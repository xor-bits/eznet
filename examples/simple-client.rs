use eznet::socket::Socket;

//

#[tokio::main]
async fn main() {
    let mut socket = Socket::connect("127.0.0.1:13331".parse().unwrap())
        .await
        .unwrap();

    println!(
        "{}",
        std::str::from_utf8(&socket.recv().await.unwrap().bytes[..]).unwrap()
    );
}
