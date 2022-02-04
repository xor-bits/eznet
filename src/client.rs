use crate::{
    connection::{CommonConnection, Connection},
    packet::PacketFlags,
};
use async_trait::async_trait;
use bytes::Bytes;
use quinn::{ClientConfig, Endpoint, NewConnection};
use rustls::{client::ServerCertVerifier, Certificate};
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};

//

pub struct Client {
    endpoint: Endpoint,
    connection: CommonConnection,
}

//

impl Client {
    pub async fn new(addr: SocketAddr) -> Self {
        let config = Self::default_config();
        let mut endpoint =
            Endpoint::client(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0).into()).unwrap();
        endpoint.set_default_client_config(config);

        let conn = endpoint.connect(addr, "localhost").unwrap().await.unwrap();

        let remote = conn.connection.remote_address();
        log::debug!("client connected: {remote}");

        Self::from_conn(conn, endpoint)
    }

    pub fn from_conn(conn: NewConnection, endpoint: Endpoint) -> Self {
        let connection = CommonConnection::new(conn);
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
}

//

#[async_trait]
impl Connection for Client {
    async fn read(&mut self) -> Bytes {
        self.connection.read().await
    }

    async fn send(&mut self, message: Bytes, flags: PacketFlags) {
        self.connection.send(message, flags).await
    }
}
