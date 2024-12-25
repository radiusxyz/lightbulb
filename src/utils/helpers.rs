use crate::domain::{AuctionId, AuctionInfo};
use sha2::{Digest, Sha256};
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

/// Creates a new `AuctionId` by hashing the SLA fields with SHA-256 and encoding the result in hex.
pub fn compute_auction_id(sla: &AuctionInfo) -> AuctionId {
    let mut hasher = Sha256::new();
    hasher.update(sla.seller_addr.as_bytes());
    hasher.update(sla.seller_signature.as_bytes());
    hasher.update(sla.block_height.to_be_bytes());
    hasher.update(sla.blockspace_size.to_be_bytes());
    hasher.update(sla.start_time.to_be_bytes());
    hasher.update(sla.end_time.to_be_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}
