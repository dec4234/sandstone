use std::fmt;
use std::fmt::{Display, Error, Formatter, Write};
use std::str::FromStr;

use anyhow::{anyhow, Result};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer};

// https://wiki.vg/Protocol#VarInt_and_VarLong
const SEGMENT_INT: i32 = 0x7F;
const SEGMENT_LONG: i64 = 0x7F;
const CONTINUE_INT: i32 = 0x80;
const CONTINUE_LONG: i64 = 0x80;
pub(crate) const CONTINUE_BYTE: u8 = 0x80; // 10000000

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, AsBytes, FromBytes, FromZeroes, Clone, Copy)]
#[repr(C)]
pub struct VarInt(pub i32);

impl VarInt {

    // Reading algorithm taken from https://wiki.vg/
    // TODO: Optimize
    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        if(bytes.len() > 5) {
            return Err(anyhow!("Max bytes of byte array is 5. VarInts are i32"));
        }

        let mut i: i32 = 0;
        let mut pos = 0;

        for b in bytes {
            let local: i32 = *b as i32;

            i |= (local & SEGMENT_INT) << pos;

            if (local & CONTINUE_INT) == 0 { // Early termination
                break;
            }

            pos += 7;

            if(pos >= 32) {
                return Err(anyhow!("Bit length is too long"));
            }
        }

        return Ok(VarInt(i));
    }

    pub fn new_from_bytes(bytes: Vec<u8>) -> Result<Self> {
        return VarInt::from_slice(bytes.as_slice());
    }

    // TODO: optimize
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = vec![];
        let mut inner = self.0;

        loop {
            if (inner & !SEGMENT_INT) == 0 {
                vec.push(inner.to_le_bytes()[0]);
                break;
            }

            vec.push(((inner & SEGMENT_INT) | CONTINUE_INT) as u8);

            // https://stackoverflow.com/questions/70212075/how-to-make-unsigned-right-shift-in-rust
            inner = {
                if inner >= 0 {
                    inner >> 7
                } else {
                    ((inner as u32) >> 7) as i32
                }
            };
        }

        return vec;
    }

    pub fn bytes(i: i32) -> Vec<u8> {
        let var = VarInt(i);

        return var.to_bytes();
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

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let bytes = s.as_bytes();

        if bytes.len() <= 0 || bytes.len() > 5 {
            return Err(Error);
        }

        let varInt = VarInt::from_slice(bytes);

        match varInt {
            Ok(varI) => {Ok(varI)}
            Err(e) => {Err(Error)}
        }
    }
}

impl McSerialize for VarInt {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> std::result::Result<(), SerializingErr> {
        serializer.serialize_vec(self.to_bytes());

        Ok(())
    }
}

impl McDeserialize for VarInt {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, VarInt> {
        let mut bytes = vec![];

        if deserializer.data.len() == 0 {
            return Err(SerializingErr::InvalidEndOfVarInt);
        }

        let mut i = 0;

        while deserializer.data[i + deserializer.index] & CONTINUE_BYTE == CONTINUE_BYTE {
            if i >= 5 {
                return Err(SerializingErr::VarTypeTooLong("VarInt must be a max of 5 bytes.".to_string()));
            }

            bytes.push(deserializer.data[i + deserializer.index]);
            i += 1;
        }

        if i == deserializer.data.len() {
            return Err(SerializingErr::InvalidEndOfVarInt);
        }

        bytes.push(deserializer.data[i + deserializer.index]);

        deserializer.increment(i + 1);

        if bytes.len() > 5 {
            return Err(SerializingErr::VarTypeTooLong("VarInt must be a max of 5 bytes.".to_string()));
        }

        let var = VarInt::new_from_bytes(bytes);

        if var.is_err() {
            return Err(SerializingErr::UnknownFailure);
        }

        return Ok(var.unwrap()); // safe to unwrap because we check for error above
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, AsBytes, FromBytes, FromZeroes, Clone, Copy)]
#[repr(C)]
pub struct VarLong(pub i64);

impl VarLong {
    // Reading algorithm taken from https://wiki.vg/
    // TODO: Optimize
    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        if(bytes.len() > 10) {
            return Err(anyhow!("Max bytes of byte array is 10. VarLongs are i64"));
        }

        let mut i: i64 = 0;
        let mut pos = 0;

        for b in bytes {
            let local: i64 = *b as i64;

            i |= (local & SEGMENT_LONG) << pos;

            if (local & CONTINUE_LONG) == 0 { // Early termination
                break;
            }

            pos += 7;

            if(pos >= 64) {
                return Err(anyhow!("Bit length is too long"));
            }
        }

        return Ok(VarLong(i));
    }

    pub fn new_from_bytes(bytes: Vec<u8>) -> Result<Self> {
        return VarLong::from_slice(bytes.as_slice());
    }

    // TODO: optimize
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = vec![];
        let mut inner = self.0;

        loop {
            if (inner & !SEGMENT_LONG) == 0 {
                vec.push(inner.to_le_bytes()[0]);
                break;
            }

            vec.push(((inner & SEGMENT_LONG) | CONTINUE_LONG) as u8);

            // https://stackoverflow.com/questions/70212075/how-to-make-unsigned-right-shift-in-rust
            inner = {
                if inner >= 0 {
                    inner >> 7
                } else {
                    ((inner as u64) >> 7) as i64
                }
            };
        }

        return vec;
    }

    pub fn bytes(i: i64) -> Vec<u8> {
        let var = VarLong(i);

        return var.to_bytes();
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

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let bytes = s.as_bytes();

        if bytes.len() <= 0 || bytes.len() > 5 {
            return Err(Error);
        }

        let varInt = VarLong::from_slice(bytes);

        match varInt {
            Ok(varI) => {Ok(varI)}
            Err(e) => {Err(Error)}
        }
    }
}

impl McSerialize for VarLong {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> std::result::Result<(), SerializingErr> {
        serializer.serialize_vec(self.to_bytes());

        Ok(())
    }
}

impl McDeserialize for VarLong {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, VarLong> {
        let mut bytes = vec![];

        if deserializer.data.len() == 0 {
            return Err(SerializingErr::InvalidEndOfVarInt);
        }

        let mut i = 0;

        while i + deserializer.index < deserializer.data.len() && deserializer.data[i + deserializer.index] & CONTINUE_BYTE == CONTINUE_BYTE {
            if i >= 10 {
                return Err(SerializingErr::VarTypeTooLong("VarLong must be a max of 10 bytes.".to_string()));
            }

            bytes.push(deserializer.data[i + deserializer.index]);
            i += 1;
        }

        if i == deserializer.data.len() {
            return Err(SerializingErr::InvalidEndOfVarInt);
        }

        bytes.push(deserializer.data[i]);

        deserializer.increment(i);

        if bytes.len() > 10 {
            return Err(SerializingErr::VarTypeTooLong("VarLong must be a max of 10 bytes.".to_string()));
        }

        let var = VarLong::new_from_bytes(bytes);

        if var.is_err() {
            return Err(SerializingErr::UnknownFailure);
        }

        return Ok(var.unwrap());
    }
}

#[cfg(test)]
mod tests {
    use crate::packets::serialization::serializer_handler::{McDeserialize, McDeserializer, McSerialize, McSerializer};
    use crate::protocol_details::datatypes::var_types::{VarInt, VarLong};

    #[test]
    fn basic_varint_from_slice() {
        assert!(VarInt::from_slice(&[221, 199, 1]).unwrap() == VarInt(25565));
        assert!(VarInt::from_slice(&[255, 255, 127]).unwrap() == VarInt(2097151));
        assert!(VarInt::from_slice(&[255, 255, 255, 255, 15]).unwrap() == VarInt(-1));
        assert!(VarInt::from_slice(&[128, 128, 128, 128, 8]).unwrap() == VarInt(-2147483648));
    }

    #[test]
    fn basic_varint_writing() {
        assert!(VarInt::from_slice(&[221, 199, 1]).unwrap().to_bytes() == vec![221, 199, 1]);
        assert!(VarInt::from_slice(&[255, 255, 127]).unwrap().to_bytes() == vec![255, 255, 127]);
        assert!(VarInt::from_slice(&[255, 255, 255, 255, 15]).unwrap().to_bytes() == vec![255, 255, 255, 255, 15]);
    }

    #[test]
    fn basic_varlong_from_slice() {
        assert!(VarLong::from_slice(&[255, 1]).unwrap() == VarLong(255));
        assert!(VarLong::from_slice(&[255, 255, 255, 255, 7]).unwrap() == VarLong(2147483647));
        assert!(VarLong::from_slice(&[255, 255, 255, 255, 255, 255, 255, 255, 255, 1]).unwrap() == VarLong(-1));
        assert!(VarLong::from_slice(&[128, 128, 128, 128, 248, 255, 255, 255, 255, 1]).unwrap() == VarLong(-2147483648));
    }

    #[test]
    fn basic_varlong_writing() {
        assert!(VarLong::from_slice(&[255, 1]).unwrap().to_bytes() == vec![255, 1]);
        assert!(VarLong::from_slice(&[255, 255, 255, 255, 7]).unwrap().to_bytes() == vec![255, 255, 255, 255, 7]);
        assert!(VarLong::from_slice(&[255, 255, 255, 255, 255, 255, 255, 255, 255, 1]).unwrap().to_bytes() == vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 1]);
        assert!(VarLong::from_slice(&[128, 128, 128, 128, 248, 255, 255, 255, 255, 1]).unwrap().to_bytes() == vec![128, 128, 128, 128, 248, 255, 255, 255, 255, 1]);
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
        assert_eq!(2097151, VarInt::mc_deserialize(&mut deserializer).unwrap().0);

        serializer.clear();
        VarInt(-2147483648).mc_serialize(&mut serializer).unwrap();
        deserializer = McDeserializer::new(&mut serializer.output);
        assert_eq!(-2147483648, VarInt::mc_deserialize(&mut deserializer).unwrap().0);

        serializer.clear();
        VarInt(-2147483648).mc_serialize(&mut serializer).unwrap();
        assert_eq!(serializer.output, vec![128, 128, 128, 128, 8]);
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
        assert_eq!(2097151, VarLong::mc_deserialize(&mut deserializer).unwrap().0);

        serializer.clear();
        VarLong(9223372036854775807).mc_serialize(&mut serializer).unwrap();
        deserializer = McDeserializer::new(&mut serializer.output);
        assert_eq!(9223372036854775807, VarLong::mc_deserialize(&mut deserializer).unwrap().0);

        serializer.clear();
        VarLong(-2147483648).mc_serialize(&mut serializer).unwrap();
        deserializer = McDeserializer::new(&mut serializer.output);
        assert_eq!(-2147483648, VarLong::mc_deserialize(&mut deserializer).unwrap().0);

        serializer.clear();
        VarLong(-9223372036854775808).mc_serialize(&mut serializer).unwrap();
        assert_eq!(serializer.output, vec![128, 128, 128, 128, 128, 128, 128, 128, 128, 1]);
    }

    #[test]
    fn test_string_serialization() {
        let mut serializer = McSerializer::new();

        "ABC".to_string().mc_serialize(&mut serializer).unwrap();
        let mut deserializer = McDeserializer::new(&mut serializer.output);
        assert_eq!("ABC".to_string(), String::mc_deserialize(&mut deserializer).unwrap());
        assert_eq!(serializer.output, vec![3, 65, 66, 67]);

        serializer.clear();

        "HELLO WORLD 123456789".to_string().mc_serialize(&mut serializer).unwrap();
        let mut deserializer = McDeserializer::new(&mut serializer.output);
        assert_eq!("HELLO WORLD 123456789".to_string(), String::mc_deserialize(&mut deserializer).unwrap());
    }
}