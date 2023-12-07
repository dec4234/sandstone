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
            let mut local: i32 = i32::from(*b);

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

    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut vec: Vec<u8> = vec![];

        for b in self.0.to_le_bytes() {

        }

        return vec.into_boxed_slice();
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol_details::datatypes::var_types::VarInt;

    #[test]
    fn basic() {
        assert!(VarInt::from_slice(&[221, 199, 1]).unwrap() == VarInt(25565));
        assert!(VarInt::from_slice(&[255, 255, 127]).unwrap() == VarInt(2097151));
    }
}