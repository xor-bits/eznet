use eznet::socket::Socket;

//

#[tokio::main]
async fn main() {
    let mut socket = Socket::connect("localhost:13331").await.unwrap();

    println!(
        "{}",
        std::str::from_utf8(&socket.recv().await.unwrap().bytes[..]).unwrap()
    );
}
