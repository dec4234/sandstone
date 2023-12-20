use std::fmt;
use std::fmt::{Display, Error, Formatter, Write};
use std::str::FromStr;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

// https://wiki.vg/Protocol#VarInt_and_VarLong
const SEGMENT_INT: i32 = 0x7F;
const SEGMENT_LONG: i64 = 0x7F;
const CONTINUE_INT: i32 = 0x80;
const CONTINUE_LONG: i64 = 0x80;

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

impl Serialize for VarInt {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_bytes(self.to_bytes().as_slice())
    }
}

impl <'de> Deserialize<'de> for VarInt { // https://serde.rs/impl-deserialize.html
    fn deserialize<D>(des: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
        des.deserialize_bytes(VarIntVisitor)
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, AsBytes, FromBytes, FromZeroes, Clone, Copy)]
#[repr(C)]
pub struct VarLong(i64);

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

impl Serialize for VarLong {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_bytes(self.to_bytes().as_slice())
    }
}

impl <'de> Deserialize<'de> for VarLong { // https://serde.rs/impl-deserialize.html
    fn deserialize<D>(des: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
        des.deserialize_bytes(VarLongVisitor)
    }
}

const CONTINUE_BYTE: u8 = 0x80; // 10000000

pub struct VarIntVisitor;

impl <'de> Visitor<'de> for VarIntVisitor {
    type Value = VarInt;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("Could not deserialize VarInt")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E> where E: serde::de::Error {
        // return Err(serde::de::Error::custom("Byte array length greater than 5"));

        let mut vec: Vec<u8> = Vec::new();
        let mut i = 0;

        while (v[i] & CONTINUE_BYTE) == CONTINUE_BYTE {
            if i > 5 {
                return Err(serde::de::Error::custom("Byte array length greater than 5"));
            }

            vec.push(v[i]);

            i += 1;
        }

        let var = VarInt::new_from_bytes(vec);

        if let Ok(v) = var{
            return Ok(v);
        }

        return Err(serde::de::Error::custom("Deserialization of VarInt failed"));
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let mut el = seq.next_element::<u8>();
        let mut vec: Vec<u8> = Vec::new();

        while el.is_ok() {
            let o = el.unwrap();

            if let Some(u) = o {
                vec.push(u);
            } else {
                break;
            }

            el = seq.next_element::<u8>();
        }

        if vec.len() > 5 {
            return Err(serde::de::Error::custom("Size is greater than 5"));
        }

        let var = VarInt::new_from_bytes(vec);

        match var {
            Ok(v) => {Ok(v)}
            Err(e) => {Err(serde::de::Error::custom("Deserialization of VarInt failed"))}
        }
    }
}

pub struct VarLongVisitor;

impl <'de> Visitor<'de> for VarLongVisitor {
    type Value = VarLong;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("Could not deserialize VarLong")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E> where E: serde::de::Error {
        // return Err(serde::de::Error::custom("Byte array length greater than 5"));

        let mut vec: Vec<u8> = Vec::new();
        let mut i = 0;

        while (v[i] & CONTINUE_BYTE) == CONTINUE_BYTE {
            if i > 10 {
                return Err(serde::de::Error::custom("Byte array length greater than 10"));
            }

            vec.push(v[i]);

            i += 1;
        }

        let var = VarLong::new_from_bytes(vec);

        if let Ok(v) = var{
            return Ok(v);
        }

        return Err(serde::de::Error::custom("Deserialization of VarLong failed"));
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        let mut el = seq.next_element::<u8>();
        let mut vec: Vec<u8> = Vec::new();

        while el.is_ok() {
            let o = el.unwrap();

            if let Some(u) = o {
                vec.push(u);
            } else {
                break;
            }

            el = seq.next_element::<u8>();
        }

        if vec.len() > 10 {
            return Err(serde::de::Error::custom("Size is greater than 10"));
        }

        let var = VarLong::new_from_bytes(vec);

        match var {
            Ok(v) => {Ok(v)}
            Err(e) => {Err(serde::de::Error::custom("Deserialization of VarLong failed"))}
        }
    }
}


#[cfg(test)]
mod tests {
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
    fn deserialize_serialize_varint() {
        let v1 = VarInt(256);
        let j1 = serde_json::to_string(&v1).unwrap();
        assert_eq!(j1.as_str(), "[128,2]");
        let v1_reverse: VarInt = serde_json::from_str(j1.as_str()).unwrap();
        assert_eq!(v1_reverse.0, 256);

        let v1 = VarInt(25565);
        let j1 = serde_json::to_string(&v1).unwrap();
        assert_eq!(j1.as_str(), "[221,199,1]");
        let v1_reverse: VarInt = serde_json::from_str(j1.as_str()).unwrap();
        assert_eq!(v1_reverse.0, 25565);

        let v1 = VarInt(2097151);
        let j1 = serde_json::to_string(&v1).unwrap();
        assert_eq!(j1.as_str(), "[255,255,127]");
        let v1_reverse: VarInt = serde_json::from_str(j1.as_str()).unwrap();
        assert_eq!(v1_reverse.0, 2097151);

        let v1 = VarInt(-2147483648);
        let j1 = serde_json::to_string(&v1).unwrap();
        assert_eq!(j1.as_str(), "[128,128,128,128,8]");
        let v1_reverse: VarInt = serde_json::from_str(j1.as_str()).unwrap();
        assert_eq!(v1_reverse.0, -2147483648);
    }

    #[test]
    fn deserialize_serialize_varlong() {
        let v1 = VarLong(256);
        let j1 = serde_json::to_string(&v1).unwrap();
        assert_eq!(j1.as_str(), "[128,2]");
        let v1_reverse: VarLong = serde_json::from_str(j1.as_str()).unwrap();
        assert_eq!(v1_reverse.0, 256);

        let v1 = VarLong(25565);
        let j1 = serde_json::to_string(&v1).unwrap();
        assert_eq!(j1.as_str(), "[221,199,1]");
        let v1_reverse: VarLong = serde_json::from_str(j1.as_str()).unwrap();
        assert_eq!(v1_reverse.0, 25565);

        let v1 = VarLong(-1);
        let j1 = serde_json::to_string(&v1).unwrap();
        assert_eq!(j1.as_str(), "[255,255,255,255,255,255,255,255,255,1]");
        let v1_reverse: VarLong = serde_json::from_str(j1.as_str()).unwrap();
        assert_eq!(v1_reverse.0, -1);

        let v1 = VarLong(-2147483648);
        let j1 = serde_json::to_string(&v1).unwrap();
        assert_eq!(j1.as_str(), "[128,128,128,128,248,255,255,255,255,1]");
        let v1_reverse: VarLong = serde_json::from_str(j1.as_str()).unwrap();
        assert_eq!(v1_reverse.0, -2147483648);
    }

    #[test]
    #[ignore]
    fn test_varint_from_bytes() {
        let vec = vec![246, 5];
        let var = VarInt::new_from_bytes(vec).unwrap();

        println!("{}", var.0);

        let mut u: u16 = 0;
        u &= 99;
        u <<= 8;
        u &= 221;

        println!("{u}");

        let vec: Vec<u8> = vec![9, 108, 111, 99, 97, 108, 104, 111, 115, 116];
        println!("{}", String::from_utf8(vec).unwrap());
        
        
    }
}