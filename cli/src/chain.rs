//! Module for chain-specific data.

use deadbeef_core::{address, hex, Contracts};
use std::{
    fmt::{self, Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};

/// A Safe supported chain.
#[derive(Clone, Copy, Debug)]
pub struct Chain(u128);

impl Chain {
    /// Returns a [`Chain`] for Ethereum Mainnet.
    pub const fn ethereum() -> Self {
        Self(1)
    }

    /// Returns the [`Contracts`] for this chain, or [`None`] if the chain is
    /// not supported.
    pub fn contracts(&self) -> Option<Contracts> {
        // Addresses can be found in the Safe deployments repository:
        // <https://github.com/safe-global/safe-deployments/tree/main/src/assets/v1.4.1>
        // The `proxyCreationCode` can be read from the from the proxy factory:
        // <https://etherscan.io/address/0xa6B71E26C5e0845f74c812102Ca7114b6a896AB2>
        match self.0 {
            1 | 100 => Some(Contracts {
                proxy_factory: address!("a6B71E26C5e0845f74c812102Ca7114b6a896AB2"),
                proxy_init_code: hex!(
                    "608060405234801561001057600080fd5b506040516101e63803806101e68339
                     818101604052602081101561003357600080fd5b810190808051906020019092
                     9190505050600073ffffffffffffffffffffffffffffffffffffffff168173ff
                     ffffffffffffffffffffffffffffffffffffff1614156100ca576040517f08c3
                     79a0000000000000000000000000000000000000000000000000000000008152
                     6004018080602001828103825260228152602001806101c46022913960400191
                     505060405180910390fd5b806000806101000a81548173ffffffffffffffffff
                     ffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffff
                     ffffffffff1602179055505060ab806101196000396000f3fe608060405273ff
                     ffffffffffffffffffffffffffffffffffffff600054167fa619486e00000000
                     0000000000000000000000000000000000000000000000006000351415605057
                     8060005260206000f35b3660008037600080366000845af43d6000803e600081
                     14156070573d6000fd5b3d6000f3fea2646970667358221220d1429297349653
                     a4918076d650332de1a1068c5f3e07c5c82360c277770b955264736f6c634300
                     07060033496e76616c69642073696e676c65746f6e2061646472657373207072
                     6f7669646564"
                )
                .to_vec(),
                singleton: address!("d9Db270c1B5E3Bd161E8c8503c55cEABeE709552"),
                fallback_handler: address!("f48f2B2d2a534e402487b3ee7C18c33Aec0Fe5e4"),
            }),
            _ => None,
        }
    }

    /// Returns the URL of the block explorer for the chain, or [`None`] if the
    /// chain is not supported.
    pub fn explorer(&self) -> Option<&str> {
        match self.0 {
            1 => Some("https://etherscan.io"),
            100 => Some("https://gnosisscan.io"),
            _ => None,
        }
    }
}

impl Default for Chain {
    fn default() -> Self {
        Self::ethereum()
    }
}

impl Display for Chain {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Chain {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, radix) = match s.strip_prefix("0x") {
            Some(s) => (s, 16),
            None => (s, 10),
        };
        let value = u128::from_str_radix(s, radix)?;

        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadbeef_core::Safe;

    #[test]
    fn proxy_init_code_digest() {
        assert_eq!(
            Chain::default()
                .contracts()
                .unwrap()
                .proxy_init_code_digest(),
            hex!("56e3081a3d1bb38ed4eed1a39f7729c3cc77c7825794c15bbf326f3047fd779c"),
        );
    }

    #[test]
    fn compute_address() {
        // <https://etherscan.io/tx/0x7b0615b648cb5b9ee366cd22af4e0e40fe90d67c0e140c6efdaabb20b3033a63>
        let mut safe = Safe::new(
            Chain::ethereum().contracts().unwrap(),
            vec![
                address!("5c8c76f2e990f194462dc5f8a8c76ba16966ed42"),
                address!("703f28830eeaaad54e786a839f6602ca098016a5"),
                address!("0e706a98f414f49a412107641c0820b0153ff5dc"),
                address!("173286fafabea063eeb3726ee5efd4ff414057b9"),
                address!("2f2806e8b288428f23707a69faa60f52bc565c17"),
                address!("4507cfb4b077d5dbddd520c701e30173d5b59fad"),
            ],
            3,
        );
        safe.update_salt_nonce(|n| {
            n.copy_from_slice(&hex!(
                "0000000000000000000000000000000000000000000000000000018bbf9209f3"
            ))
        });

        let address = safe.creation_address();
        assert_eq!(
            address,
            address!("5836152812568244760ba356b5f3838aa5b672e0")
        );
    }
}
