use tokio::time::{sleep, Duration};

use lightbulb::core::auction::AuctionManager;
use lightbulb::services::registry::{AuctionRegistry, ChainRegistry};

#[tokio::main]
async fn main() {
    // Initialize the chain and auction registries.
    let chain_registry = ChainRegistry::new();
    let auction_registry = AuctionRegistry::new(chain_registry);

    // Create an `AuctionManager` instance.
    let _manager = AuctionManager::new(auction_registry, &[1, 2, 3]);

    println!("Auction Manager started. Background worker running.");

    // Keep the main thread alive.
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
