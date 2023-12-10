use crate::nbt_type;
use serde::{Deserialize, Serialize};
use crate::protocol_details::nbt::nbt_util::{NbtTypeBody, NbtTagSerialization};

// https://wiki.vg/NBT
nbt_type!(
    NbTag => {
        // Name, Body Name, Id, Payload Byte Size
        END, EndBody, 0 => { // 0

        },

        BYTE, ByteBody, 1 => { // 1
            byte: i8
        },

        SHORT, ShortBody, 2 => { // 2
            short: i16
        },

        INT, IntBody, 3 => { // 4
            int: i32
        },

        LONG, LongBody, 4 => { // 8
            long: i64
        },

        FLOAT, FloatBody, 5 => { // 4
            float: f32
        },

        DOUBLE, DoubleBody, 6 => { // 8
            double: f64
        },

        BYTE_ARRAY, ByteArrayBody, 7 => {
            bytes: Box<[u8]>
        },

        STRING, StringBody, 8 => {
            size: u16, // prefix of the serialized byte array
            string: String
        },

        LIST, ListBody, 9 => {
            typeId: u16,
            length: i32,
            list: Vec<NbTag>
        },

        COMPOUND, CompoundBody, 10 => {
            name: String,
            list: Vec<(String, NbTag)>
        },

        INT_ARRAY, IntArrayBody, 11 => {
            size: i32,
            ints: Vec<i32>
        },

        LONG_ARRAY, LongArrayBody, 12 => {
            size: i32,
            longs: Vec<i64>
        }


});

impl NbTag {
    pub fn has_fixed_size(&self) -> bool {
        return self.get_tag_size() != -1;
    }

    pub fn get_tag_size(&self) -> i32 {
        return match self {
            NbTag::END(..) => 0,
            NbTag::BYTE(..) => 1,
            NbTag::SHORT(..) => 2,
            NbTag::INT(..) => 4,
            NbTag::LONG(..) => 8,
            NbTag::FLOAT(..) => 4,
            NbTag::DOUBLE(..) => 8,
            _ => -1,
        };
    }
}

impl NbtTagSerialization for CompoundBody {
    fn serialize(&self) -> String {
        todo!()
    }

    fn deserialize(input: String) -> Option<Self> {
        Some(Self {
            name: String::from("test"),
            list: Vec::new()
        })
    }
}