use thiserror::Error;

use crate::domain::{AuctionId, ChainId};

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

    #[error("Seller {0} is not registered on the specified chain")]
    SellerNotRegistered(String),

    #[error("Invalid seller signature")]
    InvalidSellerSignature,

    #[error("Invalid gas limit for this chain")]
    InvalidGasLimit,

    #[error("Invalid auction time settings")]
    InvalidAuctionTime,

    #[error("Chain {0} is already registered")]
    ChainAlreadyRegistered(ChainId),
}

#[derive(Error, Debug)]
pub enum BidError {
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

    #[error("Auction Error")]
    AuctionError,
}

impl From<AuctionError> for BidError {
    fn from(err: AuctionError) -> Self {
        match err {
            AuctionError::InvalidChainId(chain_id) => BidError::InvalidChainId(chain_id),
            AuctionError::InvalidAuctionId(auction_id) => BidError::InvalidAuctionId(auction_id),
            AuctionError::NoAuctions => BidError::NoAuctions,
            AuctionError::InvalidAuctionTime => BidError::InvalidAuctionTime,
            AuctionError::InvalidBuyerSignature => BidError::InvalidBuyerSignature,
            AuctionError::InsufficientFunds => BidError::InsufficientFunds,
            _ => BidError::AuctionError,
        }
    }
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

impl From<sqlx::migrate::MigrateError> for DatabaseError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        Self::DatabaseError(err.to_string())
    }
}
