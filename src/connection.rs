use crate::packet::PacketFlags;
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use futures::{SinkExt, StreamExt};
use quinn::{
    ConnectionError, Datagrams, IncomingBiStreams, IncomingUniStreams, NewConnection, RecvStream,
    SendStream,
};
use tokio::select;
use tokio_util::codec::LengthDelimitedCodec;

//

#[async_trait]
pub trait Connection {
    /// Multiple readers are allowed
    async fn read(&mut self) -> Bytes;

    /// Multiple senders are allowed
    async fn send(&mut self, message: Bytes, flags: PacketFlags);
}

//

pub struct CommonConnection {
    pub connection: quinn::Connection,
    uni_streams: IncomingUniStreams,
    bi_streams: IncomingBiStreams,
    datagrams: Datagrams,

    ordered: Option<(FramedWrite, FramedRead)>,
}

type FramedWrite = tokio_util::codec::FramedWrite<SendStream, LengthDelimitedCodec>;
type FramedRead = tokio_util::codec::FramedRead<RecvStream, LengthDelimitedCodec>;

//

#[async_trait]
impl Connection for CommonConnection {
    async fn read(&mut self) -> Bytes {
        self.read_impl().await
    }

    async fn send(&mut self, message: Bytes, flags: PacketFlags) {
        self.send_impl(message, flags).await
    }
}

//

impl CommonConnection {
    pub fn new(conn: NewConnection) -> Self {
        let NewConnection {
            connection,
            uni_streams,
            bi_streams,
            datagrams,
            ..
        } = conn;

        Self {
            connection,
            uni_streams,
            bi_streams,
            datagrams,

            ordered: None,
        }
    }

    // Readers

    async fn read_impl(&mut self) -> Bytes {
        select! {
            unreliable = self.datagrams.next() => unreliable.unwrap().unwrap(),
            Some(ordered) = Self::read_from_ordered(&mut self.ordered) => ordered.unwrap().into(),
            uni_stream = self.uni_streams.next() => self.read_uni_stream(uni_stream).await,
            bi_stream = self.bi_streams.next() => self.read_bi_stream(bi_stream).await
        }
    }

    async fn read_from_ordered(
        ordered: &mut Option<(FramedWrite, FramedRead)>,
    ) -> Option<Result<BytesMut, std::io::Error>> {
        let (_, reader) = ordered.as_mut()?;
        reader.next().await
    }

    async fn read_uni_stream(
        &self,
        uni_stream: Option<Result<RecvStream, ConnectionError>>,
    ) -> Bytes {
        // println!("uni stream");
        let reader = uni_stream.unwrap().unwrap();

        let mut framed = FramedRead::new(reader, LengthDelimitedCodec::default());
        let bytes = framed.next().await.unwrap().unwrap().into();

        bytes
    }

    async fn read_bi_stream(
        &mut self,
        bi_stream: Option<Result<(SendStream, RecvStream), ConnectionError>>,
    ) -> Bytes {
        // println!("bi stream");
        let (sender, reader) = bi_stream.unwrap().unwrap();

        let sender = FramedWrite::new(sender, LengthDelimitedCodec::default());
        let mut reader = FramedRead::new(reader, LengthDelimitedCodec::default());
        let bytes = reader.next().await.unwrap().unwrap().into();

        self.ordered = Some((sender, reader));
        bytes
    }

    // Senders

    async fn send_impl(&mut self, message: Bytes, flags: PacketFlags) {
        match flags {
            PacketFlags::IMPORTANT_ORDERED => self.send_ordered_reliable(message).await,
            PacketFlags::IMPORTANT_UNORDERED => self.send_unordered_reliable(message).await,
            PacketFlags::OPTIONAL_UNORDERED => self.send_unreliable(message).await,
            _ => todo!(),
        }
    }

    async fn send_ordered_reliable(&mut self, message: Bytes) {
        match self.ordered.as_mut() {
            Some((sender, _)) => sender.send(message).await.unwrap(),
            None => {
                let (sender, reader) = self.connection.open_bi().await.unwrap();

                let mut sender = FramedWrite::new(sender, LengthDelimitedCodec::default());
                let reader = FramedRead::new(reader, LengthDelimitedCodec::default());
                sender.send(message).await.unwrap();

                self.ordered = Some((sender, reader));
            }
        };
    }

    async fn send_unordered_reliable(&self, message: Bytes) {
        let sender = self.connection.open_uni().await.unwrap();
        let mut sender = FramedWrite::new(sender, LengthDelimitedCodec::default());
        sender.send(message).await.unwrap();
    }

    async fn send_unreliable(&self, message: Bytes) {
        let size = self.connection.max_datagram_size().unwrap();

        let hint_size = std::mem::size_of::<usize>();
        let packet_len = message.len() + hint_size;
        let packet_count = packet_len / size + (packet_len % size == 0) as usize;

        let message = Bytes::from_iter(
            packet_count
                .to_le_bytes()
                .into_iter()
                .chain(message.into_iter()),
        );

        for message in message.chunks(size) {
            self.connection
                .send_datagram(Bytes::copy_from_slice(message))
                .unwrap();
        }
    }
}
