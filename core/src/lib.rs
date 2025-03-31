#[macro_use]
mod address;
pub mod config;
mod create2;
mod keccak;
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
    while !search_iter(safe, prefix, |n| rng.fill(n)) {}
}

/// Run a single iteration of the vanity address search.
///
/// This function is publically exposed to facilitate benchmarking.
#[doc(hidden)]
pub fn search_iter(safe: &mut Safe, prefix: &[u8], update: impl FnOnce(&mut [u8; 32])) -> bool {
    safe.update_salt_nonce(update);
    safe.creation_address().0.starts_with(prefix)
}
