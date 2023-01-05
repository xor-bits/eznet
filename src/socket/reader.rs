use super::{writer::Writer, Socket, SocketStats};
use crate::packet::{Packet, PacketHeader};
use dashmap::DashMap;
use futures::{stream::SelectAll, StreamExt};
use quinn::{Connection, ConnectionError, Datagrams, Endpoint, IncomingUniStreams, RecvStream};
use std::{
    io,
    sync::atomic::{AtomicUsize, Ordering},
};
use thiserror::Error;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};
use tracing::instrument;

//

#[derive(Debug)]
pub struct Reader {
    connection: Connection,
    endpoint: Endpoint,
    uni_streams: IncomingUniStreams,
    datagrams: Datagrams,
    open_streams: SelectAll<FRead>,

    seq: DashMap<u8, u16>,
}

#[derive(Debug, Error)]
pub enum ReaderError {
    #[error(transparent)]
    Connection(#[from] quinn::ConnectionError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("Failed to send unreliable packet: {0}")]
    SendDatagram(#[from] quinn::SendDatagramError),

    #[error("Failed to decode packet: {0}")]
    Decode(#[from] bincode::Error),
}

type FRead = FramedRead<RecvStream, LengthDelimitedCodec>;

//

static RECV_DBG_ID: AtomicUsize = AtomicUsize::new(0);

//

impl Reader {
    /// Cancel safe
    #[instrument(skip_all, fields(id = %RECV_DBG_ID.fetch_add(1, Ordering::SeqCst)))]
    pub async fn recv(&mut self) -> Result<Packet, ReaderError> {
        loop {
            tracing::debug!("Attempting to read");

            // Cancel safety: this select _should_ be the only .await point
            // tracing debug macros don't await on anything, right?
            tokio::select! {
                // old dropped packets are `None` and will be ignored
                //
                // prioritize reading datagrams / from already opened streams
                Some(packet) = self.datagrams.next() => {
                    tracing::debug!({
                        err = packet.as_ref().err()
                            .map(ConnectionError::to_string)
                    }, "Got Datagram packet");
                    if let Some(packet) = self.handle_packet(&packet?)? {
                        return Ok(packet);
                    }
                },
                Some(packet) = self.open_streams.next() => {
                    tracing::debug!({
                        err = packet.as_ref().err()
                            .map(io::Error::to_string)
                    }, "Got UniStream packet");
                    if let Some(packet) = self.handle_packet(&packet?)? {
                        return Ok(packet);
                    }
                },

                // listen for new streams
                stream = self.uni_streams.next()=> {
                    tracing::debug!("New stream opened, reattempting");
                    self.handle_uni_stream(stream)?;
                    continue;
                },
            }
        }
    }

    pub fn blocking_recv(&mut self) -> Result<Packet, ReaderError> {
        futures::executor::block_on(self.recv())
    }

    /// Reunite the socket halfs
    pub fn reunite(writer: Writer, reader: Reader) -> Socket {
        Socket::reunite(writer, reader)
    }

    pub(crate) fn new(
        connection: Connection,
        endpoint: Endpoint,
        uni_streams: IncomingUniStreams,
        datagrams: Datagrams,
    ) -> Self {
        Self {
            connection,
            endpoint,
            uni_streams,
            datagrams,
            open_streams: SelectAll::new(),
            seq: DashMap::new(),
        }
    }

    fn handle_uni_stream(
        &mut self,
        stream: Option<Result<RecvStream, ConnectionError>>,
    ) -> Result<(), ReaderError> {
        let stream = stream.ok_or(ReaderError::Connection(
            quinn::ConnectionError::LocallyClosed,
        ))??;

        let stream = FRead::new(stream, <_>::default());

        self.open_streams.push(stream);

        Ok(())
    }

    fn handle_packet(&self, packet: &[u8]) -> Result<Option<Packet>, ReaderError> {
        Ok(self.drop_sequenced(Packet::decode(packet)?))
    }

    fn drop_sequenced(&self, packet: Packet) -> Option<Packet> {
        match packet.header {
            PacketHeader::Sequenced { stream_id, seq_id }
            | PacketHeader::UnreliableSequenced { stream_id, seq_id } => {
                Self::seq_id_should_drop(stream_id, seq_id, &self.seq)
            }
            _ => true,
        }
        .then_some(packet)
    }

    fn seq_id_should_drop(stream_id: u8, seq_id: u16, seq: &DashMap<u8, u16>) -> bool {
        let mut recv_seq_id = seq.entry(stream_id).or_insert(0);
        let send_seq_id = seq_id;

        // convert them to a form where it is comparable
        let rsi = u16::MAX / 2 - 1;
        let ssi = ((send_seq_id as i32 - *recv_seq_id as i32).rem_euclid(u16::MAX as i32) as u16)
            .wrapping_add(u16::MAX / 2);

        if cfg!(test) {
            dbg!(&recv_seq_id);
            dbg!(&send_seq_id);
            dbg!(&rsi);
            dbg!(&ssi);
        }

        if ssi > rsi {
            // got packet that is 'newer'
            *recv_seq_id = send_seq_id + 1;
            true
        } else {
            // got packet that is 'older'
            tracing::debug!("Dropping out of sequence packet");
            false
        }
    }
}

impl SocketStats for Reader {
    fn connection(&self) -> Connection {
        self.connection.clone()
    }

    fn endpoint(&self) -> Endpoint {
        self.endpoint.clone()
    }
}

//

#[cfg(test)]
mod tests {
    use super::Reader;
    use dashmap::DashMap;

    #[test]
    fn drop_sequenced_common_test_0() {
        let mut seq = DashMap::new();
        seq.insert(0, 0);

        assert!(Reader::seq_id_should_drop(0, 1, &seq) == true);
        assert!(Reader::seq_id_should_drop(0, 1, &seq) == false);
        assert!(Reader::seq_id_should_drop(0, 1, &seq) == false);
        assert!(Reader::seq_id_should_drop(0, 2, &seq) == true);
        assert!(Reader::seq_id_should_drop(0, 2, &seq) == false);
        assert!(Reader::seq_id_should_drop(0, 2, &seq) == false);
        assert!(Reader::seq_id_should_drop(0, 200, &seq) == true);
        assert!(Reader::seq_id_should_drop(0, 2, &seq) == false);
        assert!(Reader::seq_id_should_drop(0, u16::MAX / 4, &seq) == true);
        assert!(Reader::seq_id_should_drop(0, u16::MAX / 2, &seq) == true);
        assert!(Reader::seq_id_should_drop(0, u16::MAX / 4 * 3, &seq) == true);
        assert!(Reader::seq_id_should_drop(0, u16::MAX - 100, &seq) == true);
        assert!(Reader::seq_id_should_drop(0, u16::MAX - 100, &seq) == false);
        assert!(Reader::seq_id_should_drop(0, u16::MAX - 99, &seq) == true);
        assert!(Reader::seq_id_should_drop(0, u16::MAX - 99, &seq) == false);
        assert!(Reader::seq_id_should_drop(0, 0, &seq) == true);
        assert!(Reader::seq_id_should_drop(0, 0, &seq) == false);
    }
}
