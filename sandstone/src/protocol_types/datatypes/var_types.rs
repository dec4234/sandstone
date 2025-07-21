//! The purpose of this file is to define the custom integer types for the Minecraft protocol, VarInt and VarLong.
//! See more details here: https://wiki.vg/Protocol#VarInt_and_VarLong

use crate::network::network_error::NetworkError;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{
    McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult,
};
use std::fmt;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;
use tokio::net::TcpStream;
use uuid::Uuid;

const SEGMENT_INT: i32 = 0x7F;
const SEGMENT_LONG: i64 = 0x7F;
const SEGMENT_INT_OPP: i32 = !SEGMENT_INT; // cache these to avoid it at runtime
const SEGMENT_LONG_OPP: i64 = !SEGMENT_LONG;
const CONTINUE_INT: i32 = 0x80;
const CONTINUE_LONG: i64 = 0x80;
pub(crate) const CONTINUE_BYTE: u8 = 0x80; // 10000000

/// A VarInt is a packaged i32. It is represented in a more compressed (on average) byte format than
/// a typical i32. The most significant bit of each byte is used to indicate if there are more bytes
/// to be read, up to a max of 5.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, )]
#[repr(C)] // TODO: maybe remove
pub struct VarInt(pub i32);

impl VarInt {
    /// Convert a slice of bytes into a VarInt. Reading algorithm taken from https://wiki.vg/
    pub fn from_slice(bytes: &[u8]) -> Result<Self, SerializingErr> {
        if bytes.len() > 5 {
            return Err(SerializingErr::VarTypeTooLong(
                "VarInt must be a max of 5 bytes.".to_string(),
            ));
        }

        let mut i: i32 = 0;
        let mut pos = 0;

        for b in bytes {
            if *b == 0 {
                break;
            }

            let local: i32 = *b as i32;

            i |= (local & SEGMENT_INT) << pos;

            if (local & CONTINUE_INT) == 0 {
                // Early termination
                break;
            }

            pos += 7;

            if pos >= 32 {
                return Err(SerializingErr::UniqueFailure(
                    "Bit length is too long".to_string(),
                ));
            }
        }

        Ok(VarInt(i))
    }

    /// Extract a VarInt from a TcpStream. This reads the bytes until it finds a byte that does not have the continue bit set.
    ///
    /// This is usually used for reading the packet length VarInt from the start of a packet.
    pub fn from_tcp_stream(stream: &TcpStream) -> Result<Self, NetworkError> {
        let mut vec = Vec::with_capacity(3);

        loop {
            let var_buffer = &mut [0u8; 1];
            let len = stream.try_read(var_buffer)?;

            if len == 0 {
                return Err(NetworkError::NoDataReceived);
            }

            let b = var_buffer[0];

            if b & crate::network::CONTINUE_BIT == 0 {
                vec.push(b);
                break;
            } else {
                vec.push(b);

                if vec.len() > 3 {
                    return Err(SerializingErr::VarTypeTooLong("Packet length VarInt max bytes is 3".to_string()).into());
                }
            }
        }

        Ok(VarInt::from_slice(&vec)?)
    }

    /// Convert the VarInt into a Vec of bytes which can be serialized, or converted back to a VarInt using `from_slice`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::with_capacity(5);
        let mut inner = self.0;

        loop {
            if (inner & SEGMENT_INT_OPP) == 0 {
                vec.push(inner as u8);
                break;
            }

            vec.push((inner | CONTINUE_INT) as u8); // this is boolean simplified from the wiki.vg example

            // https://stackoverflow.com/questions/70212075/how-to-make-unsigned-right-shift-in-rust
            inner = {
                if inner >= 0 {
                    inner >> 7
                } else {
                    ((inner as u32) >> 7) as i32
                }
            };
        }

        vec
    }

    pub fn bytes(i: i32) -> Vec<u8> {
        let var = VarInt(i);

        var.to_bytes()
    }
}

impl Display for VarInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = String::from_utf8(self.to_bytes()).map_err(|_| Error)?;

        f.write_str(&s)
    }
}

impl FromStr for VarInt {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();

        if bytes.len() <= 0 || bytes.len() > 5 {
            return Err(Error);
        }

        let var_int = VarInt::from_slice(bytes);

        match var_int {
            Ok(var) => Ok(var),
            Err(_e) => Err(Error),
        }
    }
}

impl McSerialize for VarInt {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
        serializer.serialize_vec(self.to_bytes());

        Ok(())
    }
}

impl McDeserialize for VarInt {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, VarInt> {
        let mut bytes = Vec::with_capacity(5);

        if deserializer.data.len() == 0 {
            return Err(SerializingErr::InvalidEndOfVarInt);
        }

        let mut i = 0;

        while deserializer.data[i + deserializer.index] & CONTINUE_BYTE == CONTINUE_BYTE {
            if i >= 4 {
                return Err(SerializingErr::VarTypeTooLong(
                    "VarInt must be a max of 5 bytes.".to_string(),
                ));
            }

            bytes.push(deserializer.data[i + deserializer.index]);
            i += 1;
        }

        if i == deserializer.data.len() {
            return Err(SerializingErr::InvalidEndOfVarInt);
        }

        bytes.push(deserializer.data[i + deserializer.index]);

        deserializer.increment(i + 1);

        let var = VarInt::from_slice(&bytes)?;

        Ok(var)
    }
}

impl From<i32> for VarInt {
    fn from(i: i32) -> Self {
        VarInt(i)
    }
}

impl From<&[u8]> for VarInt {
    fn from(bytes: &[u8]) -> Self {
        VarInt::from_slice(bytes).unwrap()
    }
}

/// A VarLong is a packaged i64. It is represented in a more compressed (on average) byte format than
/// a typical i64. The most significant bit of each byte is used to indicate if there are more bytes
/// to be read, up to a max of 10.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy, )]
#[repr(C)] // TODO: maybe remove
pub struct VarLong(pub i64);

impl VarLong {
    /// Convert a slice of bytes into a VarLong. Reading algorithm taken from https://wiki.vg/
    pub fn from_slice(bytes: &[u8]) -> SerializingResult<Self> {
        if bytes.len() > 10 {
            return Err(SerializingErr::UniqueFailure(
                "VarLong must be a max of 10 bytes.".to_string(),
            ));
        }

        let mut i: i64 = 0;
        let mut pos = 0;

        for b in bytes {
            if *b == 0 {
                break;
            }

            let local: i64 = *b as i64;

            i |= (local & SEGMENT_LONG) << pos;

            if (local & CONTINUE_LONG) == 0 {
                // Early termination
                break;
            }

            pos += 7;

            if pos >= 64 {
                return Err(SerializingErr::UniqueFailure(
                    "Bit length is too long".to_string(),
                ));
            }
        }

        Ok(VarLong(i))
    }

    pub fn new_from_bytes(bytes: Vec<u8>) -> Result<Self, SerializingErr> {
        // cannot use SerializingResult
        VarLong::from_slice(bytes.as_slice())
    }

    /// Convert the VarLong into a Vec of bytes which can be serialized, or converted back to a VarLong using `from_slice`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::with_capacity(10);
        let mut inner = self.0;

        loop {
            if (inner & SEGMENT_LONG_OPP) == 0 {
                vec.push(inner as u8);
                break;
            }

            vec.push((inner | CONTINUE_LONG) as u8); // this is boolean simplified from the wiki.vg example

            // https://stackoverflow.com/questions/70212075/how-to-make-unsigned-right-shift-in-rust
            inner = {
                if inner >= 0 {
                    inner >> 7
                } else {
                    ((inner as u64) >> 7) as i64
                }
            };
        }

        vec
    }

    pub fn bytes(i: i64) -> Vec<u8> {
        VarLong(i).to_bytes()
    }
}

impl Display for VarLong {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = String::from_utf8(self.to_bytes()).map_err(|_| Error)?;

        f.write_str(&s)
    }
}

impl FromStr for VarLong {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();

        if bytes.len() <= 0 || bytes.len() > 5 {
            return Err(Error);
        }

        let var_int = VarLong::from_slice(bytes);

        match var_int {
            Ok(var) => Ok(var),
            Err(_) => Err(Error),
        }
    }
}

impl McSerialize for VarLong {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
        serializer.serialize_vec(self.to_bytes());

        Ok(())
    }
}

impl McDeserialize for VarLong {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, VarLong> {
        let mut bytes = Vec::with_capacity(10);

        if deserializer.data.len() == 0 {
            return Err(SerializingErr::InvalidEndOfVarInt);
        }

        let mut i = 0;

        while i + deserializer.index < deserializer.data.len()
            && deserializer.data[i + deserializer.index] & CONTINUE_BYTE == CONTINUE_BYTE
        {
            if i >= 9 {
                return Err(SerializingErr::VarTypeTooLong(
                    "VarLong must be a max of 10 bytes.".to_string(),
                ));
            }

            bytes.push(deserializer.data[i + deserializer.index]);
            i += 1;
        }

        if i == deserializer.data.len() {
            return Err(SerializingErr::InvalidEndOfVarInt);
        }

        bytes.push(deserializer.data[i]);

        deserializer.increment(i);

        let var = VarLong::from_slice(&bytes)?;

        Ok(var)
    }
}

impl From<i64> for VarLong {
    fn from(i: i64) -> Self {
        VarLong(i)
    }
}

impl From<&[u8]> for VarLong {
    fn from(bytes: &[u8]) -> Self {
        VarLong::from_slice(bytes).unwrap()
    }
}

// For rust stuff go to serializer_types.rs

// 3rd party items

impl McSerialize for Uuid {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
        self.as_u128().mc_serialize(serializer)?; // serialized as u128 in mc protocol

        Ok(())
    }
}

impl McDeserialize for Uuid {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
    where
        Self: Sized,
    {
        Ok(Uuid::from_u128(u128::mc_deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::serialization::{
        McDeserialize, McDeserializer, McSerialize, McSerializer,
    };
    use crate::protocol_types::datatypes::var_types::{VarInt, VarLong};

    #[test]
    fn basic_varint_from_slice() {
        assert_eq!(VarInt::from_slice(&[221, 199, 1]).unwrap(), VarInt(25565));
        assert_eq!(
            VarInt::from_slice(&[255, 255, 127]).unwrap(),
            VarInt(2097151)
        );
        assert_eq!(
            VarInt::from_slice(&[255, 255, 255, 255, 15]).unwrap(),
            VarInt(-1)
        );
        assert_eq!(
            VarInt::from_slice(&[128, 128, 128, 128, 8]).unwrap(),
            VarInt(-2147483648)
        );
    }

    #[test]
    fn basic_varint_writing() {
        assert_eq!(
            VarInt::from_slice(&[221, 199, 1]).unwrap().to_bytes(),
            vec![221, 199, 1]
        );
        assert_eq!(
            VarInt::from_slice(&[255, 255, 127]).unwrap().to_bytes(),
            vec![255, 255, 127]
        );
        assert_eq!(
            VarInt::from_slice(&[255, 255, 255, 255, 15])
                .unwrap()
                .to_bytes(),
            vec![255, 255, 255, 255, 15]
        );
    }

    #[test]
    fn basic_varlong_from_slice() {
        assert_eq!(VarLong::from_slice(&[255, 1]).unwrap(), VarLong(255));
        assert_eq!(
            VarLong::from_slice(&[255, 255, 255, 255, 7]).unwrap(),
            VarLong(2147483647)
        );
        assert_eq!(
            VarLong::from_slice(&[255, 255, 255, 255, 255, 255, 255, 255, 255, 1]).unwrap(),
            VarLong(-1)
        );
        assert_eq!(
            VarLong::from_slice(&[128, 128, 128, 128, 248, 255, 255, 255, 255, 1]).unwrap(),
            VarLong(-2147483648)
        );
    }

    #[test]
    fn basic_varlong_writing() {
        assert_eq!(
            VarLong::from_slice(&[255, 1]).unwrap().to_bytes(),
            vec![255, 1]
        );
        assert_eq!(
            VarLong::from_slice(&[255, 255, 255, 255, 7])
                .unwrap()
                .to_bytes(),
            vec![255, 255, 255, 255, 7]
        );
        assert_eq!(
            VarLong::from_slice(&[255, 255, 255, 255, 255, 255, 255, 255, 255, 1])
                .unwrap()
                .to_bytes(),
            vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 1]
        );
        assert_eq!(
            VarLong::from_slice(&[128, 128, 128, 128, 248, 255, 255, 255, 255, 1])
                .unwrap()
                .to_bytes(),
            vec![128, 128, 128, 128, 248, 255, 255, 255, 255, 1]
        );
    }

    #[test]
    fn test_varint_serialization() {
        let mut serializer = McSerializer::new();

        VarInt(25565).mc_serialize(&mut serializer).unwrap();
        let mut deserializer = McDeserializer::new(&mut serializer.output);
        assert_eq!(25565, VarInt::mc_deserialize(&mut deserializer).unwrap().0);

        serializer.clear();
        VarInt(2097151).mc_serialize(&mut serializer).unwrap();
        deserializer = McDeserializer::new(&mut serializer.output);
        assert_eq!(
            2097151,
            VarInt::mc_deserialize(&mut deserializer).unwrap().0
        );

        serializer.clear();
        VarInt(-2147483648).mc_serialize(&mut serializer).unwrap();
        assert_eq!(serializer.output, vec![128, 128, 128, 128, 8]);

        serializer.clear();
        VarInt(-2147483648).mc_serialize(&mut serializer).unwrap();
        deserializer = McDeserializer::new(&mut serializer.output);
        assert_eq!(
            -2147483648,
            VarInt::mc_deserialize(&mut deserializer).unwrap().0
        );
    }

    #[test]
    fn test_varlong_serialization() {
        let mut serializer = McSerializer::new();

        VarLong(25565).mc_serialize(&mut serializer).unwrap();
        let mut deserializer = McDeserializer::new(&mut serializer.output);
        assert_eq!(25565, VarLong::mc_deserialize(&mut deserializer).unwrap().0);

        serializer.clear();
        VarLong(2097151).mc_serialize(&mut serializer).unwrap();
        deserializer = McDeserializer::new(&mut serializer.output);
        assert_eq!(
            2097151,
            VarLong::mc_deserialize(&mut deserializer).unwrap().0
        );

        serializer.clear();
        VarLong(9223372036854775807)
            .mc_serialize(&mut serializer)
            .unwrap();
        deserializer = McDeserializer::new(&mut serializer.output);
        assert_eq!(
            9223372036854775807,
            VarLong::mc_deserialize(&mut deserializer).unwrap().0
        );

        serializer.clear();
        VarLong(-2147483648).mc_serialize(&mut serializer).unwrap();
        deserializer = McDeserializer::new(&mut serializer.output);
        assert_eq!(
            -2147483648,
            VarLong::mc_deserialize(&mut deserializer).unwrap().0
        );

        serializer.clear();
        VarLong(-9223372036854775808)
            .mc_serialize(&mut serializer)
            .unwrap();
        assert_eq!(
            serializer.output,
            vec![128, 128, 128, 128, 128, 128, 128, 128, 128, 1]
        );
    }

    #[test]
    fn test_zero_handling() {
        assert_eq!(
            VarInt::from_slice(&[221, 199, 1, 0]).unwrap(),
            VarInt(25565)
        );
        assert_eq!(
            VarInt::from_slice(&[255, 255, 127, 0]).unwrap(),
            VarInt(2097151)
        );
        assert_eq!(
            VarInt::from_slice(&[255, 255, 255, 255, 15]).unwrap(),
            VarInt(-1)
        );
        assert_eq!(
            VarInt::from_slice(&[128, 128, 128, 128, 8]).unwrap(),
            VarInt(-2147483648)
        );
    }
}
