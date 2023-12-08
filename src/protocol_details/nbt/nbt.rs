use crate::nbt_type;
use serde::{Deserialize, Serialize};

nbt_type!(
    NBType => {
        // Name, Body Name, Id, Payload Byte Size
        END, EndBody, 0, 0 => {

        },

        BYTE, ByteBody, 1, 1 => {
            byte: i8
        },

        SHORT, ShortBody, 2, 2 => {
            short: i16
        },

        INT, IntBody, 3, 4 => {
            int: i32
        }
    }
);