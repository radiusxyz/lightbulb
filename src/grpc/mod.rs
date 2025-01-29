pub mod client;
pub mod utils;

pub mod proto {
    pub mod auction {
        tonic::include_proto!("auction");
    }
    pub use auction::{
        auction_service_client::AuctionServiceClient, AddAuctionRequest, AuctionInfo, AuctionState,
        Bid, GetAuctionStateRequest, SubmitBidsRequest, Tx,
    };
}
