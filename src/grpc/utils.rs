// Implement From traits for converting between gRPC and Rust types

use crate::{core::domain, grpc::proto};

impl From<proto::auction::Tx> for domain::Tx {
    fn from(proto_tx: proto::auction::Tx) -> Self {
        domain::Tx {
            tx_data: proto_tx.tx_data,
        }
    }
}

impl From<domain::Tx> for proto::auction::Tx {
    fn from(tx: domain::Tx) -> Self {
        proto::auction::Tx {
            tx_data: tx.tx_data,
        }
    }
}

impl From<proto::auction::Bid> for domain::Bid {
    fn from(proto_bid: proto::auction::Bid) -> Self {
        domain::Bid {
            bidder_address: proto_bid.bidder_addr,
            bid_amount: proto_bid.bid_amount as u64,
            bidder_signature: proto_bid.bidder_signature,
            tx_list: proto_bid.tx_list.into_iter().map(|tx| tx.into()).collect(),
        }
    }
}

impl From<domain::Bid> for proto::auction::Bid {
    fn from(bid: domain::Bid) -> Self {
        proto::auction::Bid {
            bidder_addr: bid.bidder_address,
            bid_amount: bid.bid_amount as i64,
            bidder_signature: bid.bidder_signature,
            tx_list: bid.tx_list.into_iter().map(|tx| tx.into()).collect(),
        }
    }
}

impl From<proto::auction::AuctionInfo> for domain::AuctionInfo {
    fn from(proto_auction_info: proto::auction::AuctionInfo) -> Self {
        domain::AuctionInfo {
            auction_id: proto_auction_info.auction_id,
            chain_id: proto_auction_info.chain_id as domain::ChainId,
            block_number: proto_auction_info.block_number as u64,
            seller_address: proto_auction_info.seller_address,
            blockspace_size: proto_auction_info.blockspace_size as u64,
            start_time: proto_auction_info.start_time as u64,
            end_time: proto_auction_info.end_time as u64,
            seller_signature: proto_auction_info.seller_signature,
        }
    }
}

impl From<domain::AuctionInfo> for proto::auction::AuctionInfo {
    fn from(auction_info: domain::AuctionInfo) -> Self {
        proto::auction::AuctionInfo {
            auction_id: auction_info.auction_id,
            chain_id: auction_info.chain_id as i64,
            block_number: auction_info.block_number as i64,
            seller_address: auction_info.seller_address,
            blockspace_size: auction_info.blockspace_size as i64,
            start_time: auction_info.start_time as i64,
            end_time: auction_info.end_time as i64,
            seller_signature: auction_info.seller_signature,
        }
    }
}

impl From<proto::auction::AuctionState> for domain::AuctionState {
    fn from(proto_auction_state: proto::auction::AuctionState) -> Self {
        domain::AuctionState {
            auction_info: proto_auction_state.auction_info.unwrap().into(),
            bid_list: proto_auction_state
                .bid_list
                .into_iter()
                .map(|bid| bid.into())
                .collect(),
            sorted_tx_list: proto_auction_state
                .sorted_tx_list
                .into_iter()
                .map(|tx| tx.into())
                .collect(),
            is_ended: proto_auction_state.is_ended,
        }
    }
}

impl From<domain::AuctionState> for proto::auction::AuctionState {
    fn from(auction_state: domain::AuctionState) -> Self {
        proto::auction::AuctionState {
            auction_info: Some(auction_state.auction_info.into()),
            bid_list: auction_state
                .bid_list
                .into_iter()
                .map(|bid| bid.into())
                .collect(),
            sorted_tx_list: auction_state
                .sorted_tx_list
                .into_iter()
                .map(|tx| tx.into())
                .collect(),
            is_ended: auction_state.is_ended,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auction_info_from_proto() {
        let proto_auction_info = proto::auction::AuctionInfo {
            auction_id: "test_auction_id".to_string(),
            chain_id: 1,
            block_number: 100,
            seller_address: "0xTestSeller".to_string(),
            blockspace_size: 500,
            start_time: 1000,
            end_time: 5000,
            seller_signature: "0xSellerSignature".to_string(),
        };

        let auction_info: domain::AuctionInfo = proto_auction_info.into();

        assert_eq!(auction_info.auction_id, "test_auction_id");
        assert_eq!(auction_info.chain_id, 1);
        assert_eq!(auction_info.block_number, 100);
        assert_eq!(auction_info.seller_address, "0xTestSeller");
        assert_eq!(auction_info.blockspace_size, 500);
        assert_eq!(auction_info.start_time, 1000);
        assert_eq!(auction_info.end_time, 5000);
        assert_eq!(auction_info.seller_signature, "0xSellerSignature");
    }
}
