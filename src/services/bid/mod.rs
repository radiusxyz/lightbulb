use std::{collections::HashMap, sync::Arc};

use tokio::{
    sync::{Mutex, RwLock},
    task::JoinHandle,
    time::{self, Duration},
};

use crate::{
    core::AuctionManager,
    domain::{AuctionId, Bid, ChainId},
    utils::{
        errors::{AuctionError, BidError},
        types::{ArcMutexHashMap, ArcRwLockHashMap},
    },
};

/// BidService manages bids across multiple chains and periodically flushes them.
#[derive(Clone)]
pub struct BidService {
    /// Stores bids for each chain, protected by locks for thread safety.
    bid_buffer: ArcRwLockHashMap<ChainId, ArcMutexHashMap<AuctionId, Vec<Bid>>>,

    /// Specifies flush intervals for each chain.
    flush_intervals: ArcRwLockHashMap<ChainId, Duration>,

    /// Reference to the AuctionManager to handle bid submissions.
    auction_manager: Arc<AuctionManager>,
}

impl BidService {
    /// Creates a new BidService instance.
    ///
    /// Initializes bid storage, sets flush intervals, and starts background tasks for bid flushing.
    pub async fn new(
        auction_manager: Arc<AuctionManager>,
        chain_flush_intervals: HashMap<ChainId, Duration>,
    ) -> Self {
        let bid_buffer = Arc::new(RwLock::new(HashMap::new()));
        let flush_intervals = Arc::new(RwLock::new(chain_flush_intervals.clone()));

        // Initialize bid buffers for all chains.
        for chain_id in chain_flush_intervals.keys() {
            let mut buffer_lock = bid_buffer.write().await;
            buffer_lock
                .entry(*chain_id)
                .or_insert_with(|| Arc::new(Mutex::new(HashMap::new())));
        }

        // Create the BidService instance.
        BidService {
            bid_buffer,
            flush_intervals,
            auction_manager,
        }
    }

    /// Starts background tasks for bid flushing.
    ///
    /// Returns a vector of `JoinHandle`s representing the spawned tasks.
    pub async fn start_tasks(&self) -> Vec<JoinHandle<()>> {
        let flush_intervals = self.flush_intervals.read().await.clone();
        let bid_buffer = Arc::clone(&self.bid_buffer);
        let auction_manager = Arc::clone(&self.auction_manager);
        let service = self.clone();

        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        // Spawn a task for each chain's flush interval.
        for (chain_id, interval) in flush_intervals {
            let bid_buffer_clone = Arc::clone(&bid_buffer);
            let auction_manager_clone = Arc::clone(&auction_manager);
            let service_clone = service.clone();

            let handle = tokio::spawn(async move {
                loop {
                    time::sleep(interval).await;

                    if let Err(e) = service_clone
                        .flush_bids(chain_id, &bid_buffer_clone, &auction_manager_clone)
                        .await
                    {
                        eprintln!("Error flushing bids for Chain {}: {:?}", chain_id, e);
                    }
                }
            });

            handles.push(handle);
        }

        handles
    }

    /// Stores a bid for a specific chain and auction.
    ///
    /// Adds the bid to the appropriate buffer for future processing.
    pub async fn store_bid(&self, bid: Bid) -> Result<(), AuctionError> {
        let chain_id = bid.chain_id;
        let auction_id = bid.auction_id.clone();

        {
            // Acquire a read lock for the bid buffer.
            let buffer_guard = self.bid_buffer.read().await;

            // Check if the chain exists in the buffer.
            if let Some(chain_buffer_mutex) = buffer_guard.get(&chain_id) {
                let mut chain_buffer = chain_buffer_mutex.lock().await;

                // Add the bid to the auction's buffer.
                let auction_bids = chain_buffer.entry(auction_id).or_insert_with(Vec::new);
                auction_bids.push(bid);
            } else {
                // Return an error if the specified chain does not exist.
                return Err(AuctionError::InvalidChainId(chain_id));
            }
        }

        Ok(())
    }

    /// Flushes bids for a specific chain by sending them to the AuctionManager.
    ///
    /// Collects bids for the ongoing auction and submits them in a batch.
    async fn flush_bids(
        &self,
        chain_id: ChainId,
        bid_buffer: &ArcRwLockHashMap<ChainId, ArcMutexHashMap<AuctionId, Vec<Bid>>>,
        auction_manager: &Arc<AuctionManager>,
    ) -> Result<(), BidError> {
        // Retrieve the ongoing auction ID for the chain.
        let auction_id = match auction_manager.get_ongoing_auction_id(chain_id).await {
            Some(auction_id) => auction_id,
            None => return Ok(()),
        };

        // Collect and remove bids associated with the ongoing auction.
        let bids_to_flush = {
            let buffer_guard = bid_buffer.read().await;
            let chain_buffer_mutex = match buffer_guard.get(&chain_id) {
                Some(mutex) => mutex,
                None => return Err(BidError::InvalidChainId(chain_id)),
            };
            let mut chain_buffer = chain_buffer_mutex.lock().await;
            match chain_buffer.remove(&auction_id) {
                Some(bids) => bids.clone(),
                None => return Ok(()),
            }
        };

        // Submit the collected bids to the AuctionManager.
        auction_manager
            .submit_bid_batch(chain_id, auction_id.clone(), bids_to_flush)
            .await
            .map_err(|e| e.into())
    }

    /// Adds a new chain to the BidService with a specified flush interval.
    pub async fn add_chain(&self, chain_id: ChainId, flush_interval_ms: u64) {
        {
            // Update the flush interval for the new chain.
            let mut intervals_guard = self.flush_intervals.write().await;
            intervals_guard.insert(chain_id, Duration::from_millis(flush_interval_ms));
        }

        {
            // Initialize the bid buffer for the new chain.
            let mut buffer_guard = self.bid_buffer.write().await;
            buffer_guard
                .entry(chain_id)
                .or_insert_with(|| Arc::new(Mutex::new(HashMap::new())));
        }

        // Start a flush task for the new chain.
        let bid_buffer = Arc::clone(&self.bid_buffer);
        let auction_manager = Arc::clone(&self.auction_manager);
        let service_clone = self.clone();
        let interval = Duration::from_millis(flush_interval_ms);

        tokio::spawn(async move {
            loop {
                time::sleep(interval).await;

                if let Err(e) = service_clone
                    .flush_bids(chain_id, &bid_buffer, &auction_manager)
                    .await
                {
                    eprintln!("Error flushing bids for Chain {}: {:?}", chain_id, e);
                }
            }
        });
    }
}
