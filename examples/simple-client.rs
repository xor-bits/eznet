use eznet::socket::Socket;
use std::net::{Ipv4Addr, SocketAddrV4};

//

#[tokio::main]
async fn main() {
    let server_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 13331);
    let mut socket = Socket::connect(server_addr.into()).await.unwrap();

    println!(
        "{}",
        std::str::from_utf8(&socket.recv().await.unwrap().bytes[..]).unwrap()
    );
}
