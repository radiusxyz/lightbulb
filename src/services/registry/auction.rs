use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

use tokio::sync::RwLock;

use crate::{
    core::domain::{AuctionInfo, ChainId},
    services::registry::ChainRegistry,
    utils::errors::RegistryError,
};

/// `AuctionRegistry` manages queues of auction information for multiple chains.
#[derive(Default)]
pub struct AuctionRegistry {
    /// Stores auction queues for each chain, with auctions ordered by priority.
    auction_queues: HashMap<ChainId, BinaryHeap<Reverse<AuctionInfo>>>,
}

impl AuctionRegistry {
    /// Creates a new `AuctionRegistry` initialized with existing chains from the `ChainRegistry`.
    ///
    /// TODO: Delete ChainRegistry Dependency
    pub async fn new(chain_registry: &Arc<RwLock<ChainRegistry>>) -> Self {
        let mut auction_queues = HashMap::new();

        // Initialize auction queues for all registered chains.
        let chain_ids = chain_registry.read().await.get_chain_ids();
        for chain_id in chain_ids {
            auction_queues.insert(chain_id, BinaryHeap::new());
        }

        AuctionRegistry { auction_queues }
    }

    /// Removes and returns the next auction for the specified chain.
    ///
    /// Returns `None` if there are no auctions in the queue.
    pub fn pop_next_auction(&mut self, chain_id: ChainId) -> Option<AuctionInfo> {
        self.auction_queues
            .get_mut(&chain_id)
            .and_then(|queue| queue.pop())
            .map(|reverse| reverse.0)
    }

    /// Stores a new auction in the queue for the specified chain.
    ///
    /// Returns an error if the chain ID is invalid.
    pub fn store_auction_info(&mut self, auction_info: AuctionInfo) -> Result<(), RegistryError> {
        let queue = self
            .auction_queues
            .get_mut(&auction_info.chain_id)
            .ok_or(RegistryError::InvalidChainId(auction_info.chain_id))?;

        queue.push(Reverse(auction_info));
        Ok(())
    }

    /// Retrieves a reference to the next auction for the specified chain without removing it.
    ///
    /// Returns `None` if there are no auctions in the queue.
    pub fn get_next_auction_info(&self, chain_id: ChainId) -> Option<&AuctionInfo> {
        self.auction_queues
            .get(&chain_id)
            .and_then(|queue| queue.peek())
            .map(|reverse| &reverse.0)
    }

    /// Registers a new chain in the auction registry.
    ///
    /// Returns an error if the chain is already registered.
    pub fn register_chain(&mut self, chain_id: ChainId) -> Result<(), RegistryError> {
        if self.auction_queues.contains_key(&chain_id) {
            return Err(RegistryError::ChainAlreadyRegistered(chain_id));
        }

        self.auction_queues.insert(chain_id, BinaryHeap::new());
        Ok(())
    }
}
