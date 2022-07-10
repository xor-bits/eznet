use eznet::{listener::Listener, packet::Packet};

//

#[tokio::main]
async fn main() {
    let mut listener = Listener::bind("127.0.0.1:13331".parse().unwrap()).unwrap();

    while let Ok(socket) = listener.next().await {
        socket
            .send(Packet::ordered(format!("Hello {}!", socket.remote()), None))
            .await
            .unwrap();
    }
}
