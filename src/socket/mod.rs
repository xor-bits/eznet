use self::{
    filter::FilterError,
    reader::{Reader, ReaderError},
    writer::{Writer, WriterError},
};
use crate::packet::Packet;
use quinn::{Connection, Endpoint, NewConnection};
use std::{net::SocketAddr, time::Duration};

//

pub mod filter;
pub mod reader;
pub mod writer;

//

#[derive(Debug)]
pub struct Socket {
    writer: Writer,
    reader: Reader,
    endpoint: Endpoint,
}

//

impl Socket {
    pub async fn send(&self, packet: Packet) -> Result<(), WriterError> {
        self.writer.send(packet).await
    }

    pub fn blocking_send(&self, packet: Packet) -> Result<(), WriterError> {
        self.writer.blocking_send(packet)
    }

    pub async fn recv(&mut self) -> Result<Packet, ReaderError> {
        self.reader.recv().await
    }

    pub fn split(self) -> (Writer, Reader, Endpoint) {
        (self.writer, self.reader, self.endpoint)
    }

    pub fn local(&self) -> std::io::Result<SocketAddr> {
        self.endpoint.local_addr()
    }

    pub fn remote(&self) -> SocketAddr {
        self.writer.remote()
    }

    pub fn rtt(&self) -> Duration {
        self.writer.rtt()
    }

    pub fn connection(&self) -> Connection {
        self.writer.connection()
    }

    pub fn endpoint(&self) -> Endpoint {
        self.endpoint.clone()
    }

    pub(crate) async fn new(
        NewConnection {
            connection,
            mut uni_streams,
            datagrams,
            ..
        }: NewConnection,
        endpoint: Endpoint,
    ) -> Result<Self, FilterError> {
        filter::filter_unwanted(&mut uni_streams, connection.clone()).await?;

        Ok(Self {
            writer: Writer::new(connection),
            reader: Reader::new(uni_streams, datagrams),
            endpoint,
        })
    }
}
