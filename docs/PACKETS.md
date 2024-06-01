# Packets and how they are used
In order to be able to use this library, you must have a very good understanding of what packets are and how
they are used in the context of the Minecraft protocol.

First, a packet in the context of the minecraft protocol is an organized group of bytes sent from one connection to another.
This could be from the client to the server or the server to the client. All packets follow a very specific and rigid format which
is useful for efficiently decoding them.