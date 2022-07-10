use eznet::{listener::Listener, packet::Packet};

//

#[tokio::main]
async fn main() {
    let mut listener = Listener::bind("localhost:13331").unwrap();

    while let Ok(socket) = listener.next().await {
        socket
            .send(Packet::ordered(format!("Hello {}!", socket.remote()), None))
            .await
            .unwrap();
    }
}
