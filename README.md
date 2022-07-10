<div align="center">

# eznet

a simple net lib

[![dependency status](https://deps.rs/repo/github/Overpeek/eznet/status.svg)](https://deps.rs/repo/github/Overpeek/eznet)
[![build status](https://github.com/Overpeek/eznet/actions/workflows/rust.yml/badge.svg)](https://github.com/Overpeek/eznet/actions)
[![crates.io](https://img.shields.io/crates/v/eznet.svg?label=eznet)](https://crates.io/crates/eznet)
[![docs.rs](https://docs.rs/eznet/badge.svg)](https://docs.rs/eznet/)

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
let mut listener = Listener::bind("127.0.0.1:13331".parse().unwrap());

while let Ok(socket) = listener.next().await {
    socket
        .send(Packet::ordered(format!("Hello {}!", socket.remote()), None))
        .await
        .unwrap();
}

// examples/simple-client.rs
let mut socket = Socket::connect("127.0.0.1:13331".parse().unwrap())
    .await
    .unwrap();

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

- [x] Open socket magic byte test to
      filter out random scanners and
      'accidental' connections. (2)

- [ ] Disconnect message when closing. (3)

- [ ] Configurable buffer capacity. (4)

- [x] if packets are sent slightly faster
      than once per millisecond, none of them
      get actually sent, all of them are buffered. (5)

- [x] actually drop 'old' sequenced packets (6)

- [ ] list of breaking versions and
      testing it when filtering (7)

- [ ] More unit tests

## License

Licensed under either of [MIT license](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) license.

I am not a lawyer.
