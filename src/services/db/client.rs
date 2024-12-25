use crate::domain::{AuctionInfo, AuctionState, Bid, ChainInfo};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Error;

/// DBClient holds the SQLite pool and provides DB access logic.
pub struct DBClient {
    pool: SqlitePool,
}

impl DBClient {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let pool = SqlitePoolOptions::new().connect(database_url).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Provides a method to initialize necessary tables as an example.
    /// In a real service environment, it's better to use sqlx::migrate! or SQL scripts.
    pub async fn init_db(&self) -> Result<(), Error> {
        unimplemented!()
    }

    /// Get chain info from the DB (for ChainRegistry)
    pub async fn get_chain_info(&self, chain_id: i64) -> Result<ChainInfo, Error> {
        unimplemented!()
    }

    /// Insert auction info into the DB (for AuctionRegistry)
    pub async fn insert_auction_info(
        &self,
        chain_id: i64,
        auction_info: &AuctionInfo,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    /// Get next auction info from the DB (for AuctionRegistry)
    pub async fn get_next_auction_info(&self, chain_id: i64) -> Result<Option<AuctionInfo>, Error> {
        unimplemented!()
    }

    /// Insert auction state into the DB (for AuctionManager)
    pub async fn insert_auction_state(
        &self,
        chain_id: i64,
        auction_id: i64,
        auction_state: &AuctionState,
    ) -> Result<i64, Error> {
        unimplemented!()
    }

    /// Get auction state from the DB (for AuctionManager)
    pub async fn get_auction_state(
        &self,
        chain_id: i64,
        auction_id: i64,
    ) -> Result<Option<AuctionState>, Error> {
        unimplemented!()
    }

    /// Insert bid into the DB (for AuctionManager)
    pub async fn insert_bid(&self, chain_id: i64, auction_id: i64, bid: &Bid) -> Result<(), Error> {
        unimplemented!()
    }
}
