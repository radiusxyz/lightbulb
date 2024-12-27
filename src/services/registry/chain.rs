use std::collections::HashMap;

use crate::{
    domain::{ChainId, ChainInfo},
    utils::errors::RegistryError,
};

/// `ChainRegistry` manages information about registered blockchain networks.
#[derive(Default)]
pub struct ChainRegistry {
    /// Stores information about each registered chain, mapped by their `ChainId`.
    chain_info_map: HashMap<ChainId, ChainInfo>,
}

impl ChainRegistry {
    /// Creates a new `ChainRegistry` with an initial set of chain information.
    pub fn new(chain_info_map: HashMap<ChainId, ChainInfo>) -> Self {
        ChainRegistry { chain_info_map }
    }

    /// Registers a new chain with the specified `ChainId` and `ChainInfo`.
    ///
    /// Returns an error if the chain is already registered.
    pub fn register_chain(
        &mut self,
        chain_id: ChainId,
        chain_info_map: ChainInfo,
    ) -> Result<(), RegistryError> {
        if self.chain_info_map.contains_key(&chain_id) {
            return Err(RegistryError::ChainAlreadyRegistered(chain_id));
        }

        self.chain_info_map.insert(chain_id, chain_info_map);
        Ok(())
    }

    /// Returns a list of all registered chain IDs.
    pub fn get_chain_ids(&self) -> Vec<ChainId> {
        self.chain_info_map.keys().cloned().collect()
    }

    /// Validates whether the given `ChainId` is registered.
    pub fn validate_chain_id(&self, chain_id: ChainId) -> bool {
        self.chain_info_map.contains_key(&chain_id)
    }

    /// Checks if the specified seller is registered for the given chain.
    ///
    /// Returns `true` if the seller is recognized; otherwise, `false`.
    pub fn is_valid_seller(&self, chain_id: ChainId, seller: &str) -> bool {
        if let Some(info) = self.chain_info_map.get(&chain_id) {
            info.registered_sellers.contains(&seller.to_string())
        } else {
            false
        }
    }

    /// Retrieves the maximum gas limit for the specified chain, if available.
    pub fn get_max_gas_limit(&self, chain_id: ChainId) -> Option<u64> {
        self.chain_info_map
            .get(&chain_id)
            .map(|info| info.gas_limit)
    }
}
