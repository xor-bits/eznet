use futures::{join, SinkExt, StreamExt};
use quinn::{Connection, ConnectionError, IncomingUniStreams, WriteError};
use serde::{Deserialize, Serialize};
use std::{io, time::Duration};
use thiserror::Error;
use tokio::time::timeout;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tracing::instrument;

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

#[instrument(skip_all)]
pub async fn filter_unwanted(
    uni_streams: &mut IncomingUniStreams,
    connection: Connection,
) -> Result<(), FilterError> {
    let (a, b) = join!(send(connection), recv(uni_streams));
    a?;
    b?;

    tracing::debug!("Peer passed the filter test");

    Ok(())
}

#[instrument(skip_all)]
async fn send(connection: Connection) -> Result<(), FilterError> {
    // time out after 5 seconds
    // open a new stream for sending the filter test message
    timeout(Duration::from_secs(5), async move {
        tracing::debug!("Begin");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let stream = connection.open_uni().await?;
        let mut stream = FramedWrite::new(stream, LengthDelimitedCodec::default());

        tracing::debug!("Got stream");
        stream
            .send(bincode::serialize(&PACKET).unwrap().into())
            .await?;
        tracing::debug!("Finishing");
        stream.into_inner().finish().await?;

        Ok::<(), FilterError>(())
    })
    .await
    .map_err(|_| FilterError::TimedOut)??;

    tracing::debug!("Complete");

    Ok(())
}

#[instrument(skip_all)]
async fn recv(uni_streams: &mut IncomingUniStreams) -> Result<(), FilterError> {
    // time out after 5 seconds
    // open a new stream for sending the filter test message
    let packet = timeout(Duration::from_secs(5), async move {
        tracing::debug!("Begin");
        let stream = uni_streams.next().await.ok_or(FilterError::NoResponse)??;
        let mut stream = FramedRead::new(stream, LengthDelimitedCodec::default());
        tracing::debug!("Got stream");
        Ok::<_, FilterError>(stream.next().await.ok_or(FilterError::NoResponse)??)
    })
    .await
    .map_err(|_| FilterError::TimedOut)??;

    let packet: FilterPacket = bincode::deserialize(&packet[..])?;

    if packet.magic_bytes != PACKET.magic_bytes {
        tracing::debug!("Invalid filter packet {packet:?}");
        return Err(FilterError::InvalidPacketMagicBytes);
    }

    if packet.version.0 != PACKET.version.0 {
        tracing::debug!("Filter packet from incompatible peer {packet:?}");
        return Err(FilterError::NotCompatible(
            packet.version.0,
            packet.version.1,
        ));
    }
    if packet.version.1 != PACKET.version.1 {
        tracing::debug!(
            {
                peer.maj = packet.version.0,
                peer.min = packet.version.1,
                self.maj = PACKET.version.0,
                self.min = PACKET.version.1,
            },
            "Minor version mismatch, peer _should_ be compatible"
        );
    }

    // TODO: 7, see README.md

    tracing::debug!("Complete");

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
