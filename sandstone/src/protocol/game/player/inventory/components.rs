use crate::protocol::game::effects::sound::SoundEvent;
use crate::protocol::game::entity::ResolvableProfile;
use crate::protocol::game::player::inventory::slotdata::SlotData;
use crate::protocol::packets::packet_parts::block::BlockPredicate;
use crate::protocol::packets::packet_parts::effects::{ConsumeEffect, FireworkExplosion, PotionEffect};
use crate::protocol::packets::packet_parts::item::{IdOrBannerPattern, IdOrInstrument, IdOrTrimMaterial, IdOrTrimPattern, PaintingVariant};
use crate::protocol::packets::packet_parts::item_modifiers::DyeColor;
use crate::protocol::packets::packet_parts::sound::IdOrJukeboxSong;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::game_types::Position;
use crate::protocol_types::datatypes::internal_types::{IDSet, IDorX, Identifier};
use crate::protocol_types::datatypes::nbt::{NbtCompound, NbtTag};
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct EnchantmentEntry {
	pub type_id: VarInt,
	pub level: VarInt,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct EnchantmentList {
	pub entries: PrefixedArray<EnchantmentEntry>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct AttributeModifierEntry {
	pub attribute_id: VarInt,
	pub modifier_id: Identifier,
	pub value: f64,
	pub operation: VarInt,
	pub slot: VarInt,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct AttributeModifierList {
	pub entries: PrefixedArray<AttributeModifierEntry>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct CustomModelDataComponent {
	pub floats: PrefixedArray<f32>,
	pub flags: PrefixedArray<bool>,
	pub strings: PrefixedArray<String>,
	pub colors: PrefixedArray<i32>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct TooltipDisplayComponent {
	pub hide_tooltip: bool,
	pub hidden_components: PrefixedArray<VarInt>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct FoodComponent {
	pub nutrition: VarInt,
	pub saturation: f32,
	pub can_always_eat: bool,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ConsumableComponent {
	pub consume_seconds: f32,
	pub animation: VarInt,
	pub sound: IDorX<SoundEvent>,
	pub has_particles: bool,
	pub effects: PrefixedArray<ConsumeEffect>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct UseCooldownComponent {
	pub seconds: f32,
	pub cooldown_group: PrefixedOptional<Identifier>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ToolRule {
	pub blocks: IDSet,
	pub speed: PrefixedOptional<f32>,
	pub correct_drop: PrefixedOptional<bool>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ToolComponent {
	pub rules: PrefixedArray<ToolRule>,
	pub default_speed: f32,
	pub damage_per_block: VarInt,
	pub can_destroy_in_creative: bool,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct WeaponComponent {
	pub damage: VarInt,
	pub disable_blocking_for: f32,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct EquippableComponent {
	pub slot: VarInt,
	pub equip_sound: IDorX<SoundEvent>,
	pub model: PrefixedOptional<Identifier>,
	pub camera_overlay: PrefixedOptional<Identifier>,
	pub allowed_entities: PrefixedOptional<IDSet>,
	pub dispensable: bool,
	pub swappable: bool,
	pub damage_on_hurt: bool,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct DamageReduction {
	pub horizontal_angle: f32,
	pub typ: PrefixedOptional<IDSet>,
	pub base: f32,
	pub factor: f32,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BlocksAttacksComponent {
	pub block_delay: f32,
	pub disable_cooldown_scale: f32,
	pub damage_reductions: PrefixedArray<DamageReduction>,
	pub item_damage_threshold: f32,
	pub item_damage_base: f32,
	pub item_damage_factor: f32,
	pub bypassed_by: PrefixedOptional<String>,
	pub block_sound: PrefixedOptional<IDorX<SoundEvent>>,
	pub disable_sound: PrefixedOptional<IDorX<SoundEvent>>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct PotionContentsComponent {
	pub potion_id: PrefixedOptional<VarInt>,
	pub custom_color: PrefixedOptional<i32>,
	pub custom_effects: PrefixedArray<PotionEffect>,
	pub custom_name: String,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct SuspiciousStewEntry {
	pub type_id: VarInt,
	pub duration: VarInt,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct SuspiciousStewList {
	pub entries: PrefixedArray<SuspiciousStewEntry>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct WritableBookPage {
	pub raw: String,
	pub filtered: PrefixedOptional<String>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct WritableBookComponent {
	pub pages: PrefixedArray<WritableBookPage>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct WrittenBookPage {
	pub raw: TextComponent,
	pub filtered: PrefixedOptional<TextComponent>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct WrittenBookComponent {
	pub raw_title: String,
	pub filtered_title: PrefixedOptional<String>,
	pub author: String,
	pub generation: VarInt,
	pub pages: PrefixedArray<WrittenBookPage>,
	pub resolved: bool,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct TrimComponent {
	pub material: IdOrTrimMaterial,
	pub pattern: IdOrTrimPattern,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct EntityDataComponent {
	pub entity_type: VarInt,
	pub data: NbtCompound,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BlockEntityDataComponent {
	pub block_entity_type: VarInt,
	pub data: NbtCompound,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProvidesTrimMaterialComponent {
	ByName(String),
	Inline(Box<IdOrTrimMaterial>),
}

impl McSerialize for ProvidesTrimMaterialComponent {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			ProvidesTrimMaterialComponent::ByName(name) => {
				0u8.mc_serialize(serializer)?;
				name.mc_serialize(serializer)?;
			}
			ProvidesTrimMaterialComponent::Inline(material) => {
				1u8.mc_serialize(serializer)?;
				material.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for ProvidesTrimMaterialComponent {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let mode = u8::mc_deserialize(deserializer)?;
		match mode {
			0 => Ok(ProvidesTrimMaterialComponent::ByName(String::mc_deserialize(deserializer)?)),
			1 => Ok(ProvidesTrimMaterialComponent::Inline(Box::new(IdOrTrimMaterial::mc_deserialize(deserializer)?))),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid ProvidesTrimMaterial mode: {}", mode))),
		}
	}
}

impl McDefault for ProvidesTrimMaterialComponent {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		ProvidesTrimMaterialComponent::ByName(String::mc_default())
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum JukeboxPlayableComponent {
	ByName(String),
	Inline(Box<IdOrJukeboxSong>),
}

impl McSerialize for JukeboxPlayableComponent {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			JukeboxPlayableComponent::ByName(name) => {
				0u8.mc_serialize(serializer)?;
				name.mc_serialize(serializer)?;
			}
			JukeboxPlayableComponent::Inline(song) => {
				1u8.mc_serialize(serializer)?;
				song.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for JukeboxPlayableComponent {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let mode = u8::mc_deserialize(deserializer)?;
		match mode {
			0 => Ok(JukeboxPlayableComponent::ByName(String::mc_deserialize(deserializer)?)),
			1 => Ok(JukeboxPlayableComponent::Inline(Box::new(IdOrJukeboxSong::mc_deserialize(deserializer)?))),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid JukeboxPlayable mode: {}", mode))),
		}
	}
}

impl McDefault for JukeboxPlayableComponent {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		JukeboxPlayableComponent::ByName(String::mc_default())
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct LodestoneTrackerComponent {
	pub has_global_pos: bool,
	pub dimension: Option<Identifier>,
	pub position: Option<Position>,
	pub tracked: bool,
}

impl McSerialize for LodestoneTrackerComponent {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.has_global_pos.mc_serialize(serializer)?;
		if self.has_global_pos {
			self.dimension.mc_serialize(serializer)?;
			self.position.mc_serialize(serializer)?;
		}
		self.tracked.mc_serialize(serializer)?;
		Ok(())
	}
}

impl McDeserialize for LodestoneTrackerComponent {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let has_global_pos = bool::mc_deserialize(deserializer)?;
		let (dimension, position) = if has_global_pos {
			(Some(Identifier::mc_deserialize(deserializer)?), Some(Position::mc_deserialize(deserializer)?))
		} else {
			(None, None)
		};
		let tracked = bool::mc_deserialize(deserializer)?;
		Ok(Self {
			has_global_pos,
			dimension,
			position,
			tracked,
		})
	}
}

impl McDefault for LodestoneTrackerComponent {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		Self {
			has_global_pos: false,
			dimension: None,
			position: None,
			tracked: true,
		}
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct FireworksComponent {
	pub flight_duration: VarInt,
	pub explosions: PrefixedArray<FireworkExplosion>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BannerLayer {
	pub pattern: IdOrBannerPattern,
	pub color: DyeColor,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BannerPatternsComponent {
	pub layers: PrefixedArray<BannerLayer>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BlockStateProperty {
	pub name: String,
	pub value: String,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BlockStateComponent {
	pub properties: PrefixedArray<BlockStateProperty>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BeeData {
	pub entity_type: VarInt,
	pub data: NbtCompound,
	pub ticks_in_hive: VarInt,
	pub min_ticks_in_hive: VarInt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChickenVariantComponent {
	ByName(String),
	Registry(VarInt),
}

impl McSerialize for ChickenVariantComponent {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			ChickenVariantComponent::ByName(name) => {
				0u8.mc_serialize(serializer)?;
				name.mc_serialize(serializer)?;
			}
			ChickenVariantComponent::Registry(id) => {
				1u8.mc_serialize(serializer)?;
				id.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for ChickenVariantComponent {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let mode = u8::mc_deserialize(deserializer)?;
		match mode {
			0 => Ok(ChickenVariantComponent::ByName(String::mc_deserialize(deserializer)?)),
			1 => Ok(ChickenVariantComponent::Registry(VarInt::mc_deserialize(deserializer)?)),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid ChickenVariant mode: {}", mode))),
		}
	}
}

impl McDefault for ChickenVariantComponent {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		ChickenVariantComponent::Registry(VarInt(0))
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaintingVariantComponent {
	Registry(VarInt),
	Inline(Box<PaintingVariant>),
}

impl McSerialize for PaintingVariantComponent {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			PaintingVariantComponent::Registry(id) => {
				VarInt(id.0 + 1).mc_serialize(serializer)?;
			}
			PaintingVariantComponent::Inline(variant) => {
				VarInt(0).mc_serialize(serializer)?;
				variant.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for PaintingVariantComponent {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		if typ == 0 {
			Ok(PaintingVariantComponent::Inline(Box::new(PaintingVariant::mc_deserialize(deserializer)?)))
		} else {
			Ok(PaintingVariantComponent::Registry(VarInt(typ - 1)))
		}
	}
}

impl McDefault for PaintingVariantComponent {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		PaintingVariantComponent::Registry(VarInt(0))
	}
}

/// # Structured Component (Packet Part)
/// Serializes the enum ID as a VarInt first then the body of the StructuredComponent entry
///
/// The IDs are the protocol IDs of the 26.3 `minecraft:data_component_type` registry. Components
/// that are not in this enum (for example `use_effects` or `attack_range`) cannot be read yet.
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Slot_data#Structured_components>
#[derive(VarIntEnum, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum StructuredComponent {
	CustomData(NbtCompound) = 0,
	MaxStackSize(VarInt) = 1,
	MaxDamage(VarInt) = 2,
	Damage(VarInt) = 3,
	Unbreakable = 4,
	CustomName(TextComponent) = 6,
	ItemName(TextComponent) = 9,
	ItemModel(Identifier) = 10,
	Lore(PrefixedArray<TextComponent>) = 11,
	Rarity(VarInt) = 12,
	Enchantments(EnchantmentList) = 13,
	CanPlaceOn(PrefixedArray<BlockPredicate>) = 14,
	CanBreak(PrefixedArray<BlockPredicate>) = 15,
	AttributeModifiers(AttributeModifierList) = 16,
	CustomModelData(CustomModelDataComponent) = 17,
	TooltipDisplay(TooltipDisplayComponent) = 18,
	RepairCost(VarInt) = 19,
	CreativeSlotLock = 20,
	EnchantmentGlintOverride(bool) = 21,
	IntangibleProjectile(NbtCompound) = 22,
	Food(FoodComponent) = 23,
	Consumable(ConsumableComponent) = 24,
	UseRemainder(Box<SlotData>) = 25,
	UseCooldown(UseCooldownComponent) = 26,
	DamageResistant(String) = 27,
	Tool(ToolComponent) = 28,
	Weapon(WeaponComponent) = 29,
	Enchantable(VarInt) = 31,
	Equippable(EquippableComponent) = 32,
	Repairable(IDSet) = 33,
	Glider = 34,
	TooltipStyle(Identifier) = 35,
	DeathProtection(PrefixedArray<ConsumeEffect>) = 36,
	BlocksAttacks(BlocksAttacksComponent) = 37,
	StoredEnchantments(EnchantmentList) = 45,
	DyedColor(i32) = 47,
	MapId(VarInt) = 48,
	MapDecorations(NbtCompound) = 49,
	MapPostProcessing(VarInt) = 50,
	ChargedProjectiles(PrefixedArray<SlotData>) = 51,
	BundleContents(PrefixedArray<SlotData>) = 52,
	PotionContents(PotionContentsComponent) = 53,
	PotionDurationScale(f32) = 54,
	SuspiciousStewEffects(SuspiciousStewList) = 55,
	WritableBookContent(WritableBookComponent) = 56,
	WrittenBookContent(WrittenBookComponent) = 57,
	Trim(TrimComponent) = 58,
	DebugStickState(NbtCompound) = 59,
	EntityData(EntityDataComponent) = 60,
	BucketEntityData(NbtCompound) = 61,
	BlockEntityData(BlockEntityDataComponent) = 62,
	InstrumentComponent(IdOrInstrument) = 63,
	ProvidesTrimMaterial(ProvidesTrimMaterialComponent) = 64,
	OminousBottleAmplifier(VarInt) = 65,
	JukeboxPlayable(JukeboxPlayableComponent) = 66,
	ProvidesBannerPatterns(String) = 67,
	Recipes(NbtCompound) = 68,
	LodestoneTracker(LodestoneTrackerComponent) = 69,
	FireworkExplosionComponent(FireworkExplosion) = 70,
	Fireworks(FireworksComponent) = 71,
	Profile(ResolvableProfile) = 72,
	NoteBlockSound(Identifier) = 73,
	BannerPatterns(BannerPatternsComponent) = 74,
	BaseColor(DyeColor) = 75,
	PotDecorations(PrefixedArray<VarInt>) = 76,
	Container(PrefixedArray<SlotData>) = 77,
	BlockState(BlockStateComponent) = 78,
	Bees(PrefixedArray<BeeData>) = 79,
	Lock(NbtTag) = 81,
	ContainerLoot(NbtCompound) = 82,
	BreakSound(IDorX<SoundEvent>) = 83,
	VillagerVariant(VarInt) = 88,
	WolfVariant(VarInt) = 89,
	WolfSoundVariant(VarInt) = 90,
	WolfCollar(DyeColor) = 91,
	FoxVariant(VarInt) = 92,
	SalmonSize(VarInt) = 93,
	ParrotVariant(VarInt) = 94,
	TropicalFishPattern(VarInt) = 95,
	TropicalFishBaseColor(DyeColor) = 96,
	TropicalFishPatternColor(DyeColor) = 97,
	MooshroomVariant(VarInt) = 98,
	RabbitVariant(VarInt) = 99,
	PigVariant(VarInt) = 100,
	CowVariant(VarInt) = 102,
	ChickenVariant(ChickenVariantComponent) = 104,
	FrogVariant(VarInt) = 107,
	HorseVariant(VarInt) = 108,
	PaintingVariant(PaintingVariantComponent) = 109,
	LlamaVariant(VarInt) = 110,
	AxolotlVariant(VarInt) = 111,
	CatVariant(VarInt) = 112,
	CatCollar(DyeColor) = 114,
	SheepColor(DyeColor) = 115,
	ShulkerColor(DyeColor) = 116,
}

impl McDefault for StructuredComponent {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		StructuredComponent::MaxStackSize(VarInt(64))
	}
}

#[cfg(test)]
mod test {
    use super::*;

    fn ser<T: McSerialize>(v: &T) -> Vec<u8> {
		let mut s = McSerializer::new();
		v.mc_serialize(&mut s).unwrap();
		s.as_bytes().to_vec()
	}

	fn round_trip<T: McSerialize + McDeserialize + PartialEq + std::fmt::Debug>(v: T) {
		let bytes = ser(&v);
		let mut d = McDeserializer::new(&bytes);
		assert_eq!(T::mc_deserialize(&mut d).unwrap(), v);
		assert!(d.is_at_end(), "all bytes must be used");
	}

	#[test]
	fn component_ids_are_pinned() {
		// The client finds the component type from this ID. If a variant moves, every later
		// component in the slot is read with the wrong layout. These IDs are from the 26.3
		// `minecraft:data_component_type` registry (registries.json of the data generator).
		assert_eq!(ser(&StructuredComponent::MaxStackSize(VarInt(64))), vec![1, 64]);
		assert_eq!(ser(&StructuredComponent::Unbreakable), vec![4]);
		assert_eq!(ser(&StructuredComponent::CustomName(TextComponent::new("")))[0], 6);
		assert_eq!(ser(&StructuredComponent::ItemName(TextComponent::new("")))[0], 9);
		assert_eq!(ser(&StructuredComponent::Lore(PrefixedArray::new(vec![]))), vec![11, 0]);
		assert_eq!(ser(&StructuredComponent::CustomModelData(CustomModelDataComponent::mc_default()))[0], 17);
		assert_eq!(ser(&StructuredComponent::CreativeSlotLock), vec![20]);
		assert_eq!(ser(&StructuredComponent::EnchantmentGlintOverride(true)), vec![21, 1]);
		assert_eq!(ser(&StructuredComponent::Glider), vec![34]);
		assert_eq!(ser(&StructuredComponent::DyedColor(1)), vec![47, 0, 0, 0, 1]);
		assert_eq!(ser(&StructuredComponent::VillagerVariant(VarInt(2))), vec![88, 2]);
		assert_eq!(ser(&StructuredComponent::ShulkerColor(DyeColor::mc_default()))[0], 116);
	}

	#[test]
	fn components_with_bodies_round_trip() {
		round_trip(StructuredComponent::Damage(VarInt(300)));
		round_trip(StructuredComponent::Enchantments(EnchantmentList { entries: PrefixedArray::new(vec![EnchantmentEntry { type_id: VarInt(3), level: VarInt(5) }]) }));
		round_trip(StructuredComponent::Lore(PrefixedArray::new(vec![TextComponent::new("line")])));
		round_trip(StructuredComponent::UseRemainder(Box::new(SlotData::mc_default())));
		round_trip(StructuredComponent::BundleContents(PrefixedArray::new(vec![SlotData::mc_default(), SlotData::mc_default()])));
	}

	#[test]
	fn lodestone_without_target_skips_position() {
		// Without a global position, only the two bools go on the wire.
		let c = LodestoneTrackerComponent { has_global_pos: false, dimension: None, position: None, tracked: true };
		assert_eq!(ser(&c), vec![0, 1]);
		round_trip(c);
	}

	#[test]
	fn lodestone_with_target_round_trips() {
		round_trip(LodestoneTrackerComponent {
			has_global_pos: true,
			dimension: Some("minecraft:overworld".parse().unwrap()),
			position: Some(Position::new(10, 64, -10)),
			tracked: false,
		});
	}

	#[test]
	fn provides_trim_material_modes() {
		// Mode 0 is a registry name and mode 1 is an ID or inline material.
		assert_eq!(ser(&ProvidesTrimMaterialComponent::ByName("x".into())), vec![0, 1, b'x']);
		round_trip(ProvidesTrimMaterialComponent::ByName("minecraft:gold".into()));
		round_trip(ProvidesTrimMaterialComponent::Inline(Box::new(IdOrTrimMaterial::Registry(VarInt(2)))));
	}

	#[test]
	fn unknown_mode_byte_is_an_error() {
		// A bad mode must fail. If it does not fail, we read garbage for the rest of the packet.
		assert!(ProvidesTrimMaterialComponent::mc_deserialize(&mut McDeserializer::new(&[2])).is_err());
		assert!(JukeboxPlayableComponent::mc_deserialize(&mut McDeserializer::new(&[2])).is_err());
	}

	#[test]
	fn jukebox_by_name_round_trips() {
		round_trip(JukeboxPlayableComponent::ByName("minecraft:cat".into()));
	}
}
