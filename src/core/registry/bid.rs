use std::{collections::HashMap, mem::take};

use crate::core::{
    domain::{Bid, ChainId},
    errors::RegistryError,
};

#[derive(Default)]
pub struct BidRegistry {
    bids: HashMap<ChainId, Vec<Bid>>,
    processing_bids: HashMap<ChainId, Vec<Bid>>,
}

impl BidRegistry {
    pub fn new() -> Self {
        BidRegistry {
            bids: HashMap::new(),
            processing_bids: HashMap::new(),
        }
    }

    pub fn add_chain(&mut self, chain_id: ChainId) -> Result<(), RegistryError> {
        if self.bids.contains_key(&chain_id) {
            return Err(RegistryError::ChainAlreadyRegistered(chain_id));
        }
        self.bids.insert(chain_id, Vec::new());
        self.processing_bids.insert(chain_id, Vec::new());
        Ok(())
    }

    pub fn add_bid(&mut self, chain_id: ChainId, bid: Bid) -> Result<(), RegistryError> {
        match self.bids.get_mut(&chain_id) {
            Some(bids) => {
                bids.push(bid);
                Ok(())
            }
            None => Err(RegistryError::InvalidChainId(chain_id)),
        }
    }

    pub fn get_bids(&self, chain_id: ChainId) -> Option<Vec<Bid>> {
        self.bids.get(&chain_id).cloned()
    }

    pub fn take_bids_for_processing(
        &mut self,
        chain_id: ChainId,
    ) -> Result<Vec<Bid>, RegistryError> {
        match self.bids.get_mut(&chain_id) {
            Some(bids) => {
                let taken_bids = take(bids);
                self.processing_bids
                    .entry(chain_id)
                    .or_default()
                    .extend(taken_bids.clone());
                Ok(taken_bids)
            }
            None => Err(RegistryError::InvalidChainId(chain_id)),
        }
    }

    pub fn clear_processed_bids(&mut self, chain_id: ChainId) {
        if let Some(bids) = self.processing_bids.get_mut(&chain_id) {
            bids.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bid_registry() {
        let mut bid_registry = BidRegistry::new();
        bid_registry.add_chain(1).unwrap();
        let bid = Bid {
            bidder_address: "0xAlice".to_string(),
            bid_amount: 100,
            bidder_signature: "AliceSignature".to_string(),
            tx_list: vec![],
        };
        bid_registry.add_bid(1, bid.clone()).unwrap();

        let bids = bid_registry.get_bids(1);
        assert_eq!(bids, Some(vec![bid.clone()]));
    }
}
