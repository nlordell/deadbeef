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
        // <https://etherscan.io/address/0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67>
        match self.0 {
            1 | 100 => Some(Contracts {
                proxy_factory: address!("4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67"),
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
                     14156070573d6000fd5b3d6000f3fea264697066735822122003d1488ee65e08
                     fa41e58e888a9865554c535f2c77126a82cb4c0f917f31441364736f6c634300
                     07060033496e76616c69642073696e676c65746f6e2061646472657373207072
                     6f7669646564"
                )
                .to_vec(),
                singleton: address!("41675C099F32341bf84BFc5382aF534df5C7461a"),
                fallback_handler: address!("fd0732Dc9E303f09fCEf3a7388Ad10A83459Ec99"),
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
            hex!("76733d705f71b79841c0ee960a0ca880f779cde7ef446c989e6d23efc0a4adfb"),
        );
    }

    #[test]
    fn compute_address() {
        // <https://etherscan.io/tx/0xdac58edb65c2af3f86f03586eeec7caa7ee245d6d06679a913e5dda16617658e>
        let mut safe = Safe::new(
            Chain::ethereum().contracts().unwrap(),
            vec![
                address!("34f845773D4364999f2fbC7AA26ABDeE902cBb46"),
                address!("E2Df39d8c1c393BDe653D96a09852508CA2816e5"),
                address!("000000000dD7Bc0bcCE4392698dc3e11004F20eB"),
                address!("Cbd6073f486714E6641bf87c22A9CEc25aCf5804"),
            ],
            2,
        );
        safe.update_salt_nonce(|n| {
            n.copy_from_slice(&hex!(
                "c437564b491906978ae4396733fbc0835f87e6b2578193331caa87645ebe9bdc"
            ))
        });

        let address = safe.creation_address();
        assert_eq!(
            address,
            address!("000000000034065b3a94C2118CFe5B4C0067B615")
        );
    }
}
