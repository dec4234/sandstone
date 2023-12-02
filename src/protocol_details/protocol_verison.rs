use crate::enumize;

// https://wiki.vg/Protocol_History
enumize!(ProtocolVerison, i16 => {
    One_Eight, 57,
    One_Nine, 107,
    One_Ten, 210,
    One_Eleven, 315,
    One_Twelve_Two, 340,
    One_Thirteen_Two, 404,
    One_Fourteen_Four, 498,
    One_Fifteen_Two, 578,
    One_Sixteen_Three, 753,
    One_Seventeen_One, 756,
    One_Eighteen_Two, 758,
    One_Nineteen_Four, 762,
    One_Twenty_Two, 764,
    One_TwentyOne, -1
});