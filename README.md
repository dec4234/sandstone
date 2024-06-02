# sandstone
[![Crates.io](https://img.shields.io/crates/v/sandstone)](https://crates.io/crates/sandstone)
[![Docs.rs](https://docs.rs/sandstone/badge.svg)](https://docs.rs/sandstone)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/ironcraft/sandstone/CI)

sandstone is a **Minecraft: Java Edition** networking library. It is not a server implementation, but rather a library that can be used to create your own server-sided software.
The ultimate goal is to use this library in a future implementation of a Minecraft: Java Edition server
in Rust. 

This library is provided as a structured baseline and as an open source software solution for anyone looking
to create specialized **Minecraft: Java Edition** servers. It is built with convenient optimizations and abstractions in mind. 

The library currently has a fully custom packet serializer and deserializer, as well as a client connection handler.

Here is a current example of handling the server list status.

This project will be a continuous work in progress, and may see periods of no activity.

```rust
fn main() {
    SimpleLogger::new().init().unwrap();
	debug!("Starting server");

	let server = TcpListener::bind("127.0.0.1:25565").await.unwrap();

	loop {
		let (socket, a) = server.accept().await.unwrap();
		
		let mut client = CraftClient::from_connection(socket).unwrap();
		
		let mut response = UniversalStatusResponse::new(ProtocolVerison::V1_20, "&a&lThis is a test description &b§kttt");
		response.set_player_info(1, 0, vec![PlayerSample::new_random("&6&lTest")]);
		
		let image = image::open("src/server-icon.png").unwrap();
		response.set_favicon_image(image);
		
		DefaultHandshakeHandler::handle_handshake(&mut client).await.unwrap();
		DefaultStatusHandler::handle_status(&mut client, StatusResponseBody::new(response), DefaultPingHandler).await.unwrap();
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
- [ ] Begin basic login procedure handler?
- [ ] Compression support
- [ ] Encryption support
- [ ] Begin server structure 
- [ ] Documentation
  - [ ] Give explainer line for every file
  - [ ] Document all public functions
  - [ ] Copyright notices

## Disclaimer
Please note that this project is under heavy development and functions might not be heavily optimized yet.<br>
Please also note that encryption has not been rigorously tested for security, so please use online features with caution.