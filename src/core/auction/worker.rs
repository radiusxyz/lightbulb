use std::sync::Arc;

use tokio::{
    sync::{mpsc::Sender, RwLock},
    time::{sleep, Duration},
};

use crate::domain::{
    AuctionId, AuctionInfo, AuctionState, Bid, ChainId, Tx, WorkerMessage, WorkerMessageType,
};
use crate::utils::{errors::AuctionError, helpers::current_unix_ms};

/// `AuctionWorker` manages the state (`AuctionState`) for a specific `ChainId`.
/// If there is no ongoing auction, it remains idle. When an auction starts,
/// it updates the state and handles all auction-related logic internally.
#[derive(Clone)]
pub struct AuctionWorker {
    /// The chain that this worker is responsible for
    chain_id: ChainId,

    /// Tracks the current `AuctionState`. If there is no active auction, it is `None`.
    state: Arc<RwLock<Option<AuctionState>>>,

    /// Sender for notifying the manager when an auction ends or is processing
    result_sender: Sender<WorkerMessage>,
}

impl AuctionWorker {
    /// Creates a new `AuctionWorker`.
    /// Initially, there is no active auction, so the `state` is `None`.
    pub fn new(chain_id: ChainId, result_sender: Sender<WorkerMessage>) -> Self {
        AuctionWorker {
            chain_id,
            state: Arc::new(RwLock::new(None)),
            result_sender,
        }
    }

    /// Main worker loop. If there is an active auction, it periodically checks
    /// whether it has ended, sorts bids, determines the highest bidder, etc.
    pub async fn run(&self) {
        loop {
            if let Err(e) = self.process_auction().await {
                eprintln!("[Worker {}] Error processing auction: {}", self.chain_id, e);
            }

            // Sleep 500ms before checking again
            sleep(Duration::from_millis(500)).await;
        }
    }

    // ------------------------------------------------------------------------
    // Auction management methods
    // ------------------------------------------------------------------------

    /// Starts a new auction. Overwrites any existing auction state if one was already in progress.
    pub async fn start_auction(
        &self,
        auction_id: AuctionId,
        info: AuctionInfo,
    ) -> Result<(), AuctionError> {
        let mut guard = self.state.write().await;
        let new_state = AuctionState::new(info);
        println!(
            "[Worker {}] Starting new auction with ID: {}",
            self.chain_id, auction_id
        );
        *guard = Some(new_state);
        Ok(())
    }

    /// Submits a bid. Returns an error if the auction is already ended or does not exist.
    pub async fn submit_bid(
        &self,
        auction_id: AuctionId,
        bid: Bid,
    ) -> Result<String, AuctionError> {
        let mut guard = self.state.write().await;
        if let Some(ref mut auction_state) = *guard {
            if auction_state.is_ended {
                return Err(AuctionError::AuctionEnded);
            }

            // Potential place to check if the provided auction_id matches the current state's ID
            if auction_state.auction_info.id != auction_id {
                return Err(AuctionError::InvalidAuctionId(auction_id));
            }

            auction_state.bids.push(bid);

            Ok(format!(
                "[Worker {}] ACK: Auction {} bid accepted.",
                self.chain_id, auction_id
            ))
        } else {
            Err(AuctionError::NoAuctions)
        }
    }

    /// Returns the current auction's ID and `AuctionInfo`.
    pub async fn request_sale_info(&self) -> Result<(AuctionId, AuctionInfo), AuctionError> {
        let guard = self.state.read().await;
        if let Some(ref auction_state) = *guard {
            let info = auction_state.auction_info.clone();
            let auction_id = info.id.clone();
            Ok((auction_id, info))
        } else {
            Err(AuctionError::NoAuctions)
        }
    }

    /// Returns the most recent ToB (Top-of-Block) information, i.e., the list of transactions
    /// from the current highest bidder. If there is no winner yet, returns an empty list.
    pub async fn get_latest_tob(&self) -> Result<Vec<Tx>, AuctionError> {
        let guard = self.state.read().await;
        if let Some(ref auction_state) = *guard {
            if let Some(ref winner_addr) = auction_state.winner {
                let tx_list = auction_state
                    .bids
                    .iter()
                    .find(|b| &b.bidder_addr == winner_addr)
                    .map(|b| b.tx_list.clone())
                    .unwrap_or_default();
                Ok(tx_list)
            } else {
                Ok(Vec::new())
            }
        } else {
            Err(AuctionError::NoAuctions)
        }
    }

    /// Returns the entire current auction state.
    pub async fn get_auction_state(&self) -> Result<AuctionState, AuctionError> {
        let guard = self.state.read().await;
        if let Some(ref auction_state) = *guard {
            Ok(auction_state.clone())
        } else {
            Err(AuctionError::NoAuctions)
        }
    }

    // ------------------------------------------------------------------------
    // Internal loop handling: determines if an auction should end, picks the highest bidder, etc.
    // ------------------------------------------------------------------------
    async fn process_auction(&self) -> Result<(), String> {
        let mut guard = self.state.write().await;
        if let Some(ref mut auction_state) = *guard {
            // If the auction is already ended, do nothing
            if auction_state.is_ended {
                return Ok(());
            }

            let now = current_unix_ms();
            let info = &auction_state.auction_info;

            // If the auction hasn't started yet, do nothing
            if now < info.start_time {
                return Ok(());
            }

            // Check if auction has ended
            if now >= info.end_time {
                auction_state.is_ended = true;
                // Sort bids by highest amount
                auction_state
                    .bids
                    .sort_unstable_by(|a, b| b.bid_amount.cmp(&a.bid_amount));
                if let Some(top_bid) = auction_state.bids.first() {
                    auction_state.highest_bid = top_bid.bid_amount;
                    auction_state.winner = Some(top_bid.bidder_addr.clone());
                }

                let auction_id = info.id.clone();
                self.send_worker_message(WorkerMessageType::AuctionEnded, auction_id)
                    .await?;

                return Ok(());
            }

            // If the auction is ongoing, you could sort bids to always know the highest
            auction_state
                .bids
                .sort_unstable_by(|a, b| b.bid_amount.cmp(&a.bid_amount));
            if let Some(top_bid) = auction_state.bids.first() {
                auction_state.highest_bid = top_bid.bid_amount;
                auction_state.winner = Some(top_bid.bidder_addr.clone());
            }
            self.send_worker_message(WorkerMessageType::AuctionProcessing, info.id.clone())
                .await?;
        }
        Ok(())
    }

    // ------------------------------------------------------------------------
    // Helper methods
    // ------------------------------------------------------------------------

    /// Sends a `WorkerMessage` to the manager.
    async fn send_worker_message(
        &self,
        message_type: WorkerMessageType,
        auction_id: AuctionId,
    ) -> Result<(), String> {
        let message = WorkerMessage {
            message_type,
            chain_id: self.chain_id,
            auction_id,
        };
        self.result_sender
            .send(message)
            .await
            .map_err(|e| format!("Failed to send auction message: {}", e))
    }
}
