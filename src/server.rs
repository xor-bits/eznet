use crate::socket::{ConnectError, Socket};
use futures::StreamExt;
use once_cell::sync::OnceCell;
use quinn::{Endpoint, Incoming, ServerConfig};
use rustls::{Certificate, PrivateKey};
use std::{io, net::SocketAddr};
use thiserror::Error;

//

#[derive(Debug, Default)]
pub struct Server {
    sockets: Vec<(Endpoint, Incoming)>,
    config: OnceCell<ServerConfig>,
}

#[derive(Debug, Error)]
pub enum BindError {
    #[error("Failed to bind socket: {0}")]
    IoError(#[from] io::Error),
}

//

impl Server {
    /// Self signed certificate
    pub fn new() -> Self {
        Self::default()
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
    pub fn server_config_insecure(&mut self) {
        self.update_config(Self::default_server_config())
    }

    /// Bind one address
    pub fn bind(&mut self, addr: SocketAddr) -> Result<Endpoint, BindError> {
        let (endpoint, incoming) = Endpoint::server(
            self.config.get_or_init(Self::default_server_config).clone(),
            addr,
        )?;
        self.sockets.push((endpoint.clone(), incoming));
        Ok(endpoint)
    }

    /// Bind multiple addresses
    pub fn bind_all<'s>(
        &'s mut self,
        addr: impl Iterator<Item = SocketAddr> + 's,
    ) -> impl Iterator<Item = Result<Endpoint, BindError>> + 's {
        let config = self.config.get_or_init(Self::default_server_config);
        addr.map(|addr| {
            let (endpoint, incoming) = Endpoint::server(config.clone(), addr)?;
            self.sockets.push((endpoint.clone(), incoming));
            Ok(endpoint)
        })
    }

    /// Take the next connection
    pub async fn next(&mut self) -> Result<Socket, ConnectError> {
        let (connecting, index, _) = futures::future::select_all(
            self.sockets.iter_mut().map(|(_, incoming)| incoming.next()),
        )
        .await;

        let connecting =
            connecting.ok_or(ConnectError::Connect(quinn::ConnectError::EndpointStopping))?;

        // TODO: move to socket
        let connection = connecting.await?;

        Ok(Socket::new(
            connection,
            self.sockets.get(index).unwrap().0.clone(),
        ))
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
