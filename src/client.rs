use crate::{connection::CommonConnection, packet::PacketFlags};
use quinn::{ClientConfig, Endpoint, NewConnection};
use rustls::{client::ServerCertVerifier, Certificate};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};

//

pub struct Client<S, R>
where
    S: Send + Serialize + Unpin + 'static,
    R: Send + DeserializeOwned + Unpin + 'static,
{
    endpoint: Endpoint,
    connection: CommonConnection<S, R>,
}

//

impl<S, R> Client<S, R>
where
    S: Send + Serialize + Unpin + 'static,
    R: Send + DeserializeOwned + Unpin + 'static,
{
    pub async fn new(addr: SocketAddr) -> Self {
        let config = Self::default_config();
        let mut endpoint =
            Endpoint::client(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0).into()).unwrap();
        endpoint.set_default_client_config(config);

        let conn = endpoint.connect(addr, "localhost").unwrap().await.unwrap();

        let remote = conn.connection.remote_address();
        log::debug!("client connected: {remote}");

        Self::from_conn(conn, endpoint).await
    }

    pub async fn from_conn(conn: NewConnection, endpoint: Endpoint) -> Self {
        let connection = CommonConnection::new(conn).await;
        Self {
            endpoint,
            connection,
        }
    }

    /// Self signed certificate verifier
    ///
    /// i.e. accepts everything
    ///
    /// vulnerable to MITM
    pub fn default_config() -> ClientConfig {
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

        let crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(Verifier))
            .with_no_client_auth();
        ClientConfig::new(Arc::new(crypto))
    }

    pub async fn wait_idle(&self) {
        self.endpoint.wait_idle().await;
    }

    pub async fn read(&mut self) -> Option<R> {
        self.connection.read().await
    }

    pub async fn send(&mut self, message: S, flags: PacketFlags) -> Option<()> {
        self.connection.send(message, flags).await
    }
}
