use crate::{connection::CommonConnection, packet::PacketFlags};
use bincode::config::Configuration;
use bytes::Bytes;
use quinn::NewConnection;
use serde::{de::DeserializeOwned, Serialize};

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

    pub async fn read_ty<T>(&mut self) -> Option<T>
    where
        T: DeserializeOwned,
    {
        const CONFIG: Configuration = bincode::config::standard();
        let bytes = self.read().await?;
        Some(bincode::serde::decode_from_slice(&bytes, CONFIG).unwrap().0)
    }

    pub async fn send_ty<T>(&mut self, message: &T, flags: PacketFlags) -> Option<()>
    where
        T: Serialize,
    {
        const CONFIG: Configuration = bincode::config::standard();
        let message = bincode::serde::encode_to_vec(message, CONFIG).unwrap();
        self.send(message.into(), flags).await
    }
}
