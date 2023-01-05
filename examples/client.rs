use eznet::{client::Client, packet::Packet, server::Server, socket::Socket};
use std::{time::Duration, net::SocketAddr};
use tokio::time::sleep;

//

#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt::init();

    let mut client = Client::new();

    let mut socket = client.connect(, server_name);

    let mut socket = Socket::connect("localhost:13331").await.unwrap();

    tracing::info!("Start sending ordered");

    for i in 0..20000_u16 {
        socket
            .send(Packet::ordered(&i.to_be_bytes(), None))
            .await
            .unwrap();
    }

    tracing::info!("all sent");

    socket.recv().await.unwrap();

    tracing::info!("Start sending unordered");

    for i in 0..20000_u16 {
        socket
            .send(Packet::reliable_unordered(&i.to_be_bytes()))
            .await
            .unwrap();
    }

    tracing::info!("all sent");

    socket.recv().await.unwrap();

    tracing::info!("Start sending unreliable");
    for i in 0..20000_u16 {
        socket
            .send(Packet::unreliable_sequenced(&i.to_be_bytes()[..], None))
            .await
            .unwrap();
    }

    tracing::info!("all sent");

    // give a bit more time for unreliable packets to be sent
    sleep(Duration::from_millis(100)).await;
}
