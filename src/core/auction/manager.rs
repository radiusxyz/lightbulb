use std::{collections::HashMap, sync::Arc};

use tokio::{
    sync::{mpsc, RwLock},
    task::JoinHandle,
};

use crate::{
    core::auction::AuctionWorker,
    domain::{AuctionId, AuctionInfo, Bid, ChainId, Tx, WorkerMessage, WorkerMessageType},
    services::registry::{AuctionRegistry, RegistryService},
    utils::{
        errors::AuctionError,
        helpers::current_unix_ms,
        types::{ArcRwLock, ArcRwLockHashMap},
    },
};

/// `AuctionManager` is responsible for scheduling auctions (e.g., starting new auctions, handling bids, requests for information).
/// The actual state of an auction (`AuctionState`) is fully managed inside `AuctionWorker`.
#[derive(Clone)]
pub struct AuctionManager {
    /// Manages registration (scheduling) and validation logic for auctions
    pub auction_registry: ArcRwLock<AuctionRegistry>,

    /// Maps a `ChainId` to the `AuctionInfo` of the ongoing auction (if any)
    pub ongoing_auctions: ArcRwLockHashMap<ChainId, AuctionInfo>,

    /// Maps a `ChainId` to its dedicated `AuctionWorker`
    pub workers: ArcRwLockHashMap<ChainId, Arc<AuctionWorker>>,

    /// Maps a `ChainId` to a worker's background task handle
    pub worker_handles: ArcRwLockHashMap<ChainId, JoinHandle<()>>,

    /// Used by a worker to send a `WorkerMessage` when an auction event occurs
    pub message_sender: mpsc::Sender<WorkerMessage>,
}

impl AuctionManager {
    /// Creates a new `AuctionManager`.
    /// If desired, you could spawn a worker for every `ChainId` at creation time,
    /// or you can spawn a worker only when a new `ChainId` is introduced.
    /// TODO: Delete RegistryService dependency
    pub async fn new(registry_service: &RegistryService) -> Self {
        let (message_sender, mut message_receiver) = mpsc::channel(100);

        let chain_ids = registry_service.get_chain_ids().await;
        let auction_registry = registry_service.get_auction_registry();

        let manager = AuctionManager {
            auction_registry,
            ongoing_auctions: Arc::new(RwLock::new(HashMap::new())),
            workers: Arc::new(RwLock::new(HashMap::new())),
            worker_handles: Arc::new(RwLock::new(HashMap::new())),
            message_sender,
        };

        // Clone the manager for the background task
        // This can be implemented with concurrency with tokio::spawn and a semaphore
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            while let Some(result) = message_receiver.recv().await {
                manager_clone.handle_worker_message(result).await;
            }
        });

        // Create a worker for each initial `ChainId`
        for &chain_id in chain_ids.iter() {
            manager.start_worker_for_chain(chain_id).await;
        }

        manager
    }

    /// Creates and runs an `AuctionWorker` for a specified chain in the background.
    /// Does nothing if a worker for that chain already exists.
    pub async fn start_worker_for_chain(&self, chain_id: ChainId) {
        // Acquire a read lock to check if the worker already exists
        {
            let workers_guard = self.workers.read().await;
            if workers_guard.contains_key(&chain_id) {
                return;
            }
        }

        // Acquire a write lock to insert the new worker
        let mut workers_guard = self.workers.write().await;
        if workers_guard.contains_key(&chain_id) {
            // Double-check to prevent race conditions
            return;
        }

        let worker = Arc::new(AuctionWorker::new(chain_id, self.message_sender.clone()));
        workers_guard.insert(chain_id, worker.clone());

        drop(workers_guard); // Release the write lock before spawning the task

        let handle = tokio::spawn(async move {
            worker.run().await;
        });

        // Insert the worker handle
        let mut handles_guard = self.worker_handles.write().await;
        handles_guard.insert(chain_id, handle);
    }

    /// Processes a finished auction (received from the worker).
    /// This is where you could update a database, log events, send notifications, etc.
    async fn handle_worker_message(&self, message: WorkerMessage) {
        println!("[Manager] Received worker message: {:?}", message);
        match message.message_type {
            WorkerMessageType::AuctionEnded => {
                let chain_id = message.chain_id;
                let auction_id = message.auction_id.clone();

                // Acquire a read lock to check the ongoing auction
                let ongoing_auction_opt = {
                    let ongoing_guard = self.ongoing_auctions.read().await;
                    ongoing_guard.get(&chain_id).cloned()
                };

                if let Some(ongoing_auction_info) = ongoing_auction_opt {
                    if ongoing_auction_info.id == auction_id {
                        // Acquire a write lock to remove the auction
                        let mut ongoing_guard = self.ongoing_auctions.write().await;
                        ongoing_guard.remove(&chain_id);
                        println!(
                            "[Manager] Auction with ID {} on Chain {} has ended and was removed.",
                            auction_id, chain_id
                        );
                    }
                }
            }
            WorkerMessageType::AuctionProcessing => {
                // Handle other message types if necessary
            }
            WorkerMessageType::Idle => {
                // Handle idle state if necessary
            }
        }
    }

    // ------------------------------------------------------------------------
    // Methods for scheduling / controlling auctions
    // ------------------------------------------------------------------------

    /// Requests to start the next auction. Retrieves scheduling info from `AuctionRegistry`
    /// and forwards the request to the appropriate worker. Returns the new auction ID or `None`.
    pub async fn start_next_auction(&self, chain_id: ChainId) -> Option<AuctionId> {
        // Step 1: Retrieve the next auction info
        let next_info = {
            let registry_guard = self.auction_registry.read().await;
            registry_guard.get_next_auction_info(chain_id)?.clone()
        };

        // Step 2: Check if the auction can start
        if current_unix_ms() < next_info.start_time {
            return None;
        }

        // Step 3: Remove the next auction from the registry
        {
            let mut registry_guard = self.auction_registry.write().await;
            registry_guard.pop_next_auction(chain_id);
        }

        let auction_id = next_info.id.clone();

        // Step 4: Retrieve the worker for the chain
        let worker_opt = {
            let workers_guard = self.workers.read().await;
            workers_guard.get(&chain_id).cloned()
        };

        // Step 5: Start the auction
        if let Some(worker) = worker_opt {
            if worker
                .start_auction(auction_id.clone(), next_info.clone())
                .await
                .is_err()
            {
                println!(
                    "[Manager] Failed to start auction {} on Chain {}.",
                    auction_id, chain_id
                );
            }
            {
                // Insert into ongoing auctions
                let mut ongoing_guard = self.ongoing_auctions.write().await;
                ongoing_guard.insert(chain_id, next_info.clone());
                println!(
                    "[Manager] Auction {} started on Chain {}.",
                    auction_id, chain_id
                );
                Some(auction_id)
            }
        } else {
            println!(
                "[Manager] No worker found for Chain {}. Cannot start auction.",
                chain_id
            );
            None
        }
    }

    // ------------------------------------------------------------------------
    // Getters
    // ------------------------------------------------------------------------

    /// Retrieves the ongoing auction ID for a given chain.
    pub async fn get_ongoing_auction_id(&self, chain_id: ChainId) -> Option<AuctionId> {
        let ongoing_guard = self.ongoing_auctions.read().await;
        ongoing_guard
            .get(&chain_id)
            .map(|auction_info| auction_info.id.clone())
    }

    pub async fn get_all_ongoing_auction_ids(&self) -> HashMap<ChainId, AuctionId> {
        let ongoing_guard = self.ongoing_auctions.read().await;
        ongoing_guard
            .iter()
            .map(|(chain_id, auction_info)| (*chain_id, auction_info.id.clone()))
            .collect()
    }

    // ------------------------------------------------------------------------
    // Methods for communicating with AuctionWorkers
    // ------------------------------------------------------------------------

    /// For submitting a bid. We assume validation (e.g., signature checks) is already done at the `AuctionRegistry` level.
    pub async fn submit_bid(
        &self,
        chain_id: ChainId,
        auction_id: AuctionId,
        bid: Bid,
    ) -> Result<String, AuctionError> {
        let worker_opt = {
            let workers_guard = self.workers.read().await;
            workers_guard.get(&chain_id).cloned()
        };

        if let Some(worker) = worker_opt {
            worker.submit_bid(auction_id, bid).await
        } else {
            Err(AuctionError::NoAuctions)
        }
    }

    pub async fn submit_bid_batch(
        &self,
        chain_id: ChainId,
        auction_id: AuctionId,
        bids: Vec<Bid>,
    ) -> Result<(), AuctionError> {
        let worker_opt = {
            let workers_guard = self.workers.read().await;
            workers_guard.get(&chain_id).cloned()
        };

        if let Some(worker) = worker_opt {
            worker.submit_bid_batch(auction_id, bids).await
        } else {
            Err(AuctionError::NoAuctions)
        }
    }

    /// Requests the latest ToB (Top-of-Block) info for the current auction.
    pub async fn request_latest_tob(&self, chain_id: ChainId) -> Result<Vec<Tx>, AuctionError> {
        let worker_opt = {
            let workers_guard = self.workers.read().await;
            workers_guard.get(&chain_id).cloned()
        };

        if let Some(worker) = worker_opt {
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
        let worker_opt = {
            let workers_guard = self.workers.read().await;
            workers_guard.get(&chain_id).cloned()
        };

        if let Some(worker) = worker_opt {
            worker.get_auction_state().await
        } else {
            Err(AuctionError::NoAuctions)
        }
    }
}
