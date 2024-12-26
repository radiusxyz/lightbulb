use crate::domain::{AuctionId, AuctionInfo, AuctionResult, ChainId};
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

    /// Insert auction info into the DB (for AuctionRegistry)
    pub async fn insert_auction_info(
        &self,
        chain_id: ChainId,
        auction_info: &AuctionInfo,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    /// Get auction info from the DB (for AuctionRegistry)
    pub async fn get_auction_info(
        &self,
        chain_id: ChainId,
        auction_id: &AuctionId,
    ) -> Result<AuctionInfo, Error> {
        unimplemented!()
    }

    /// Insert auction result into the DB (for AuctionRegistry)
    pub async fn insert_auction_result(&self, auction_result: &AuctionResult) -> Result<(), Error> {
        unimplemented!()
    }

    /// Get auction result from the DB (for AuctionRegistry)
    pub async fn get_auction_result(
        &self,
        chain_id: ChainId,
        auction_id: &AuctionId,
    ) -> Result<AuctionResult, Error> {
        unimplemented!()
    }
}
