use crate::socket::{filter::FilterError, Socket};
use futures::{stream::FuturesUnordered, StreamExt};
use once_cell::sync::OnceCell;
use quinn::{ClientConfig, Endpoint};
use rustls::{client::ServerCertVerifier, Certificate};
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    sync::Arc,
};
use thiserror::Error;
use tokio::net::{lookup_host, ToSocketAddrs};

//

// Lazy initialized dual stack (IPv4 + IPv6) client
#[derive(Debug, Default)]
pub struct Client {
    client_v4: OnceCell<Endpoint>,
    client_v6: OnceCell<Endpoint>,
    config: OnceCell<ClientConfig>,
}

#[derive(Debug, Error)]
pub enum ClientConnectError {
    #[error("Failed to create a client Endpoint: {0}")]
    SocketBind(#[from] std::io::Error),

    #[error("Failed to create a client Endpoint: No addresses given")]
    NoSocketAddrs,

    #[error(transparent)]
    Connect(#[from] quinn::ConnectError),

    #[error(transparent)]
    Connection(#[from] quinn::ConnectionError),

    #[error(transparent)]
    Filter(#[from] FilterError),
}

//

impl Client {
    /// The default configuration is vulnerable to MITM attacks
    ///
    /// use [`Self::client_config`] to set
    pub fn new() -> Self {
        Self::default()
    }

    /// Custom client configuration
    ///
    /// Allows server validation
    pub fn client_config(&mut self, config: rustls::ClientConfig) {
        self.update_config(ClientConfig::new(Arc::new(config)));
    }

    /// Does not validate the server
    ///
    /// Vulnerable to MITM attacks
    ///
    /// This is the default
    pub fn client_config_insecure(&mut self) {
        self.update_config(Self::default_client_config());
    }

    pub async fn connect(
        &self,
        addr: SocketAddr,
        server_name: &str,
    ) -> Result<Socket, ClientConnectError> {
        // a client can take any port that's available
        let (listen, endpoint) = match addr {
            SocketAddr::V4(_) => (
                SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0).into(),
                &self.client_v4,
            ),
            SocketAddr::V6(_) => (
                SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0).into(),
                &self.client_v6,
            ),
        };

        let endpoint = endpoint.get_or_try_init(|| {
            let mut client = quinn::Endpoint::client(listen)?;
            client.set_default_client_config(
                self.config.get_or_init(Self::default_client_config).clone(),
            );
            Ok::<_, ClientConnectError>(client)
        })?;

        let conn = endpoint.connect(addr, server_name)?.await?;

        Ok(Socket::new(conn, endpoint.clone()).await?)
    }

    pub async fn connect_to<A: ToSocketAddrs>(
        &self,
        addr: A,
        server_name: &str,
    ) -> Result<Socket, ClientConnectError> {
        let mut futures = lookup_host(addr)
            .await?
            .map(|addr| self.connect(addr, server_name))
            .collect::<FuturesUnordered<_>>();

        let mut err = ClientConnectError::NoSocketAddrs;
        while let Some(socket) = futures.next().await {
            match socket {
                socket @ Ok(_) => return socket,
                Err(e) => err = e,
            }
        }
        Err(err)
    }

    fn update_config(&mut self, config: ClientConfig) {
        *self = <_>::default();
        _ = self.config.set(config);
    }

    fn default_client_config() -> ClientConfig {
        struct Verifier;
        impl ServerCertVerifier for Verifier {
            fn verify_server_cert(
                &self,
                _end_entity: &Certificate,
                _intermediates: &[Certificate],
                _server_name: &rustls::ServerName,
                _scts: &mut dyn Iterator<Item = &[u8]>,
                _ocsp_response: &[u8],
                _now: std::time::SystemTime,
            ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
                Ok(rustls::client::ServerCertVerified::assertion())
            }
        }

        ClientConfig::new(Arc::new(
            rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(Arc::new(Verifier))
                .with_no_client_auth(),
        ))
    }
}
