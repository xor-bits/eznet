use crate::{
    attempt_all,
    socket::{ConnectError, Socket},
};
use futures::StreamExt;
use quinn::{Endpoint, Incoming, ServerConfig};
use rustls::Certificate;
use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
};
use thiserror::Error;

//

pub struct Listener {
    endpoint: Endpoint,
    incoming: Incoming,
}

#[derive(Debug, Error)]
pub enum BindError {
    #[error("bind error ({0})")]
    IoError(#[from] io::Error),

    #[error("invalid socket address ({0})")]
    InvalidSocketAddress(io::Error),

    #[error("no socket address")]
    NoSocketAddress,

    #[error("tls error ({0})")]
    TLSError(#[from] rustls::Error),

    #[error("self signed cert error ({0})")]
    SelfSignedCertError(#[from] rcgen::RcgenError),
}

//

impl Listener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self, BindError> {
        let config = Self::default_config()?;
        let addrs = addr
            .to_socket_addrs()
            .map_err(BindError::InvalidSocketAddress)?;

        attempt_all(
            addrs,
            move |addr| Self::from_config(addr, config.clone()),
            BindError::NoSocketAddress,
        )
    }

    /// Self signed certificate
    pub fn default_config() -> Result<ServerConfig, BindError> {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
        let cert_der = cert.serialize_der()?;
        let priv_key = cert.serialize_private_key_der();
        let priv_key = rustls::PrivateKey(priv_key);
        let cert_chain = vec![Certificate(cert_der)];

        Ok(ServerConfig::with_single_cert(cert_chain, priv_key)?)
    }

    pub fn from_config(addr: SocketAddr, config: ServerConfig) -> Result<Self, BindError> {
        let (endpoint, incoming) = Endpoint::server(config, addr)?;
        Ok(Self { endpoint, incoming })
    }

    pub async fn next(&mut self) -> Result<Socket, ConnectError> {
        let connecting = self
            .incoming
            .next()
            .await
            .ok_or(ConnectError::Connect(quinn::ConnectError::EndpointStopping))?;
        let connection = connecting.await?;
        Socket::new(connection, self.endpoint.clone()).await
    }
}
