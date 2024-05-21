use crate::versions;

// https://wiki.vg/Protocol_History
versions!(ProtocolVerison, i16 => {
    v1_8, 47, "1.8.9",
    v1_9, 110, "1.9.4",
    v1_10, 210, "1.10.2",
    v1_11, 316, "1.11.2",
    v1_12, 340, "1.12.2",
    v1_13, 404, "1.13.2",
    v1_14, 498, "1.14.4",
    v1_15, 578, "1.15.2",
    v1_16, 754, "1.16.5",
    v1_17, 756, "1.17.1",
    v1_18, 758, "1.18.2",
    v1_19, 762, "1.19.4",
    v1_20, 766, "1.20.4",
    v1_21, -1, ""
});