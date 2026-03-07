use crate::protocol::game::effects::sound::SoundEvent;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::internal_types::{IDSet, IDorX};
use crate::protocol_types::datatypes::nbt::nbt::NbtCompound;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum DyeColor {
	White,
	Orange,
	Magenta,
	LightBlue,
	Yellow,
	Lime,
	Pink,
	Gray,
	LightGray,
	Cyan,
	Purple,
	Blue,
	Brown,
	Green,
	Red,
	Black,
}

impl McSerialize for DyeColor {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let id = match self {
			DyeColor::White => 0,
			DyeColor::Orange => 1,
			DyeColor::Magenta => 2,
			DyeColor::LightBlue => 3,
			DyeColor::Yellow => 4,
			DyeColor::Lime => 5,
			DyeColor::Pink => 6,
			DyeColor::Gray => 7,
			DyeColor::LightGray => 8,
			DyeColor::Cyan => 9,
			DyeColor::Purple => 10,
			DyeColor::Blue => 11,
			DyeColor::Brown => 12,
			DyeColor::Green => 13,
			DyeColor::Red => 14,
			DyeColor::Black => 15,
		};
		VarInt(id).mc_serialize(serializer)
	}
}

impl McDeserialize for DyeColor {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let id = VarInt::mc_deserialize(deserializer)?.0;
		match id {
			0 => Ok(DyeColor::White),
			1 => Ok(DyeColor::Orange),
			2 => Ok(DyeColor::Magenta),
			3 => Ok(DyeColor::LightBlue),
			4 => Ok(DyeColor::Yellow),
			5 => Ok(DyeColor::Lime),
			6 => Ok(DyeColor::Pink),
			7 => Ok(DyeColor::Gray),
			8 => Ok(DyeColor::LightGray),
			9 => Ok(DyeColor::Cyan),
			10 => Ok(DyeColor::Purple),
			11 => Ok(DyeColor::Blue),
			12 => Ok(DyeColor::Brown),
			13 => Ok(DyeColor::Green),
			14 => Ok(DyeColor::Red),
			15 => Ok(DyeColor::Black),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid DyeColor id: {}", id)))
		}
	}
}

impl McDefault for DyeColor {
	fn mc_default() -> Self {
		DyeColor::White
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct InlineSoundEvent {
	pub name: String,
	pub has_fixed_range: bool,
	#[mc(deserialize_if = has_fixed_range)]
	pub fixed_range: Option<f32>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ProfileProperty {
	pub name: String,
	pub value: String,
	pub signature: PrefixedOptional<String>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ResolvableProfile {
	pub name: PrefixedOptional<String>,
	pub uuid: PrefixedOptional<Uuid>,
	pub properties: PrefixedArray<ProfileProperty>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockPredicate {
	pub blocks: PrefixedOptional<IDSet>,
	pub properties: PrefixedOptional<PrefixedArray<BlockProperty>>,
	pub nbt: PrefixedOptional<NbtCompound>,
}

impl McSerialize for BlockPredicate {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.blocks.mc_serialize(serializer)?;
		self.properties.mc_serialize(serializer)?;
		self.nbt.mc_serialize(serializer)?;
		Ok(())
	}
}

impl McDeserialize for BlockPredicate {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		Ok(Self {
			blocks: PrefixedOptional::mc_deserialize(deserializer)?,
			properties: PrefixedOptional::mc_deserialize(deserializer)?,
			nbt: PrefixedOptional::mc_deserialize(deserializer)?,
		})
	}
}

impl McDefault for BlockPredicate {
	fn mc_default() -> Self {
		Self {
			blocks: PrefixedOptional::new(None),
			properties: PrefixedOptional::new(None),
			nbt: PrefixedOptional::new(None),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockProperty {
	pub name: String,
	pub is_exact: bool,
	pub exact_value: Option<String>,
	pub min_value: Option<String>,
	pub max_value: Option<String>,
}

impl McSerialize for BlockProperty {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.name.mc_serialize(serializer)?;
		self.is_exact.mc_serialize(serializer)?;
		if self.is_exact {
			self.exact_value.mc_serialize(serializer)?;
		} else {
			self.min_value.mc_serialize(serializer)?;
			self.max_value.mc_serialize(serializer)?;
		}
		Ok(())
	}
}

impl McDeserialize for BlockProperty {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let name = String::mc_deserialize(deserializer)?;
		let is_exact = bool::mc_deserialize(deserializer)?;
		if is_exact {
			Ok(Self {
				name,
				is_exact,
				exact_value: Some(String::mc_deserialize(deserializer)?),
				min_value: None,
				max_value: None,
			})
		} else {
			Ok(Self {
				name,
				is_exact,
				exact_value: None,
				min_value: Some(String::mc_deserialize(deserializer)?),
				max_value: Some(String::mc_deserialize(deserializer)?),
			})
		}
	}
}

impl McDefault for BlockProperty {
	fn mc_default() -> Self {
		Self {
			name: String::mc_default(),
			is_exact: true,
			exact_value: Some(String::mc_default()),
			min_value: None,
			max_value: None,
		}
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct FireworkExplosion {
	pub shape: VarInt,
	pub colors: PrefixedArray<i32>,
	pub fade_colors: PrefixedArray<i32>,
	pub has_trail: bool,
	pub has_twinkle: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PotionEffectDetail {
	pub amplifier: VarInt,
	pub duration: VarInt,
	pub ambient: bool,
	pub show_particles: bool,
	pub show_icon: bool,
	pub hidden_effect: PrefixedOptional<Box<PotionEffectDetail>>,
}

impl McSerialize for PotionEffectDetail {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.amplifier.mc_serialize(serializer)?;
		self.duration.mc_serialize(serializer)?;
		self.ambient.mc_serialize(serializer)?;
		self.show_particles.mc_serialize(serializer)?;
		self.show_icon.mc_serialize(serializer)?;
		self.hidden_effect.mc_serialize(serializer)?;
		Ok(())
	}
}

impl McDeserialize for PotionEffectDetail {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		Ok(Self {
			amplifier: VarInt::mc_deserialize(deserializer)?,
			duration: VarInt::mc_deserialize(deserializer)?,
			ambient: bool::mc_deserialize(deserializer)?,
			show_particles: bool::mc_deserialize(deserializer)?,
			show_icon: bool::mc_deserialize(deserializer)?,
			hidden_effect: PrefixedOptional::mc_deserialize(deserializer)?,
		})
	}
}

impl McDefault for PotionEffectDetail {
	fn mc_default() -> Self {
		Self {
			amplifier: VarInt(1),
			duration: VarInt(100),
			ambient: false,
			show_particles: true,
			show_icon: true,
			hidden_effect: PrefixedOptional::new(None),
		}
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct PotionEffect {
	pub type_id: VarInt,
	pub detail: PotionEffectDetail,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsumeEffect {
	ApplyEffects(PrefixedArray<PotionEffect>, f32),
	RemoveEffects(IDSet),
	ClearAllEffects,
	TeleportRandomly(f32),
	PlaySound(IDorX<SoundEvent>),
}

impl McSerialize for ConsumeEffect {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			ConsumeEffect::ApplyEffects(effects, probability) => {
				VarInt(0).mc_serialize(serializer)?;
				effects.mc_serialize(serializer)?;
				probability.mc_serialize(serializer)?;
			}
			ConsumeEffect::RemoveEffects(id_set) => {
				VarInt(1).mc_serialize(serializer)?;
				id_set.mc_serialize(serializer)?;
			}
			ConsumeEffect::ClearAllEffects => {
				VarInt(2).mc_serialize(serializer)?;
			}
			ConsumeEffect::TeleportRandomly(diameter) => {
				VarInt(3).mc_serialize(serializer)?;
				diameter.mc_serialize(serializer)?;
			}
			ConsumeEffect::PlaySound(sound) => {
				VarInt(4).mc_serialize(serializer)?;
				sound.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for ConsumeEffect {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		match typ {
			0 => {
				let effects = PrefixedArray::mc_deserialize(deserializer)?;
				let probability = f32::mc_deserialize(deserializer)?;
				Ok(ConsumeEffect::ApplyEffects(effects, probability))
			}
			1 => Ok(ConsumeEffect::RemoveEffects(IDSet::mc_deserialize(deserializer)?)),
			2 => Ok(ConsumeEffect::ClearAllEffects),
			3 => Ok(ConsumeEffect::TeleportRandomly(f32::mc_deserialize(deserializer)?)),
			4 => Ok(ConsumeEffect::PlaySound(IDorX::mc_deserialize(deserializer)?)),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid ConsumeEffect type: {}", typ)))
		}
	}
}

impl McDefault for ConsumeEffect {
	fn mc_default() -> Self {
		ConsumeEffect::ClearAllEffects
	}
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
	Inline(TrimMaterial),
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
			Ok(IdOrTrimMaterial::Inline(TrimMaterial::mc_deserialize(deserializer)?))
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
	Inline(TrimPattern),
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
			Ok(IdOrTrimPattern::Inline(TrimPattern::mc_deserialize(deserializer)?))
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
	Inline(Instrument),
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
			Ok(IdOrInstrument::Inline(Instrument::mc_deserialize(deserializer)?))
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
pub struct JukeboxSong {
	pub sound_event: IDorX<SoundEvent>,
	pub description: TextComponent,
	pub duration: f32,
	pub output: VarInt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdOrJukeboxSong {
	Registry(VarInt),
	Inline(JukeboxSong),
}

impl McSerialize for IdOrJukeboxSong {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			IdOrJukeboxSong::Registry(id) => {
				VarInt(id.0 + 1).mc_serialize(serializer)?;
			}
			IdOrJukeboxSong::Inline(song) => {
				VarInt(0).mc_serialize(serializer)?;
				song.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for IdOrJukeboxSong {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		if typ == 0 {
			Ok(IdOrJukeboxSong::Inline(JukeboxSong::mc_deserialize(deserializer)?))
		} else {
			Ok(IdOrJukeboxSong::Registry(VarInt(typ - 1)))
		}
	}
}

impl McDefault for IdOrJukeboxSong {
	fn mc_default() -> Self {
		IdOrJukeboxSong::Registry(VarInt(0))
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
	Inline(PaintingVariant),
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
			Ok(IdOrPaintingVariant::Inline(PaintingVariant::mc_deserialize(deserializer)?))
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
