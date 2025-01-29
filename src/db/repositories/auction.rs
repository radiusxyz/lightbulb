use async_trait::async_trait;

use crate::{
    core::domain::{AuctionInfo, AuctionRepository},
    db::pool::DbPool,
    utils::errors::DatabaseError,
};

/// `SqliteAuctionRepository` provides SQLite-based implementations for managing auction data.
pub struct SqliteAuctionRepository {
    /// Database connection pool.
    db_pool: DbPool,
}

impl SqliteAuctionRepository {
    /// Creates a new instance of `SqliteAuctionRepository`.
    pub fn new(db_pool: DbPool) -> Self {
        SqliteAuctionRepository { db_pool }
    }
}

#[async_trait]
impl AuctionRepository for SqliteAuctionRepository {
    /// Inserts a new auction into the database.
    async fn create_auction(&self, auction_info: AuctionInfo) -> Result<(), DatabaseError> {
        let query = r#"
            INSERT INTO auctions (id, chain_id, block_number, seller_address, blockspace_size, start_time, end_time, seller_signature)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(&auction_info.auction_id)
            .bind(auction_info.chain_id as i64)
            .bind(auction_info.block_number as i64)
            .bind(&auction_info.seller_address)
            .bind(auction_info.blockspace_size as i64)
            .bind(auction_info.start_time as i64)
            .bind(auction_info.end_time as i64)
            .bind(&auction_info.seller_signature)
            .execute(&self.db_pool.pool)
            .await?;

        Ok(())
    }

    /// Retrieves auction information by ID.
    async fn get_auction_info(
        &self,
        auction_id: &str,
    ) -> Result<Option<AuctionInfo>, DatabaseError> {
        let query = r#"
            SELECT id, chain_id, block_number, seller_address, blockspace_size, start_time, end_time, seller_signature
            FROM auctions
            WHERE id = ?
        "#;

        let auction = sqlx::query_as::<_, AuctionInfo>(query)
            .bind(auction_id)
            .fetch_optional(&self.db_pool.pool)
            .await?;

        Ok(auction)
    }

    /// Lists all auctions stored in the database.
    async fn list_auctions(&self) -> Result<Vec<AuctionInfo>, DatabaseError> {
        let query = r#"
            SELECT id, chain_id, block_number, seller_address, blockspace_size, start_time, end_time, seller_signature
            FROM auctions
        "#;

        let auctions = sqlx::query_as::<_, AuctionInfo>(query)
            .fetch_all(&self.db_pool.pool)
            .await?;

        Ok(auctions)
    }

    /// Deletes an auction by ID.
    async fn delete_auction(&self, auction_id: &str) -> Result<(), DatabaseError> {
        let query = r#"
            DELETE FROM auctions WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(auction_id)
            .execute(&self.db_pool.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbPool;

    #[tokio::test]
    async fn test_create_and_get_auction() -> Result<(), DatabaseError> {
        // Setup test database
        let db_pool = DbPool::new("sqlite::memory:").await?;
        let repo = SqliteAuctionRepository::new(db_pool.clone());

        // Create AuctionInfo for testing
        let auction_info = AuctionInfo {
            auction_id: "test_auction".to_string(),
            chain_id: 1,
            block_number: 100,
            seller_address: "test_seller".to_string(),
            blockspace_size: 500,
            start_time: 1633036800,
            end_time: 1633123200,
            seller_signature: "test_signature".to_string(),
        };

        // Test create_auction
        repo.create_auction(auction_info.clone()).await?;

        // Test get_auction_info
        let fetched = repo.get_auction_info("test_auction").await?;
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.auction_id, auction_info.auction_id);
        assert_eq!(
            fetched.block_number as i64,
            auction_info.block_number as i64
        );
        assert_eq!(fetched.seller_address, auction_info.seller_address);
        assert_eq!(
            fetched.blockspace_size as i64,
            auction_info.blockspace_size as i64
        );
        assert_eq!(fetched.start_time as i64, auction_info.start_time as i64);
        assert_eq!(fetched.end_time as i64, auction_info.end_time as i64);
        assert_eq!(fetched.seller_signature, auction_info.seller_signature);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_auctions() -> Result<(), DatabaseError> {
        // Setup test database
        let db_pool = DbPool::new("sqlite::memory:").await?;
        let repo = SqliteAuctionRepository::new(db_pool.clone());

        // Create and insert two AuctionInfo instances
        let auction1 = AuctionInfo {
            auction_id: "auction1".to_string(),
            chain_id: 1,
            block_number: 101,
            seller_address: "seller1".to_string(),
            blockspace_size: 600,
            start_time: 1633036801,
            end_time: 1633123201,
            seller_signature: "signature1".to_string(),
        };

        let auction2 = AuctionInfo {
            auction_id: "auction2".to_string(),
            chain_id: 2,
            block_number: 102,
            seller_address: "seller2".to_string(),
            blockspace_size: 700,
            start_time: 1633036802,
            end_time: 1633123202,
            seller_signature: "signature2".to_string(),
        };

        repo.create_auction(auction1.clone()).await?;
        repo.create_auction(auction2.clone()).await?;

        // Test list_auctions
        let auctions = repo.list_auctions().await?;
        assert_eq!(auctions.len(), 2);
        assert!(auctions.contains(&auction1));
        assert!(auctions.contains(&auction2));

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_auction() -> Result<(), DatabaseError> {
        // Setup test database
        let db_pool = DbPool::new("sqlite::memory:").await?;
        let repo = SqliteAuctionRepository::new(db_pool.clone());

        // Create and insert AuctionInfo
        let auction = AuctionInfo {
            auction_id: "auction_to_delete".to_string(),
            chain_id: 1,
            block_number: 103,
            seller_address: "seller3".to_string(),
            blockspace_size: 800,
            start_time: 1633036803,
            end_time: 1633123203,
            seller_signature: "signature3".to_string(),
        };

        repo.create_auction(auction.clone()).await?;

        // Verify existence before deletion
        let fetched_before = repo.get_auction_info("auction_to_delete").await?;
        assert!(fetched_before.is_some());

        // Test delete_auction
        repo.delete_auction("auction_to_delete").await?;

        // Verify non-existence after deletion
        let fetched_after = repo.get_auction_info("auction_to_delete").await?;
        assert!(fetched_after.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_duplicate_auction() -> Result<(), DatabaseError> {
        // Setup test database
        let db_pool = DbPool::new("sqlite::memory:").await?;
        let repo = SqliteAuctionRepository::new(db_pool.clone());

        // Create and insert AuctionInfo
        let auction = AuctionInfo {
            auction_id: "duplicate_auction".to_string(),
            chain_id: 1,
            block_number: 104,
            seller_address: "seller4".to_string(),
            blockspace_size: 900,
            start_time: 1633036804,
            end_time: 1633123204,
            seller_signature: "signature4".to_string(),
        };

        // First insertion
        repo.create_auction(auction.clone()).await?;

        // Attempt duplicate insertion
        let result = repo.create_auction(auction.clone()).await;
        assert!(result.is_err());

        // Check that the error is DatabaseError::DatabaseError
        match result {
            Err(DatabaseError::DatabaseError(msg)) => {
                // SQLite returns "UNIQUE constraint failed" on duplicate keys
                assert!(msg.contains("UNIQUE constraint failed"));
            }
            _ => panic!("Expected DatabaseError::DatabaseError due to duplicate key"),
        }

        Ok(())
    }
}
