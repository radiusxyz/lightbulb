use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::core::auction::AuctionManager;
use crate::utils::helpers::current_unix_ms;

/// The `AuctionWorker` is an actor-like structure that runs in the background,
/// periodically checking auctions for updates (e.g., sorting bids by highest amount, ending auctions).
#[derive(Clone)]
pub struct AuctionWorker {
    manager: Arc<AuctionManager>,
}

impl AuctionWorker {
    /// Creates a new `AuctionWorker`.
    pub fn new(manager: Arc<AuctionManager>) -> Self {
        AuctionWorker { manager }
    }

    /// An infinite loop that processes auctions every 500ms.
    /// In production, you might adjust the interval or make it configurable.
    pub async fn run(&self) {
        loop {
            if let Err(e) = self.process_auctions().await {
                eprintln!("Error processing auctions: {}", e);
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    /// Processes all ongoing auctions, marking them as ended if the end time has passed,
    /// and selecting the top bid as the winner if there are any bids.
    async fn process_auctions(&self) -> Result<(), String> {
        let now = current_unix_ms();
        let mut auctions = self.manager.ongoing_auctions.write().await;

        for (_chain_id, auction_state) in auctions.iter_mut() {
            if auction_state.is_ended {
                continue;
            }

            let sla = &auction_state.auction_info;

            // Mark auction as ended if the end time has passed
            if now >= sla.end_time {
                auction_state.is_ended = true;
                continue;
            }

            // Skip processing if the auction hasn't started yet
            if now < sla.start_time {
                continue;
            }

            // Sort bids in descending order by bid_amount
            auction_state
                .bids
                .sort_unstable_by(|a, b| b.bid_amount.cmp(&a.bid_amount));

            // Assign the top bidder as the winner, if any
            if let Some(top_bid) = auction_state.bids.first() {
                auction_state.highest_bid = top_bid.bid_amount;
                auction_state.winner = Some(top_bid.bidder_addr.clone());
            }
        }

        Ok(())
    }
}
