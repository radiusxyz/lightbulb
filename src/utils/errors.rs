use crate::domain::{AuctionId, ChainId};
use thiserror::Error;

/// A set of possible errors that can occur in the auction workflow.
#[derive(Error, Debug)]
pub enum AuctionError {
    #[error("Invalid chain ID: {0}")]
    InvalidChainId(ChainId),

    #[error("Invalid auction ID: {0}")]
    InvalidAuctionId(AuctionId),

    #[error("No auctions found for the specified chain")]
    NoAuctions,

    #[error("Seller is not registered on the specified chain")]
    SellerNotRegistered,

    #[error("Invalid seller signature")]
    InvalidSellerSignature,

    #[error("Invalid gas limit for this chain")]
    InvalidGasLimit,

    #[error("Invalid auction time settings")]
    InvalidAuctionTime,

    #[error("Invalid buyer signature")]
    InvalidBuyerSignature,

    #[error("Insufficient funds for the bid")]
    InsufficientFunds,

    #[error("Auction has not started yet")]
    AuctionNotStarted,

    #[error("Auction has already ended")]
    AuctionEnded,
}

/// A set of possible errors that can occur in the registry workflow.
#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Invalid chain ID: {0}")]
    InvalidChainId(ChainId),

    #[error("Seller is not registered on the specified chain")]
    SellerNotRegistered,

    #[error("Invalid seller signature")]
    InvalidSellerSignature,

    #[error("Invalid gas limit for this chain")]
    InvalidGasLimit,

    #[error("Invalid auction time settings")]
    InvalidAuctionTime,

    #[error("Database error: {0}")]
    DatabaseError(String),
}
