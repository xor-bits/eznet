<div align="center">

# eznet

a simple net lib

</div>

[ENet](http://enet.bespin.org/)/[laminar](https://github.com/TimonPost/laminar)
style, [Quinn](https://github.com/quinn-rs/quinn) ([QUIC](https://en.wikipedia.org/wiki/QUIC))
based, simple to use and async net lib with configurable reliability and ordering.

## Features:

 - Packets are encrypted (but not really securely [TODO](#todo): 1)
 
 - Reliable ordered, reliable sequenced, reliable unordered, unreliable sequenced and unreliable unordered packets
 
 - Easy to use
 
 - Async/await

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

- [ ] Encryption doesn't protect
      from MITM attacks at the moment.
      Only self signed server side
      certificates are used and clients
      accept everything. Add certificates,
      private keys, server names and DNS. (1)

- [ ] Open socket magic byte test to
      filter out random scanners and
      'accidental' connections. (2)

- [ ] Disconnect message when closing. (3)

- [ ] Configurable buffer capacity. (4)

- [ ] if packets are sent slightly faster
      than once per millisecond, none of them
      get actually sent, all of them are buffered. (5)

- [ ] actually drop 'old' sequenced packets (6)
