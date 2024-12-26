use tokio::time::{sleep, Duration};

use lightbulb::core::auction::AuctionManager;
use lightbulb::services::registry::RegistryService;

#[tokio::main]
async fn main() {
    // Initialize the chain and auction registries.
    let (auction_registry, chain_registry) = RegistryService::create_registry().await;

    // Create a `RegistryService` instance.
    let registry_service = RegistryService::new(auction_registry.clone(), chain_registry.clone());

    // Create an `AuctionManager` instance.
    let _manager = AuctionManager::new(&registry_service).await;

    println!("Auction Manager started. Background worker running.");

    // Keep the main thread alive.
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
