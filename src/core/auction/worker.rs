use crate::core::{
    domain::{AuctionId, AuctionInfo, AuctionState, Bid, ChainId},
    errors::AuctionError,
};

#[async_trait::async_trait]
pub trait AuctionWorkerClient {
    async fn add_auction(&mut self, auction_info: AuctionInfo) -> Result<bool, AuctionError>;
    async fn submit_bids(
        &mut self,
        chain_id: ChainId,
        auction_id: AuctionId,
        bid: Vec<Bid>,
    ) -> Result<bool, AuctionError>;
    async fn get_auction_state(&mut self, chain_id: ChainId) -> Result<AuctionState, AuctionError>;
}
