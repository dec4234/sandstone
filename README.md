# sandstone
[![Crates.io](https://img.shields.io/crates/v/sandstone)](https://crates.io/crates/sandstone)
[![Docs.rs](https://docs.rs/sandstone/badge.svg)](https://docs.rs/sandstone)

sandstone is a **Minecraft: Java Edition** networking library. It is not a server implementation, but rather a library that can be used to create your own server-sided software.
The ultimate goal is to use this library in a future implementation of a Minecraft: Java Edition server
in Rust. 

**This project will be a continuous work in progress, and may see months of no activity.**

This library is provided as a structured baseline and as an open source software solution for anyone looking
to create specialized **Minecraft: Java Edition** servers. It is built with convenient optimizations and abstractions in mind. 

The library currently has a fully custom packet serializer and deserializer, as well as a client connection handler.

Here is a current example of handling the server list status.

```rust
#[tokio::main]
async fn main() { 
  SimpleLogger::new().init().unwrap();
  debug!("Starting server");

  let server = TcpListener::bind("127.0.0.1:25565").await.unwrap();

  loop {
      let (socket, _) = server.accept().await.unwrap();
      
      let mut client = CraftClient::from_connection(socket).unwrap();
      
      let mut response = StatusResponseSpec::new(ProtocolVerison::V1_20, "&a&lThis is a test description &b§kttt");
      response.set_player_info(1, 0, vec![PlayerSample::new_random("&6&lTest")]);
      
      let image = image::open("src/server-icon.png").unwrap();
      response.set_favicon_image(image);
      
      DefaultHandshakeHandler::handle_handshake(&mut client).await.unwrap();
      DefaultStatusHandler::handle_status(&mut client, StatusResponsePacket::new(response), DefaultPingHandler).await.unwrap();
  }
}
```

Which produces this...<br>
<a href="https://gyazo.com/b9b3907a5f3c62898e06b8634cbe8b9f"><img src="https://i.gyazo.com/b9b3907a5f3c62898e06b8634cbe8b9f.gif" alt="Image from Gyazo" width="618"/></a>

More examples can be found in the [examples/](examples) folder.

## TODO
The actual TODO list is massive, but here are the current priorities for the project.

- [x] Figure out what to do with packets and begin implementing a full version
  - [ ] Deserialize given standard info tests
  - [x] How to handle optional fields ... See Packet::LoginPluginResponse
  - [x] Implement Java's [bitset](https://docs.oracle.com/javase/8/docs/api/java/util/BitSet.html) for bit fields
  - [ ] Maybe implement an Identifier struct? - See minecraft api types
- [x] Utilities
  - [ ] Thread pool for new connections
  - [ ] Auto generate serialization/deserialization tests for all packets?
    - [ ] Macro
    - [ ] Default field trait? Derive?
- [ ] Begin basic login procedure handler?
- [ ] Compression support
- [ ] Encryption support
- [ ] Begin server structure 
- [ ] Documentation
  - [ ] Give explainer line for every file
  - [ ] Document all public functions
    - [ ] Better examples and documentation for packet reading
  - [ ] Copyright notices

## Disclaimer
Please note that this project is under heavy development and functions might not be heavily optimized yet.<br>
Please also note that encryption has not been rigorously tested for security, so please use online features with caution.

## References
- [Protocol Wiki](https://minecraft.wiki/w/Java_Edition_protocol) = The main resource for implementing new functions for the library. This wiki has extensive documentation
on everything relating to the Minecraft protocol, both Bedrock and Java Edition. It also has a lot of articles relating to encryption,
the Mojang protocol, and other hidden details about the game. 

Some other projects were consulted for general design and handling for the minecraft protocol.

- [feather](https://github.com/feather-rs/feather)
- [valence](https://github.com/valence-rs/valence)
- [MCHPRS](https://github.com/MCHPR/MCHPRS)

The following projects were significantly useful for understanding some quirks with the Minecraft protocol. It also guided
me on early version of McSerializer and McDeserializer (instead of using serde).

- [mcproto-rs](https://github.com/Twister915/mcproto-rs)
- [craftio-rs](https://github.com/Twister915/craftio-rs)