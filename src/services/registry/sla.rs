use std::collections::HashMap;

use crate::domain::{ChainId, SLA};
use crate::services::registry::ChainRegistry;
use crate::utils::{errors::AuctionError, helpers::verify_signature};

pub struct SlaRegistry {
    pub chain_registry: ChainRegistry,
    pub sla_storage: HashMap<ChainId, Vec<SLA>>,
}

impl SlaRegistry {
    /// Creates a new `SlaRegistry` instance with the given `ChainRegistry`.
    pub fn new(chain_registry: ChainRegistry) -> Self {
        SlaRegistry {
            chain_registry,
            sla_storage: HashMap::new(),
        }
    }

    /// Retrieves the next SLA for the given chain ID.
    pub fn get_next_sla(&self, chain_id: ChainId) -> Option<&SLA> {
        self.sla_storage
            .get(&chain_id)
            .and_then(|sla_list| sla_list.first())
    }

    /// Submits an SLA to the registry, validating it before storage.
    ///
    /// Returns `Ok(())` if the SLA is valid and successfully stored,
    /// or an `AuctionError` if validation fails.
    pub fn submit_sale_info(&mut self, chain_id: ChainId, sla: SLA) -> Result<(), AuctionError> {
        // Perform validations using our helper function.
        self.validate_sla(chain_id, &sla)?;

        // Insert into storage only if validations succeed.
        self.insert_sla(chain_id, sla);
        Ok(())
    }

    /// Inserts an SLA into the appropriate chain's list and sorts the list by `start_time`.
    fn insert_sla(&mut self, chain_id: ChainId, sla: SLA) {
        // If this chain doesn't exist yet, create a new vector.
        self.sla_storage.entry(chain_id).or_default().push(sla);

        // Sort by `start_time`.
        if let Some(sla_list) = self.sla_storage.get_mut(&chain_id) {
            sla_list.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        }
    }

    /// Validates the SLA before storing it.
    ///
    /// Returns `Ok(())` if all validations pass, otherwise
    /// returns the first encountered `AuctionError`.
    fn validate_sla(&self, chain_id: ChainId, sla: &SLA) -> Result<(), AuctionError> {
        // Validate chain ID
        self.validate_chain_id(chain_id)?;

        // Validate seller
        self.validate_seller(chain_id, &sla.seller_addr)?;

        // Validate seller signature
        self.validate_seller_signature(sla)?;

        // Validate gas limit
        self.validate_gas_limit(chain_id, sla.blockspace_size)?;

        // Validate timings
        self.validate_timings(sla.start_time, sla.end_time)?;

        Ok(())
    }

    /// Validates the chain ID.
    fn validate_chain_id(&self, chain_id: ChainId) -> Result<(), AuctionError> {
        if !self.chain_registry.validate_chain_id(chain_id) {
            Err(AuctionError::InvalidChainId)
        } else {
            Ok(())
        }
    }

    /// Validates the seller's registration on the chain.
    fn validate_seller(&self, chain_id: ChainId, seller_addr: &str) -> Result<(), AuctionError> {
        if !self.chain_registry.is_valid_seller(chain_id, seller_addr) {
            Err(AuctionError::SellerNotRegistered)
        } else {
            Ok(())
        }
    }

    /// Validates the seller's signature (mock).
    fn validate_seller_signature(&self, sla: &SLA) -> Result<(), AuctionError> {
        if !verify_signature(&sla.seller_addr, &sla.seller_signature) {
            Err(AuctionError::InvalidSellerSignature)
        } else {
            Ok(())
        }
    }

    /// Validates the auction's gas limit.
    fn validate_gas_limit(
        &self,
        chain_id: ChainId,
        blockspace_size: u64,
    ) -> Result<(), AuctionError> {
        match self.chain_registry.get_max_gas_limit(chain_id) {
            Some(max_gas) if blockspace_size <= max_gas => Ok(()),
            _ => Err(AuctionError::InvalidGasLimit),
        }
    }

    fn validate_timings(&self, start_time: u64, end_time: u64) -> Result<(), AuctionError> {
        if end_time < start_time + 500 {
            return Err(AuctionError::InvalidAuctionTime);
        }
        Ok(())
    }
}
