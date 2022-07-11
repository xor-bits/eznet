use eznet::listener::Listener;
use tokio::{select, sync::broadcast};

//

#[tokio::main]
async fn main() {
    let mut listener = Listener::bind("localhost:13331").unwrap();

    let (tx, rx) = broadcast::channel(256);

    while let Ok(mut socket) = listener.next().await {
        let (tx, mut rx) = (tx.clone(), rx.resubscribe());

        tokio::spawn(async move {
            let (socket_tx, mut socket_rx) = socket.split();

            loop {
                select! {
                    Ok(from_broadcast) = rx.recv() => {
                        socket_tx
                        .send(from_broadcast)
                        .await
                        .unwrap();
                    }
                    Some(to_broadcast) = socket_rx.recv() => {
                        tx.send(to_broadcast).unwrap();
                    }
                }
            }
        });
    }
}
