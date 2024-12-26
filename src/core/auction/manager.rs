use std::sync::Arc;

use dashmap::DashMap;
use tokio::{
    sync::{mpsc, RwLock},
    task::JoinHandle,
};

use crate::core::auction::AuctionWorker;
use crate::domain::{AuctionId, AuctionInfo, Bid, ChainId, Tx, WorkerMessage, WorkerMessageType};
use crate::services::registry::{AuctionRegistry, RegistryService};
use crate::utils::{errors::AuctionError, helpers::current_unix_ms};

/// `AuctionManager` is responsible for scheduling auctions (e.g., starting new auctions, handling bids, requests for information).
/// The actual state of an auction (`AuctionState`) is fully managed inside `AuctionWorker`.
#[derive(Clone)]
pub struct AuctionManager {
    /// Manages registration (scheduling) and validation logic for auctions
    pub auction_registry: Arc<RwLock<AuctionRegistry>>,

    /// Maps a `ChainId` to the ID of the ongoing auction (if any)
    pub ongoing_auctions: Arc<DashMap<ChainId, AuctionInfo>>,

    /// Maps a `ChainId` to its dedicated `AuctionWorker`
    pub workers: Arc<DashMap<ChainId, Arc<AuctionWorker>>>,

    /// Maps a `ChainId` to a worker's background task handle
    pub worker_handles: Arc<DashMap<ChainId, JoinHandle<()>>>,

    /// Used by a worker to send an `AuctionResult` when an auction ends
    pub message_sender: mpsc::Sender<WorkerMessage>,
}

impl AuctionManager {
    /// Creates a new `AuctionManager`.
    /// If desired, you could spawn a worker for every `ChainId` at creation time,
    /// or you can spawn a worker only when a new `ChainId` is introduced.
    pub async fn new(registry_service: &RegistryService) -> Self {
        let (message_sender, mut message_receiver) = mpsc::channel(100);

        let chain_ids = registry_service.get_chain_ids().await;
        let auction_registry = registry_service.get_auction_registry();

        let manager = AuctionManager {
            auction_registry,
            ongoing_auctions: Arc::new(DashMap::new()),
            workers: Arc::new(DashMap::new()),
            worker_handles: Arc::new(DashMap::new()),
            message_sender,
        };

        // Spawn a background task to receive results from workers
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            while let Some(result) = message_receiver.recv().await {
                manager_clone.handle_worker_message(result).await;
            }
        });

        // Create a worker for each initial `ChainId`
        for &chain_id in chain_ids.iter() {
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

        let worker = Arc::new(AuctionWorker::new(chain_id, self.message_sender.clone()));
        self.workers.insert(chain_id, worker.clone());

        let handle = tokio::spawn(async move {
            worker.run().await;
        });

        self.worker_handles.insert(chain_id, handle);
    }

    /// Processes a finished auction (received from the worker).
    /// This is where you could update a database, log events, send notifications, etc.
    async fn handle_worker_message(&self, message: WorkerMessage) {
        match message.message_type {
            WorkerMessageType::AuctionEnded => {
                let chain_id = message.chain_id;
                if let Some(ongoing_auction_info) = self.ongoing_auctions.get(&chain_id) {
                    if ongoing_auction_info.id == message.auction_id {
                        self.ongoing_auctions.remove(&chain_id);
                    }
                }
            }
            WorkerMessageType::AuctionProcessing => {
                // Do nothing
            }
            WorkerMessageType::Idle => {
                // Do nothing
            }
        }
    }

    // ------------------------------------------------------------------------
    // Methods for scheduling / controlling auctions
    // ------------------------------------------------------------------------

    /// Requests to start the next auction. Retrieves scheduling info from `AuctionRegistry`
    /// and forwards the request to the appropriate worker. Returns the new auction ID or `None`.
    pub async fn start_next_auction(&self, chain_id: ChainId) -> Option<AuctionId> {
        let guard = self.auction_registry.read().await;
        let next_info = guard.get_next_auction_info(chain_id)?;

        if current_unix_ms() < next_info.start_time {
            return None;
        }

        {
            let mut guard = self.auction_registry.write().await;
            guard.pop_next_auction(chain_id);
        }

        let auction_id = &next_info.id;

        if let Some(worker) = self.workers.get(&chain_id) {
            worker
                .start_auction(auction_id.clone(), next_info.clone())
                .await;
            self.ongoing_auctions.insert(chain_id, next_info.clone());
            Some(auction_id.clone())
        } else {
            None
        }
    }

    /// Retrieves the ongoing auction ID for a given chain.
    pub async fn get_ongoing_auction_id(&self, chain_id: ChainId) -> Option<AuctionId> {
        self.ongoing_auctions
            .get(&chain_id)
            .map(|auction_info| auction_info.id.clone())
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

    /// Requests the latest ToB(Top-of-Block) info for the current auction.
    pub async fn request_latest_tob(&self, chain_id: ChainId) -> Result<Vec<Tx>, AuctionError> {
        if let Some(worker) = self.workers.get(&chain_id) {
            worker.get_latest_tob().await
        } else {
            Err(AuctionError::NoAuctions)
        }
    }

    /// Retrieves the full internal auction state.
    pub async fn request_auction_state(
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
