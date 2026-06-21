use crate::protocol::game::info::inventory::components::StructuredComponent;
use crate::protocol::game::info::inventory::slotdata::SlotData;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};

/// The icon type displayed on a map for a Map Icon entry.
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum MapIconType {
	WhiteArrow = 0,
	GreenArrow = 1,
	RedArrow = 2,
	BlueArrow = 3,
	WhiteCross = 4,
	RedPointer = 5,
	WhiteCircle = 6,
	SmallWhiteCircle = 7,
	Mansion = 8,
	Monument = 9,
	WhiteBanner = 10,
	OrangeBanner = 11,
	MagentaBanner = 12,
	LightBlueBanner = 13,
	YellowBanner = 14,
	LimeBanner = 15,
	PinkBanner = 16,
	GrayBanner = 17,
	LightGrayBanner = 18,
	CyanBanner = 19,
	PurpleBanner = 20,
	BlueBanner = 21,
	BrownBanner = 22,
	GreenBanner = 23,
	RedBanner = 24,
	BlackBanner = 25,
	TreasureMarker = 26,
	DesertVillage = 27,
	PlainsVillage = 28,
	SavannaVillage = 29,
	SnowyVillage = 30,
	TaigaVillage = 31,
	JungleTemple = 32,
	SwampHut = 33,
	TrialChambers = 34,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct MapIcons {
	pub typ: MapIconType,
	pub x: i8,
	pub z: i8,
	pub direction: i8,
	pub display_name: PrefixedOptional<TextComponent>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct MapColorPatch {
	pub columns: u8,
	#[mc(deserialize_if = columns == 0)]
	pub rows: Option<u8>,
	#[mc(deserialize_if = columns == 0)]
	pub color_x: Option<u8>,
	#[mc(deserialize_if = columns == 0)]
	pub color_z: Option<u8>,
	#[mc(deserialize_if = columns == 0)]
	pub data: Option<PrefixedArray<u8>>
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct Trade {
	pub input_item_1: TradeItem,
	pub output_item: SlotData,
	pub input_item_2: PrefixedOptional<TradeItem>,
	pub trade_disabled: bool,
	pub number_of_trade_uses: i32,
	pub xp: i32,
	/// Can be zero or negative.
	pub special_price: i32,
	pub price_multiplier: f32,
	pub demand: i32
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct TradeItem {
	pub item_id: VarInt,
	pub item_count: VarInt,
	pub structured_components: PrefixedArray<StructuredComponent>
}