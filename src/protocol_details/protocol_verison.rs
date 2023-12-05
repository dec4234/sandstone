use crate::enumize;

// https://wiki.vg/Protocol_History
enumize!(ProtocolVerison, i16 => {
    v1_8, 57,
    v1_9, 107,
    v1_10, 210,
    v1_11, 315,
    v1_12_2, 340,
    v1_13_2, 404,
    v1_14_4, 498,
    v1_15_2, 578,
    v1_16_3, 753,
    v1_17_1, 756,
    v1_18_2, 758,
    v1_19_2, 762,
    v1_20_2, 764,
    v1_21, -1
});