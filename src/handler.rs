use crate::{connection::CommonConnection, packet::PacketFlags};
use bytes::Bytes;
use quinn::NewConnection;

//

pub struct Handler {
    connection: CommonConnection,
}

//

impl Handler {
    pub async fn new(conn: NewConnection) -> Self {
        let remote = conn.connection.remote_address();
        log::debug!("server connected: {remote}");

        let connection = CommonConnection::new(conn).await;
        Self { connection }
    }

    pub async fn read(&mut self) -> Option<Bytes> {
        self.connection.read().await
    }

    pub async fn send(&mut self, message: Bytes, flags: PacketFlags) -> Option<()> {
        self.connection.send(message, flags).await
    }
}
