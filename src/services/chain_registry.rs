use crate::domain::ChainId;
use crate::domain::SLAConfig;
use crate::domain::SLA;
use std::collections::HashMap;

#[derive(Default)]
pub struct ChainRegistry {
    /// A mapping of `ChainId` to the maximum allowed gas limit.
    pub max_gas_limit: HashMap<ChainId, u64>,
    /// A mapping of `ChainId` to a list of valid (registered) seller addresses.
    pub registered_sellers: HashMap<ChainId, Vec<String>>,
    /// A mapping of `ChainId` to the current block height.
    pub current_block_height: HashMap<ChainId, u64>,
    /// A mapping of `ChainId` to SLA-related configuration parameters.
    pub sla_config: HashMap<ChainId, SLAConfig>,
}

impl ChainRegistry {
    /// Creates a new `ChainRegistry` with mock data for demonstration.
    pub fn new() -> Self {
        let mut max_gas_limit = HashMap::new();
        // For example, chain 1 has a max gas limit of 2,000,000
        max_gas_limit.insert(1, 2_000_000u64);

        let mut registered_sellers = HashMap::new();
        registered_sellers.insert(1, vec!["0xSellerAddress".to_string()]);

        let mut current_block_height = HashMap::new();
        current_block_height.insert(1, 10_000u64);

        let mut sla_config = HashMap::new();
        sla_config.insert(
            1,
            SLAConfig {
                min_end_time_offset_ms: 500,
            },
        );

        ChainRegistry {
            max_gas_limit,
            registered_sellers,
            current_block_height,
            sla_config,
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

    /// Validates the SLA for the specified chain.
    pub fn is_valid_sla(&self, chain_id: ChainId, sla: &SLA, recent_sla: &SLA) -> bool {
        self.is_valid_seller(chain_id, &sla.seller_addr)
            && self.validate_future_block_height(chain_id, recent_sla)
            && self.is_below_chain_gas_limit(chain_id, sla.blockspace_size)
            && self.is_future_start_time(sla.start_time)
            && self.validate_end_time_offset(chain_id, sla.start_time, sla.end_time)
    }

    /// Retrieves the maximum gas limit for the specified chain.
    pub fn get_max_gas_limit(&self, chain_id: ChainId) -> Option<u64> {
        self.max_gas_limit.get(&chain_id).copied()
    }

    /// Retrieves the current block height for the specified chain, if available.
    pub fn get_current_block_height(&self, chain_id: ChainId) -> Option<u64> {
        self.current_block_height.get(&chain_id).copied()
    }

    /// Updates the current block height for the specified chain.
    pub fn update_current_block_height(&mut self, chain_id: ChainId, height: u64) {
        self.current_block_height.insert(chain_id, height);
    }

    /// Fetches the SLA configuration for the specified chain.
    pub fn get_sla_config(&self, chain_id: ChainId) -> Option<&SLAConfig> {
        self.sla_config.get(&chain_id)
    }

    /// Validates that the specified block height is greater than the most recent auction's block height.
    pub fn validate_future_block_height(&self, chain_id: ChainId, recent_sla: &SLA) -> bool {
        recent_sla.block_height < self.get_current_block_height(chain_id).unwrap_or(0)
    }

    /// Checks whether the given blockspace size is below or equal to the max gas limit of the specified chain.
    pub fn is_below_chain_gas_limit(&self, chain_id: ChainId, blockspace_size: u64) -> bool {
        if let Some(max_gas) = self.get_max_gas_limit(chain_id) {
            blockspace_size <= max_gas
        } else {
            false
        }
    }

    /// Checks if the start_time is strictly greater than the provided current_time (i.e., it is in the future).
    pub fn is_future_start_time(&self, start_time: u64) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        start_time > current_time
    }

    /// Checks if the end_time meets the minimal offset requirement (e.g., 500ms) after the start_time for the given chain.
    pub fn validate_end_time_offset(
        &self,
        chain_id: ChainId,
        start_time: u64,
        end_time: u64,
    ) -> bool {
        if let Some(cfg) = self.get_sla_config(chain_id) {
            end_time >= start_time + cfg.min_end_time_offset_ms
        } else {
            false
        }
    }
}
