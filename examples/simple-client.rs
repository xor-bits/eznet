use eznet::client::Client;

//

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::new();

    loop {
        let mut socket = client
            .connect_wait(([127, 0, 0, 1], 13331), "simple-server")
            .await?;

        let packet = &socket.recv().await?.bytes[..];
        let message = std::str::from_utf8(packet)?;
        println!("{message}");
    }

    Ok(())
}
