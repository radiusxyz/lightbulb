use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::core::auction::AuctionWorker;
use crate::domain::{AuctionId, AuctionInfo, AuctionState, Bid, ChainId, Tx};
use crate::services::registry::AuctionRegistry;
use crate::utils::{
    errors::AuctionError,
    helpers::{compute_auction_id, current_unix_ms, verify_signature},
};

/// `AuctionManager` manages ongoing auctions for each chain.
/// Registration and validation of auctions are handled by `AuctionRegistry`,
/// while `AuctionManager` retrieves auction information from `AuctionRegistry`,
/// initiates/terminates auctions, and handles bids during an auction.
#[derive(Clone)]
pub struct AuctionManager {
    /// Current auction IDs for each chain
    pub ongoing_auction_ids: Arc<RwLock<HashMap<ChainId, AuctionId>>>,
    /// Current auction states for each chain
    pub ongoing_auctions: Arc<RwLock<HashMap<ChainId, AuctionState>>>,
    /// Registry of registered auctions and validation logic
    pub auction_registry: Arc<AuctionRegistry>,
}

impl AuctionManager {
    /// Creates a new instance of `AuctionManager`.
    pub fn new(auction_registry: AuctionRegistry) -> Self {
        AuctionManager {
            ongoing_auction_ids: Arc::new(RwLock::new(HashMap::new())),
            ongoing_auctions: Arc::new(RwLock::new(HashMap::new())),
            auction_registry: Arc::new(auction_registry),
        }
    }

    /// Starts the `AuctionWorker` responsible for background tasks related to auctions.
    /// Examples: checking auction end time, automatic settlement, event handling, etc.
    pub fn start_worker(self: &Arc<Self>) -> JoinHandle<()> {
        let worker = AuctionWorker::new(self.clone());
        tokio::spawn(async move {
            worker.run().await;
        })
    }

    /// Retrieves the next auction information from `AuctionRegistry`,
    /// and sets the ongoing auction in `ongoing_auction_ids` and `ongoing_auctions`.
    ///
    /// - If an auction is already ongoing, it can be overwritten or raise an error.
    /// - Returns `None` if no auction is available or if the auction start time has not yet been reached.
    pub async fn start_next_auction(&self, chain_id: ChainId) -> Option<AuctionId> {
        // Fetch the next auction information
        let next_auction_info = self.auction_registry.get_next_auction_info(chain_id)?;

        // Check if the auction start time has passed
        if current_unix_ms() < next_auction_info.start_time {
            return None;
        }

        // Compute auction ID
        let auction_id = compute_auction_id(next_auction_info);

        // Create AuctionState (initial state)
        let new_auction_state = AuctionState::new(next_auction_info.clone());

        {
            // Acquire write lock and update ongoing auction state
            let mut ids = self.ongoing_auction_ids.write().await;
            let mut states = self.ongoing_auctions.write().await;

            ids.insert(chain_id, auction_id.clone());
            states.insert(chain_id, new_auction_state);
        }

        // Return the auction ID
        Some(auction_id)
    }

    /// Requests the `AuctionInfo` of the current ongoing auction.
    /// - Returns `AuctionError::NoAuctions` if no auction is found.
    pub async fn request_sale_info(
        &self,
        chain_id: ChainId,
    ) -> Result<(AuctionId, AuctionInfo), AuctionError> {
        let ids = self.ongoing_auction_ids.read().await;
        let states = self.ongoing_auctions.read().await;

        let auction_id = ids.get(&chain_id).ok_or(AuctionError::NoAuctions)?.clone();
        let state = states.get(&chain_id).ok_or(AuctionError::NoAuctions)?;

        Ok((auction_id, state.auction_info.clone()))
    }

    /// Submits a bid to the auction.
    /// - Assumes seller signature and chain verification have already passed in `AuctionRegistry`.
    /// - Simply checks if the auction is ongoing and registers the bid.
    pub async fn submit_bid(
        &self,
        chain_id: ChainId,
        auction_id: AuctionId,
        bid: Bid,
    ) -> Result<String, AuctionError> {
        // Check if the auction is ongoing
        {
            let ids = self.ongoing_auction_ids.read().await;
            let ongoing_id = ids.get(&chain_id).ok_or(AuctionError::NoAuctions)?;
            if *ongoing_id != auction_id {
                return Err(AuctionError::InvalidAuctionId(auction_id));
            }
        }

        // Validate the bid
        self.validate_bid(&bid)?;

        // Record bid in the auction
        let mut states = self.ongoing_auctions.write().await;
        let auction_state = states.get_mut(&chain_id).ok_or(AuctionError::NoAuctions)?;

        if auction_state.is_ended {
            return Err(AuctionError::AuctionEnded);
        }

        auction_state.bids.push(bid);
        Ok(format!(
            "ACK: Auction {} on Chain {} bid accepted.",
            auction_id, chain_id
        ))
    }

    /// Retrieves the latest Top-of-Book (winner) transaction information of the current auction.
    /// - Returns an empty `Vec` if no winner is set.
    pub async fn request_latest_tob_info(
        &self,
        chain_id: ChainId,
        auction_id: AuctionId,
    ) -> Result<Vec<Tx>, AuctionError> {
        // Verify if the ongoing auction ID matches
        let ids = self.ongoing_auction_ids.read().await;
        let ongoing_id = ids.get(&chain_id).ok_or(AuctionError::NoAuctions)?;
        if *ongoing_id != auction_id {
            return Err(AuctionError::InvalidAuctionId(auction_id));
        }

        let states = self.ongoing_auctions.read().await;
        let auction_state = states.get(&chain_id).ok_or(AuctionError::NoAuctions)?;

        if let Some(ref winner_addr) = auction_state.winner {
            let tx_list = auction_state
                .bids
                .iter()
                .find(|b| &b.bidder_addr == winner_addr)
                .map(|b| b.tx_list.clone())
                .unwrap_or_default();
            Ok(tx_list)
        } else {
            Ok(Vec::new())
        }
    }

    /// Retrieves the full state of the current ongoing auction.
    /// - Returns an error if no auction is ongoing or IDs do not match.
    pub async fn get_auction_state(
        &self,
        chain_id: ChainId,
        auction_id: AuctionId,
    ) -> Result<AuctionState, AuctionError> {
        let ids = self.ongoing_auction_ids.read().await;
        let ongoing_id = ids.get(&chain_id).ok_or(AuctionError::NoAuctions)?;
        if *ongoing_id != auction_id {
            return Err(AuctionError::InvalidAuctionId(auction_id));
        }

        let states = self.ongoing_auctions.read().await;
        let auction_state = states.get(&chain_id).ok_or(AuctionError::NoAuctions)?;
        Ok(auction_state.clone())
    }

    // ------------------------ Private Helper Functions ------------------------

    fn validate_bid(&self, bid: &Bid) -> Result<(), AuctionError> {
        self.validate_buyer_signature(bid)?;
        self.validate_bid_amount(bid.bid_amount)
    }

    /// (Mock) Verifies the buyer's signature.
    fn validate_buyer_signature(&self, bid: &Bid) -> Result<(), AuctionError> {
        if !verify_signature(&bid.bidder_addr, &bid.bidder_signature) {
            Err(AuctionError::InvalidBuyerSignature)
        } else {
            Ok(())
        }
    }

    /// (Mock) Validates the bid amount.
    fn validate_bid_amount(&self, bid_amount: u64) -> Result<(), AuctionError> {
        if bid_amount > 1_000_000_000 {
            Err(AuctionError::InsufficientFunds)
        } else {
            Ok(())
        }
    }
}
