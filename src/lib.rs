pub mod domain;
pub mod services;
pub mod utils;

#[cfg(test)]
mod tests {
    use super::domain::{Bid, Tx, SLA};
    use super::services::auction::{AuctionManager, AuctionWorker};
    use super::utils::helpers::current_unix_ms;
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    /// Basic test for submitting an SLA to the `AuctionManager`.
    /// It checks that an Auction ID (SHA256 hash) is generated and that the auction is stored.
    #[tokio::test]
    async fn test_submit_sale_info() {
        let manager = AuctionManager::new();
        let chain_id = 1;
        let sla = SLA {
            block_height: 123456,
            seller_addr: "0xSellerAddress".to_string(),
            blockspace_size: 1_000_000,
            start_time: current_unix_ms() + 1_000, // Starts 1 second in the future
            end_time: current_unix_ms() + 2_000,   // Ends 2 seconds in the future
            seller_signature: "MockSellerSignature".to_string(),
        };

        let result = manager.submit_sale_info(chain_id, sla.clone()).await;
        assert!(result.is_ok(), "Failed to submit sale info");

        let (auction_id, server_signature) = result.unwrap();
        println!("Generated Auction ID (hash) = {}", auction_id);
        // SHA256 hex string is 64 characters in length
        assert_eq!(auction_id.len(), 64);

        assert!(server_signature.contains("ServerSig-Chain:1-Auction:"));

        let state = manager
            .get_auction_state(chain_id, auction_id.clone())
            .await
            .expect("AuctionState not found");
        assert_eq!(state.sla.block_height, sla.block_height);
        assert_eq!(state.sla.seller_addr, sla.seller_addr);
    }

    /// Tests submitting a bid to an auction to ensure it is stored properly.
    #[tokio::test]
    async fn test_submit_bid() {
        let manager = AuctionManager::new();
        let chain_id = 1;
        let sla = SLA {
            block_height: 123456,
            seller_addr: "0xSellerAddress".to_string(),
            blockspace_size: 1_000_000,
            start_time: current_unix_ms() + 500, // Starts shortly
            end_time: current_unix_ms() + 10_000, // Ends after 10 seconds
            seller_signature: "MockSellerSignature".to_string(),
        };

        let (auction_id, _) = manager.submit_sale_info(chain_id, sla).await.unwrap();

        let bid = Bid {
            bidder_addr: "0xBuyerAddress".to_string(),
            bid_amount: 10_000,
            bidder_signature: "MockBuyerSignature".to_string(),
            tx_list: vec![Tx {
                tx_data: "dummy_tx_data".to_string(),
            }],
        };

        let res = manager.submit_bid(chain_id, auction_id.clone(), bid).await;
        assert!(res.is_ok(), "Failed to submit a valid bid");

        let state = manager
            .get_auction_state(chain_id, auction_id.clone())
            .await
            .expect("Failed to fetch AuctionState after bid submission");
        assert_eq!(state.bids.len(), 1);
    }

    /// Tests the auction worker's ability to determine a winner correctly.
    #[tokio::test]
    async fn test_auction_worker_winner_determination() {
        let manager = Arc::new(AuctionManager::new());
        let chain_id = 1;
        let sla = SLA {
            block_height: 123456,
            seller_addr: "0xSellerAddress".to_string(),
            blockspace_size: 1_000_000,
            start_time: current_unix_ms() + 500, // Starts shortly
            end_time: current_unix_ms() + 2_000, // Ends in 2 seconds
            seller_signature: "MockSellerSignature".to_string(),
        };

        let (auction_id, _) = manager.submit_sale_info(chain_id, sla).await.unwrap();

        let bid1 = Bid {
            bidder_addr: "0xBuyer1".to_string(),
            bid_amount: 10_000,
            bidder_signature: "MockBuyerSignature1".to_string(),
            tx_list: vec![Tx {
                tx_data: "tx1".to_string(),
            }],
        };

        let bid2 = Bid {
            bidder_addr: "0xBuyer2".to_string(),
            bid_amount: 20_000,
            bidder_signature: "MockBuyerSignature2".to_string(),
            tx_list: vec![Tx {
                tx_data: "tx2".to_string(),
            }],
        };

        // Submit bids
        manager
            .submit_bid(chain_id, auction_id.clone(), bid1)
            .await
            .unwrap();
        manager
            .submit_bid(chain_id, auction_id.clone(), bid2)
            .await
            .unwrap();

        // Start the worker
        let worker = AuctionWorker::new(manager.clone());
        tokio::spawn(async move {
            worker.run().await;
        });

        // Wait for the auction to end
        sleep(Duration::from_millis(3_000)).await;

        // Retrieve auction state
        let state = manager
            .get_auction_state(chain_id, auction_id.clone())
            .await
            .expect("Failed to fetch AuctionState after worker processing");

        assert!(state.is_ended, "Auction should be ended");
        assert_eq!(state.winner, Some("0xBuyer2".to_string()));
        assert_eq!(state.highest_bid, 20_000);
    }
}
