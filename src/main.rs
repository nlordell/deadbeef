use hex_literal::hex;
use rand::Rng as _;
use std::{env, process, sync::mpsc, thread};
use tiny_keccak::{Hasher as _, Keccak};

fn initializer(address: [u8; 20]) -> [u8; 356] {
    let mut initializer = hex!(
        "b63e800d
         0000000000000000000000000000000000000000000000000000000000000100
         0000000000000000000000000000000000000000000000000000000000000001
         0000000000000000000000000000000000000000000000000000000000000000
         0000000000000000000000000000000000000000000000000000000000000140
         000000000000000000000000f48f2b2d2a534e402487b3ee7c18c33aec0fe5e4
         0000000000000000000000000000000000000000000000000000000000000000
         0000000000000000000000000000000000000000000000000000000000000000
         0000000000000000000000000000000000000000000000000000000000000000
         0000000000000000000000000000000000000000000000000000000000000001
         000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
         0000000000000000000000000000000000000000000000000000000000000000"
    );
    initializer[304..][..20].copy_from_slice(&address);
    initializer
}

#[derive(Clone)]
struct Salt([u8; 64]);

impl Salt {
    fn new(address: [u8; 20]) -> Self {
        let mut bytes = [0xff; 64];

        let mut hasher = Keccak::v256();
        hasher.update(&initializer(address));
        hasher.finalize(&mut bytes[..32]);

        Self(bytes)
    }

    fn nonce_mut(&mut self) -> &mut [u8] {
        &mut self.0[32..]
    }

    fn nonce(&self) -> [u8; 32] {
        let mut nonce = [0_u8; 32];
        nonce.copy_from_slice(&self.0[32..]);
        nonce
    }

    fn value(&self, output: &mut [u8]) {
        let mut hasher = Keccak::v256();
        hasher.update(&self.0);
        hasher.finalize(output);
    }
}

struct Create2([u8; 85]);

impl Create2 {
    fn new() -> Self {
        Self(hex!(
            "ff
             a6b71e26c5e0845f74c812102ca7114b6a896ab2
             0000000000000000000000000000000000000000000000000000000000000000
             56e3081a3d1bb38ed4eed1a39f7729c3cc77c7825794c15bbf326f3047fd779c"
        ))
    }

    fn factory(&self) -> [u8; 20] {
        let mut factory = [0_u8; 20];
        factory.copy_from_slice(&self.0[1..][..20]);
        factory
    }

    fn salt(&mut self, salt: &Salt) {
        salt.value(&mut self.0[21..][..32]);
    }

    fn creation_address(&self) -> [u8; 20] {
        let mut buffer = [0_u8; 32];
        let mut hasher = Keccak::v256();
        hasher.update(&self.0);
        hasher.finalize(&mut buffer);

        // SAFETY: ok because we know the length is correct.
        unsafe { *buffer[12..].as_ptr().cast() }
    }
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("USAGE: deadbeef ADDRESS PREFIX");
        process::exit(1);
    }

    let address = {
        let mut buffer = [0_u8; 20];
        hex::decode_to_slice(strip_ox(&args[0]), &mut buffer).unwrap_or_else(|err| {
            eprintln!("ERROR: invalid address: {err}");
            process::exit(1);
        });
        buffer
    };
    let prefix = hex::decode(strip_ox(&args[1])).unwrap_or_else(|err| {
        eprintln!("ERROR: invalid address: {err}");
        process::exit(1);
    });

    let (sender, receiver) = mpsc::channel();
    let threads = (0..num_cpus::get())
        .map(|_| {
            thread::spawn({
                let prefix = prefix.clone();
                let result = sender.clone();
                move || search_address(address, &prefix, result)
            })
        })
        .collect::<Vec<_>>();

    let safe = receiver.recv().expect("missing result");
    println!("address:    0x{}", hex::encode(&safe.address));
    println!("factory:    0x{}", hex::encode(&safe.factory));
    println!("salt_nonce: 0x{}", hex::encode(&safe.salt_nonce));
    println!("calldata:   0x{}", hex::encode(&safe.calldata));

    let _ = threads;
    process::exit(0);
}

fn strip_ox(s: &str) -> &str {
    s.strip_prefix("0x").unwrap_or(s)
}

struct VanitySafe {
    address: [u8; 20],
    factory: [u8; 20],
    salt_nonce: [u8; 32],
    calldata: Vec<u8>,
}

fn search_address(address: [u8; 20], prefix: &[u8], result: mpsc::Sender<VanitySafe>) {
    let mut rng = rand::thread_rng();

    let mut salt = Salt::new(address);
    let mut create2 = Create2::new();

    while !create2.creation_address().starts_with(prefix) {
        rng.fill(salt.nonce_mut());
        create2.salt(&salt);
    }

    let calldata = {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&hex!(
            "1688f0b9
             000000000000000000000000d9db270c1b5e3bd161e8c8503c55ceabee709552
             0000000000000000000000000000000000000000000000000000000000000060"
        ));
        buffer.extend_from_slice(salt.nonce_mut());
        buffer.extend_from_slice(&hex!(
            "0000000000000000000000000000000000000000000000000000000000000164"
        ));
        buffer.extend_from_slice(&initializer(address));
        buffer.extend_from_slice(&[0_u8; 28]);
        buffer
    };

    let _ = result.send(VanitySafe {
        address: create2.creation_address(),
        factory: create2.factory(),
        salt_nonce: salt.nonce(),
        calldata,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_address() {
        let mut salt = Salt::new(hex!("85108e6bEE0E6E4d317b72751365d5A5D2Ee66a5"));
        let mut create2 = Create2::new();

        salt.nonce_mut().copy_from_slice(&hex!(
            "0000000000000000000000000000000000000000000000000000017e63b10d14"
        ));
        create2.salt(&salt);

        let address = create2.creation_address();
        assert_eq!(address, hex!("8c166d8d0d6d884e433196e06d44cca2be9a21c9"));
    }
}
