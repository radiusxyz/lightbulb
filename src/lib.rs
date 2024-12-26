pub mod core;
pub mod db;
pub mod domain;
pub mod services;
pub mod utils;

#[cfg(test)]
mod tests {
    use crate::core::auction::AuctionManager;
    use crate::domain::{AuctionInfo, Bid};
    use crate::services::registry::{AuctionRegistry, ChainRegistry};
    use crate::utils::helpers::current_unix_ms;

    use tokio::time::{sleep, Duration};

    /// A high-level integration test that checks creating and finishing a single auction flow.
    #[tokio::test]
    async fn test_auction_flow() -> Result<(), Box<dyn std::error::Error>> {
        // 1) Set up a ChainRegistry with mock data. (You may already have one from default)
        let chain_registry = ChainRegistry::new();

        // 2) Create an AuctionRegistry using that chain registry.
        let mut auction_registry = AuctionRegistry::new(chain_registry);

        // 3) Pick a chain ID recognized by the ChainRegistry (e.g., 1).
        let chain_id = 1_u64;

        // Make sure our AuctionRegistry has a storage entry for chain_id = 1.
        // If your real code requires you to initialize this manually, do so here:
        auction_registry
            .auction_storage
            .insert(chain_id, Vec::new());

        // 4) Create an AuctionInfo that starts now and ends soon.
        let start_time = current_unix_ms();
        let end_time = start_time + 2_000; // 2 seconds from now
        let auction_info = AuctionInfo {
            id: "test-auction-id".to_string(), // Normally you might use AuctionInfo::new(...).
            block_number: 12345,
            seller_addr: "0xSellerAddress".to_string(), // Must match a registered seller in ChainRegistry
            blockspace_size: 500,
            start_time,
            end_time,
            seller_signature: "some_mock_signature".to_string(),
        };

        // 5) Submit this AuctionInfo to the registry.
        //    This validates and stores the auction under chain_id = 1.
        auction_registry.submit_sale_info(chain_id, auction_info)?;

        // 6) Create the AuctionManager with the chain ID = [1].
        let chain_ids = [chain_id];
        let manager = AuctionManager::new(auction_registry, &chain_ids);

        // 7) Start the auction for this chain.
        let maybe_auction_id = manager.start_next_auction(chain_id).await;
        assert!(
            maybe_auction_id.is_some(),
            "Auction was not started properly."
        );
        let auction_id = maybe_auction_id.unwrap();
        println!("Started auction with ID: {}", auction_id);

        // 8) Submit a mock bid
        let mock_bid = Bid {
            bidder_addr: "0xDemoBidder".to_string(),
            bid_amount: 999,
            bidder_signature: "bidder_signature".to_string(),
            tx_list: vec![], // no actual Tx for this example
        };

        let submit_result = manager
            .submit_bid(chain_id, auction_id.clone(), mock_bid)
            .await;
        assert!(
            submit_result.is_ok(),
            "Failed to submit a bid: {:?}",
            submit_result
        );

        // 9) Sleep longer than the auction duration (2s). This allows the worker loop
        //    to detect the end_time and finish the auction.
        sleep(Duration::from_secs(3)).await;

        // 10) Retrieve and verify the final auction state
        let final_state = manager.get_auction_state(chain_id).await?;
        assert!(
            final_state.is_ended,
            "Expected the auction to be ended by now."
        );

        // Confirm the highest_bid / winner
        println!(
            "Auction ended with highest bid = {}, winner = {:?}",
            final_state.highest_bid, final_state.winner
        );
        assert_eq!(
            final_state.highest_bid, 999,
            "Expected highest bid to be 999."
        );
        assert_eq!(
            final_state.winner,
            Some("0xDemoBidder".to_string()),
            "Expected the winner to be 0xDemoBidder."
        );

        Ok(())
    }
}
