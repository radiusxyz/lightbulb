use std::sync::Arc;
use tokio::time::{sleep, Duration};

use lightbulb::services::{
    auction::AuctionManager,
    registry::{chain, AuctionRegistry, ChainRegistry},
};

#[tokio::main]
async fn main() {
    // Initialize the chain and auction registries.
    let chain_registry = ChainRegistry::new();
    let auction_registry = AuctionRegistry::new(chain_registry);

    // Create the auction manager with the auction registry.
    let manager = Arc::new(AuctionManager::new(auction_registry));

    // Start the background worker in an actor-like pattern to periodically process auctions.
    let _jh = manager.start_worker();

    println!("Auction Manager started. Background worker running.");

    // For demonstration purposes, keep the main thread alive.
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
