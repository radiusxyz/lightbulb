use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

/// Returns the current Unix timestamp in milliseconds.
pub fn current_unix_ms() -> u64 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    now.as_millis() as u64
}

/// Mock function for signature verification. Always returns `true` in this demo.
/// Replace with a real cryptographic check in production.
pub fn verify_signature(_addr: &str, _signature: &str) -> bool {
    true
}

/// Computes a SHA-256 hash of the provided inputs and returns the result as a hex-encoded string.
pub fn compute_hash(inputs: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    for input in inputs {
        hasher.update(input);
    }
    let result = hasher.finalize();
    hex::encode(result)
}
