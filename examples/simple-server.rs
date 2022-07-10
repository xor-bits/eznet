use eznet::{listener::Listener, packet::Packet};

//

#[tokio::main]
async fn main() {
    let mut listener = Listener::bind("127.0.0.1:13331".parse().unwrap());

    while let Ok(socket) = listener.next().await {
        socket
            .send(Packet::ordered_from(
                format!("Hello {}!", socket.remote()).as_bytes(),
                None,
            ))
            .await
            .unwrap();
    }
}
