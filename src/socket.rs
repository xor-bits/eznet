use crate::{attempt_all_async, filter::FilterError, inner::SocketInner, packet::Packet};
use quinn::{ClientConfig, Endpoint, NewConnection};
use quinn_proto::ConnectionStats;
use rustls::{client::ServerCertVerifier, Certificate};
use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs},
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};
use thiserror::Error;
use tokio::sync::mpsc::{
    self,
    error::{TryRecvError, TrySendError},
};

//

#[derive(Debug)]
pub struct Socket {
    inner: Option<SocketInner>,
}

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("connect error ({0})")]
    Connect(#[from] quinn::ConnectError),

    #[error("connection error ({0})")]
    Connection(#[from] quinn::ConnectionError),

    #[error("failed to bind socket ({0})")]
    IoError(#[from] io::Error),

    #[error("invalid socket address ({0})")]
    InvalidSocketAddress(io::Error),

    #[error("no socket address")]
    NoSocketAddress,

    #[error("peer filtered out ({0})")]
    FilterError(#[from] FilterError),
}

//

impl Socket {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, ConnectError> {
        let config = Self::default_config();
        let addrs = addr
            .to_socket_addrs()
            .map_err(ConnectError::InvalidSocketAddress)?;

        attempt_all_async(
            addrs,
            move |addr| Self::connect_config(addr, config.clone()),
            ConnectError::NoSocketAddress,
        )
        .await
    }

    pub async fn connect_config(
        addr: SocketAddr,
        config: ClientConfig,
    ) -> Result<Self, ConnectError> {
        // TODO: 1, see README.md

        let listen = if addr.is_ipv6() {
            SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0).into()
        } else {
            SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0).into()
        };

        let mut endpoint = Endpoint::client(listen)?;
        endpoint.set_default_client_config(config);
        let conn = endpoint.connect(addr, "localhost")?.await?;

        Self::new(conn, endpoint).await
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

    /// panics if socket is split
    pub async fn recv(&mut self) -> Option<Packet> {
        self.receiver().recv().await
    }

    /// panics if socket is split
    pub fn try_recv(&mut self) -> Result<Packet, TryRecvError> {
        self.receiver().try_recv()
    }

    /// panics if socket is split
    pub async fn send(&self, packet: Packet) -> Option<()> {
        self.sender().send(packet).await.ok()
    }

    /// panics if socket is split
    pub fn try_send(&self, packet: Packet) -> Result<(), TrySendError<Packet>> {
        self.sender().try_send(packet)
    }

    /// returns the sender and receiver parts
    ///
    /// this socket should still be kept
    ///
    /// panics if split twice
    pub fn split(&mut self) -> (mpsc::Sender<Packet>, mpsc::Receiver<Packet>) {
        self.channels.take().expect("channels already taken")
    }

    /// _unsplit_
    pub fn unite(&mut self, channels: (mpsc::Sender<Packet>, mpsc::Receiver<Packet>)) {
        self.channels = Some(channels);
    }

    /// panics if socket is split
    pub fn channels(&self) -> &(mpsc::Sender<Packet>, mpsc::Receiver<Packet>) {
        self.channels.as_ref().expect("channels already taken")
    }

    /// panics if socket is split
    pub fn channels_mut(&mut self) -> &mut (mpsc::Sender<Packet>, mpsc::Receiver<Packet>) {
        self.channels.as_mut().expect("channels already taken")
    }

    /// panics if socket is split
    pub fn sender(&self) -> &mpsc::Sender<Packet> {
        &self.channels().0
    }

    /// panics if socket is split
    pub fn receiver(&mut self) -> &mut mpsc::Receiver<Packet> {
        &mut self.channels_mut().1
    }

    pub async fn wait_idle(&self) {
        self.endpoint.wait_idle().await
    }

    pub fn remote(&self) -> SocketAddr {
        self.connection.remote_address()
    }

    pub fn local(&self) -> SocketAddr {
        self.endpoint.local_addr().unwrap()
    }

    pub fn stats(&self) -> ConnectionStats {
        self.connection.stats()
    }

    /// Round trip time estimation
    pub fn rtt(&self) -> Duration {
        self.connection.rtt()
    }

    pub(crate) async fn new(conn: NewConnection, endpoint: Endpoint) -> Result<Self, ConnectError> {
        Ok(Self {
            inner: Some(SocketInner::new(conn, endpoint).await?),
        })
    }
}

impl Deref for Socket {
    type Target = SocketInner;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect("Socket used after drop")
    }
}

impl DerefMut for Socket {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().expect("Socket used after drop")
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        if let Some(s) = self.inner.take() {
            s.close();
        }
    }
}
