# ironcraft
ironcraft is a **Minecraft: Java Edition** networking library. It is not a server implementation, but rather a library that can be used to create your own servers and clients.
The ultimate goal is to use this library in a future implementation of a Minecraft: Java Edition server
in Rust. 

This library is provided as a structured baseline and as an open source software solution for anyone looking
to create specialized **Minecraft: Java Edition** servers or clients. However, the library will be mostly built from a server perspective, with optimizations and
features made for a server environment.

Here is a current example of handling the server list status.

```rust
fn main() {
    SimpleLogger::new().init().unwrap();
    debug!("Starting server");

    let server = TcpListener::bind("127.0.0.1:25565").await.unwrap();

    loop {
        let (socket, a) = server.accept().await.unwrap();
    
        let mut client = CraftClient::from_connection(socket).unwrap();
    
        let response = UniversalStatusResponse::new(ProtocolVerison::v1_20, "§a§lThis is a test description §b§kttt");
    
    
        DefaultHandshakeHandler::handle_handshake(&mut client).await.unwrap();
        DefaultStatusHandler::handle_status(&mut client, response, DefaultPingHandler).await.unwrap();
    }
}
```