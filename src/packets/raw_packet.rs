use std::fmt::{Debug, Display, Error, Formatter};
use serde::{ser, Serialize, Serializer};
use crate::packets::packet_definer::Packet;
use crate::protocol_details::datatypes::var_types::{VarInt};
use anyhow::Result;

/*pub struct RawPacket {
    Length: VarInt,
    PacketID: VarInt,
    Data: Vec<u8>
}*/

pub trait RawPacketBodyTrait {
    fn from_packet<P: Packet>(p: P) -> Self;
}

pub struct UncompressedRawPacketBody {
    length: VarInt,
    packetID: VarInt,
    data: Vec<u8>
}

impl RawPacketBodyTrait for UncompressedRawPacketBody {
    fn from_packet<P: Packet>(p: P) -> Self {
        todo!()
    }
}

pub struct CompressedRawPacketBody {
    compressedLength: VarInt,
    dataLength: VarInt,
    packetID: VarInt,
    data: Vec<u8>
}

pub enum RawPacket {
    UNCOMPRESSED(UncompressedRawPacketBody),
    COMPRESSED(CompressedRawPacketBody)
}

impl Serialize for RawPacket {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer,
    {
        match self {
            RawPacket::UNCOMPRESSED(body) => {
                todo!()
            }
            RawPacket::COMPRESSED(body) => {
                todo!()
            }
        }
    }
}