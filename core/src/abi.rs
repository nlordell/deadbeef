use crate::address::Address;
use hex_literal::hex;
use std::mem;
use tiny_keccak::{Hasher as _, Keccak};

pub fn safe_to_l2_setup(singleton_l2: Address) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&hex!("fe51f643"));
    buffer.extend_from_slice(&addr(singleton_l2));
    buffer
}

pub fn safe_setup(
    owners: &[Address],
    threshold: usize,
    to: Address,
    data: &[u8],
    fallback_handler: Address,
) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&hex!("b63e800d"));
    buffer.extend_from_slice(&num(0x100)); // owners.offset
    buffer.extend_from_slice(&num(threshold));
    buffer.extend_from_slice(&addr(to));
    buffer.extend_from_slice(&num(0x120 + 0x20 * owners.len())); // data.offset
    buffer.extend_from_slice(&addr(fallback_handler));
    buffer.extend_from_slice(&addr(Address::zero())); // paymentToken
    buffer.extend_from_slice(&num(0)); // payment
    buffer.extend_from_slice(&addr(Address::zero())); // paymentReceiver
    buffer.extend_from_slice(&num(owners.len())); // owners.length
    for owner in owners {
        buffer.extend_from_slice(&addr(*owner));
    }
    buffer.extend_from_slice(&num(data.len()));
    buffer.extend_from_slice(data);
    buffer.extend_from_slice(padding(data.len()));
    buffer
}

/// Returns the proxy init code digest.
pub fn proxy_init_code_hash(proxy_init_code: &[u8], singleton: Address) -> [u8; 32] {
    let mut output = [0_u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(proxy_init_code);
    hasher.update(&addr(singleton));
    hasher.finalize(&mut output);
    output
}

/// Returns the calldata for the `createProxyWithNonce` call on the proxy
/// factory.
pub fn create_proxy_with_nonce(
    singleton: Address,
    initializer: &[u8],
    salt_nonce: [u8; 32],
) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&hex!("1688f0b9"));
    buffer.extend_from_slice(&addr(singleton));
    buffer.extend_from_slice(&num(0x60)); // initializer.offset
    buffer.extend_from_slice(&salt_nonce);
    buffer.extend_from_slice(&num(initializer.len()));
    buffer.extend_from_slice(&initializer);
    buffer.extend_from_slice(&[0_u8; 28]); // padding
    buffer
}

fn num(a: usize) -> [u8; 32] {
    let mut b = [0_u8; 32];
    b[(32 - mem::size_of::<usize>())..].copy_from_slice(&a.to_be_bytes());
    b
}
fn addr(a: Address) -> [u8; 32] {
    let mut b = [0_u8; 32];
    b[12..].copy_from_slice(&a.0);
    b
}
fn padding(len: usize) -> &'static [u8] {
    static B: [u8; 32] = [0; 32];
    let l = (32 - len % 32) % 32;
    &B[..l]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializer_bytes() {
        assert_eq!(
            &safe_setup(
                &[
                    address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                    address!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                    address!("cccccccccccccccccccccccccccccccccccccccc"),
                ],
                2,
                address!("0000000000000000000000000000000000000000"),
                &[],
                address!("3333333333333333333333333333333333333333"),
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
        assert_eq!(
            &safe_setup(
                &[
                    address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                    address!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                    address!("cccccccccccccccccccccccccccccccccccccccc"),
                ],
                2,
                address!("3333333333333333333333333333333333333333"),
                &hex!(
                    "fe51f643
                     0000000000000000000000004444444444444444444444444444444444444444"
                ),
                address!("5555555555555555555555555555555555555555")
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
}
