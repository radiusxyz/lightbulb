use crate::db::pool::DbPool;
use crate::domain::{AuctionInfo, AuctionRepository};
use crate::utils::errors::RegistryError;
use async_trait::async_trait;

pub struct SqliteAuctionRepository {
    db_pool: DbPool,
}

impl SqliteAuctionRepository {
    pub fn new(db_pool: DbPool) -> Self {
        SqliteAuctionRepository { db_pool }
    }
}

#[async_trait]
impl AuctionRepository for SqliteAuctionRepository {
    async fn create_auction(&self, auction_info: AuctionInfo) -> Result<(), RegistryError> {
        let query = r#"
            INSERT INTO auctions (id, block_height, seller_addr, blockspace_size, start_time, end_time, seller_signature)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(&auction_info.id)
            .bind(auction_info.block_height as i64)
            .bind(&auction_info.seller_addr)
            .bind(auction_info.blockspace_size as i64)
            .bind(auction_info.start_time as i64)
            .bind(auction_info.end_time as i64)
            .bind(&auction_info.seller_signature)
            .execute(&self.db_pool.pool)
            .await
            .map_err(|e| RegistryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_auction_info(
        &self,
        auction_id: &str,
    ) -> Result<Option<AuctionInfo>, RegistryError> {
        let query = r#"
            SELECT id, block_height, seller_addr, blockspace_size, start_time, end_time, seller_signature
            FROM auctions
            WHERE id = ?
        "#;

        let auction = sqlx::query_as::<_, AuctionInfo>(query)
            .bind(auction_id)
            .fetch_optional(&self.db_pool.pool)
            .await
            .map_err(|e| RegistryError::DatabaseError(e.to_string()))?;

        Ok(auction)
    }

    async fn list_auctions(&self) -> Result<Vec<AuctionInfo>, RegistryError> {
        let query = r#"
            SELECT id, block_height, seller_addr, blockspace_size, start_time, end_time, seller_signature
            FROM auctions
        "#;

        let auctions = sqlx::query_as::<_, AuctionInfo>(query)
            .fetch_all(&self.db_pool.pool)
            .await
            .map_err(|e| RegistryError::DatabaseError(e.to_string()))?;

        Ok(auctions)
    }

    async fn delete_auction(&self, auction_id: &str) -> Result<(), RegistryError> {
        let query = r#"
            DELETE FROM auctions WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(auction_id)
            .execute(&self.db_pool.pool)
            .await
            .map_err(|e| RegistryError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use sqlx::SqlitePool;

    /// Helper function to create an in-memory SQLite pool and apply migrations.
    async fn setup_test_db() -> Result<DbPool, RegistryError> {
        // In-memory SQLite database URL
        let database_url = "sqlite:./test.db";

        // Create and connect the SQLx pool
        let pool = SqlitePool::connect(database_url)
            .await
            .map_err(|e| RegistryError::DatabaseError(e.to_string()))?;

        // Apply migrations
        // Assumes migration files are located in the project's root "migrations" folder
        // The sqlx::migrate! macro includes migrations at compile time
        // Therefore, the same migrations are applied during test execution
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| RegistryError::DatabaseError(e.to_string()))?;

        // Create DbPool instance
        Ok(DbPool { pool })
    }

    #[tokio::test]
    async fn test_create_and_get_auction() -> Result<(), RegistryError> {
        // Setup test database
        let db_pool = setup_test_db().await?;
        let repo = SqliteAuctionRepository::new(db_pool.clone());

        // Create AuctionInfo for testing
        let auction_info = AuctionInfo {
            id: "test_auction".to_string(),
            block_height: 100,
            seller_addr: "test_seller".to_string(),
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
        assert_eq!(fetched.id, auction_info.id);
        assert_eq!(
            fetched.block_height as i64,
            auction_info.block_height as i64
        );
        assert_eq!(fetched.seller_addr, auction_info.seller_addr);
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
    async fn test_list_auctions() -> Result<(), RegistryError> {
        // Setup test database
        let db_pool = setup_test_db().await?;
        let repo = SqliteAuctionRepository::new(db_pool.clone());

        // Create and insert two AuctionInfo instances
        let auction1 = AuctionInfo {
            id: "auction1".to_string(),
            block_height: 101,
            seller_addr: "seller1".to_string(),
            blockspace_size: 600,
            start_time: 1633036801,
            end_time: 1633123201,
            seller_signature: "signature1".to_string(),
        };

        let auction2 = AuctionInfo {
            id: "auction2".to_string(),
            block_height: 102,
            seller_addr: "seller2".to_string(),
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
    async fn test_delete_auction() -> Result<(), RegistryError> {
        // Setup test database
        let db_pool = setup_test_db().await?;
        let repo = SqliteAuctionRepository::new(db_pool.clone());

        // Create and insert AuctionInfo
        let auction = AuctionInfo {
            id: "auction_to_delete".to_string(),
            block_height: 103,
            seller_addr: "seller3".to_string(),
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
    async fn test_create_duplicate_auction() -> Result<(), RegistryError> {
        // Setup test database
        let db_pool = setup_test_db().await?;
        let repo = SqliteAuctionRepository::new(db_pool.clone());

        // Create and insert AuctionInfo
        let auction = AuctionInfo {
            id: "duplicate_auction".to_string(),
            block_height: 104,
            seller_addr: "seller4".to_string(),
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

        // Check that the error is RegistryError::DatabaseError
        match result {
            Err(RegistryError::DatabaseError(msg)) => {
                // SQLite returns "UNIQUE constraint failed" on duplicate keys
                assert!(msg.contains("UNIQUE constraint failed"));
            }
            _ => panic!("Expected RegistryError::DatabaseError due to duplicate key"),
        }

        Ok(())
    }
}
