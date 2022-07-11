use eznet::{packet::Packet, socket::Socket};
use futures::StreamExt;
use tokio::{io::stdin, select};
use tokio_util::codec::{FramedRead, LinesCodec};

//

#[tokio::main]
async fn main() {
    let mut socket = Socket::connect("localhost:13331").await.unwrap();

    let name = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("Anonymous".to_owned());

    let stdin = stdin();
    let mut stdin = FramedRead::new(stdin, LinesCodec::default());
    let (tx, mut rx) = socket.split();

    loop {
        let _ = &socket;
        select! {
            from_broadcast = rx.recv() => {
                if let Some(from_broadcast) = from_broadcast {
                    if let Ok(s) = std::str::from_utf8(&from_broadcast.bytes[..]) {
                        println!("{s}");
                    }
                } else {
                    break;
                }
            }
            Some(Ok(to_broadcast)) = stdin.next() => {
                // *zero* client verification
                if tx.send(Packet::ordered(format!("[{name}]: {to_broadcast}"), None)).await.is_err() {
                    break;
                }
            }
        }
    }
}
