use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

use tokio::sync::RwLock;

use crate::{
    domain::{AuctionInfo, ChainId},
    services::registry::ChainRegistry,
    utils::errors::RegistryError,
};

#[derive(Default)]
pub struct AuctionRegistry {
    auction_queues: HashMap<ChainId, BinaryHeap<Reverse<AuctionInfo>>>,
}

impl AuctionRegistry {
    /// TODO: Delete ChainRegistry Dependency
    pub async fn new(chain_registry: &Arc<RwLock<ChainRegistry>>) -> Self {
        let mut auction_queues = HashMap::new();
        let chain_ids = chain_registry.read().await.get_chain_ids();

        for chain_id in chain_ids {
            auction_queues.insert(chain_id, BinaryHeap::new());
        }

        AuctionRegistry { auction_queues }
    }

    pub fn pop_next_auction(&mut self, chain_id: ChainId) -> Option<AuctionInfo> {
        self.auction_queues
            .get_mut(&chain_id)
            .and_then(|queue| queue.pop())
            .map(|reverse| reverse.0)
    }

    pub fn store_auction_info(&mut self, auction_info: AuctionInfo) -> Result<(), RegistryError> {
        let queue = self
            .auction_queues
            .get_mut(&auction_info.chain_id)
            .ok_or(RegistryError::InvalidChainId(auction_info.chain_id))?;

        queue.push(Reverse(auction_info));
        Ok(())
    }

    pub fn get_next_auction_info(&self, chain_id: ChainId) -> Option<&AuctionInfo> {
        self.auction_queues
            .get(&chain_id)
            .and_then(|queue| queue.peek())
            .map(|reverse| &reverse.0)
    }

    pub fn register_chain(&mut self, chain_id: ChainId) -> Result<(), RegistryError> {
        if self.auction_queues.contains_key(&chain_id) {
            return Err(RegistryError::ChainAlreadyRegistered(chain_id));
        }

        self.auction_queues.insert(chain_id, BinaryHeap::new());
        Ok(())
    }
}
