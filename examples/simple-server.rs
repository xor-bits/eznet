use eznet::{packet::Packet, server::Server, socket::SocketStats};

//

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let mut server = Server::from(([127, 0, 0, 1], 13331)).unwrap();

    while let Ok(socket) = server.next().await {
        tokio::spawn(async move {
            if let Err(err) = async move {
                let socket = socket.await?;
                socket
                    .send(Packet::ordered(format!("Hello {}!", socket.remote()), 0))
                    .await?;
                socket.endpoint().wait_idle().await;

                Ok::<_, anyhow::Error>(())
            }
            .await
            {
                tracing::error!("Network error: {err}")
            }
        });
    }
}
