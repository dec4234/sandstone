//! Defines the protocol version numbers for the final patch of each Minecraft version. This is important
//! for verifying client protocol versions.

use crate::versions;

// https://wiki.vg/Protocol_History
versions!(ProtocolVerison, i16 => {
    V1_8, 47, "1.8.9",
    V1_9, 110, "1.9.4",
    V1_10, 210, "1.10.2",
    V1_11, 316, "1.11.2",
    V1_12, 340, "1.12.2",
    V1_13, 404, "1.13.2",
    V1_14, 498, "1.14.4",
    V1_15, 578, "1.15.2",
    V1_16, 754, "1.16.5",
    V1_17, 756, "1.17.1",
    V1_18, 758, "1.18.2",
    V1_19, 762, "1.19.4",
    V1_20, 766, "1.20.6",
    V1_21, 770, "1.21.5"
});