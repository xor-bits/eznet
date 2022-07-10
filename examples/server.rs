use eznet::{listener::Listener, packet::Packet};
use std::net::{Ipv4Addr, SocketAddrV4};

//

#[tokio::main]
pub async fn main() {
    env_logger::init();

    let mut listener =
        Listener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 13331).into()).unwrap();
    let mut socket = listener.next().await.unwrap();

    log::info!("Start receiving ordered");

    let mut i = 0;
    for _ in 0..20000_u16 {
        let j = u16::from_be_bytes(
            (&socket.recv().await.unwrap().bytes[..])
                .try_into()
                .unwrap(),
        );
        if j < i {
            log::error!("Out of order packet");
        }
        i = j;
    }

    log::info!("all received");

    socket
        .send(Packet::ordered_static(b"continue", None))
        .await
        .unwrap();

    log::info!("Start receiving unordered");

    let mut i = 0;
    let mut c = 0;
    for _ in 0..20000_u16 {
        let j = u16::from_be_bytes(
            (&socket.recv().await.unwrap().bytes[..])
                .try_into()
                .unwrap(),
        );
        if j < i {
            c += 1;
        }
        i = j;
    }

    log::info!("all received, {c}/20000 out of order");

    socket
        .send(Packet::ordered_static(b"continue", None))
        .await
        .unwrap();

    log::info!("Start receiving unreliable sequenced");

    let mut i = 0;
    let mut c = 0;
    while let Some(packet) = socket.recv().await {
        let j = u16::from_be_bytes((&packet.bytes[..]).try_into().unwrap());

        c += 1;
        if j < i {
            log::error!("Out of order packet");
        }
        i = j;
    }

    log::info!("Got {c}/20000 packets");
}
