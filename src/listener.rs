use crate::socket::{ConnectError, Socket};
use futures::StreamExt;
use quinn::{Endpoint, Incoming, ServerConfig};
use rustls::Certificate;
use std::{io, net::SocketAddr};
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

    #[error("tls error ({0})")]
    TLSError(#[from] rustls::Error),

    #[error("self signed cert error ({0})")]
    SelfSignedCertError(#[from] rcgen::RcgenError),
}

//

impl Listener {
    pub fn bind(addr: SocketAddr) -> Result<Self, BindError> {
        let config = Self::default_config()?;
        Self::from_config(config, addr)
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

    pub fn from_config(config: ServerConfig, addr: SocketAddr) -> Result<Self, BindError> {
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
