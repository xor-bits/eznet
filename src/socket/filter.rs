use futures::{join, SinkExt, StreamExt};
use quinn::{Connection, ConnectionError, IncomingUniStreams, WriteError};
use serde::{Deserialize, Serialize};
use std::{io, time::Duration};
use thiserror::Error;
use tokio::time::timeout;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

//

#[derive(Debug, Error)]
pub enum FilterError {
    #[error("Timed out opening a test sender stream")]
    TimedOut,

    #[error(transparent)]
    ConnectionError(#[from] ConnectionError),

    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("Failed to send the filter packet: {0}")]
    WriteError(#[from] WriteError),

    #[error("Invalid filter packet: {0}")]
    PacketParseError(#[from] bincode::Error),

    #[error("Invalid filter packet: Invalid magic bytes")]
    InvalidPacketMagicBytes,

    // TODO: 7, see README.md
    #[error("The peer (version {0}.{1}) is not compatible with version {}.{}", PACKET.version.0, PACKET.version.1)]
    NotCompatible(u16, u16),

    #[error("The peer didn't respond")]
    NoResponse,
}

//

pub async fn filter_unwanted(
    uni_streams: &mut IncomingUniStreams,
    connection: Connection,
) -> Result<(), FilterError> {
    let (a, b) = join!(send_filter_test(connection), recv_filter_test(uni_streams));
    a?;
    b?;

    Ok(())
}

async fn send_filter_test(connection: Connection) -> Result<(), FilterError> {
    // time out after 5 seconds
    // open a new stream for sending the filter test message
    let stream = timeout(Duration::from_secs(5), connection.open_uni())
        .await
        .map_err(|_| FilterError::TimedOut)??;
    let mut stream = FramedWrite::new(stream, LengthDelimitedCodec::default());

    stream
        .send(bincode::serialize(&PACKET).unwrap().into())
        .await?;
    stream.into_inner().finish().await?;

    Ok(())
}

async fn recv_filter_test(uni_streams: &mut IncomingUniStreams) -> Result<(), FilterError> {
    // time out after 5 seconds
    // open a new stream for sending the filter test message
    let packet = timeout(Duration::from_secs(5), async move {
        let stream = uni_streams.next().await.ok_or(FilterError::NoResponse)??;
        let mut stream = FramedRead::new(stream, LengthDelimitedCodec::default());
        Ok::<_, FilterError>(stream.next().await.ok_or(FilterError::NoResponse)??)
    })
    .await
    .map_err(|_| FilterError::TimedOut)??;

    let packet: FilterPacket = bincode::deserialize(&packet[..])?;

    if packet.magic_bytes != PACKET.magic_bytes {
        tracing::debug!("Invalid filter packet {packet:?}");
        return Err(FilterError::InvalidPacketMagicBytes);
    }

    if packet.version != PACKET.version {
        tracing::debug!("Filter packet from incompatible peer {packet:?}");
        return Err(FilterError::NotCompatible(
            packet.version.0,
            packet.version.1,
        ));
    }

    // TODO: 7, see README.md

    Ok(())
}

//

#[derive(Debug, Serialize, Deserialize)]
struct FilterPacket {
    magic_bytes: u64,
    version: (u16, u16),
}

//

static PACKET: FilterPacket = FilterPacket {
    // Just a random u64 I generated
    //
    // Not intended filter out malicious
    // connections, just accidental
    // connections and incompatible versions
    magic_bytes: 0x87213c5b6657d98a,

    version: include!(concat!(env!("OUT_DIR"), "/version")),
};

// TODO: 7, see README.md
// static BREAKING_VERSIONS: &[(u16, u16, u16)] = &[];
