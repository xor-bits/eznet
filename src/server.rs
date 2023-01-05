use crate::socket::{filter::FilterError, Socket};
use futures::{stream::SelectAll, Future, Stream, StreamExt};
use once_cell::sync::OnceCell;
use quinn::{Connecting, Endpoint, Incoming, ServerConfig};
use rustls::{Certificate, PrivateKey};
use std::{io, net::SocketAddr};
use thiserror::Error;
use tracing::instrument;

//

#[derive(Debug, Default)]
pub struct Server {
    sockets: SelectAll<ServerSocket>,
    config: OnceCell<ServerConfig>,
}

#[derive(Debug, Error)]
pub enum BindError {
    #[error("Failed to bind socket: {0}")]
    IoError(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error(transparent)]
    Connect(#[from] quinn::ConnectError),

    #[error(transparent)]
    Connection(#[from] quinn::ConnectionError),

    #[error("Failed to bind socket: {0}")]
    IoError(std::io::Error),

    #[error("Invalid socket address: {0}")]
    InvalidSocketAddress(std::io::Error),

    #[error("No socket address")]
    NoSocketAddress,

    #[error(transparent)]
    FilterError(#[from] FilterError),
}

#[derive(Debug)]
struct ServerSocket {
    endpoint: Endpoint,
    incoming: Incoming,
}

impl Stream for ServerSocket {
    type Item = (Connecting, Endpoint);

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.incoming
            .poll_next_unpin(cx)
            .map(|res| res.map(|conn| (conn, self.endpoint.clone())))
    }
}

//

impl Server {
    /// Self signed certificate
    pub fn new() -> Self {
        Self::default()
    }

    /// Self signed certificate
    pub fn from(addr: impl Into<SocketAddr>) -> Result<Self, BindError> {
        let mut s = Self::new();
        s.bind(addr.into())?;
        Ok(s)
    }

    /// Custom server configuration
    ///
    /// Allows server validation
    pub fn server_config(
        &mut self,
        cert_chain: Vec<Certificate>,
        key: PrivateKey,
    ) -> Result<(), rustls::Error> {
        self.update_config(ServerConfig::with_single_cert(cert_chain, key)?);
        Ok(())
    }

    /// Use self signed certificate
    ///
    /// Clients cannot validate the server
    ///
    /// This is the default
    pub fn insecure_server_config(&mut self) {
        self.update_config(Self::default_server_config())
    }

    /// Bind one address
    pub fn bind(&mut self, addr: SocketAddr) -> Result<Endpoint, BindError> {
        let (endpoint, incoming) = Endpoint::server(
            self.config.get_or_init(Self::default_server_config).clone(),
            addr,
        )?;
        self.sockets.push(ServerSocket {
            endpoint: endpoint.clone(),
            incoming,
        });
        Ok(endpoint)
    }

    /// Take the next connection
    #[instrument(skip_all, fields(sockets = %self.sockets.len()))]
    pub async fn next(
        &mut self,
    ) -> Result<impl Future<Output = Result<Socket, FilterError>>, ConnectError> {
        tracing::debug!("Waiting for the next client");
        let (connecting, incoming) = self
            .sockets
            .next()
            .await
            .ok_or(ConnectError::Connect(quinn::ConnectError::EndpointStopping))?;

        tracing::debug!("A client is connecting");
        Ok(Socket::new(connecting, incoming))
    }

    pub async fn next_wait(&mut self) -> Result<Socket, ConnectError> {
        Ok(self.next().await?.await?)
    }

    fn update_config(&mut self, config: ServerConfig) {
        let _ = self.config.take();
        _ = self.config.set(config);
    }

    fn default_server_config() -> ServerConfig {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let cert_der = cert.serialize_der().unwrap();
        let priv_key = cert.serialize_private_key_der();
        let priv_key = rustls::PrivateKey(priv_key);
        let cert_chain = vec![Certificate(cert_der)];
        ServerConfig::with_single_cert(cert_chain, priv_key).unwrap()
    }
}
