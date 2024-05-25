# ironcraft
ironcraft is a **Minecraft: Java Edition** networking library. It is not a server implementation, but rather a library that can be used to create your own server-sided software.
The ultimate goal is to use this library in a future implementation of a Minecraft: Java Edition server
in Rust. 

This library is provided as a structured baseline and as an open source software solution for anyone looking
to create specialized **Minecraft: Java Edition** servers. It is built with convenient optimizations and abstractions in mind. 

The library currently has a fully custom packet serializer and deserializer, as well as a client connection handler.

Here is a current example of handling the server list status.

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
		DefaultStatusHandler::handle_status(&mut client, response, DefaultPingHandler).await.unwrap();
	}
}
```

Which produces this...
<a href="https://gyazo.com/b9b3907a5f3c62898e06b8634cbe8b9f"><img src="https://i.gyazo.com/b9b3907a5f3c62898e06b8634cbe8b9f.gif" alt="Image from Gyazo" width="618"/></a>

More examples can be found in the [examples/](examples) folder.

## TODO
The actual TODO list is massive, but here are the current focuses of the project. Main priorities are listed first.

- [x] Figure out what to do with packets and begin implementing a full version
  - [ ] Deserialize given standard info tests
- [ ] Begin basic login procedure handler?
- [ ] Compression support
- [ ] Encryption support

## Disclaimer
Please note that this project is under heavy development and functions might not be heavily optimized yet.
Please also note that encryption might not be entirely secure, so please use online features with caution.