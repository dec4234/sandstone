# Packets and how they are used
In order to be able to use this library, you must have a very good understanding of what packets are and how
they are used in the context of the Minecraft protocol.

## Basic internal structure

First, a packet in the context of the minecraft protocol is an organized group of bytes sent from one connection to another.
This could be from the client to the server or the server to the client. All packets follow a very specific and rigid format which
is useful for efficiently decoding them.

The format of a packet is as follows:
```
+----------------+----------------+----------------+----------------+
| Packet Length  | Packet ID      | Packet Data...                 |
| (VarInt)       | (VarInt)       | (Byte Array)                   |
+----------------+----------------+----------------+----------------+
```

Ok, we now have some new ideas here to dissect. First, the `Packet Length` is a VarInt. A VarInt is a variable length integer type
that is specific to the Minecraft API. It allows for more efficient byte usage overall, as the vast majority of integers can be represented
in less than 4 bytes. However, in some rare circumstances a VarInt could take 5 bytes to represent a 4 byte integer, but this is rare enough
that is cuts down on the overall byte usage.

The packet length describes the length in bytes of the following `Packet ID` and `Packet Data`. We can use this to determine how many bytes
we should read from the connection to get the full packet.

The `Packet ID` is a VarInt that describes the type of packet that is being sent. When used in conjunction with the current network "state", we
can determine the specific packet that we should try to deserialize.

Now that was just the internal data for a packet. On top of that, we also have to account for compression and then encryption. The compression threshold
is determined when a user starts the login sequence to a server. The server will send a `SetCompression` packet to the client, and all subsequent packets
will be compressed if they are larger than the threshold. This is done to save bandwidth and increase the speed of the connection.

## Building and sending packets
The power of this library comes from the ability to effortlessly create packets and distribute them to clients, without having to worry about all the stuff
above. 