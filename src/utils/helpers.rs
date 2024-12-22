use std::time::{SystemTime, UNIX_EPOCH};

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
