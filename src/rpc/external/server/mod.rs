pub mod bid;

use std::net::SocketAddr;

use jsonrpsee::{
    server::{Server, ServerBuilder, ServerHandle},
    RpcModule,
};
use tower::{
    layer::util::{Identity, Stack},
    ServiceBuilder,
};
use tower_http::cors::CorsLayer;

use crate::rpc::{errors::RpcError, utils::create_cors_layer};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ServerKind {
    Http(SocketAddr),
    WS(SocketAddr),
    WsHttp(SocketAddr),
}

impl std::fmt::Display for ServerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerKind::Http(addr) => write!(f, "{} (HTTP-RPC server)", addr),
            ServerKind::WS(addr) => write!(f, "{} (WS-RPC server)", addr),
            ServerKind::WsHttp(addr) => write!(f, "{} (WS-HTTP-RPC server)", addr),
        }
    }
}

/// Enum representing a server built either with or without CORS middleware.
pub enum WsHttpServerKind {
    Plain(Server),
    WithCors(Server<Stack<CorsLayer, Identity>>),
}

impl WsHttpServerKind {
    /// Builds a server using the provided ServerBuilder.
    /// If `cors_origin` is Some, a CorsLayer is created via `create_cors_layer`
    /// and added as middleware.
    pub async fn build(
        builder: ServerBuilder<Identity, Identity>,
        socket_addr: SocketAddr,
        cors_origin: Option<String>,
        server_kind: ServerKind,
    ) -> Result<Self, RpcError> {
        if let Some(origin) = cors_origin {
            let cors = create_cors_layer(&origin).map_err(|e| RpcError::Custom(e.to_string()))?;
            let server = builder
                .set_http_middleware(ServiceBuilder::new().layer(cors))
                .build(socket_addr)
                .await
                .map_err(|err| RpcError::IoError(server_kind, err))?;
            Ok(WsHttpServerKind::WithCors(server))
        } else {
            let server = builder
                .build(socket_addr)
                .await
                .map_err(|err| RpcError::IoError(server_kind, err))?;
            Ok(WsHttpServerKind::Plain(server))
        }
    }

    /// Returns the local address of the server.
    pub fn local_addr(&self) -> Result<SocketAddr, RpcError> {
        match self {
            WsHttpServerKind::Plain(server) => server
                .local_addr()
                .map_err(|e| RpcError::Custom(e.to_string())),
            WsHttpServerKind::WithCors(server) => server
                .local_addr()
                .map_err(|e| RpcError::Custom(e.to_string())),
        }
    }

    /// Starts the server with the provided RPC module and returns a ServerHandle.
    pub async fn start(self, module: RpcModule<()>) -> Result<ServerHandle, RpcError> {
        match self {
            WsHttpServerKind::Plain(server) => Ok(server.start(module)),
            WsHttpServerKind::WithCors(server) => Ok(server.start(module)),
        }
    }
}

#[derive(Default)]
pub struct RpcServerConfig {
    http_addr: Option<SocketAddr>,
    ws_addr: Option<SocketAddr>,
    cors_origin: Option<String>,
}

impl RpcServerConfig {
    /// Creates a new RPC server configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the HTTP server binding address.
    pub fn with_http_addr(mut self, addr: SocketAddr) -> Self {
        self.http_addr = Some(addr);
        self
    }

    /// Sets the WS server binding address.
    pub fn with_ws_addr(mut self, addr: SocketAddr) -> Self {
        self.ws_addr = Some(addr);
        self
    }

    /// Sets the allowed CORS origin(s). For example: "*" or "http://example.com, http://other.com"
    pub fn with_cors_origin(mut self, origin: impl Into<String>) -> Self {
        self.cors_origin = Some(origin.into());
        self
    }

    /// Builds the RPC server using the current configuration.
    /// Returns an RpcServer instance.
    pub async fn build(self) -> Result<RpcServer, RpcError> {
        let http_addr = self.http_addr.ok_or_else(|| {
            RpcError::Custom("HTTP address not set in configuration.".to_string())
        })?;
        let ws_addr = self
            .ws_addr
            .ok_or_else(|| RpcError::Custom("WS address not set in configuration.".to_string()))?;

        let http_builder = ServerBuilder::default().http_only();
        let ws_builder = ServerBuilder::default().ws_only();

        let http_server = WsHttpServerKind::build(
            http_builder,
            http_addr,
            self.cors_origin.clone(),
            ServerKind::Http(http_addr),
        )
        .await?;
        let ws_server = WsHttpServerKind::build(
            ws_builder,
            ws_addr,
            self.cors_origin,
            ServerKind::WS(ws_addr),
        )
        .await?;

        // Return an RpcServer instance.
        Ok(RpcServer {
            http_server,
            ws_server,
        })
    }
}

pub struct RpcServer {
    http_server: WsHttpServerKind,
    ws_server: WsHttpServerKind,
}

impl RpcServer {
    /// Starts the RPC server with the provided RPC module.
    /// Returns an RpcServerHandle for controlling the running servers.
    pub async fn start(self, module: RpcModule<()>) -> Result<RpcServerHandle, RpcError> {
        let http_handle = self.http_server.start(module.clone()).await?;
        let ws_handle = self.ws_server.start(module).await?;
        Ok(RpcServerHandle {
            http: Some(http_handle),
            ws: Some(ws_handle),
        })
    }
}

pub struct RpcServerHandle {
    pub http: Option<ServerHandle>,
    pub ws: Option<ServerHandle>,
}

impl RpcServerHandle {
    /// Stops both the HTTP and WS servers.
    pub fn stop(&self) -> Result<(), RpcError> {
        if let Some(handle) = &self.http {
            handle.stop().map_err(|e| RpcError::Custom(e.to_string()))?;
        }
        if let Some(handle) = &self.ws {
            handle.stop().map_err(|e| RpcError::Custom(e.to_string()))?;
        }
        Ok(())
    }
}

impl Drop for RpcServerHandle {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    use jsonrpsee::{server::ServerBuilder, RpcModule};

    use super::*;

    #[tokio::test]
    async fn test_create_cors_layer_wildcard() {
        let cors_layer = create_cors_layer("*").unwrap();
        println!("{:?}", cors_layer);
    }

    #[tokio::test]
    async fn test_create_cors_layer_valid_domains() {
        let cors_layer = create_cors_layer("http://example.com,http://other.com").unwrap();
        println!("{:?}", cors_layer);
    }

    #[tokio::test]
    async fn test_create_cors_layer_wildcard_in_list() {
        let cors_layer = create_cors_layer("http://example.com,*");
        assert!(cors_layer.is_err());
    }

    #[tokio::test]
    async fn test_ws_http_server_kind_build_plain() {
        let builder = ServerBuilder::default().http_only();
        let socket_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080));
        let server_kind = ServerKind::Http(socket_addr);
        let server = WsHttpServerKind::build(builder, socket_addr, None, server_kind).await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_ws_http_server_kind_build_with_cors() {
        let builder = ServerBuilder::default().http_only();
        let socket_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080));
        let server_kind = ServerKind::Http(socket_addr);
        let cors_origin = Some("http://example.com".to_string());
        let server = WsHttpServerKind::build(builder, socket_addr, cors_origin, server_kind).await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_rpc_server_config_build_and_start() {
        let http_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080));
        let ws_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8081));
        let mut module = RpcModule::new(());
        module
            .register_method(
                "say_hello",
                |_, _, _| -> Result<&str, jsonrpsee_types::ErrorCode> { Ok("Hello, world!") },
            )
            .expect("Method registration failed");
        let config = RpcServerConfig::new()
            .with_http_addr(http_addr)
            .with_ws_addr(ws_addr)
            .with_cors_origin("http://example.com");
        let rpc_server = config.build().await;
        assert!(rpc_server.is_ok());
        let server_handle = rpc_server.unwrap().start(module).await;
        assert!(server_handle.is_ok());
    }
}
