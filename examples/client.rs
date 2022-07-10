use futures::future::join_all;
use rnet::{packet::Packet, socket::Socket};
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};
use tokio::time::sleep;

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
            .send(Packet::ordered_from(&i.to_be_bytes(), None))
            .await
            .unwrap();
    }

    log::info!("all sent");

    socket.recv().await.unwrap();

    log::info!("Start sending unordered");

    for i in 0..20000_u16 {
        socket
            .send(Packet::reliable_unordered_from(&i.to_be_bytes()))
            .await
            .unwrap();
    }

    log::info!("all sent");

    socket.recv().await.unwrap();

    log::info!("Start sending unreliable");
    for i in 0..20000_u16 {
        socket
            .send(Packet::unreliable_sequenced_from(
                &i.to_be_bytes()[..],
                None,
            ))
            .await
            .unwrap();
    }

    log::info!("all sent");

    // give a bit more time for unreliable packets to be sent
    sleep(Duration::from_millis(100)).await;
}
