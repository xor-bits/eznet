use self::{ordered::Ordered, unordered::Unordered, unreliable::Unreliable};
use crate::packet::{Packet, PacketFlags};
use quinn::{Connection, NewConnection, RecvStream, SendStream};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::{join, select};
use tokio_serde::formats::Bincode;
use tokio_util::codec::LengthDelimitedCodec;

//

pub mod ordered;
pub mod unordered;
pub mod unreliable;

//

/* #[async_trait]
pub trait Connection {
    /// Multiple readers are allowed
    async fn read(&mut self) -> Bytes;

    /// Multiple senders are allowed
    async fn send(&mut self, message: Bytes, flags: PacketFlags);
} */

//

pub struct CommonConnection<S, R>
where
    S: Send + Serialize + Unpin + 'static,
    R: Send + DeserializeOwned + Unpin + 'static,
{
    pub connection: Arc<Connection>,
    ordered: Ordered<S, R>,
    unordered: Unordered<S, R>,
    unreliable: Unreliable<S, R>,
}

pub type FramedWrite = tokio_util::codec::FramedWrite<SendStream, LengthDelimitedCodec>;
pub type FramedRead = tokio_util::codec::FramedRead<RecvStream, LengthDelimitedCodec>;
pub type Framed<P, T> = tokio_serde::Framed<T, Packet<P>, Packet<P>, Bincode<Packet<P>, Packet<P>>>;

//

impl<S, R> CommonConnection<S, R>
where
    S: Send + Serialize + Unpin + 'static,
    R: Send + DeserializeOwned + Unpin + 'static,
{
    pub async fn new(conn: NewConnection) -> Self {
        let NewConnection {
            connection,
            uni_streams,
            bi_streams,
            datagrams,
            ..
        } = conn;

        let connection = Arc::new(connection);
        let ordered = Ordered::new(connection.clone(), bi_streams);
        let unordered = Unordered::new(connection.clone(), uni_streams);
        let unreliable = Unreliable::new(connection.clone(), datagrams);
        let (ordered, unordered, unreliable) = join!(ordered, unordered, unreliable);

        Self {
            connection,
            ordered,
            unordered,
            unreliable,
        }
    }

    // Readers

    pub async fn read(&mut self) -> Option<R> {
        let a = async { self.ordered.read().await };
        let b = async { self.unordered.read().await };
        let c = async { self.unreliable.read().await };

        select! {
            message = a => message,
            message = b => message,
            message = c => message
        }
    }

    // Senders

    pub async fn send(&mut self, message: S, flags: PacketFlags) -> Option<()> {
        const RO: PacketFlags = PacketFlags::RELIABLE.union(PacketFlags::ORDERED);
        const RS: PacketFlags = PacketFlags::RELIABLE.union(PacketFlags::SEQUENCED);
        const RU: PacketFlags = PacketFlags::RELIABLE.union(PacketFlags::UNORDERED);
        const UR: PacketFlags = PacketFlags::UNRELIABLE.union(PacketFlags::ORDERED);
        const US: PacketFlags = PacketFlags::UNRELIABLE.union(PacketFlags::SEQUENCED);
        const UU: PacketFlags = PacketFlags::UNRELIABLE.union(PacketFlags::UNORDERED);

        match flags {
            RO => self.ordered.write(message).await,
            RS => self.unordered.write(message, true).await,
            RU => self.unordered.write(message, false).await,
            UR => panic!("Unreliable ordered packets are not supported"),
            US => self.unreliable.write(message, true).await,
            UU => self.unreliable.write(message, false).await,
            _ => todo!(),
        }
    }
}
