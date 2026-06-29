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
use crate::protocol_types::datatypes::internal_types::{IDSet, IDorX};
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
	pub modifier_id: String,
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
	pub cooldown_group: PrefixedOptional<String>,
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
	pub model: PrefixedOptional<String>,
	pub camera_overlay: PrefixedOptional<String>,
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
	fn mc_default() -> Self {
		JukeboxPlayableComponent::ByName(String::mc_default())
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct LodestoneTrackerComponent {
	pub has_global_pos: bool,
	pub dimension: Option<String>,
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
			(Some(String::mc_deserialize(deserializer)?), Some(Position::mc_deserialize(deserializer)?))
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
	fn mc_default() -> Self {
		PaintingVariantComponent::Registry(VarInt(0))
	}
}

/// # Structured Component (Packet Part)
/// Serializes the enum ID as a VarInt first then the body of the StructuredComponent entry
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Slot_data#Structured_components
#[derive(VarIntEnum, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum StructuredComponent {
	CustomData(NbtCompound) = 0,
	MaxStackSize(VarInt) = 1,
	MaxDamage(VarInt) = 2,
	Damage(VarInt) = 3,
	Unbreakable = 4,
	CustomName(TextComponent) = 5,
	ItemName(TextComponent) = 6,
	ItemModel(String) = 7,
	Lore(PrefixedArray<TextComponent>) = 8,
	Rarity(VarInt) = 9,
	Enchantments(EnchantmentList) = 10,
	CanPlaceOn(PrefixedArray<BlockPredicate>) = 11,
	CanBreak(PrefixedArray<BlockPredicate>) = 12,
	AttributeModifiers(AttributeModifierList) = 13,
	CustomModelData(CustomModelDataComponent) = 14,
	TooltipDisplay(TooltipDisplayComponent) = 15,
	RepairCost(VarInt) = 16,
	CreativeSlotLock = 17,
	EnchantmentGlintOverride(bool) = 18,
	IntangibleProjectile(NbtCompound) = 19,
	Food(FoodComponent) = 20,
	Consumable(ConsumableComponent) = 21,
	UseRemainder(Box<SlotData>) = 22,
	UseCooldown(UseCooldownComponent) = 23,
	DamageResistant(String) = 24,
	Tool(ToolComponent) = 25,
	Weapon(WeaponComponent) = 26,
	Enchantable(VarInt) = 27,
	Equippable(EquippableComponent) = 28,
	Repairable(IDSet) = 29,
	Glider = 30,
	TooltipStyle(String) = 31,
	DeathProtection(PrefixedArray<ConsumeEffect>) = 32,
	BlocksAttacks(BlocksAttacksComponent) = 33,
	StoredEnchantments(EnchantmentList) = 34,
	DyedColor(i32) = 35,
	MapColor(i32) = 36,
	MapId(VarInt) = 37,
	MapDecorations(NbtCompound) = 38,
	MapPostProcessing(VarInt) = 39,
	ChargedProjectiles(PrefixedArray<SlotData>) = 40,
	BundleContents(PrefixedArray<SlotData>) = 41,
	PotionContents(PotionContentsComponent) = 42,
	PotionDurationScale(f32) = 43,
	SuspiciousStewEffects(SuspiciousStewList) = 44,
	WritableBookContent(WritableBookComponent) = 45,
	WrittenBookContent(WrittenBookComponent) = 46,
	Trim(TrimComponent) = 47,
	DebugStickState(NbtCompound) = 48,
	EntityData(EntityDataComponent) = 49,
	BucketEntityData(NbtCompound) = 50,
	BlockEntityData(BlockEntityDataComponent) = 51,
	InstrumentComponent(IdOrInstrument) = 52,
	ProvidesTrimMaterial(ProvidesTrimMaterialComponent) = 53,
	OminousBottleAmplifier(VarInt) = 54,
	JukeboxPlayable(JukeboxPlayableComponent) = 55,
	ProvidesBannerPatterns(String) = 56,
	Recipes(NbtCompound) = 57,
	LodestoneTracker(LodestoneTrackerComponent) = 58,
	FireworkExplosionComponent(FireworkExplosion) = 59,
	Fireworks(FireworksComponent) = 60,
	Profile(ResolvableProfile) = 61,
	NoteBlockSound(String) = 62,
	BannerPatterns(BannerPatternsComponent) = 63,
	BaseColor(DyeColor) = 64,
	PotDecorations(PrefixedArray<VarInt>) = 65,
	Container(PrefixedArray<SlotData>) = 66,
	BlockState(BlockStateComponent) = 67,
	Bees(PrefixedArray<BeeData>) = 68,
	Lock(NbtTag) = 69,
	ContainerLoot(NbtCompound) = 70,
	BreakSound(IDorX<SoundEvent>) = 71,
	VillagerVariant(VarInt) = 72,
	WolfVariant(VarInt) = 73,
	WolfSoundVariant(VarInt) = 74,
	WolfCollar(DyeColor) = 75,
	FoxVariant(VarInt) = 76,
	SalmonSize(VarInt) = 77,
	ParrotVariant(VarInt) = 78,
	TropicalFishPattern(VarInt) = 79,
	TropicalFishBaseColor(DyeColor) = 80,
	TropicalFishPatternColor(DyeColor) = 81,
	MooshroomVariant(VarInt) = 82,
	RabbitVariant(VarInt) = 83,
	PigVariant(VarInt) = 84,
	CowVariant(VarInt) = 85,
	ChickenVariant(ChickenVariantComponent) = 86,
	FrogVariant(VarInt) = 87,
	HorseVariant(VarInt) = 88,
	PaintingVariant(PaintingVariantComponent) = 89,
	LlamaVariant(VarInt) = 90,
	AxolotlVariant(VarInt) = 91,
	CatVariant(VarInt) = 92,
	CatCollar(DyeColor) = 93,
	SheepColor(DyeColor) = 94,
	ShulkerColor(DyeColor) = 95,
}

impl McDefault for StructuredComponent {
	fn mc_default() -> Self {
		StructuredComponent::MaxStackSize(VarInt(64))
	}
}
