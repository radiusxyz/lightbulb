use crate::{
    core::{
        domain::{AuctionId, AuctionInfo, AuctionState, Bid, ChainId},
        errors::AuctionError,
    },
    rpc::internal::{api::auction_worker::AuctionWorkerApi, proto},
};

#[derive(Clone)]
pub struct GrpcAuctionWorkerClient {
    client: proto::AuctionServiceClient<tonic::transport::Channel>,
}

impl GrpcAuctionWorkerClient {
    pub async fn connect(addr: &str) -> Result<Self, tonic::transport::Error> {
        let client = proto::AuctionServiceClient::connect(addr.to_string()).await?;
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl AuctionWorkerApi for GrpcAuctionWorkerClient {
    async fn add_auction(&mut self, auction_info: AuctionInfo) -> Result<bool, AuctionError> {
        let request = tonic::Request::new(proto::AddAuctionRequest {
            auction_info: Some(auction_info.into()),
        });

        let response = self.client.add_auction(request).await;

        match response {
            Ok(resp) => Ok(resp.into_inner().success),
            Err(status) => Err(AuctionError::GrpcError(status.message().to_string())),
        }
    }

    async fn submit_bids(
        &mut self,
        chain_id: ChainId,
        auction_id: AuctionId,
        bid_list: Vec<Bid>,
    ) -> Result<bool, AuctionError> {
        let request = tonic::Request::new(proto::SubmitBidsRequest {
            chain_id: chain_id as i64,
            auction_id,
            bid_list: bid_list.into_iter().map(|bid| bid.into()).collect(),
        });

        let response = self.client.submit_bids(request).await;

        match response {
            Ok(resp) => Ok(resp.into_inner().success),
            Err(status) => Err(AuctionError::GrpcError(status.message().to_string())),
        }
    }

    async fn get_auction_state(&mut self, chain_id: ChainId) -> Result<AuctionState, AuctionError> {
        let request = tonic::Request::new(proto::GetAuctionStateRequest {
            chain_id: chain_id as i64,
        });

        let response = self.client.get_auction_state(request).await;

        match response {
            Ok(resp) => Ok(resp.into_inner().state.unwrap().into()),
            Err(status) => Err(AuctionError::GrpcError(status.message().to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio;

    use super::*;
    use crate::core::{
        domain::{AuctionInfo, Bid, Tx},
        utils::helpers::current_unix_ms,
    };

    const TEST_SERVER_ADDR: &str = "http://localhost:50051";

    #[tokio::test]
    async fn test_connect() {
        let client = GrpcAuctionWorkerClient::connect(TEST_SERVER_ADDR).await;
        assert!(client.is_ok(), "Failed to connect to gRPC server");
    }

    #[tokio::test]
    async fn test_add_auction() {
        let mut client = GrpcAuctionWorkerClient::connect(TEST_SERVER_ADDR)
            .await
            .unwrap();

        let auction_info = AuctionInfo {
            auction_id: "test_auction_id".to_string(),
            chain_id: 1,
            block_number: 1,
            seller_address: "0xSeller".to_string(),
            blockspace_size: 100,
            start_time: current_unix_ms() + 1000,
            end_time: current_unix_ms() + 6000,
            seller_signature: "SellerSignature".to_string(),
        };

        let result = client.add_auction(auction_info).await;
        assert!(result.is_ok(), "Failed to add auction");
        assert!(result.unwrap(), "Auction was not added successfully");
    }

    #[tokio::test]
    async fn test_submit_bids() {
        let mut client = GrpcAuctionWorkerClient::connect(TEST_SERVER_ADDR)
            .await
            .unwrap();

        let chain_id = 1;
        let auction_id = "test_auction_id".to_string();

        let bid_list = vec![
            Bid {
                bidder_address: "0xAlice".to_string(),
                bid_amount: 100,
                bidder_signature: "AliceSignature".to_string(),
                tx_list: vec![Tx {
                    tx_data: "tx_data".to_string(),
                }],
            },
            Bid {
                bidder_address: "0xBob".to_string(),
                bid_amount: 120,
                bidder_signature: "BobSignature".to_string(),
                tx_list: vec![Tx {
                    tx_data: "tx_data".to_string(),
                }],
            },
        ];

        let result = client.submit_bids(chain_id, auction_id, bid_list).await;
        assert!(result.is_ok(), "Failed to submit bids");
        assert!(result.unwrap(), "Bids were not submitted successfully");
    }

    #[tokio::test]
    async fn test_get_auction_state() {
        let mut client = GrpcAuctionWorkerClient::connect(TEST_SERVER_ADDR)
            .await
            .unwrap();

        let chain_id = 1;
        let result = client.get_auction_state(chain_id).await;

        assert!(result.is_ok(), "Failed to get auction state");

        let auction_state = result.unwrap();
        println!("Auction state: {:?}", auction_state);
    }
}
