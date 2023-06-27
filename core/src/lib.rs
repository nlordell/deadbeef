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

/// For the specified Safe parameters
pub fn search(mut safe: Safe, prefix: &[u8]) -> Transaction {
    let mut rng = SmallRng::from_entropy();
    while !safe.creation_address().0.starts_with(prefix) {
        safe.update_salt_nonce(|n| rng.fill(n));
    }
    safe.transaction()
}
