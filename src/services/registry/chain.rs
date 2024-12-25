use crate::domain::{ChainId, ChainInfo};
use std::collections::HashMap;

#[derive(Default)]
pub struct ChainRegistry {
    chain_info: HashMap<ChainId, ChainInfo>,
}

impl ChainRegistry {
    /// Creates a new `ChainRegistry` with mock data for demonstration.
    pub fn new() -> Self {
        let mut chain_info = HashMap::new();

        chain_info.insert(
            1,
            ChainInfo {
                gas_limit: 2_000_000,
                registered_sellers: vec!["0xSellerAddress".to_string()],
            },
        );

        chain_info.insert(
            2,
            ChainInfo {
                gas_limit: 1_000_000,
                registered_sellers: vec!["0xSellerAddress2".to_string()],
            },
        );

        ChainRegistry { chain_info }
    }

    /// Checks whether the given chain ID is recognized in our registry.
    pub fn validate_chain_id(&self, chain_id: ChainId) -> bool {
        self.chain_info.contains_key(&chain_id)
    }

    /// Checks if the given seller address is registered for the specified chain.
    pub fn is_valid_seller(&self, chain_id: ChainId, seller: &str) -> bool {
        if let Some(info) = self.chain_info.get(&chain_id) {
            info.registered_sellers.contains(&seller.to_string())
        } else {
            false
        }
    }

    /// Retrieves the maximum gas limit for the specified chain.
    pub fn get_max_gas_limit(&self, chain_id: ChainId) -> Option<u64> {
        self.chain_info.get(&chain_id).map(|info| info.gas_limit)
    }
}
