use crate::utils::helpers::compute_hash;

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

/// Represents a Service Level Agreement (AuctionInfo) provided by the seller, which is the basis for an auction.
#[derive(Debug, Clone)]
pub struct AuctionInfo {
    pub id: AuctionId,
    pub block_height: u64,
    pub seller_addr: String,
    pub blockspace_size: u64,
    /// Start time in Unix milliseconds.
    pub start_time: u64,
    /// End time in Unix milliseconds.
    pub end_time: u64,
    pub seller_signature: String,
}

impl AuctionInfo {
    /// Creates a new AuctionInfo instance with the given parameters.
    pub fn new(
        block_height: u64,
        seller_addr: String,
        blockspace_size: u64,
        start_time: u64,
        end_time: u64,
        seller_signature: String,
    ) -> Self {
        AuctionInfo {
            id: compute_hash(&[
                block_height.to_be_bytes().as_ref(),
                seller_addr.as_bytes(),
                blockspace_size.to_be_bytes().as_ref(),
                start_time.to_be_bytes().as_ref(),
                end_time.to_be_bytes().as_ref(),
                seller_signature.as_bytes(),
            ]),
            block_height,
            seller_addr,
            blockspace_size,
            start_time,
            end_time,
            seller_signature,
        }
    }
}

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

// ------------------------ Type Aliases ------------------------

pub type ChainId = u64;
pub type AuctionId = String;
