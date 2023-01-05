use std::net::SocketAddr;

use eznet::{
    packet::Packet,
    server::Server,
    socket::{filter::FilterError, Socket},
};
use futures::Future;
use tokio::{
    select,
    sync::broadcast::{self, Receiver, Sender},
};

//

#[tokio::main]
async fn main() {
    let mut listener = Server::from("localhost:13331".parse::<SocketAddr>().unwrap()).unwrap();

    let (tx, rx) = broadcast::channel(256);

    while let Ok(socket) = listener.next().await {
        let ch = (tx.clone(), rx.resubscribe());

        tokio::spawn(async move {
            if let Err(err) = handle_client(socket, ch).await {
                tracing::error!("Net err: {err}")
            }
        });
    }
}

async fn handle_client(
    socket: impl Future<Output = Result<Socket, FilterError>>,
    (tx, mut rx): (Sender<Packet>, Receiver<Packet>),
) -> anyhow::Result<()> {
    let socket = socket.await?;

    let (socket_tx, mut socket_rx) = socket.split();

    loop {
        select! {
            Ok(from_broadcast) = rx.recv() => {
                socket_tx
                .send(from_broadcast)
                .await?;
            }
            Ok(to_broadcast) = socket_rx.recv() => {
                tx.send(to_broadcast)?;
            }
        }
    }
}
