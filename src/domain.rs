use async_trait::async_trait;

use crate::utils::{errors::DatabaseError, helpers::compute_hash};

/// Represents a transaction submitted by a bidder (mock).
#[derive(Debug, Clone)]
pub struct Tx {
    pub tx_data: String,
}

/// Represents a bid submitted by a buyer, including bidder address, amount, signature, and transaction list.
#[derive(Debug, Clone)]
pub struct Bid {
    pub bidder_addr: String,
    pub bid_amount: u64,
    pub bidder_signature: String,
    pub tx_list: Vec<Tx>,
}

pub struct ChainInfo {
    pub gas_limit: u64,
    pub registered_sellers: Vec<String>,
}

/// Represents a Service Level Agreement (AuctionInfo) provided by the seller, which is the basis for an auction.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuctionInfo {
    pub id: AuctionId,
    pub chain_id: ChainId,
    pub block_number: u64,
    pub seller_address: String,
    pub blockspace_size: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub seller_signature: String,
}

impl AuctionInfo {
    /// Creates a new AuctionInfo instance with the given parameters.
    pub fn new(
        chain_id: ChainId,
        block_number: u64,
        seller_address: String,
        blockspace_size: u64,
        start_time: u64,
        end_time: u64,
        seller_signature: String,
    ) -> Self {
        AuctionInfo {
            id: compute_hash(&[
                chain_id.to_be_bytes().as_ref(),
                block_number.to_be_bytes().as_ref(),
                seller_address.as_bytes(),
                blockspace_size.to_be_bytes().as_ref(),
                start_time.to_be_bytes().as_ref(),
                end_time.to_be_bytes().as_ref(),
                seller_signature.as_bytes(),
            ]),
            chain_id,
            block_number,
            seller_address,
            blockspace_size,
            start_time,
            end_time,
            seller_signature,
        }
    }
}

impl Ord for AuctionInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start_time.cmp(&other.start_time)
    }
}

impl PartialOrd for AuctionInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AuctionInfo {
    fn eq(&self, other: &Self) -> bool {
        self.start_time == other.start_time
    }
}

impl Eq for AuctionInfo {}

/// Represents the state of an auction, including the AuctionInfo, current highest bid, winner, all bids, and whether it is ended.
#[derive(Debug, Clone)]
pub struct AuctionState {
    pub auction_info: AuctionInfo,
    pub highest_bid: u64,
    pub winner: Option<String>,
    pub bids: Vec<Bid>,
    pub is_ended: bool,
}

impl AuctionState {
    /// Creates a new `AuctionState` based on the provided AuctionInfo.
    pub fn new(auction_info: AuctionInfo) -> Self {
        AuctionState {
            auction_info,
            highest_bid: 0,
            winner: None,
            bids: Vec::new(),
            is_ended: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuctionResult {
    pub chain_id: ChainId,
    pub auction_id: AuctionId,
    pub winner: String,
}

#[derive(Debug)]
pub struct WorkerMessage {
    pub message_type: WorkerMessageType,
    pub chain_id: ChainId,
    pub auction_id: AuctionId,
}

#[derive(Debug)]
pub enum WorkerMessageType {
    AuctionEnded,
    AuctionProcessing,
    Idle,
}

// ------------------------------------------------------------------------
// Type aliases
// ------------------------------------------------------------------------

pub type ChainId = u64;
pub type AuctionId = String;

// ------------------------------------------------------------------------
// Repository Traits
// ------------------------------------------------------------------------

#[async_trait]
pub trait AuctionRepository {
    async fn create_auction(&self, auction_info: AuctionInfo) -> Result<(), DatabaseError>;
    async fn get_auction_info(
        &self,
        auction_id: &str,
    ) -> Result<Option<AuctionInfo>, DatabaseError>;
    async fn list_auctions(&self) -> Result<Vec<AuctionInfo>, DatabaseError>;
    async fn delete_auction(&self, auction_id: &str) -> Result<(), DatabaseError>;
}
