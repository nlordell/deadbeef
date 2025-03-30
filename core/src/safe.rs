//! Module containing Safe

use crate::{address::Address, create2::Create2};
use hex_literal::hex;
use tiny_keccak::{Hasher as _, Keccak};

/// Safe deployment for computing deterministic addresses.
#[derive(Clone)]
pub struct Safe {
    contracts: Contracts,
    initializer: Vec<u8>,
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
    /// The optional `SafeToL2Setup` setup to use.
    pub setup: Option<SafeToL2Setup>,
    /// The fallback handler address to use (for example, the
    /// `CompatibilityFallbackHandler`).
    pub fallback_handler: Address,
}

/// Safe multi-chain setup.
#[derive(Clone)]
pub struct SafeToL2Setup {
    /// The addres of the setup contract.
    pub address: Address,
    /// The `SafeL2` singleton for the setup.
    pub singleton_l2: Address,
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
        let initializer = contracts.initializer(&owners, threshold);

        let mut salt = [0_u8; 64];
        let mut hasher = Keccak::v256();
        hasher.update(&initializer);
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
        let calldata = self
            .contracts
            .create_proxy_with_nonce(&self.initializer, self.salt_nonce());
        Transaction {
            to: self.contracts.proxy_factory,
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
    fn initializer(&self, owners: &[Address], threshold: usize) -> Vec<u8> {
        use abi::*;

        let (to, data) = match &self.setup {
            Some(setup) => {
                let mut buffer = Vec::new();
                buffer.extend_from_slice(&hex!("fe51f643"));
                buffer.extend_from_slice(&addr(setup.singleton_l2));
                (setup.address, buffer)
            }
            None => (Address::zero(), Vec::new()),
        };

        let mut buffer = Vec::new();
        buffer.extend_from_slice(&hex!("b63e800d"));
        buffer.extend_from_slice(&num(0x100)); // owners.offset
        buffer.extend_from_slice(&num(threshold));
        buffer.extend_from_slice(&addr(to));
        buffer.extend_from_slice(&num(0x120 + 0x20 * owners.len())); // data.offset
        buffer.extend_from_slice(&addr(self.fallback_handler));
        buffer.extend_from_slice(&addr(Address::zero())); // paymentToken
        buffer.extend_from_slice(&num(0)); // payment
        buffer.extend_from_slice(&addr(Address::zero())); // paymentReceiver
        buffer.extend_from_slice(&num(owners.len())); // owners.length
        for owner in owners {
            buffer.extend_from_slice(&addr(*owner));
        }
        buffer.extend_from_slice(&num(data.len()));
        buffer.extend_from_slice(&padded(data));
        buffer
    }

    /// Returns the calldata for the `createProxyWithNonce` call on the proxy
    /// factory.
    fn create_proxy_with_nonce(&self, initializer: &[u8], salt_nonce: [u8; 32]) -> Vec<u8> {
        use abi::*;

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
    pub fn padded(mut d: Vec<u8>) -> Vec<u8> {
        let b = [0_u8; 32];
        let l = (32 - d.len() % 32) % 32;
        d.extend_from_slice(&b[..l]);
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn initializer_bytes() {
        let contracts = Contracts {
            proxy_factory: address!("1111111111111111111111111111111111111111"),
            proxy_init_code: vec![],
            singleton: address!("2222222222222222222222222222222222222222"),
            setup: None,
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
    fn safe_to_l2_setup() {
        let contracts = Contracts {
            proxy_factory: address!("1111111111111111111111111111111111111111"),
            proxy_init_code: vec![],
            singleton: address!("2222222222222222222222222222222222222222"),
            setup: Some(SafeToL2Setup {
                address: address!("3333333333333333333333333333333333333333"),
                singleton_l2: address!("4444444444444444444444444444444444444444"),
            }),
            fallback_handler: address!("5555555555555555555555555555555555555555"),
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
                 0000000000000000000000003333333333333333333333333333333333333333
                 0000000000000000000000000000000000000000000000000000000000000180
                 0000000000000000000000005555555555555555555555555555555555555555
                 0000000000000000000000000000000000000000000000000000000000000000
                 0000000000000000000000000000000000000000000000000000000000000000
                 0000000000000000000000000000000000000000000000000000000000000000
                 0000000000000000000000000000000000000000000000000000000000000003
                 000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
                 000000000000000000000000bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
                 000000000000000000000000cccccccccccccccccccccccccccccccccccccccc
                 0000000000000000000000000000000000000000000000000000000000000024
                 fe51f643
                 0000000000000000000000004444444444444444444444444444444444444444
                         00000000000000000000000000000000000000000000000000000000"
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
