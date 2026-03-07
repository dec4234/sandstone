use crate::protocol::game::effects::sound::SoundEvent;
use crate::protocol::game::info::inventory::component_types::*;
use crate::protocol::game::info::inventory::slotdata::SlotData;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::game_types::Position;
use crate::protocol_types::datatypes::internal_types::{IDSet, IDorX};
use crate::protocol_types::datatypes::nbt::nbt::{NbtCompound, NbtTag};
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};

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
	Inline(IdOrTrimMaterial),
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
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let mode = u8::mc_deserialize(deserializer)?;
		match mode {
			0 => Ok(ProvidesTrimMaterialComponent::ByName(String::mc_deserialize(deserializer)?)),
			1 => Ok(ProvidesTrimMaterialComponent::Inline(IdOrTrimMaterial::mc_deserialize(deserializer)?)),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid ProvidesTrimMaterial mode: {}", mode)))
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
	Inline(IdOrJukeboxSong),
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
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let mode = u8::mc_deserialize(deserializer)?;
		match mode {
			0 => Ok(JukeboxPlayableComponent::ByName(String::mc_deserialize(deserializer)?)),
			1 => Ok(JukeboxPlayableComponent::Inline(IdOrJukeboxSong::mc_deserialize(deserializer)?)),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid JukeboxPlayable mode: {}", mode)))
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
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let has_global_pos = bool::mc_deserialize(deserializer)?;
		let (dimension, position) = if has_global_pos {
			(Some(String::mc_deserialize(deserializer)?), Some(Position::mc_deserialize(deserializer)?))
		} else {
			(None, None)
		};
		let tracked = bool::mc_deserialize(deserializer)?;
		Ok(Self { has_global_pos, dimension, position, tracked })
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
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let mode = u8::mc_deserialize(deserializer)?;
		match mode {
			0 => Ok(ChickenVariantComponent::ByName(String::mc_deserialize(deserializer)?)),
			1 => Ok(ChickenVariantComponent::Registry(VarInt::mc_deserialize(deserializer)?)),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid ChickenVariant mode: {}", mode)))
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
	Inline(PaintingVariant),
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
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		if typ == 0 {
			Ok(PaintingVariantComponent::Inline(PaintingVariant::mc_deserialize(deserializer)?))
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

#[derive(Debug, Clone, PartialEq)]
pub enum StructuredComponent {
	CustomData(NbtCompound),                        // 0
	MaxStackSize(VarInt),                           // 1
	MaxDamage(VarInt),                              // 2
	Damage(VarInt),                                 // 3
	Unbreakable,                                    // 4
	CustomName(TextComponent),                      // 5
	ItemName(TextComponent),                        // 6
	ItemModel(String),                              // 7
	Lore(PrefixedArray<TextComponent>),             // 8
	Rarity(VarInt),                                 // 9
	Enchantments(EnchantmentList),                  // 10
	CanPlaceOn(PrefixedArray<BlockPredicate>),      // 11
	CanBreak(PrefixedArray<BlockPredicate>),        // 12
	AttributeModifiers(AttributeModifierList),      // 13
	CustomModelData(CustomModelDataComponent),      // 14
	TooltipDisplay(TooltipDisplayComponent),        // 15
	RepairCost(VarInt),                             // 16
	CreativeSlotLock,                               // 17
	EnchantmentGlintOverride(bool),                 // 18
	IntangibleProjectile(NbtCompound),              // 19
	Food(FoodComponent),                            // 20
	Consumable(ConsumableComponent),                // 21
	UseRemainder(Box<SlotData>),                    // 22
	UseCooldown(UseCooldownComponent),              // 23
	DamageResistant(String),                        // 24
	Tool(ToolComponent),                            // 25
	Weapon(WeaponComponent),                        // 26
	Enchantable(VarInt),                            // 27
	Equippable(EquippableComponent),                // 28
	Repairable(IDSet),                              // 29
	Glider,                                         // 30
	TooltipStyle(String),                           // 31
	DeathProtection(PrefixedArray<ConsumeEffect>),  // 32
	BlocksAttacks(BlocksAttacksComponent),          // 33
	StoredEnchantments(EnchantmentList),            // 34
	DyedColor(i32),                                 // 35
	MapColor(i32),                                  // 36
	MapId(VarInt),                                  // 37
	MapDecorations(NbtCompound),                    // 38
	MapPostProcessing(VarInt),                      // 39
	ChargedProjectiles(PrefixedArray<SlotData>),    // 40
	BundleContents(PrefixedArray<SlotData>),        // 41
	PotionContents(PotionContentsComponent),        // 42
	PotionDurationScale(f32),                       // 43
	SuspiciousStewEffects(SuspiciousStewList),      // 44
	WritableBookContent(WritableBookComponent),     // 45
	WrittenBookContent(WrittenBookComponent),       // 46
	Trim(TrimComponent),                            // 47
	DebugStickState(NbtCompound),                   // 48
	EntityData(EntityDataComponent),                // 49
	BucketEntityData(NbtCompound),                  // 50
	BlockEntityData(BlockEntityDataComponent),      // 51
	InstrumentComponent(IdOrInstrument),            // 52
	ProvidesTrimMaterial(ProvidesTrimMaterialComponent), // 53
	OminousBottleAmplifier(VarInt),                 // 54
	JukeboxPlayable(JukeboxPlayableComponent),      // 55
	ProvidesBannerPatterns(String),                 // 56
	Recipes(NbtCompound),                           // 57
	LodestoneTracker(LodestoneTrackerComponent),    // 58
	FireworkExplosionComponent(FireworkExplosion),   // 59
	Fireworks(FireworksComponent),                  // 60
	Profile(ResolvableProfile),                     // 61
	NoteBlockSound(String),                         // 62
	BannerPatterns(BannerPatternsComponent),        // 63
	BaseColor(DyeColor),                            // 64
	PotDecorations(PrefixedArray<VarInt>),          // 65
	Container(PrefixedArray<SlotData>),             // 66
	BlockState(BlockStateComponent),                // 67
	Bees(PrefixedArray<BeeData>),                   // 68
	Lock(NbtTag),                                   // 69
	ContainerLoot(NbtCompound),                     // 70
	BreakSound(IDorX<SoundEvent>),                  // 71
	VillagerVariant(VarInt),                        // 72
	WolfVariant(VarInt),                            // 73
	WolfSoundVariant(VarInt),                       // 74
	WolfCollar(DyeColor),                           // 75
	FoxVariant(VarInt),                             // 76
	SalmonSize(VarInt),                             // 77
	ParrotVariant(VarInt),                          // 78
	TropicalFishPattern(VarInt),                    // 79
	TropicalFishBaseColor(DyeColor),                // 80
	TropicalFishPatternColor(DyeColor),             // 81
	MooshroomVariant(VarInt),                       // 82
	RabbitVariant(VarInt),                          // 83
	PigVariant(VarInt),                             // 84
	CowVariant(VarInt),                             // 85
	ChickenVariant(ChickenVariantComponent),        // 86
	FrogVariant(VarInt),                            // 87
	HorseVariant(VarInt),                           // 88
	PaintingVariant(PaintingVariantComponent),      // 89
	LlamaVariant(VarInt),                           // 90
	AxolotlVariant(VarInt),                         // 91
	CatVariant(VarInt),                             // 92
	CatCollar(DyeColor),                            // 93
	SheepColor(DyeColor),                           // 94
	ShulkerColor(DyeColor),                         // 95
}

impl McSerialize for StructuredComponent {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			StructuredComponent::CustomData(v) => { VarInt(0).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::MaxStackSize(v) => { VarInt(1).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::MaxDamage(v) => { VarInt(2).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Damage(v) => { VarInt(3).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Unbreakable => { VarInt(4).mc_serialize(serializer)?; }
			StructuredComponent::CustomName(v) => { VarInt(5).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::ItemName(v) => { VarInt(6).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::ItemModel(v) => { VarInt(7).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Lore(v) => { VarInt(8).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Rarity(v) => { VarInt(9).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Enchantments(v) => { VarInt(10).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::CanPlaceOn(v) => { VarInt(11).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::CanBreak(v) => { VarInt(12).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::AttributeModifiers(v) => { VarInt(13).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::CustomModelData(v) => { VarInt(14).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::TooltipDisplay(v) => { VarInt(15).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::RepairCost(v) => { VarInt(16).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::CreativeSlotLock => { VarInt(17).mc_serialize(serializer)?; }
			StructuredComponent::EnchantmentGlintOverride(v) => { VarInt(18).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::IntangibleProjectile(v) => { VarInt(19).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Food(v) => { VarInt(20).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Consumable(v) => { VarInt(21).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::UseRemainder(v) => { VarInt(22).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::UseCooldown(v) => { VarInt(23).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::DamageResistant(v) => { VarInt(24).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Tool(v) => { VarInt(25).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Weapon(v) => { VarInt(26).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Enchantable(v) => { VarInt(27).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Equippable(v) => { VarInt(28).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Repairable(v) => { VarInt(29).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Glider => { VarInt(30).mc_serialize(serializer)?; }
			StructuredComponent::TooltipStyle(v) => { VarInt(31).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::DeathProtection(v) => { VarInt(32).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::BlocksAttacks(v) => { VarInt(33).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::StoredEnchantments(v) => { VarInt(34).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::DyedColor(v) => { VarInt(35).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::MapColor(v) => { VarInt(36).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::MapId(v) => { VarInt(37).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::MapDecorations(v) => { VarInt(38).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::MapPostProcessing(v) => { VarInt(39).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::ChargedProjectiles(v) => { VarInt(40).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::BundleContents(v) => { VarInt(41).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::PotionContents(v) => { VarInt(42).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::PotionDurationScale(v) => { VarInt(43).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::SuspiciousStewEffects(v) => { VarInt(44).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::WritableBookContent(v) => { VarInt(45).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::WrittenBookContent(v) => { VarInt(46).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Trim(v) => { VarInt(47).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::DebugStickState(v) => { VarInt(48).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::EntityData(v) => { VarInt(49).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::BucketEntityData(v) => { VarInt(50).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::BlockEntityData(v) => { VarInt(51).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::InstrumentComponent(v) => { VarInt(52).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::ProvidesTrimMaterial(v) => { VarInt(53).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::OminousBottleAmplifier(v) => { VarInt(54).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::JukeboxPlayable(v) => { VarInt(55).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::ProvidesBannerPatterns(v) => { VarInt(56).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Recipes(v) => { VarInt(57).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::LodestoneTracker(v) => { VarInt(58).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::FireworkExplosionComponent(v) => { VarInt(59).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Fireworks(v) => { VarInt(60).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Profile(v) => { VarInt(61).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::NoteBlockSound(v) => { VarInt(62).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::BannerPatterns(v) => { VarInt(63).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::BaseColor(v) => { VarInt(64).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::PotDecorations(v) => { VarInt(65).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Container(v) => { VarInt(66).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::BlockState(v) => { VarInt(67).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Bees(v) => { VarInt(68).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::Lock(v) => { VarInt(69).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::ContainerLoot(v) => { VarInt(70).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::BreakSound(v) => { VarInt(71).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::VillagerVariant(v) => { VarInt(72).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::WolfVariant(v) => { VarInt(73).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::WolfSoundVariant(v) => { VarInt(74).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::WolfCollar(v) => { VarInt(75).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::FoxVariant(v) => { VarInt(76).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::SalmonSize(v) => { VarInt(77).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::ParrotVariant(v) => { VarInt(78).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::TropicalFishPattern(v) => { VarInt(79).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::TropicalFishBaseColor(v) => { VarInt(80).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::TropicalFishPatternColor(v) => { VarInt(81).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::MooshroomVariant(v) => { VarInt(82).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::RabbitVariant(v) => { VarInt(83).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::PigVariant(v) => { VarInt(84).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::CowVariant(v) => { VarInt(85).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::ChickenVariant(v) => { VarInt(86).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::FrogVariant(v) => { VarInt(87).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::HorseVariant(v) => { VarInt(88).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::PaintingVariant(v) => { VarInt(89).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::LlamaVariant(v) => { VarInt(90).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::AxolotlVariant(v) => { VarInt(91).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::CatVariant(v) => { VarInt(92).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::CatCollar(v) => { VarInt(93).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::SheepColor(v) => { VarInt(94).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
			StructuredComponent::ShulkerColor(v) => { VarInt(95).mc_serialize(serializer)?; v.mc_serialize(serializer)?; }
		}
		Ok(())
	}
}

impl McDeserialize for StructuredComponent {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let id = VarInt::mc_deserialize(deserializer)?.0;
		match id {
			0 => Ok(StructuredComponent::CustomData(NbtCompound::mc_deserialize(deserializer)?)),
			1 => Ok(StructuredComponent::MaxStackSize(VarInt::mc_deserialize(deserializer)?)),
			2 => Ok(StructuredComponent::MaxDamage(VarInt::mc_deserialize(deserializer)?)),
			3 => Ok(StructuredComponent::Damage(VarInt::mc_deserialize(deserializer)?)),
			4 => Ok(StructuredComponent::Unbreakable),
			5 => Ok(StructuredComponent::CustomName(TextComponent::mc_deserialize(deserializer)?)),
			6 => Ok(StructuredComponent::ItemName(TextComponent::mc_deserialize(deserializer)?)),
			7 => Ok(StructuredComponent::ItemModel(String::mc_deserialize(deserializer)?)),
			8 => Ok(StructuredComponent::Lore(PrefixedArray::mc_deserialize(deserializer)?)),
			9 => Ok(StructuredComponent::Rarity(VarInt::mc_deserialize(deserializer)?)),
			10 => Ok(StructuredComponent::Enchantments(EnchantmentList::mc_deserialize(deserializer)?)),
			11 => Ok(StructuredComponent::CanPlaceOn(PrefixedArray::mc_deserialize(deserializer)?)),
			12 => Ok(StructuredComponent::CanBreak(PrefixedArray::mc_deserialize(deserializer)?)),
			13 => Ok(StructuredComponent::AttributeModifiers(AttributeModifierList::mc_deserialize(deserializer)?)),
			14 => Ok(StructuredComponent::CustomModelData(CustomModelDataComponent::mc_deserialize(deserializer)?)),
			15 => Ok(StructuredComponent::TooltipDisplay(TooltipDisplayComponent::mc_deserialize(deserializer)?)),
			16 => Ok(StructuredComponent::RepairCost(VarInt::mc_deserialize(deserializer)?)),
			17 => Ok(StructuredComponent::CreativeSlotLock),
			18 => Ok(StructuredComponent::EnchantmentGlintOverride(bool::mc_deserialize(deserializer)?)),
			19 => Ok(StructuredComponent::IntangibleProjectile(NbtCompound::mc_deserialize(deserializer)?)),
			20 => Ok(StructuredComponent::Food(FoodComponent::mc_deserialize(deserializer)?)),
			21 => Ok(StructuredComponent::Consumable(ConsumableComponent::mc_deserialize(deserializer)?)),
			22 => Ok(StructuredComponent::UseRemainder(Box::new(SlotData::mc_deserialize(deserializer)?))),
			23 => Ok(StructuredComponent::UseCooldown(UseCooldownComponent::mc_deserialize(deserializer)?)),
			24 => Ok(StructuredComponent::DamageResistant(String::mc_deserialize(deserializer)?)),
			25 => Ok(StructuredComponent::Tool(ToolComponent::mc_deserialize(deserializer)?)),
			26 => Ok(StructuredComponent::Weapon(WeaponComponent::mc_deserialize(deserializer)?)),
			27 => Ok(StructuredComponent::Enchantable(VarInt::mc_deserialize(deserializer)?)),
			28 => Ok(StructuredComponent::Equippable(EquippableComponent::mc_deserialize(deserializer)?)),
			29 => Ok(StructuredComponent::Repairable(IDSet::mc_deserialize(deserializer)?)),
			30 => Ok(StructuredComponent::Glider),
			31 => Ok(StructuredComponent::TooltipStyle(String::mc_deserialize(deserializer)?)),
			32 => Ok(StructuredComponent::DeathProtection(PrefixedArray::mc_deserialize(deserializer)?)),
			33 => Ok(StructuredComponent::BlocksAttacks(BlocksAttacksComponent::mc_deserialize(deserializer)?)),
			34 => Ok(StructuredComponent::StoredEnchantments(EnchantmentList::mc_deserialize(deserializer)?)),
			35 => Ok(StructuredComponent::DyedColor(i32::mc_deserialize(deserializer)?)),
			36 => Ok(StructuredComponent::MapColor(i32::mc_deserialize(deserializer)?)),
			37 => Ok(StructuredComponent::MapId(VarInt::mc_deserialize(deserializer)?)),
			38 => Ok(StructuredComponent::MapDecorations(NbtCompound::mc_deserialize(deserializer)?)),
			39 => Ok(StructuredComponent::MapPostProcessing(VarInt::mc_deserialize(deserializer)?)),
			40 => Ok(StructuredComponent::ChargedProjectiles(PrefixedArray::mc_deserialize(deserializer)?)),
			41 => Ok(StructuredComponent::BundleContents(PrefixedArray::mc_deserialize(deserializer)?)),
			42 => Ok(StructuredComponent::PotionContents(PotionContentsComponent::mc_deserialize(deserializer)?)),
			43 => Ok(StructuredComponent::PotionDurationScale(f32::mc_deserialize(deserializer)?)),
			44 => Ok(StructuredComponent::SuspiciousStewEffects(SuspiciousStewList::mc_deserialize(deserializer)?)),
			45 => Ok(StructuredComponent::WritableBookContent(WritableBookComponent::mc_deserialize(deserializer)?)),
			46 => Ok(StructuredComponent::WrittenBookContent(WrittenBookComponent::mc_deserialize(deserializer)?)),
			47 => Ok(StructuredComponent::Trim(TrimComponent::mc_deserialize(deserializer)?)),
			48 => Ok(StructuredComponent::DebugStickState(NbtCompound::mc_deserialize(deserializer)?)),
			49 => Ok(StructuredComponent::EntityData(EntityDataComponent::mc_deserialize(deserializer)?)),
			50 => Ok(StructuredComponent::BucketEntityData(NbtCompound::mc_deserialize(deserializer)?)),
			51 => Ok(StructuredComponent::BlockEntityData(BlockEntityDataComponent::mc_deserialize(deserializer)?)),
			52 => Ok(StructuredComponent::InstrumentComponent(IdOrInstrument::mc_deserialize(deserializer)?)),
			53 => Ok(StructuredComponent::ProvidesTrimMaterial(ProvidesTrimMaterialComponent::mc_deserialize(deserializer)?)),
			54 => Ok(StructuredComponent::OminousBottleAmplifier(VarInt::mc_deserialize(deserializer)?)),
			55 => Ok(StructuredComponent::JukeboxPlayable(JukeboxPlayableComponent::mc_deserialize(deserializer)?)),
			56 => Ok(StructuredComponent::ProvidesBannerPatterns(String::mc_deserialize(deserializer)?)),
			57 => Ok(StructuredComponent::Recipes(NbtCompound::mc_deserialize(deserializer)?)),
			58 => Ok(StructuredComponent::LodestoneTracker(LodestoneTrackerComponent::mc_deserialize(deserializer)?)),
			59 => Ok(StructuredComponent::FireworkExplosionComponent(FireworkExplosion::mc_deserialize(deserializer)?)),
			60 => Ok(StructuredComponent::Fireworks(FireworksComponent::mc_deserialize(deserializer)?)),
			61 => Ok(StructuredComponent::Profile(ResolvableProfile::mc_deserialize(deserializer)?)),
			62 => Ok(StructuredComponent::NoteBlockSound(String::mc_deserialize(deserializer)?)),
			63 => Ok(StructuredComponent::BannerPatterns(BannerPatternsComponent::mc_deserialize(deserializer)?)),
			64 => Ok(StructuredComponent::BaseColor(DyeColor::mc_deserialize(deserializer)?)),
			65 => Ok(StructuredComponent::PotDecorations(PrefixedArray::mc_deserialize(deserializer)?)),
			66 => Ok(StructuredComponent::Container(PrefixedArray::mc_deserialize(deserializer)?)),
			67 => Ok(StructuredComponent::BlockState(BlockStateComponent::mc_deserialize(deserializer)?)),
			68 => Ok(StructuredComponent::Bees(PrefixedArray::mc_deserialize(deserializer)?)),
			69 => Ok(StructuredComponent::Lock(NbtTag::mc_deserialize(deserializer)?)),
			70 => Ok(StructuredComponent::ContainerLoot(NbtCompound::mc_deserialize(deserializer)?)),
			71 => Ok(StructuredComponent::BreakSound(IDorX::mc_deserialize(deserializer)?)),
			72 => Ok(StructuredComponent::VillagerVariant(VarInt::mc_deserialize(deserializer)?)),
			73 => Ok(StructuredComponent::WolfVariant(VarInt::mc_deserialize(deserializer)?)),
			74 => Ok(StructuredComponent::WolfSoundVariant(VarInt::mc_deserialize(deserializer)?)),
			75 => Ok(StructuredComponent::WolfCollar(DyeColor::mc_deserialize(deserializer)?)),
			76 => Ok(StructuredComponent::FoxVariant(VarInt::mc_deserialize(deserializer)?)),
			77 => Ok(StructuredComponent::SalmonSize(VarInt::mc_deserialize(deserializer)?)),
			78 => Ok(StructuredComponent::ParrotVariant(VarInt::mc_deserialize(deserializer)?)),
			79 => Ok(StructuredComponent::TropicalFishPattern(VarInt::mc_deserialize(deserializer)?)),
			80 => Ok(StructuredComponent::TropicalFishBaseColor(DyeColor::mc_deserialize(deserializer)?)),
			81 => Ok(StructuredComponent::TropicalFishPatternColor(DyeColor::mc_deserialize(deserializer)?)),
			82 => Ok(StructuredComponent::MooshroomVariant(VarInt::mc_deserialize(deserializer)?)),
			83 => Ok(StructuredComponent::RabbitVariant(VarInt::mc_deserialize(deserializer)?)),
			84 => Ok(StructuredComponent::PigVariant(VarInt::mc_deserialize(deserializer)?)),
			85 => Ok(StructuredComponent::CowVariant(VarInt::mc_deserialize(deserializer)?)),
			86 => Ok(StructuredComponent::ChickenVariant(ChickenVariantComponent::mc_deserialize(deserializer)?)),
			87 => Ok(StructuredComponent::FrogVariant(VarInt::mc_deserialize(deserializer)?)),
			88 => Ok(StructuredComponent::HorseVariant(VarInt::mc_deserialize(deserializer)?)),
			89 => Ok(StructuredComponent::PaintingVariant(PaintingVariantComponent::mc_deserialize(deserializer)?)),
			90 => Ok(StructuredComponent::LlamaVariant(VarInt::mc_deserialize(deserializer)?)),
			91 => Ok(StructuredComponent::AxolotlVariant(VarInt::mc_deserialize(deserializer)?)),
			92 => Ok(StructuredComponent::CatVariant(VarInt::mc_deserialize(deserializer)?)),
			93 => Ok(StructuredComponent::CatCollar(DyeColor::mc_deserialize(deserializer)?)),
			94 => Ok(StructuredComponent::SheepColor(DyeColor::mc_deserialize(deserializer)?)),
			95 => Ok(StructuredComponent::ShulkerColor(DyeColor::mc_deserialize(deserializer)?)),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid StructuredComponent id: {}", id)))
		}
	}
}

impl McDefault for StructuredComponent {
	fn mc_default() -> Self {
		StructuredComponent::MaxStackSize(VarInt(64))
	}
}
