use crate::nbt_type;
use serde::{Deserialize, Serialize};

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
        }


});

impl NbTag {
    pub fn has_fixed_size(&self) -> bool {
        return false;
    }

    pub fn get_tag_size(&self) -> u16 {
        return 0;
    }
}