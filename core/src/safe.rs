//! Module containing Safe

use crate::{address::Address, create2::Create2};
use hex_literal::hex;
use tiny_keccak::{Hasher as _, Keccak};

/// Safe deployment for computing deterministic addresses.
#[derive(Clone)]
pub struct Safe {
    contracts: Contracts,
    owners: Vec<Address>,
    threshold: usize,
    salt: [u8; 64],
    create2: Create2,
}

/// Safe contract data on a given chain.
#[derive(Clone)]
pub struct Contracts {
    /// The address of the `SafeProxyFactory` contract.
    pub proxy_factory: Address,
    /// The `SafeProxy` init code.
    pub proxy_init_code: Vec<u8>,
    /// The `Safe` singleton address.
    pub singleton: Address,
    /// The default `CompatabilityFallbackHandler` address to use.
    pub fallback_handler: Address,
}

/// Safe deployment transaction information.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    /// The final address that the safe will end up on.
    pub creation_address: Address,
    /// The address of the proxy factory for deploying the Safe.
    pub factory: Address,
    /// The address of the Safe singleton contract.
    pub singleton: Address,
    /// The owners of the safe.
    pub owners: Vec<Address>,
    /// The signature threshold to use with the safe.
    pub threshold: usize,
    /// The address of the fallback handler that the safe was initialized with.
    pub fallback_handler: Address,
    /// The calldata to send to the proxy factory to create this Safe.
    pub calldata: Vec<u8>,
}

impl Safe {
    /// Creates a new safe from deployment parameters.
    pub fn new(contracts: Contracts, owners: Vec<Address>, threshold: usize) -> Self {
        let mut salt = [0_u8; 64];
        let mut hasher = Keccak::v256();
        hasher.update(&contracts.initializer(&owners, threshold));
        hasher.finalize(&mut salt[0..32]);

        let mut create2 = Create2::new(
            contracts.proxy_factory,
            Default::default(),
            contracts.proxy_init_code_digest(),
        );
        let mut hasher = Keccak::v256();
        hasher.update(&salt);
        hasher.finalize(create2.salt_mut());

        Self {
            contracts,
            owners,
            threshold,
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

    /// Updates the salt nonce and recomputes the `CREATE2` salt.
    pub fn update_salt_nonce(&mut self, f: impl FnOnce(&mut [u8])) {
        let salt_nonce = unsafe { self.salt.get_unchecked_mut(32..64) };
        f(salt_nonce);

        let mut hasher = Keccak::v256();
        hasher.update(&self.salt);
        hasher.finalize(self.create2.salt_mut());
    }

    /// Returns the transaction information for the current safe deployment.
    pub fn transaction(self) -> Transaction {
        let calldata =
            self.contracts
                .proxy_calldata(&self.owners, self.threshold, self.salt_nonce());
        Transaction {
            creation_address: self.creation_address(),
            factory: self.contracts.proxy_factory,
            singleton: self.contracts.singleton,
            owners: self.owners,
            threshold: self.threshold,
            fallback_handler: self.contracts.fallback_handler,
            calldata,
        }
    }
}

impl Contracts {
    /// Returns the proxy init code digest.
    pub fn proxy_init_code_digest(&self) -> [u8; 32] {
        let mut output = [0_u8; 32];
        let mut hasher = Keccak::v256();
        hasher.update(&self.proxy_init_code);
        hasher.update(&abi::addr(self.singleton));
        hasher.finalize(&mut output);
        output
    }

    /// Computes the initializer calldata for the specified Safe parameters.
    pub fn initializer(&self, owners: &[Address], threshold: usize) -> Vec<u8> {
        use abi::*;

        let mut buffer = Vec::new();
        buffer.extend_from_slice(&hex!("b63e800d"));
        buffer.extend_from_slice(&num(0x100)); // owners.offset
        buffer.extend_from_slice(&num(threshold));
        buffer.extend_from_slice(&addr(Address::zero())); // to
        buffer.extend_from_slice(&num(0x120 + 0x20 * owners.len())); // data.offset
        buffer.extend_from_slice(&addr(self.fallback_handler));
        buffer.extend_from_slice(&addr(Address::zero())); // paymentToken
        buffer.extend_from_slice(&num(0)); // payment
        buffer.extend_from_slice(&addr(Address::zero())); // paymentReceiver
        buffer.extend_from_slice(&num(owners.len())); // owners.length
        for owner in owners {
            buffer.extend_from_slice(&addr(*owner)); // owners.length
        }
        buffer.extend_from_slice(&num(0)); // data.length
        buffer
    }

    /// Returns the calldata required for the transaction to deploy the proxy.
    pub fn proxy_calldata(
        &self,
        owners: &[Address],
        threshold: usize,
        salt_nonce: [u8; 32],
    ) -> Vec<u8> {
        use abi::*;

        let initializer = self.initializer(owners, threshold);
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&hex!("1688f0b9"));
        buffer.extend_from_slice(&addr(self.singleton));
        buffer.extend_from_slice(&num(0x60)); // initializer.offset
        buffer.extend_from_slice(&salt_nonce);
        buffer.extend_from_slice(&num(initializer.len()));
        buffer.extend_from_slice(&initializer);
        buffer.extend_from_slice(&[0_u8; 28]); // padding
        buffer
    }
}

/// Poor man's ABI encode.
mod abi {
    use crate::address::Address;
    use std::mem;

    pub fn num(a: usize) -> [u8; 32] {
        let mut b = [0_u8; 32];
        b[(32 - mem::size_of::<usize>())..].copy_from_slice(&a.to_be_bytes());
        b
    }
    pub fn addr(a: Address) -> [u8; 32] {
        let mut b = [0_u8; 32];
        b[12..].copy_from_slice(&a.0);
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializer_bytes() {
        let contracts = Contracts {
            proxy_factory: address!("1111111111111111111111111111111111111111"),
            proxy_init_code: vec![],
            singleton: address!("2222222222222222222222222222222222222222"),
            fallback_handler: address!("3333333333333333333333333333333333333333"),
        };

        assert_eq!(
            &contracts.initializer(
                &[
                    address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                    address!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                    address!("cccccccccccccccccccccccccccccccccccccccc"),
                ],
                2,
            ),
            &hex!(
                "b63e800d
                 0000000000000000000000000000000000000000000000000000000000000100
                 0000000000000000000000000000000000000000000000000000000000000002
                 0000000000000000000000000000000000000000000000000000000000000000
                 0000000000000000000000000000000000000000000000000000000000000180
                 0000000000000000000000003333333333333333333333333333333333333333
                 0000000000000000000000000000000000000000000000000000000000000000
                 0000000000000000000000000000000000000000000000000000000000000000
                 0000000000000000000000000000000000000000000000000000000000000000
                 0000000000000000000000000000000000000000000000000000000000000003
                 000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
                 000000000000000000000000bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
                 000000000000000000000000cccccccccccccccccccccccccccccccccccccccc
                 0000000000000000000000000000000000000000000000000000000000000000"
            ),
        );
    }

    #[test]
    fn transaction() {
        let mut safe = Safe::new(
            Contracts {
                proxy_factory: address!("1111111111111111111111111111111111111111"),
                proxy_init_code: vec![],
                singleton: address!("2222222222222222222222222222222222222222"),
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
                creation_address: address!("cDa7814460beF6D0BF5dc1b34AB29605b36c3bC4"),
                factory: address!("1111111111111111111111111111111111111111"),
                singleton: address!("2222222222222222222222222222222222222222"),
                owners: vec![
                    address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                    address!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                    address!("cccccccccccccccccccccccccccccccccccccccc"),
                ],
                threshold: 2,
                fallback_handler: address!("3333333333333333333333333333333333333333"),
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
