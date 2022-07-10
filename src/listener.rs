use crate::socket::{ConnectError, Socket};
use futures::StreamExt;
use quinn::{Endpoint, Incoming, ServerConfig};
use rustls::Certificate;
use std::net::SocketAddr;

//

pub struct Listener {
    endpoint: Endpoint,
    incoming: Incoming,
}

//

impl Listener {
    pub fn bind(addr: SocketAddr) -> Self {
        let config = Self::default_config();
        Self::from_config(config, addr)
    }

    /// Self signed certificate
    pub fn default_config() -> ServerConfig {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let cert_der = cert.serialize_der().unwrap();
        let priv_key = cert.serialize_private_key_der();
        let priv_key = rustls::PrivateKey(priv_key);
        let cert_chain = vec![Certificate(cert_der)];

        ServerConfig::with_single_cert(cert_chain, priv_key).unwrap()
    }

    pub fn from_config(config: ServerConfig, addr: SocketAddr) -> Self {
        let (endpoint, incoming) = Endpoint::server(config, addr).unwrap();
        Self { endpoint, incoming }
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
