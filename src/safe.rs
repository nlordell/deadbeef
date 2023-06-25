//! Module containing Safe

use crate::{
    address::{address, Address},
    create2::Create2,
};
use hex_literal::hex;
use tiny_keccak::{Hasher as _, Keccak};

const PROXY_FACTORY: Address = address!("4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67");
const PROXY_INIT_CODE_DIGEST: [u8; 32] =
    hex!("76733d705f71b79841c0ee960a0ca880f779cde7ef446c989e6d23efc0a4adfb");
const SINGLETON: Address = address!("41675C099F32341bf84BFc5382aF534df5C7461a");
const FALLBACK_HANDLER: Address = address!("fd0732Dc9E303f09fCEf3a7388Ad10A83459Ec99");

/// Safe deployment for computing deterministic addresses.
#[derive(Clone)]
pub struct Safe {
    owners: Vec<Address>,
    threshold: usize,
    salt: [u8; 64],
    create2: Create2,
}

/// Safe deployment transaction information.
pub struct Info {
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
    pub fn new(owners: Vec<Address>, threshold: usize) -> Self {
        let mut salt = [0_u8; 64];
        let mut hasher = Keccak::v256();
        hasher.update(&initializer(&owners, threshold));
        hasher.finalize(&mut salt[0..32]);

        let mut create2 = Create2::new(PROXY_FACTORY, Default::default(), PROXY_INIT_CODE_DIGEST);
        let mut hasher = Keccak::v256();
        hasher.update(&salt);
        hasher.finalize(create2.salt_mut());

        Self {
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
    pub fn info(self) -> Info {
        let calldata = proxy_calldata(&self.owners, self.threshold, self.salt_nonce());
        Info {
            creation_address: self.creation_address(),
            factory: PROXY_FACTORY,
            singleton: SINGLETON,
            owners: self.owners,
            threshold: self.threshold,
            fallback_handler: FALLBACK_HANDLER,
            calldata,
        }
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

/// Computes the initializer calldata for the specified Safe parameters.
fn initializer(owners: &[Address], threshold: usize) -> Vec<u8> {
    use abi::*;

    let mut buffer = Vec::new();
    buffer.extend_from_slice(&hex!("b63e800d"));
    buffer.extend_from_slice(&num(0x100)); // owners.offset
    buffer.extend_from_slice(&num(threshold));
    buffer.extend_from_slice(&addr(Address::zero())); // to
    buffer.extend_from_slice(&num(0x120 + 0x20 * owners.len())); // data.offset
    buffer.extend_from_slice(&addr(FALLBACK_HANDLER));
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
fn proxy_calldata(owners: &[Address], threshold: usize, salt_nonce: [u8; 32]) -> Vec<u8> {
    use abi::*;

    let initializer = initializer(owners, threshold);
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&hex!("1688f0b9"));
    buffer.extend_from_slice(&addr(SINGLETON));
    buffer.extend_from_slice(&num(0x60)); // initializer.offset
    buffer.extend_from_slice(&salt_nonce);
    buffer.extend_from_slice(&num(initializer.len()));
    buffer.extend_from_slice(&initializer);
    buffer.extend_from_slice(&[0_u8; 28]); // padding
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializer_bytes() {
        assert_eq!(
            &initializer(
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
                 000000000000000000000000fd0732dc9e303f09fcef3a7388ad10a83459ec99
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
    fn compute_address() {
        // <https://etherscan.io/tx/0xdac58edb65c2af3f86f03586eeec7caa7ee245d6d06679a913e5dda16617658e>
        let mut safe = Safe::new(
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
            address!("000000000034065b3a94c2118cfe5b4c0067b615")
        );
    }

    #[test]
    fn proxy_init_code_digest() {
        // Proxy factory address retrieved from the Safe deployments repository:
        // <https://github.com/safe-global/safe-deployments/tree/main/src/assets/v1.4.1>
        // The `proxyCreationCode` can be read from the from the proxy factory:
        // <https://etherscan.io/address/0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67>
        const PROXY_INIT_CODE: &[u8] = &hex!(
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
        );

        let mut output = [0_u8; 32];
        let mut hasher = Keccak::v256();
        hasher.update(PROXY_INIT_CODE);
        hasher.update(&abi::addr(SINGLETON));
        hasher.finalize(&mut output);

        assert_eq!(output, PROXY_INIT_CODE_DIGEST);
    }
}
