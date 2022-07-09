use rnet::{listener::Listener, packet::Packet};
use std::net::{Ipv4Addr, SocketAddrV4};

//

#[tokio::main]
pub async fn main() {
    env_logger::init();

    let mut listener = Listener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 13331).into());
    let mut socket = listener.next().await.unwrap();

    log::info!("Start receiving ordered");

    let mut i = 0;
    for _ in 0..20000_u16 {
        let j = u16::from_be_bytes((&socket.recv().await.unwrap()[..]).try_into().unwrap());
        log::info!("{j}");
        if j < i {
            log::error!("Out of order packet");
        }
        i = j;
    }

    log::info!("all received");

    socket
        .send(Packet::copy_from_slice(b"continue"))
        .await
        .unwrap();

    log::info!("Start receiving unordered");

    let mut i = 0;
    let mut c = 0;
    for _ in 0..20000_u16 {
        let j = u16::from_be_bytes((&socket.recv().await.unwrap()[..]).try_into().unwrap());
        if j < i {
            c += 1;
        }
        i = j;
    }

    log::info!("all received, {c}/20000 out of order");

    socket
        .send(Packet::copy_from_slice(b"continue"))
        .await
        .unwrap();

    log::info!("Start receiving unreliable");

    let mut c = 0;
    while let Some(_) = socket.recv().await {
        c += 1
    }

    log::info!("Got {c}/20000 unreliable packets");
}
