use crate::address::{Address, NonZeroAddress};
use hex_literal::hex;
use tiny_keccak::{Hasher as _, Keccak};

/// The Safe smart account creation configuration.
#[derive(Clone)]
pub struct Configuration {
    /// Proxy creation configuration.
    pub proxy: Proxy,
    /// The account configuration.
    pub account: Account,
}

/// The Safe proxy configuration.
#[derive(Clone)]
pub struct Proxy {
    /// The address of the `SafeProxyFactory` contract.
    pub factory: NonZeroAddress,
    /// The `SafeProxy` init code.
    pub init_code: Vec<u8>,
    /// The `Safe` singleton implementation address.
    pub singleton: NonZeroAddress,
}

impl Proxy {
    /// Returns the proxy init code digest.
    pub fn init_code_hash(&self) -> [u8; 32] {
        let mut output = [0_u8; 32];
        let mut hasher = Keccak::v256();
        hasher.update(&self.init_code);
        hasher.update(&abi::addr(self.singleton.get()));
        hasher.finalize(&mut output);
        output
    }

    /// Returns the calldata for the `createProxyWithNonce` call on the proxy
    /// factory.
    pub fn create_proxy_with_nonce(&self, initializer: &[u8], salt_nonce: [u8; 32]) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&hex!("1688f0b9"));
        buffer.extend_from_slice(&abi::addr(self.singleton.get()));
        buffer.extend_from_slice(&abi::num(0x60)); // initializer.offset
        buffer.extend_from_slice(&salt_nonce);
        buffer.extend_from_slice(&abi::num(initializer.len()));
        buffer.extend_from_slice(initializer);
        buffer.extend_from_slice(abi::padding(initializer)); // padding
        buffer
    }
}

/// The `Safe` smart account configuration.
#[derive(Clone)]
pub struct Account {
    /// The initial owners of the account.
    pub owners: Vec<NonZeroAddress>,
    /// The signature threshold for the account.
    pub threshold: usize,
    /// The optional `SafeToL2Setup` setup to use.
    pub setup: Option<SafeToL2Setup>,
    /// The optional fallback handler address to use.
    pub fallback_handler: Option<NonZeroAddress>,
    /// The optional address for tagging the Safe.
    pub identifier: Option<Address>,
}

impl Account {
    /// Eencodes the Safe `setup` call initializer bytes.
    pub fn initializer(&self) -> Vec<u8> {
        let (to, data) = self
            .setup
            .as_ref()
            .map(|setup| (setup.address.get(), setup.encode()))
            .unwrap_or_default();
        let fallback_handler = self
            .fallback_handler
            .map(NonZeroAddress::get)
            .unwrap_or_default();
        let payment_receiver = self.identifier.unwrap_or_default();

        let mut buffer = Vec::new();
        buffer.extend_from_slice(&hex!("b63e800d"));
        buffer.extend_from_slice(&abi::num(0x100)); // owners.offset
        buffer.extend_from_slice(&abi::num(self.threshold));
        buffer.extend_from_slice(&abi::addr(to));
        buffer.extend_from_slice(&abi::num(0x120 + 0x20 * self.owners.len())); // data.offset
        buffer.extend_from_slice(&abi::addr(fallback_handler));
        buffer.extend_from_slice(&abi::addr(Address::zero())); // paymentToken
        buffer.extend_from_slice(&abi::num(0)); // payment
        buffer.extend_from_slice(&abi::addr(payment_receiver));
        buffer.extend_from_slice(&abi::num(self.owners.len())); // owners.length
        for owner in &self.owners {
            buffer.extend_from_slice(&abi::addr(owner.get()));
        }
        buffer.extend_from_slice(&abi::num(data.len()));
        buffer.extend_from_slice(&data);
        buffer.extend_from_slice(abi::padding(&data));
        buffer
    }
}

/// Safe multi-chain setup using the `SafeToL2Setup` contract.
#[derive(Clone)]
pub struct SafeToL2Setup {
    /// The addres of the setup contract.
    pub address: NonZeroAddress,
    /// The `SafeL2` singleton for the setup.
    pub l2_singleton: NonZeroAddress,
}

impl SafeToL2Setup {
    /// Encodes the call to `safeToL2Setup` call on the setup contract.
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&hex!("fe51f643"));
        buffer.extend_from_slice(&abi::addr(self.l2_singleton.get()));
        buffer
    }
}

/// Poor man's Solidity ABI encoding.
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

    pub fn padding(d: &[u8]) -> &'static [u8] {
        static B: [u8; 32] = [0; 32];
        let l = (32 - d.len() % 32) % 32;
        &B[..l]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_initializer() {
        let account = Account {
            owners: vec![
                address!(nz "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                address!(nz "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                address!(nz "cccccccccccccccccccccccccccccccccccccccc"),
            ],
            threshold: 2,
            setup: None,
            fallback_handler: None,
            identifier: None,
        };
        assert_eq!(
            &account.initializer(),
            &hex!(
                "b63e800d
                 0000000000000000000000000000000000000000000000000000000000000100
                 0000000000000000000000000000000000000000000000000000000000000002
                 0000000000000000000000000000000000000000000000000000000000000000
                 0000000000000000000000000000000000000000000000000000000000000180
                 0000000000000000000000000000000000000000000000000000000000000000
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
    fn full_initializer() {
        let account = Account {
            owners: vec![
                address!(nz "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                address!(nz "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                address!(nz "cccccccccccccccccccccccccccccccccccccccc"),
            ],
            threshold: 2,
            setup: Some(SafeToL2Setup {
                address: address!(nz "1111111111111111111111111111111111111111"),
                l2_singleton: address!(nz "2222222222222222222222222222222222222222"),
            }),
            fallback_handler: Some(address!(nz "ffffffffffffffffffffffffffffffffffffffff")),
            identifier: None,
        };
        assert_eq!(
            &account.initializer(),
            &hex!(
                "b63e800d
                 0000000000000000000000000000000000000000000000000000000000000100
                 0000000000000000000000000000000000000000000000000000000000000002
                 0000000000000000000000001111111111111111111111111111111111111111
                 0000000000000000000000000000000000000000000000000000000000000180
                 000000000000000000000000ffffffffffffffffffffffffffffffffffffffff
                 0000000000000000000000000000000000000000000000000000000000000000
                 0000000000000000000000000000000000000000000000000000000000000000
                 0000000000000000000000000000000000000000000000000000000000000000
                 0000000000000000000000000000000000000000000000000000000000000003
                 000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
                 000000000000000000000000bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
                 000000000000000000000000cccccccccccccccccccccccccccccccccccccccc
                 0000000000000000000000000000000000000000000000000000000000000024
                 fe51f64300000000000000000000000022222222222222222222222222222222
                 2222222200000000000000000000000000000000000000000000000000000000"
            ),
        );
    }
}
