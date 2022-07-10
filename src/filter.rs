use crate::VERSION;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use quinn::{Connection, ConnectionError, IncomingUniStreams, WriteError};
use serde::{Deserialize, Serialize};
use std::{io, time::Duration};
use thiserror::Error;
use tokio::{join, select, time::sleep};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

//

#[derive(Debug, Error)]
pub enum FilterError {
    #[error("Timed out opening a test sender stream")]
    TimedOut,

    #[error("Connection error ({0})")]
    ConnectionError(#[from] ConnectionError),

    #[error("Connection error ({0})")]
    IoError(#[from] io::Error),

    #[error("Connection error ({0})")]
    WriteError(#[from] WriteError),

    #[error("Invalid filter packet ({0})")]
    PacketParseError(#[from] bincode::Error),

    #[error("Invalid filter packet (invalid magic bytes)")]
    InvalidPacketMagicBytes,

    // TODO: 7, see README.md
    #[error("peer is not compatible with {}", crate::VERSION)]
    NotCompatible,
}

//

pub async fn filter_unwanted(
    uni_streams: &mut IncomingUniStreams,
    connection: &Connection,
) -> Result<(), FilterError> {
    let (a, b) = join!(send_filter_test(connection), recv_filter_test(uni_streams));
    a?;
    b?;

    Ok(())
}

async fn send_filter_test(connection: &Connection) -> Result<(), FilterError> {
    // time out after 5 seconds
    // open a new stream for sending the filter test message
    let mut stream = select! {
        timeout = filter_test_time_out() => return timeout,
        stream = connection.open_uni() => FramedWrite::new(stream?, LengthDelimitedCodec::default())
    };

    stream.send(PACKET.clone()).await?;
    stream.into_inner().finish().await?;

    Ok(())
}

async fn recv_filter_test(uni_streams: &mut IncomingUniStreams) -> Result<(), FilterError> {
    // time out after 5 seconds
    // open a new stream for sending the filter test message
    let mut stream = select! {
        timeout = filter_test_time_out() => return timeout,
        Some(stream) = uni_streams.next() => FramedRead::new(stream?, LengthDelimitedCodec::default()),
    };

    let packet = select! {
        timeout = filter_test_time_out() => return timeout,
        Some(packet) = stream.next() => packet?,
    };

    let packet: FilterPacket = bincode::deserialize(&packet[..])?;

    if packet.magic_bytes != MAGIC_BYTES {
        log::debug!("Invalid filter packet {packet:?}");
        return Err(FilterError::InvalidPacketMagicBytes);
    }

    // TODO: 7, see README.md

    Ok(())
}

async fn filter_test_time_out() -> Result<(), FilterError> {
    sleep(Duration::from_secs(5)).await;
    Err(FilterError::TimedOut)
}

//

#[derive(Debug, Serialize, Deserialize)]
struct FilterPacket<'a> {
    magic_bytes: u64,
    version: &'a str,
}

lazy_static::lazy_static! {
    static ref PACKET: Bytes = {
        bincode::serialize(&FilterPacket {
            magic_bytes: MAGIC_BYTES,
            version: VERSION,
        }).unwrap().into()
    };
}

// just a random u64 i generated
// Not intended filter out malicious
// connections, just accidental
// connections and port scanners
static MAGIC_BYTES: u64 = 0x87213c5b6657d98a;

// TODO: 7, see README.md
// static BREAKING_VERSIONS: &[(u16, u16, u16)] = &[];
