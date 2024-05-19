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

impl ProtocolVerison {
	pub fn get_string(&self) -> String {
		match self {
			ProtocolVerison::v1_8 => "1.8",
			ProtocolVerison::v1_9 => "1.9",
			ProtocolVerison::v1_10 => "1.10",
			ProtocolVerison::v1_11 => "1.11",
			ProtocolVerison::v1_12_2 => "1.12.2",
			ProtocolVerison::v1_13_2 => "1.13.2",
			ProtocolVerison::v1_14_4 => "1.14.4",
			ProtocolVerison::v1_15_2 => "1.15.2",
			ProtocolVerison::v1_16_3 => "1.16.3",
			ProtocolVerison::v1_17_1 => "1.17.1",
			ProtocolVerison::v1_18_2 => "1.18.2",
			ProtocolVerison::v1_19_2 => "1.19.2",
			ProtocolVerison::v1_20_2 => "1.20.2",
			ProtocolVerison::v1_21 => "1.21",
		}.to_string()
	}
}