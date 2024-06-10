# Server - Client Handling
The purpose of this is to roughly outline the server-client handling of the project.

First, the server will capture a hashmap of all the clients where the key is the client's entity ID or
UUID, or some other unique identifier.

There will be a threadpool that handles status requests, and another threadpool that handles client logins.
Each client will receive a dedicated thread for reading all incoming requests.

The dedicated handler will receive groups of packets, making sure to check and follow that one 
packet that denotes groups of packets. Packets will either then be sent individually or as the groups
they come in to an MPSC channel to the server.

A single thread on the server will receive all the packets and use the provided unique identifier to understand
the specific player that it refers to. The server will then handle the packets accordingly.

Somehow we have to separate packet groups across ticks, maybe the MPSC channel is cleared after each tick?

See more in this file from MCHPRS: [packet_handler.rs](https://github.com/MCHPR/MCHPRS/blob/master/crates/core/src/plot/packet_handlers.rs)