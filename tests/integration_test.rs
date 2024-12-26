use tokio::time::sleep;

use lightbulb::{
    core::auction::AuctionManager,
    domain::{AuctionInfo, Bid, ChainId, ChainInfo, Tx},
    services::registry::RegistryService,
    utils::helpers::current_unix_ms,
};

#[tokio::test]
async fn test_auction_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test: test_auction_lifecycle");

    // 1. Setup RegistryService
    let (auction_registry, chain_registry) = RegistryService::create_registry().await;
    let registry_service = RegistryService::new(auction_registry, chain_registry);

    println!("RegistryService created");

    // Define the ChainId to use for the test
    let test_chain_id: ChainId = 1;

    registry_service
        .register_chain(
            test_chain_id,
            ChainInfo {
                gas_limit: 1000,
                registered_sellers: vec!["0xTestSeller".to_string()],
            },
        )
        .await?;

    println!("ChainId 1 registered with seller 0xTestSeller");

    let chain_ids = registry_service.get_chain_ids().await;
    assert_eq!(chain_ids.len(), 1, "ChainId registration failed");
    assert_eq!(
        chain_ids[0], 1,
        "ChainId does not match the registered value"
    );
    println!("ChainIds registered: {:?}", chain_ids);

    // 2. Setup AuctionManager
    let auction_manager = AuctionManager::new(&registry_service).await;
    println!("AuctionManager created");

    // 3. Create AuctionInfo with start_time in the past and end_time shortly in the future
    let now = current_unix_ms();
    let auction_info = AuctionInfo::new(
        test_chain_id,
        100, // block_number
        "0xTestSeller".to_string(),
        500,        // blockspace_size
        now - 1000, // start_time: 1 second in the past
        now + 2000, // end_time: 2 seconds in the future
        "0xSellerSignature".to_string(),
    );

    println!("AuctionInfo created: {:?}", auction_info);

    // 4. Submit AuctionInfo to the registry
    registry_service
        .submit_auction_info(auction_info.clone())
        .await?;

    println!("AuctionInfo submitted to registry");

    // 5. Start the next auction
    let auction_id_option = auction_manager.start_next_auction(test_chain_id).await;
    assert!(
        auction_id_option.is_some(),
        "Failed to start the next auction"
    );

    let auction_id = auction_id_option.unwrap();
    println!("Auction started with ID: {}", auction_id);

    assert_eq!(
        auction_id, auction_info.id,
        "Auction ID does not match the submitted AuctionInfo"
    );

    // 6. Verify that the auction is ongoing
    let ongoing_auction_id = auction_manager.get_ongoing_auction_id(test_chain_id).await;
    assert_eq!(
        ongoing_auction_id,
        Some(auction_id.clone()),
        "Auction is not marked as ongoing"
    );

    println!(
        "Auction is ongoing with ID: {}",
        ongoing_auction_id.unwrap()
    );

    // 7. Submit bids
    let bid1 = Bid {
        bidder_addr: "0xBidder1".to_string(),
        bid_amount: 1000,
        bidder_signature: "0xBidder1Signature".to_string(),
        tx_list: vec![Tx {
            tx_data: "tx1".to_string(),
        }],
    };

    let bid2 = Bid {
        bidder_addr: "0xBidder2".to_string(),
        bid_amount: 1500, // Highest bid
        bidder_signature: "0xBidder2Signature".to_string(),
        tx_list: vec![Tx {
            tx_data: "tx2".to_string(),
        }],
    };

    let bid3 = Bid {
        bidder_addr: "0xBidder3".to_string(),
        bid_amount: 1200,
        bidder_signature: "0xBidder3Signature".to_string(),
        tx_list: vec![Tx {
            tx_data: "tx3".to_string(),
        }],
    };

    println!("Bids created");

    // Submit bids asynchronously
    let bid1_result = auction_manager
        .submit_bid(test_chain_id, auction_id.clone(), bid1.clone())
        .await;
    assert!(bid1_result.is_ok(), "Failed to submit bid1");
    println!("Bid1 submitted successfully");

    let bid2_result = auction_manager
        .submit_bid(test_chain_id, auction_id.clone(), bid2.clone())
        .await;
    assert!(bid2_result.is_ok(), "Failed to submit bid2");
    println!("Bid2 submitted successfully");

    let bid3_result = auction_manager
        .submit_bid(test_chain_id, auction_id.clone(), bid3.clone())
        .await;
    assert!(bid3_result.is_ok(), "Failed to submit bid3");
    println!("Bid3 submitted successfully");

    // 8. Wait for the auction to end
    // Since the auction ends in ~2 seconds, wait for 3 seconds to ensure it has concluded
    println!("Waiting for auction to end...");
    sleep(tokio::time::Duration::from_secs(3)).await;

    // 9. Verify that the auction has ended
    let ongoing_auction_id_after = auction_manager.get_ongoing_auction_id(test_chain_id).await;
    assert!(
        ongoing_auction_id_after.is_none(),
        "Auction is still marked as ongoing after end time"
    );

    println!("Auction has ended");

    // 10. Retrieve the auction state to verify the winner
    let auction_state_result = auction_manager.request_auction_state(test_chain_id).await;
    match auction_state_result {
        Ok(state) => {
            assert!(state.is_ended, "Auction state is not marked as ended");
            assert_eq!(
                state.winner,
                Some("0xBidder2".to_string()),
                "Winner does not match the highest bid"
            );
            assert_eq!(
                state.highest_bid, 1500,
                "Highest bid amount does not match the expected value"
            );
            assert_eq!(
                state.bids.len(),
                3,
                "Number of bids does not match the expected count"
            );
            println!(
                "Auction state verified: Winner: {}, Highest Bid: {}",
                state.winner.unwrap(),
                state.highest_bid
            );
        }
        Err(e) => panic!("Failed to retrieve auction state: {}", e),
    }

    // 11. Optionally, verify that the worker has processed the auction end message
    println!("Test completed successfully");
    Ok(())
}
