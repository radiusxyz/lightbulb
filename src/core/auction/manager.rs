use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::core::{
    auction::AuctionWorkerClient,
    domain::{AuctionId, AuctionInfo, Bid, ChainId},
    errors::AuctionError,
    registry::{AuctionRegistry, RegistryService},
    utils::types::{ArcRwLock, ArcRwLockHashMap},
};

/// `AuctionManager` is responsible for scheduling auctions (e.g., starting new auctions, handling bids, requests for information).
/// The actual state of an auction (`AuctionState`) is fully managed inside `AuctionWorker`.
#[derive(Clone)]
pub struct AuctionManager<W: AuctionWorkerClient> {
    /// Manages registration (scheduling) and validation logic for auctions
    pub auction_registry: ArcRwLock<AuctionRegistry>,

    /// Maps a `ChainId` to the `AuctionInfo` of the ongoing auction (if any)
    pub ongoing_auctions: ArcRwLockHashMap<ChainId, AuctionInfo>,

    /// Reference to the AuctionWorkerClient to handle bid submissions
    pub auction_worker_client: W,
}

impl<W: AuctionWorkerClient> AuctionManager<W> {
    /// Creates a new `AuctionManager`.
    /// If desired, you could spawn a worker for every `ChainId` at creation time,
    /// or you can spawn a worker only when a new `ChainId` is introduced.
    /// TODO: Delete RegistryService dependency
    pub async fn new(registry_service: &RegistryService, auction_worker_client: W) -> Self {
        let auction_registry = registry_service.get_auction_registry();

        AuctionManager {
            auction_registry,
            ongoing_auctions: Arc::new(RwLock::new(HashMap::new())),
            auction_worker_client,
        }
    }

    pub async fn get_ongoing_auction_id(&self, chain_id: ChainId) -> Option<AuctionId> {
        let ongoing_auctions = self.ongoing_auctions.read().await;
        let auction_info = ongoing_auctions.get(&chain_id)?;
        Some(auction_info.auction_id.clone())
    }

    pub async fn add_auction(&mut self, auction_info: AuctionInfo) -> Result<(), AuctionError> {
        let mut ongoing_auctions = self.ongoing_auctions.write().await;
        ongoing_auctions.insert(auction_info.chain_id, auction_info.clone());

        let response = self.auction_worker_client.add_auction(auction_info).await?;

        match response {
            true => Ok(()),
            false => Err(AuctionError::AuctionNotAdded),
        }
    }

    pub async fn submit_bids(
        &mut self,
        chain_id: ChainId,
        auction_id: AuctionId,
        bids: Vec<Bid>,
    ) -> Result<bool, AuctionError> {
        self.auction_worker_client
            .submit_bids(chain_id, auction_id, bids)
            .await
    }
}
