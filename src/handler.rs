use crate::{connection::CommonConnection, packet::PacketFlags};
use quinn::NewConnection;
use serde::{de::DeserializeOwned, Serialize};

//

pub struct Handler<S, R>
where
    S: Send + Serialize + Unpin + 'static,
    R: Send + DeserializeOwned + Unpin + 'static,
{
    connection: CommonConnection<S, R>,
}

//

impl<S, R> Handler<S, R>
where
    S: Send + Serialize + Unpin + 'static,
    R: Send + DeserializeOwned + Unpin + 'static,
{
    pub async fn new(conn: NewConnection) -> Self {
        let remote = conn.connection.remote_address();
        log::debug!("server connected: {remote}");

        let connection = CommonConnection::new(conn).await;
        Self { connection }
    }

    pub async fn read(&mut self) -> Option<R> {
        self.connection.read().await
    }

    pub async fn send(&mut self, message: S, flags: PacketFlags) -> Option<()> {
        self.connection.send(message, flags).await
    }
}
