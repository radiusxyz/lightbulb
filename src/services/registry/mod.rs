pub mod auction;
pub mod chain;

use std::sync::Arc;

pub use auction::AuctionRegistry;
pub use chain::ChainRegistry;
use tokio::sync::RwLock;

use crate::{
    domain::{AuctionInfo, ChainId, ChainInfo},
    utils::{errors::RegistryError, types::ArcRwLock},
};

/// `RegistryService` handles the registration and validation of chains and auctions.
pub struct RegistryService {
    /// Stores auction-related data in a thread-safe manner.
    auction_registry: ArcRwLock<AuctionRegistry>,

    /// Stores chain-related data in a thread-safe manner.
    chain_registry: ArcRwLock<ChainRegistry>,
}

impl RegistryService {
    /// Creates a new `RegistryService` instance with the provided registries.
    pub fn new(
        auction_registry: ArcRwLock<AuctionRegistry>,
        chain_registry: ArcRwLock<ChainRegistry>,
    ) -> Self {
        RegistryService {
            auction_registry,
            chain_registry,
        }
    }

    /// Initializes new registries for chains and auctions.
    pub async fn create_registry() -> (ArcRwLock<AuctionRegistry>, ArcRwLock<ChainRegistry>) {
        let chain_registry = Arc::new(RwLock::new(ChainRegistry::default()));
        let auction_registry = Arc::new(RwLock::new(AuctionRegistry::new(&chain_registry).await));

        (auction_registry, chain_registry)
    }

    /// Provides a clone of the auction registry.
    pub fn get_auction_registry(&self) -> ArcRwLock<AuctionRegistry> {
        self.auction_registry.clone()
    }

    /// Retrieves a list of all registered chain IDs.
    pub async fn get_chain_ids(&self) -> Vec<ChainId> {
        let chain_registry = self.chain_registry.read().await;
        chain_registry.get_chain_ids()
    }

    /// Registers a new chain with the given `ChainId` and `ChainInfo`.
    pub async fn register_chain(
        &self,
        chain_id: ChainId,
        chain_info: ChainInfo,
    ) -> Result<(), RegistryError> {
        {
            // Update the chain registry with the new chain info.
            let mut chain_registry_guard = self.chain_registry.write().await;
            chain_registry_guard.register_chain(chain_id, chain_info)?;
        }

        {
            // Notify the auction registry about the new chain.
            let mut auction_registry_guard = self.auction_registry.write().await;
            auction_registry_guard.register_chain(chain_id)
        }
    }

    /// Submits new auction information after validation.
    pub async fn submit_auction_info(
        &self,
        auction_info: AuctionInfo,
    ) -> Result<(), RegistryError> {
        // Validate the auction information.
        self.validate_auction_info(&auction_info).await?;

        // Store the auction information.
        let mut auction_registry = self.auction_registry.write().await;
        auction_registry.store_auction_info(auction_info)
    }

    /// Validates the provided auction information.
    pub async fn validate_auction_info(
        &self,
        auction_info: &AuctionInfo,
    ) -> Result<(), RegistryError> {
        let chain_registry = self.chain_registry.read().await;

        // Ensure the chain ID is valid.
        if !chain_registry.validate_chain_id(auction_info.chain_id) {
            return Err(RegistryError::InvalidChainId(auction_info.chain_id));
        }

        // Ensure the seller is registered for the given chain.
        if !chain_registry.is_valid_seller(auction_info.chain_id, &auction_info.seller_address) {
            return Err(RegistryError::SellerNotRegistered(
                auction_info.seller_address.clone(),
            ));
        }

        Ok(())
    }
}
