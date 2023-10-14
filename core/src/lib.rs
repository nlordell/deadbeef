#[macro_use]
mod address;
mod create2;
mod safe;

pub use self::{
    address::Address,
    safe::{Contracts, Safe, Transaction},
};
pub use hex_literal::hex;
use rand::{rngs::SmallRng, Rng as _, SeedableRng as _};

/// Search for a vanity address with the specified Safe parameters and prefix.
pub fn search(safe: &mut Safe, prefix: &[u8]) {
    let mut rng = SmallRng::from_entropy();
    while !starts_with_nibbles(&safe.creation_address().0[..], prefix) {
        safe.update_salt_nonce(|n| rng.fill(n));
    }
}

fn starts_with_nibbles(data: &[u8], prefix_nibbles: &[u8]) -> bool {
    let data_nibbles: Vec<u8> = data
        .iter()
        .flat_map(|&byte| vec![byte >> 4, byte & 0x0F])
        .collect();

    data_nibbles.starts_with(prefix_nibbles)
}
