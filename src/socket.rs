use crate::{packet::Packet, reader::reader_worker_job, writer::writer_worker_job};
use futures::future::join;
use quinn::{ClientConfig, Connection, Endpoint, NewConnection};
use quinn_proto::ConnectionStats;
use rustls::{client::ServerCertVerifier, Certificate};
use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};
use thiserror::Error;
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};

//

pub struct Socket {
    inner: Option<SocketInner>,
}

pub struct SocketInner {
    endpoint: Endpoint,
    connection: Connection,

    send: mpsc::Sender<Packet>,
    recv: mpsc::Receiver<Packet>,

    write_worker: JoinHandle<()>,
    read_worker: JoinHandle<()>,
    should_stop: broadcast::Sender<()>,
}

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("connect error ({0})")]
    Connect(#[from] quinn::ConnectError),

    #[error("connection error ({0})")]
    Connection(#[from] quinn::ConnectionError),

    #[error("failed to bind socket ({0})")]
    IoError(#[from] io::Error),
}

//

impl Socket {
    pub async fn connect(addr: SocketAddr) -> Result<Self, ConnectError> {
        let config = Self::default_config();
        Self::connect_config(addr, config).await
    }

    pub async fn connect_config(
        addr: SocketAddr,
        config: ClientConfig,
    ) -> Result<Self, ConnectError> {
        // TODO: encryption doesn't protect
        // from MITM attacks at the moment.
        // Add certificates, private keys,
        // server names and DNS

        let listen = if addr.is_ipv6() {
            SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0).into()
        } else {
            SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0).into()
        };

        let mut endpoint = Endpoint::client(listen)?;
        endpoint.set_default_client_config(config);
        let conn = endpoint.connect(addr, "localhost")?.await?;

        Ok(Self::new(conn, endpoint))
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

    pub async fn recv(&mut self) -> Option<Packet> {
        self.recv.recv().await
    }

    pub async fn send(&self, packet: Packet) -> Option<()> {
        self.send.send(packet).await.ok()
    }

    pub async fn wait_idle(&self) {
        self.endpoint.wait_idle().await
    }

    pub fn remote(&self) -> SocketAddr {
        self.connection.remote_address()
    }

    pub fn stats(&self) -> ConnectionStats {
        self.connection.stats()
    }

    /// Round trip time estimation
    pub fn rtt(&self) -> Duration {
        self.connection.rtt()
    }

    pub(crate) fn new(conn: NewConnection, endpoint: Endpoint) -> Self {
        let NewConnection {
            connection,
            uni_streams,
            datagrams,
            ..
        } = conn;

        // TODO: configurable buffer capacity
        let (worker_send, recv) = mpsc::channel(256);
        let (send, worker_recv) = mpsc::channel(256);

        let (should_stop, worker_should_stop_1) = broadcast::channel(1);
        let worker_should_stop_2 = worker_should_stop_1.resubscribe();

        // spawn writer worker
        let write_worker = tokio::spawn(writer_worker_job(
            connection.clone(),
            worker_recv,
            worker_should_stop_1,
        ));

        // spawn reader worker
        let read_worker = tokio::spawn(reader_worker_job(
            uni_streams,
            datagrams,
            worker_send,
            worker_should_stop_2,
        ));

        Self {
            inner: Some(SocketInner {
                endpoint,
                connection,

                send,
                recv,

                write_worker,
                read_worker,
                should_stop,
            }),
        }
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
        if let Some(SocketInner {
            send,
            recv,
            write_worker,
            read_worker,
            should_stop,
            connection,
            endpoint,
            ..
        }) = self.inner.take()
        {
            futures::executor::block_on(async move {
                let _ = should_stop.send(());
                let _ = join(write_worker, read_worker).await;
                let _ = (send, recv, connection, endpoint);
                log::debug!("Closing socket");
            });
        }
    }
}
