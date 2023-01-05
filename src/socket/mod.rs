use self::{
    filter::FilterError,
    reader::{Reader, ReaderError},
    writer::{Writer, WriterError},
};
use crate::packet::Packet;
use quinn::{Connecting, Connection, Endpoint, NewConnection};
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
}

pub trait SocketStats {
    /// Socket local address
    ///
    /// [`Endpoint::local_addr`]
    fn local(&self) -> std::io::Result<SocketAddr> {
        self.endpoint().local_addr()
    }

    /// Connection remote address
    ///
    /// [`Connection::remote_address`]
    fn remote(&self) -> SocketAddr {
        self.connection().remote_address()
    }

    /// Connection round trip time
    ///
    /// [`Connection::rtt`]
    fn rtt(&self) -> Duration {
        self.connection().rtt()
    }

    fn connection(&self) -> Connection;

    fn endpoint(&self) -> Endpoint;
}

//

impl Socket {
    /// Read [`Writer::send`]
    pub async fn send(&self, packet: Packet) -> Result<(), WriterError> {
        self.writer.send(packet).await
    }

    /// Read [`Writer::blocking_send`]
    pub fn blocking_send(&self, packet: Packet) -> Result<(), WriterError> {
        self.writer.blocking_send(packet)
    }

    /// Read [`Reader::recv`]
    pub async fn recv(&mut self) -> Result<Packet, ReaderError> {
        self.reader.recv().await
    }

    /// Read [`Reader::blocking_recv`]
    pub fn blocking_recv(&mut self) -> Result<Packet, ReaderError> {
        self.reader.blocking_recv()
    }

    /// Split the socket into its writer and reader halfs
    pub fn split(self) -> (Writer, Reader) {
        (self.writer, self.reader)
    }

    /// Reunite the socket halfs
    pub fn reunite(writer: Writer, reader: Reader) -> Self {
        Self { writer, reader }
    }

    pub(crate) async fn new(conn: Connecting, endpoint: Endpoint) -> Result<Self, FilterError> {
        let NewConnection {
            connection,
            mut uni_streams,
            datagrams,
            ..
        } = conn.await?;

        filter::filter_unwanted(&mut uni_streams, connection.clone()).await?;

        tracing::debug!("Connected");

        Ok(Self {
            writer: Writer::new(connection.clone(), endpoint.clone()),
            reader: Reader::new(connection, endpoint, uni_streams, datagrams),
        })
    }
}

impl SocketStats for Socket {
    fn connection(&self) -> Connection {
        self.reader.connection()
    }

    fn endpoint(&self) -> Endpoint {
        self.reader.endpoint()
    }
}
