use rnet::{listener::Listener, packet::Packet};
use std::net::{Ipv4Addr, SocketAddrV4};

//

#[tokio::main]
async fn main() {
    let bind_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 13331);
    let mut listener = Listener::bind(bind_addr.into());

    while let Some(socket) = listener.next().await {
        socket
            .send(Packet::ordered_from(
                format!("Hello {}!", socket.remote()).as_bytes(),
                None,
            ))
            .await
            .unwrap();
    }
}
