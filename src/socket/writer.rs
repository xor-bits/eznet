use crate::packet::{Packet, PacketHeader};
use dashmap::{
    mapref::{entry::Entry, one::RefMut},
    DashMap,
};
use futures::SinkExt;
use quinn::Connection;
use std::{
    fmt::Debug,
    net::SocketAddr,
    sync::atomic::{AtomicU16, Ordering},
    time::Duration,
};
use thiserror::Error;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

//

#[derive(Debug)]
pub struct Writer {
    connection: Connection,
    streams: DashMap<u8, Stream>,
}

#[derive(Debug)]
pub struct Stream {
    write: FWrite,
    seq_id: AtomicU16,
}

#[derive(Debug, Error)]
pub enum WriterError {
    #[error(transparent)]
    ConnectionError(#[from] quinn::ConnectionError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Failed to send an unreliable packet: {0}")]
    SendDatagramError(#[from] quinn::SendDatagramError),

    #[error("Failed to encode packet: {0}")]
    DecodeError(#[from] bincode::Error),
}

type FWrite = FramedWrite<quinn::SendStream, LengthDelimitedCodec>;

//

impl Writer {
    pub async fn send(&self, packet: Packet) -> Result<(), WriterError> {
        match packet.header {
            PacketHeader::Ordered { stream_id } | PacketHeader::Sequenced { stream_id, .. } => {
                self.get_stream(stream_id)
                    .await?
                    .value_mut()
                    .send(packet)
                    .await?
            }
            PacketHeader::Unordered => self.new_stream().await?.send(packet).await?,
            PacketHeader::UnreliableSequenced { stream_id, .. } => self.connection.send_datagram(
                packet
                    .with_seq_id(self.get_stream(stream_id).await?.value().next())
                    .encode()?,
            )?,
            PacketHeader::UnreliableUnordered => {
                self.connection.send_datagram(packet.encode()?)?;
            }
        };

        Ok(())
    }

    pub fn blocking_send(&self, packet: Packet) -> Result<(), WriterError> {
        futures::executor::block_on(self.send(packet))
    }

    pub(crate) fn new(connection: Connection) -> Self {
        Self {
            connection,
            streams: Default::default(),
        }
    }

    async fn get_stream(
        &self,
        stream_id: u8,
    ) -> Result<RefMut<u8, Stream>, quinn::ConnectionError> {
        match self.streams.entry(stream_id) {
            Entry::Occupied(v) => Ok(v.into_ref()),
            Entry::Vacant(spot) => Ok(spot.insert(self.new_stream().await?)),
        }
    }

    async fn new_stream(&self) -> Result<Stream, quinn::ConnectionError> {
        Ok(Stream {
            write: FramedWrite::new(self.connection.open_uni().await?, <_>::default()),
            seq_id: 0.into(),
        })
    }

    pub fn rtt(&self) -> Duration {
        self.connection.rtt()
    }

    pub fn remote(&self) -> SocketAddr {
        self.connection.remote_address()
    }

    pub fn connection(&self) -> Connection {
        self.connection.clone()
    }
}

impl Stream {
    fn next(&self) -> u16 {
        self.seq_id.fetch_add(1, Ordering::SeqCst)
    }

    async fn send(&mut self, packet: Packet) -> Result<(), WriterError> {
        self.write
            .send(packet.with_seq_id(self.next()).encode()?)
            .await?;
        Ok(())
    }
}
