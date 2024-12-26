use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::core::auction::AuctionWorker;
use crate::domain::{AuctionId, AuctionInfo, AuctionResult, Bid, ChainId, Tx};
use crate::services::registry::AuctionRegistry;
use crate::utils::{errors::AuctionError, helpers::current_unix_ms};

/// `AuctionManager` is responsible for scheduling auctions (e.g., starting new auctions, handling bids, requests for information).
/// The actual state of an auction (`AuctionState`) is fully managed inside `AuctionWorker`.
#[derive(Clone)]
pub struct AuctionManager {
    /// Manages registration (scheduling) and validation logic for auctions
    pub auction_registry: Arc<AuctionRegistry>,

    /// Maps a `ChainId` to its dedicated `AuctionWorker`
    pub workers: Arc<DashMap<ChainId, Arc<AuctionWorker>>>,

    /// Maps a `ChainId` to a worker's background task handle
    pub worker_handles: Arc<DashMap<ChainId, JoinHandle<()>>>,

    /// Used by a worker to send an `AuctionResult` when an auction ends
    pub result_sender: mpsc::Sender<AuctionResult>,
}

impl AuctionManager {
    /// Creates a new `AuctionManager`.
    /// If desired, you could spawn a worker for every `ChainId` at creation time,
    /// or you can spawn a worker only when a new `ChainId` is introduced.
    pub fn new(auction_registry: AuctionRegistry, chain_ids: &[ChainId]) -> Self {
        let (result_sender, mut result_receiver) = mpsc::channel(100);

        let manager = AuctionManager {
            auction_registry: Arc::new(auction_registry),
            workers: Arc::new(DashMap::new()),
            worker_handles: Arc::new(DashMap::new()),
            result_sender,
        };

        // Spawn a background task to receive results from workers
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            while let Some(result) = result_receiver.recv().await {
                manager_clone.handle_auction_result(result).await;
            }
        });

        // Create a worker for each initial `ChainId`
        for &chain_id in chain_ids {
            manager.start_worker_for_chain(chain_id);
        }

        manager
    }

    /// Creates and runs an `AuctionWorker` for a specified chain in the background.
    /// Does nothing if a worker for that chain already exists.
    pub fn start_worker_for_chain(&self, chain_id: ChainId) {
        if self.workers.contains_key(&chain_id) {
            return;
        }

        let worker = Arc::new(AuctionWorker::new(chain_id, self.result_sender.clone()));
        self.workers.insert(chain_id, worker.clone());

        let handle = tokio::spawn(async move {
            worker.run().await;
        });

        self.worker_handles.insert(chain_id, handle);
    }

    /// Processes a finished auction (received from the worker).
    /// This is where you could update a database, log events, send notifications, etc.
    async fn handle_auction_result(&self, result: AuctionResult) {
        println!(
            "[Manager] Auction {} on chain {} ended with winner: {}",
            result.auction_id, result.chain_id, result.winner
        );
    }

    // ------------------------------------------------------------------------
    // Methods for scheduling / controlling auctions
    // ------------------------------------------------------------------------

    /// Requests to start the next auction. Retrieves scheduling info from `AuctionRegistry`
    /// and forwards the request to the appropriate worker. Returns the new auction ID or `None`.
    pub async fn start_next_auction(&self, chain_id: ChainId) -> Option<AuctionId> {
        let next_info = self.auction_registry.get_next_auction_info(chain_id)?;

        if current_unix_ms() < next_info.start_time {
            return None;
        }

        let auction_id = &next_info.id;

        if let Some(worker) = self.workers.get(&chain_id) {
            worker
                .start_auction(auction_id.clone(), next_info.clone())
                .await;
            Some(auction_id.clone())
        } else {
            None
        }
    }

    /// For submitting a bid. We assume validation (e.g., signature checks) is already done at the `AuctionRegistry` level.
    pub async fn submit_bid(
        &self,
        chain_id: ChainId,
        auction_id: AuctionId,
        bid: Bid,
    ) -> Result<String, AuctionError> {
        if let Some(worker) = self.workers.get(&chain_id) {
            worker.submit_bid(auction_id, bid).await
        } else {
            Err(AuctionError::NoAuctions)
        }
    }

    /// Requests the current auction's info (the auction ID and `AuctionInfo`).
    pub async fn request_sale_info(
        &self,
        chain_id: ChainId,
    ) -> Result<(AuctionId, AuctionInfo), AuctionError> {
        if let Some(worker) = self.workers.get(&chain_id) {
            worker.request_sale_info().await
        } else {
            Err(AuctionError::NoAuctions)
        }
    }

    /// Requests the latest ToB(Top-of-Block) info for the current auction.
    pub async fn request_latest_tob_info(
        &self,
        chain_id: ChainId,
    ) -> Result<Vec<Tx>, AuctionError> {
        if let Some(worker) = self.workers.get(&chain_id) {
            worker.request_latest_tob_info().await
        } else {
            Err(AuctionError::NoAuctions)
        }
    }

    /// Retrieves the full internal auction state.
    pub async fn get_auction_state(
        &self,
        chain_id: ChainId,
    ) -> Result<crate::domain::AuctionState, AuctionError> {
        if let Some(worker) = self.workers.get(&chain_id) {
            worker.get_auction_state().await
        } else {
            Err(AuctionError::NoAuctions)
        }
    }
}
