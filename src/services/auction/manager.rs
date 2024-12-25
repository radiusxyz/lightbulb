use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use hex;
use sha2::{Digest, Sha256};

use crate::domain::{AuctionId, AuctionState, Bid, ChainId, Tx, SLA};
use crate::services::{auction::AuctionWorker, registry::ChainRegistry};
use crate::utils::errors::AuctionError;
use crate::utils::helpers::{current_unix_ms, verify_signature};

/// The `AuctionManager` maintains an in-memory data store of auctions per chain.
#[derive(Clone)]
pub struct AuctionManager {
    /// A mapping of ChainId -> AuctionId -> AuctionState.
    pub auctions: Arc<RwLock<HashMap<ChainId, HashMap<AuctionId, AuctionState>>>>,
    /// A reference to a `ChainRegistry` for chain-specific data, such as max gas limits, registered sellers, etc.
    pub chain_registry: Arc<ChainRegistry>,
}

impl Default for AuctionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AuctionManager {
    /// Creates a new `AuctionManager` instance with default mock data.
    pub fn new() -> Self {
        AuctionManager {
            auctions: Arc::new(RwLock::new(HashMap::new())),
            chain_registry: Arc::new(ChainRegistry::new()),
        }
    }

    /// Starts the `AuctionWorker` in a background task. This worker periodically processes auctions.
    pub fn start_worker(self: &Arc<Self>) -> JoinHandle<()> {
        let worker = AuctionWorker::new(self.clone());
        tokio::spawn(async move {
            worker.run().await;
        })
    }

    /// Creates a new `AuctionId` by hashing the SLA fields with SHA-256 and encoding the result in hex.
    fn compute_auction_id(sla: &SLA) -> AuctionId {
        let mut hasher = Sha256::new();
        hasher.update(sla.seller_addr.as_bytes());
        hasher.update(sla.seller_signature.as_bytes());
        hasher.update(sla.block_height.to_be_bytes());
        hasher.update(sla.blockspace_size.to_be_bytes());
        hasher.update(sla.start_time.to_be_bytes());
        hasher.update(sla.end_time.to_be_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Submits SLA (sale info), validates it, and creates a new auction.
    /// Returns the generated `AuctionId` and a mock server signature.
    pub async fn submit_sale_info(
        &self,
        chain_id: ChainId,
        sla: SLA,
    ) -> Result<(AuctionId, String), AuctionError> {
        // Validate chain ID
        self.validate_chain(chain_id)?;

        // Validate seller
        self.validate_seller(chain_id, &sla.seller_addr)?;

        // Validate seller signature
        self.validate_seller_signature(&sla)?;

        // Validate gas limit
        self.validate_gas_limit(chain_id, sla.blockspace_size)?;

        // Validate auction timings
        self.validate_timings(sla.start_time, sla.end_time)?;

        // Generate Auction ID
        let auction_id = Self::compute_auction_id(&sla);

        // Create and store AuctionState
        self.store_auction(chain_id, auction_id.clone(), sla.clone())
            .await;

        // Generate mock server signature
        let server_signature = format!("ServerSig-Chain:{}-Auction:{}", chain_id, auction_id);

        Ok((auction_id, server_signature))
    }

    /// Requests the first auction's information on a given chain.
    pub async fn request_sale_info(
        &self,
        chain_id: ChainId,
    ) -> Result<(AuctionId, SLA), AuctionError> {
        let auctions = self.auctions.read().await;
        let chain_auctions = auctions
            .get(&chain_id)
            .ok_or(AuctionError::InvalidChainId)?;

        chain_auctions
            .iter()
            .next()
            .map(|(id, state)| (id.clone(), state.sla.clone()))
            .ok_or(AuctionError::NoAuctions)
    }

    /// Returns the top-of-book (highest bid) for the specified auction, verifying the seller signature (mock).
    pub async fn request_tob(
        &self,
        chain_id: ChainId,
        auction_id: AuctionId,
        seller_signature: &str,
    ) -> Result<u64, AuctionError> {
        // Verify seller's signature (mock)
        self.verify_seller_signature(seller_signature)?;

        // Retrieve highest bid
        let auctions = self.auctions.read().await;
        let chain_auctions = auctions
            .get(&chain_id)
            .ok_or(AuctionError::InvalidChainId)?;
        let auction_state = chain_auctions
            .get(&auction_id)
            .ok_or(AuctionError::InvalidAuctionId)?;

        Ok(auction_state.highest_bid)
    }

    /// Submits a new `Bid` to the specified auction.
    pub async fn submit_bid(
        &self,
        chain_id: ChainId,
        auction_id: AuctionId,
        bid: Bid,
    ) -> Result<String, AuctionError> {
        // Validate buyer's signature (mock)
        self.validate_buyer_signature(&bid)?;

        // Validate bidder's funds
        self.validate_bid_amount(bid.bid_amount)?;

        // Record the bid
        self.record_bid(chain_id, auction_id, bid).await
    }

    /// Retrieves the transactions associated with the winning bid. If no winner is set yet, returns an empty list.
    pub async fn request_latest_tob_info(
        &self,
        chain_id: ChainId,
        auction_id: AuctionId,
    ) -> Result<Vec<Tx>, AuctionError> {
        let auctions = self.auctions.read().await;
        let chain_auctions = auctions
            .get(&chain_id)
            .ok_or(AuctionError::InvalidChainId)?;
        let auction_state = chain_auctions
            .get(&auction_id)
            .ok_or(AuctionError::InvalidAuctionId)?;

        if let Some(ref winner_addr) = auction_state.winner {
            Ok(auction_state
                .bids
                .iter()
                .find(|b| &b.bidder_addr == winner_addr)
                .map(|b| b.tx_list.clone())
                .unwrap_or_else(Vec::new))
        } else {
            Ok(Vec::new())
        }
    }

    /// Retrieves the full auction state for the specified chain and auction ID.
    pub async fn get_auction_state(
        &self,
        chain_id: ChainId,
        auction_id: AuctionId,
    ) -> Result<AuctionState, AuctionError> {
        let auctions = self.auctions.read().await;
        let chain_auctions = auctions
            .get(&chain_id)
            .ok_or(AuctionError::InvalidChainId)?;
        chain_auctions
            .get(&auction_id)
            .cloned()
            .ok_or(AuctionError::InvalidAuctionId)
    }

    // ------------------------ Helper Functions ------------------------

    /// Validates the chain ID.
    fn validate_chain(&self, chain_id: ChainId) -> Result<(), AuctionError> {
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

    /// Validates the auction's start and end times.
    fn validate_timings(&self, start_time: u64, end_time: u64) -> Result<(), AuctionError> {
        let now = current_unix_ms();
        if start_time <= now {
            return Err(AuctionError::InvalidAuctionTime);
        }
        if end_time < start_time + 500 {
            return Err(AuctionError::InvalidAuctionTime);
        }
        Ok(())
    }

    /// Stores the auction in the in-memory data store.
    async fn store_auction(&self, chain_id: ChainId, auction_id: AuctionId, sla: SLA) {
        let mut auctions = self.auctions.write().await;
        auctions
            .entry(chain_id)
            .or_insert_with(HashMap::new)
            .insert(auction_id, AuctionState::new(sla));
    }

    /// Verifies the seller's signature (mock).
    fn verify_seller_signature(&self, _seller_signature: &str) -> Result<(), AuctionError> {
        // Implement actual verification logic here if needed
        Ok(())
    }

    /// Validates the buyer's signature (mock).
    fn validate_buyer_signature(&self, bid: &Bid) -> Result<(), AuctionError> {
        if !verify_signature(&bid.bidder_addr, &bid.bidder_signature) {
            Err(AuctionError::InvalidBuyerSignature)
        } else {
            Ok(())
        }
    }

    /// Validates the bid amount against mock funds.
    fn validate_bid_amount(&self, bid_amount: u64) -> Result<(), AuctionError> {
        if bid_amount > 1_000_000_000 {
            Err(AuctionError::InsufficientFunds)
        } else {
            Ok(())
        }
    }

    /// Records the bid in the specified auction.
    async fn record_bid(
        &self,
        chain_id: ChainId,
        auction_id: AuctionId,
        bid: Bid,
    ) -> Result<String, AuctionError> {
        let mut auctions = self.auctions.write().await;
        let chain_auctions = auctions
            .get_mut(&chain_id)
            .ok_or(AuctionError::InvalidChainId)?;

        let auction_state = chain_auctions
            .get_mut(&auction_id)
            .ok_or(AuctionError::InvalidAuctionId)?;

        if auction_state.is_ended {
            return Err(AuctionError::AuctionEnded);
        }

        auction_state.bids.push(bid);

        Ok(format!(
            "ACK: Auction {} on Chain {} bid accepted.",
            auction_id, chain_id
        ))
    }
}
