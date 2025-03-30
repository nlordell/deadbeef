//! Module containing Safe

use crate::{abi, address::Address, contracts::Contracts, create2::Create2};
use tiny_keccak::{Hasher as _, Keccak};

/// Safe deployment for computing deterministic addresses.
#[derive(Clone)]
pub struct Safe {
    contracts: Contracts,
    initializer: Vec<u8>,
    salt: [u8; 64],
    create2: Create2,
}

/// Safe deployment transaction information.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    /// The `to` address for the Ethereum transaction.
    pub to: Address,
    /// The calldata to send to the proxy factory to create this Safe.
    pub calldata: Vec<u8>,
}

impl Safe {
    /// Creates a new safe from deployment parameters.
    pub fn new(contracts: Contracts, owners: Vec<Address>, threshold: usize) -> Self {
        let (to, data) = contracts
            .setup
            .as_ref()
            .map(|setup| (setup.address, abi::safe_to_l2_setup(setup.singleton_l2)))
            .unwrap_or_default();
        let initializer =
            abi::safe_setup(&owners, threshold, to, &data, contracts.fallback_handler);

        let mut salt = [0_u8; 64];
        let mut hasher = Keccak::v256();
        hasher.update(&initializer);
        hasher.finalize(&mut salt[0..32]);

        let mut create2 = Create2::new(
            contracts.proxy_factory,
            Default::default(),
            abi::proxy_init_code_hash(&contracts.proxy_init_code, contracts.singleton),
        );
        let mut hasher = Keccak::v256();
        hasher.update(&salt);
        hasher.finalize(create2.salt_mut());

        Self {
            contracts,
            initializer,
            salt,
            create2,
        }
    }

    /// Returns the creation address for the Safe.
    pub fn creation_address(&self) -> Address {
        self.create2.creation_address()
    }

    /// Returns the current salt nonce value for the Safe deployment.
    pub fn salt_nonce(&self) -> [u8; 32] {
        self.salt[32..64].try_into().unwrap()
    }

    /// Returns the initializer calldata for the Safe.
    pub fn initializer(&self) -> &[u8] {
        &self.initializer
    }

    /// Updates the salt nonce and recomputes the `CREATE2` salt.
    pub fn update_salt_nonce(&mut self, f: impl FnOnce(&mut [u8])) {
        let salt_nonce = unsafe { self.salt.get_unchecked_mut(32..64) };
        f(salt_nonce);

        let mut hasher = Keccak::v256();
        hasher.update(&self.salt);
        hasher.finalize(self.create2.salt_mut());
    }

    /// Returns the transaction information for the current safe deployment.
    pub fn transaction(&self) -> Transaction {
        Transaction {
            to: self.contracts.proxy_factory,
            calldata: abi::create_proxy_with_nonce(
                self.contracts.singleton,
                &self.initializer,
                self.salt_nonce(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn transaction() {
        let mut safe = Safe::new(
            Contracts {
                proxy_factory: address!("1111111111111111111111111111111111111111"),
                proxy_init_code: vec![],
                singleton: address!("2222222222222222222222222222222222222222"),
                setup: None,
                fallback_handler: address!("3333333333333333333333333333333333333333"),
            },
            vec![
                address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                address!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                address!("cccccccccccccccccccccccccccccccccccccccc"),
            ],
            2,
        );
        safe.update_salt_nonce(|nonce| nonce.fill(0xee));

        assert_eq!(
            safe.transaction(),
            Transaction {
                to: address!("1111111111111111111111111111111111111111"),
                calldata: hex!(
                    "1688f0b9
                     0000000000000000000000002222222222222222222222222222222222222222
                     0000000000000000000000000000000000000000000000000000000000000060
                     eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
                     00000000000000000000000000000000000000000000000000000000000001a4
                     b63e800d00000000000000000000000000000000000000000000000000000000
                     0000010000000000000000000000000000000000000000000000000000000000
                     0000000200000000000000000000000000000000000000000000000000000000
                     0000000000000000000000000000000000000000000000000000000000000000
                     0000018000000000000000000000000033333333333333333333333333333333
                     3333333300000000000000000000000000000000000000000000000000000000
                     0000000000000000000000000000000000000000000000000000000000000000
                     0000000000000000000000000000000000000000000000000000000000000000
                     0000000000000000000000000000000000000000000000000000000000000000
                     00000003000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
                     aaaaaaaa000000000000000000000000bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
                     bbbbbbbb000000000000000000000000cccccccccccccccccccccccccccccccc
                     cccccccc00000000000000000000000000000000000000000000000000000000
                     0000000000000000000000000000000000000000000000000000000000000000"
                )
                .to_vec(),
            }
        );
    }
}
