use eznet::{listener::Listener, packet::Packet};

//

#[tokio::main]
pub async fn main() {
    env_logger::init();

    let mut listener = Listener::bind("localhost:13331").unwrap();
    let mut socket = listener.next().await.unwrap();

    tracing::info!("Start receiving ordered");

    let mut i = 0;
    for _ in 0..20000_u16 {
        let j = u16::from_be_bytes(
            (&socket.recv().await.unwrap().bytes[..])
                .try_into()
                .unwrap(),
        );
        if j < i {
            tracing::error!("Out of order packet {i} {j}");
        }
        i = j;
    }

    tracing::info!("all received");

    socket
        .send(Packet::ordered_static(b"continue", None))
        .await
        .unwrap();

    tracing::info!("Start receiving unordered");

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

    tracing::info!("all received, {c}/20000 out of order");

    socket
        .send(Packet::ordered_static(b"continue", None))
        .await
        .unwrap();

    tracing::info!("Start receiving unreliable sequenced");

    let mut i = 0;
    let mut c = 0;
    while let Some(packet) = socket.recv().await {
        let j = u16::from_be_bytes((&packet.bytes[..]).try_into().unwrap());

        c += 1;
        if j < i {
            tracing::error!("Out of order packet");
        }
        i = j;
    }

    tracing::info!("Got {c}/20000 packets");
}
