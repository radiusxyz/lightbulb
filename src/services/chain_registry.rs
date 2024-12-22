use crate::domain::ChainId;
use std::collections::HashMap;

#[derive(Default)]
pub struct ChainRegistry {
    /// A mapping of `ChainId` to the maximum allowed gas limit.
    pub max_gas_limit: HashMap<ChainId, u64>,
    /// A mapping of `ChainId` to a list of valid (registered) seller addresses.
    pub registered_sellers: HashMap<ChainId, Vec<String>>,
}

impl ChainRegistry {
    /// Creates a new `ChainRegistry` with mock data for demonstration.
    pub fn new() -> Self {
        let mut max_gas_limit = HashMap::new();
        // For example, chain 1 has a max gas limit of 2,000,000
        max_gas_limit.insert(1, 2_000_000u64);

        let mut registered_sellers = HashMap::new();
        registered_sellers.insert(1, vec!["0xSellerAddress".to_string()]);

        ChainRegistry {
            max_gas_limit,
            registered_sellers,
        }
    }

    /// Checks whether the given chain ID is recognized in our registry.
    pub fn validate_chain_id(&self, chain_id: ChainId) -> bool {
        self.max_gas_limit.contains_key(&chain_id)
    }

    /// Checks if the given seller address is registered for the specified chain.
    pub fn is_valid_seller(&self, chain_id: ChainId, seller: &str) -> bool {
        self.registered_sellers
            .get(&chain_id)
            .map_or(false, |sellers| sellers.contains(&seller.to_string()))
    }

    /// Retrieves the maximum gas limit for the specified chain.
    pub fn get_max_gas_limit(&self, chain_id: ChainId) -> Option<u64> {
        self.max_gas_limit.get(&chain_id).copied()
    }
}
