use thiserror::Error;

use crate::rpc::external::server::ServerKind;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("{0} server error: {1}")]
    IoError(ServerKind, #[source] std::io::Error),
    #[error("Custom error: {0}")]
    Custom(String),
}
