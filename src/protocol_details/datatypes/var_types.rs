use std::fmt::Error;
use std::ops::{AddAssign, BitAnd, BitOr, BitOrAssign};
use anyhow::{anyhow, Result};

// https://wiki.vg/Protocol#VarInt_and_VarLong
const SEGMENT: i32 = 0x7F;
const CONTINUE: i32 = 0x80;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct VarInt(i32);

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

            i |= (local & SEGMENT) << pos;

            if (local & CONTINUE) == 0 { // Early termination
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
    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut vec: Vec<u8> = vec![];
        let mut inner = self.0;

        loop {
            if (inner & !SEGMENT) == 0 {
                vec.push(inner.to_le_bytes()[0]);
                break;
            }

            vec.push(((inner & SEGMENT) | CONTINUE) as u8);

            // https://stackoverflow.com/questions/70212075/how-to-make-unsigned-right-shift-in-rust
            inner = {
                if inner >= 0 {
                    inner >> 7
                } else {
                    ((inner as u32) >> 7) as i32
                }
            };
        }

        return vec.into_boxed_slice();
    }

    pub fn bytes(i: i32) -> Box<[u8]> {
        let var = VarInt(i);

        return var.to_bytes();
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol_details::datatypes::var_types::VarInt;

    #[test]
    fn basic_varint_from_slice() {
        assert!(VarInt::from_slice(&[221, 199, 1]).unwrap() == VarInt(25565));
        assert!(VarInt::from_slice(&[255, 255, 127]).unwrap() == VarInt(2097151));
        assert!(VarInt::from_slice(&[255, 255, 255, 255, 15]).unwrap() == VarInt(-1));
        assert!(VarInt::from_slice(&[128, 128, 128, 128, 8]).unwrap() == VarInt(-2147483648));
    }

    #[test]
    fn basic_varint_writing() {
        assert!(VarInt::from_slice(&[221, 199, 1]).unwrap().to_bytes() == Box::new([221, 199, 1]));
        assert!(VarInt::from_slice(&[255, 255, 127]).unwrap().to_bytes() == Box::new([255, 255, 127]));
        assert!(VarInt::from_slice(&[255, 255, 255, 255, 15]).unwrap().to_bytes() == Box::new([255, 255, 255, 255, 15]));
    }

    #[test]
    fn basic_varlong() {

    }
}