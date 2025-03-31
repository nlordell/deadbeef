use sha3::{Digest as _, Keccak256};

/// Compute the Keccak-256 hash of the input bytes.
#[inline]
pub fn v256(bytes: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut hasher = Keccak256::new();
    hasher.update(bytes);
    hasher.finalize_into((&mut output).into());
    output
}

/// Compute the Keccak-256 hash of the input byte chunks.
pub fn v256_chunked(chunks: &[&[u8]]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut hasher = Keccak256::new();
    for chunk in chunks {
        hasher.update(chunk);
    }
    hasher.finalize_into((&mut output).into());
    output
}
