<div align="center">

# rnet

a simple net lib

</div>

[ENet](http://enet.bespin.org/)/[laminar](https://github.com/TimonPost/laminar)
style, [Quinn](https://github.com/quinn-rs/quinn) ([QUIC](https://en.wikipedia.org/wiki/QUIC))
based, simple to use and async net lib with configurable reliability and ordering.

## Example:

```rust
// examples/simple-server.rs
let bind_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 13331);
let mut listener = Listener::bind(bind_addr.into());

while let Some(socket) = listener.next().await {
    socket
        .send(Packet::ordered_from(
            format!("Hello {}!", socket.remote()).as_bytes(),
            None,
        ))
        .await
        .unwrap();
}

// examples/simple-client.rs
let server_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 13331);
let mut socket = Socket::connect(server_addr.into()).await.unwrap();

println!(
    "{}",
    std::str::from_utf8(&socket.recv().await.unwrap().bytes[..]).unwrap()
);
```

## TODO:

- 1: Encryption doesn't protect
  from MITM attacks at the moment.
  Add certificates, private keys,
  server names and DNS

- 2: Open socket magic byte test to
  filter out random scanners and
  'accidental' connections.

- 3: Disconnect message when closing

- 4: Configurable buffer capacity
