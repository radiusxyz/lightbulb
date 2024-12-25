use std::collections::HashMap;

use crate::domain::{AuctionInfo, ChainId};
use crate::services::registry::ChainRegistry;
use crate::utils::{errors::RegistryError, helpers::verify_signature};

pub struct AuctionRegistry {
    pub chain_registry: ChainRegistry,
    pub auction_storage: HashMap<ChainId, Vec<AuctionInfo>>,
}

impl AuctionRegistry {
    /// Creates a new `AuctionRegistry` instance with the given `ChainRegistry`.
    pub fn new(chain_registry: ChainRegistry) -> Self {
        AuctionRegistry {
            chain_registry,
            auction_storage: HashMap::new(),
        }
    }

    /// Retrieves the next auction_info for the given chain ID.
    pub fn get_next_auction_info(&self, chain_id: ChainId) -> Option<&AuctionInfo> {
        self.auction_storage
            .get(&chain_id)
            .and_then(|auction_list| auction_list.first())
    }

    /// Submits an auction_info to the registry, validating it before storage.
    ///
    /// Returns `Ok(())` if the auction_info is valid and successfully stored,
    /// or an `RegistryError` if validation fails.
    pub fn submit_sale_info(
        &mut self,
        chain_id: ChainId,
        auction_info: AuctionInfo,
    ) -> Result<(), RegistryError> {
        // Perform validations using our helper function.
        self.validate_auction_info(chain_id, &auction_info)?;

        // Insert into storage only if validations succeed.
        self.insert_auction_info(chain_id, auction_info);
        Ok(())
    }

    /// Inserts an auction_info into the appropriate chain's list and sorts the list by `start_time`.
    fn insert_auction_info(&mut self, chain_id: ChainId, auction_info: AuctionInfo) {
        // Insert the auction info
        if let Some(auction_list) = self.auction_storage.get_mut(&chain_id) {
            auction_list.push(auction_info);

            // Sort by `start_time`
            auction_list.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        }
    }

    /// Validates the auction_info before storing it.
    ///
    /// Returns `Ok(())` if all validations pass, otherwise
    /// returns the first encountered `RegistryError`.
    fn validate_auction_info(
        &self,
        chain_id: ChainId,
        auction_info: &AuctionInfo,
    ) -> Result<(), RegistryError> {
        // Validate chain ID
        self.validate_chain_id(chain_id)?;

        // Validate seller
        self.validate_seller(chain_id, &auction_info.seller_addr)?;

        // Validate seller signature
        self.validate_seller_signature(auction_info)?;

        // Validate gas limit
        self.validate_gas_limit(chain_id, auction_info.blockspace_size)?;

        // Validate timings
        self.validate_timings(auction_info.start_time, auction_info.end_time)?;

        Ok(())
    }

    // ------------------------ Validation Helpers ------------------------

    /// Validates the chain ID.
    fn validate_chain_id(&self, chain_id: ChainId) -> Result<(), RegistryError> {
        if !self.chain_registry.validate_chain_id(chain_id)
            || !self.auction_storage.contains_key(&chain_id)
        {
            Err(RegistryError::InvalidChainId(chain_id))
        } else {
            Ok(())
        }
    }

    /// Validates the seller's registration on the chain.
    fn validate_seller(&self, chain_id: ChainId, seller_addr: &str) -> Result<(), RegistryError> {
        if !self.chain_registry.is_valid_seller(chain_id, seller_addr) {
            Err(RegistryError::SellerNotRegistered)
        } else {
            Ok(())
        }
    }

    /// Validates the seller's signature (mock).
    fn validate_seller_signature(&self, auction_info: &AuctionInfo) -> Result<(), RegistryError> {
        if !verify_signature(&auction_info.seller_addr, &auction_info.seller_signature) {
            Err(RegistryError::InvalidSellerSignature)
        } else {
            Ok(())
        }
    }

    /// Validates the auction's gas limit.
    fn validate_gas_limit(
        &self,
        chain_id: ChainId,
        blockspace_size: u64,
    ) -> Result<(), RegistryError> {
        match self.chain_registry.get_max_gas_limit(chain_id) {
            Some(max_gas) if blockspace_size <= max_gas => Ok(()),
            _ => Err(RegistryError::InvalidGasLimit),
        }
    }

    fn validate_timings(&self, start_time: u64, end_time: u64) -> Result<(), RegistryError> {
        if end_time < start_time + 500 {
            return Err(RegistryError::InvalidAuctionTime);
        }
        Ok(())
    }
}
