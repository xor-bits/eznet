use futures::future::join_all;
use rnet::{packet::Packet, socket::Socket};
use std::net::{Ipv4Addr, SocketAddrV4};

//

#[tokio::main]
pub async fn main() {
    env_logger::init();

    let mut socket = Socket::connect(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 13331).into())
        .await
        .unwrap();

    log::info!("Start sending ordered");

    for i in 0..20000_u16 {
        socket
            .send(Packet::copy_from_slice(&i.to_be_bytes()).as_reliable_ordered(None))
            .await
            .unwrap();
    }

    log::info!("all sent");

    socket.recv().await.unwrap();

    log::info!("Start sending unordered");

    for i in 0..20000_u16 {
        socket
            .send(Packet::copy_from_slice(&i.to_be_bytes()).as_reliable_unordered())
            .await
            .unwrap();
    }

    log::info!("all sent");

    socket.recv().await.unwrap();

    log::info!("Start sending unreliable");

    join_all((0..20000_u16).map(|_| {
        let socket = &socket;
        async move {
            socket
                .send(Packet::copy_from_slice(b"some unreliable bytes").as_unreliable())
                .await
                .unwrap();
        }
    }))
    .await;

    log::info!("all sent");
}
