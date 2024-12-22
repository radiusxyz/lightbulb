use std::sync::Arc;
use tokio::time::{sleep, Duration};

use lightbulb::services::auction::AuctionManager;

#[tokio::main]
async fn main() {
    let manager = Arc::new(AuctionManager::new());

    // Start the background worker in an actor-like pattern to periodically process auctions.
    let _jh = manager.start_worker();

    println!("Auction Manager started. Background worker running.");

    // For demonstration purposes, keep the main thread alive.
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
