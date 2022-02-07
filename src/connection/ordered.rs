use super::{Framed, FramedRead, FramedWrite};
use crate::packet::Packet;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use quinn::{Connection, IncomingBiStreams};
use std::sync::Arc;
use tokio_serde::formats::Bincode;
use tokio_util::codec::LengthDelimitedCodec;

//

pub struct Ordered {
    connection: Arc<Connection>,
    incoming: IncomingBiStreams,

    writer: Option<Framed<FramedWrite>>,
    reader: Option<Framed<FramedRead>>,
}

//

impl Ordered {
    pub async fn new(connection: Arc<Connection>, incoming: IncomingBiStreams) -> Self {
        Self {
            connection,
            incoming,

            reader: None,
            writer: None,
        }
    }

    async fn get_writer(&mut self) -> &'_ mut Framed<FramedWrite> {
        let stream = &mut self.writer;
        match stream {
            Some(stream) => stream,
            None => {
                let (writer, _) = self.connection.open_bi().await.unwrap();
                let writer = FramedWrite::new(writer, LengthDelimitedCodec::default());
                let writer = Framed::new(writer, Bincode::default());
                *stream = Some(writer);

                stream.as_mut().unwrap()
            }
        }
    }

    async fn get_reader(&mut self) -> &'_ mut Framed<FramedRead> {
        let stream = &mut self.reader;
        match stream {
            Some(stream) => stream,
            None => {
                let (_, reader) = self.incoming.next().await.unwrap().unwrap();
                let reader = FramedRead::new(reader, LengthDelimitedCodec::default());
                let reader = Framed::new(reader, Bincode::default());
                *stream = Some(reader);

                stream.as_mut().unwrap()
            }
        }
    }

    pub async fn write(&mut self, message: Bytes) -> Option<()> {
        self.get_writer()
            .await
            .send(Packet::new(None, message))
            .await
            .ok()
    }

    pub async fn read(&mut self) -> Option<Bytes> {
        Some(self.get_reader().await.next().await?.ok()?.payload)
    }
}
