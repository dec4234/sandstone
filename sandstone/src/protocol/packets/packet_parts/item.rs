use crate::protocol::game::effects::sound::SoundEvent;
use crate::protocol::game::player::inventory::components::StructuredComponent;
use crate::protocol::game::player::inventory::slotdata::SlotData;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::internal_types::IDorX;
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

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct TrimMaterial {
	pub suffix: String,
	pub overrides: PrefixedArray<TrimMaterialOverride>,
	pub description: TextComponent,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct TrimMaterialOverride {
	pub armor_material: String,
	pub asset_name: String,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct TrimPattern {
	pub asset_name: String,
	pub template_item: VarInt,
	pub description: TextComponent,
	pub decal: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdOrTrimMaterial {
	Registry(VarInt),
	Inline(Box<TrimMaterial>),
}

impl McSerialize for IdOrTrimMaterial {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			IdOrTrimMaterial::Registry(id) => {
				VarInt(id.0 + 1).mc_serialize(serializer)?;
			}
			IdOrTrimMaterial::Inline(material) => {
				VarInt(0).mc_serialize(serializer)?;
				material.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for IdOrTrimMaterial {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		if typ == 0 {
			Ok(IdOrTrimMaterial::Inline(Box::new(TrimMaterial::mc_deserialize(deserializer)?)))
		} else {
			Ok(IdOrTrimMaterial::Registry(VarInt(typ - 1)))
		}
	}
}

impl McDefault for IdOrTrimMaterial {
	fn mc_default() -> Self {
		IdOrTrimMaterial::Registry(VarInt(0))
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdOrTrimPattern {
	Registry(VarInt),
	Inline(Box<TrimPattern>),
}

impl McSerialize for IdOrTrimPattern {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			IdOrTrimPattern::Registry(id) => {
				VarInt(id.0 + 1).mc_serialize(serializer)?;
			}
			IdOrTrimPattern::Inline(pattern) => {
				VarInt(0).mc_serialize(serializer)?;
				pattern.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for IdOrTrimPattern {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		if typ == 0 {
			Ok(IdOrTrimPattern::Inline(Box::new(TrimPattern::mc_deserialize(deserializer)?)))
		} else {
			Ok(IdOrTrimPattern::Registry(VarInt(typ - 1)))
		}
	}
}

impl McDefault for IdOrTrimPattern {
	fn mc_default() -> Self {
		IdOrTrimPattern::Registry(VarInt(0))
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct Instrument {
	pub sound_event: IDorX<SoundEvent>,
	pub use_duration: f32,
	pub range: f32,
	pub description: TextComponent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdOrInstrument {
	Registry(VarInt),
	Inline(Box<Instrument>),
}

impl McSerialize for IdOrInstrument {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			IdOrInstrument::Registry(id) => {
				VarInt(id.0 + 1).mc_serialize(serializer)?;
			}
			IdOrInstrument::Inline(instrument) => {
				VarInt(0).mc_serialize(serializer)?;
				instrument.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for IdOrInstrument {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		if typ == 0 {
			Ok(IdOrInstrument::Inline(Box::new(Instrument::mc_deserialize(deserializer)?)))
		} else {
			Ok(IdOrInstrument::Registry(VarInt(typ - 1)))
		}
	}
}

impl McDefault for IdOrInstrument {
	fn mc_default() -> Self {
		IdOrInstrument::Registry(VarInt(0))
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BannerPatternDef {
	pub asset_id: String,
	pub translation_key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdOrBannerPattern {
	Registry(VarInt),
	Inline(BannerPatternDef),
}

impl McSerialize for IdOrBannerPattern {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			IdOrBannerPattern::Registry(id) => {
				VarInt(id.0 + 1).mc_serialize(serializer)?;
			}
			IdOrBannerPattern::Inline(pattern) => {
				VarInt(0).mc_serialize(serializer)?;
				pattern.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for IdOrBannerPattern {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		if typ == 0 {
			Ok(IdOrBannerPattern::Inline(BannerPatternDef::mc_deserialize(deserializer)?))
		} else {
			Ok(IdOrBannerPattern::Registry(VarInt(typ - 1)))
		}
	}
}

impl McDefault for IdOrBannerPattern {
	fn mc_default() -> Self {
		IdOrBannerPattern::Registry(VarInt(0))
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct PaintingVariant {
	pub width: VarInt,
	pub height: VarInt,
	pub asset_id: String,
	pub title: PrefixedOptional<TextComponent>,
	pub author: PrefixedOptional<TextComponent>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdOrPaintingVariant {
	Registry(VarInt),
	Inline(Box<PaintingVariant>),
}

impl McSerialize for IdOrPaintingVariant {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			IdOrPaintingVariant::Registry(id) => {
				VarInt(id.0 + 1).mc_serialize(serializer)?;
			}
			IdOrPaintingVariant::Inline(variant) => {
				VarInt(0).mc_serialize(serializer)?;
				variant.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for IdOrPaintingVariant {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		if typ == 0 {
			Ok(IdOrPaintingVariant::Inline(Box::new(PaintingVariant::mc_deserialize(deserializer)?)))
		} else {
			Ok(IdOrPaintingVariant::Registry(VarInt(typ - 1)))
		}
	}
}

impl McDefault for IdOrPaintingVariant {
	fn mc_default() -> Self {
		IdOrPaintingVariant::Registry(VarInt(0))
	}
}