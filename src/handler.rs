use crate::{
    connection::{CommonConnection, Connection},
    packet::PacketFlags,
};
use async_trait::async_trait;
use bytes::Bytes;
use quinn::NewConnection;

//

pub struct Handler {
    connection: CommonConnection,
}

//

impl Handler {
    pub fn new(conn: NewConnection) -> Self {
        let remote = conn.connection.remote_address();
        log::debug!("server connected: {remote}");

        let connection = CommonConnection::new(conn);
        Self { connection }
    }
}

//

#[async_trait]
impl Connection for Handler {
    async fn read(&mut self) -> Bytes {
        self.connection.read().await
    }

    async fn send(&mut self, message: Bytes, flags: PacketFlags) {
        self.connection.send(message, flags).await
    }
}
