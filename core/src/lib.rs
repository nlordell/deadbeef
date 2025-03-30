#[macro_use]
mod address;
pub mod config;
mod create2;
mod safe;

pub use self::{
    address::{Address, NonZeroAddress},
    config::Configuration,
    safe::{Safe, Transaction},
};
pub use hex_literal::hex;
use rand::{rngs::SmallRng, Rng as _, SeedableRng as _};

/// Search for a vanity address with the specified Safe parameters and prefix.
pub fn search(safe: &mut Safe, prefix: &[u8]) {
    let mut rng = SmallRng::from_os_rng();
    while !safe.creation_address().0.starts_with(prefix) {
        safe.update_salt_nonce(|n| rng.fill(n));
    }
}
