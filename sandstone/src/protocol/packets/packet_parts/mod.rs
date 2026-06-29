//! Parts of packet bodies that are shared or complex.

pub mod auth;
pub mod block;
pub mod debug;
pub mod effects;
pub mod entity;
pub mod item;
pub mod item_modifiers;
pub mod player;
pub mod scoreboard;
pub mod sound;
pub mod stats;

use crate::bitflag;
use crate::protocol::game::player::inventory::slotdata::SlotData;
use crate::protocol::game::player::inventory::slots::{RecipeDisplay, SlotDisplay};
use crate::protocol::game::world::chunk::BiomeByteData;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::game_types::EquipmentSlot;
use crate::protocol_types::datatypes::internal_types::IDSet;
use crate::protocol_types::datatypes::nbt::NbtCompound;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, TypeEnum, VarIntEnum};
use uuid::Uuid;

bitflag!(PlayerAbilityFlags: u8 {
	invulnerable, flying, allow_flying, creative_mode
});

bitflag!(PlayerPositionFlags: u8 {
	on_ground, pushing_against_wall
});

bitflag!(PlayerInputFlags: u8 {
	forward, backward, left, right, jumping, sneaking, sprinting
});

bitflag!(BossBarFlags: u8 {
	should_darken_sky, is_dragon_bar, create_fog
});

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoginPluginSpec {
	pub(crate) message_id: VarInt,
	pub(crate) success: bool,
	#[mc(deserialize_if = success)]
	pub(crate) data: Option<Vec<u8>>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddResourcePackSpec {
	pub(crate) uuid: Uuid,
	pub(crate) url: String,
	pub(crate) hash: String,
	pub(crate) forced: bool,
	pub(crate) has_prompt_message: bool,
	#[mc(deserialize_if = has_prompt_message)]
	pub(crate) prompt_message: Option<String>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct LoginCookieResponseSpec {
	key: String,
	has_payload: bool,
	payload_length: VarInt,
	#[mc(deserialize_if = has_payload)]
	payload: Option<Vec<u8>>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct ResourcePackEntry {
	pub namespace: String,
	pub id: String,
	pub version: String,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct Tag {
	pub identifier: String,
	pub entries: PrefixedArray<VarInt>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProtocolPropertyElement {
	pub name: String,
	pub value: String,
	pub signature: PrefixedOptional<String>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PropertySet {
	pub identifier: String,
	pub items: PrefixedArray<VarInt>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct RecipeBookEntry {
	pub recipe_id: VarInt,
	pub display: RecipeDisplay,
	pub group_id: VarInt,
	pub category_id: VarInt,
	pub ingredients: PrefixedOptional<PrefixedArray<IDSet>>,
	pub flags: u8,
}

#[derive(McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct StonecutterRecipe {
	pub id_set: IDSet,
	pub slot_display: SlotDisplay,
}

impl McDefault for StonecutterRecipe {
	fn mc_default() -> Self {
		Self {
			id_set: IDSet {
				typ: VarInt(4),
				tag_name: None,
				ids: Some(vec![VarInt(0), VarInt(1), VarInt(2)]),
			},
			slot_display: SlotDisplay::Empty,
		}
	}
}

#[derive(TypeEnum, McDefault, Debug, Clone, PartialEq)]
#[type_enum(u8)]
pub enum GameEventType {
	NoRespawnBlockAvailable = 0,
	BeginRaining = 1,
	EndRaining = 2,
	ChangeGameMode = 3,
	WinGame = 4,
	DemoEvent = 5,
	ArrowHitPlayer = 6,
	RainLevelChange = 7,
	ThunderLevelChange = 8,
	PlayPufferfishStingSound = 9,
	PlayElderGuardianAppearance = 10,
	EnableRespawnScreen = 11,
	LimitedCrafting = 12,
	StartWaitingForLevelChunks = 13,
}

#[derive(McSerialize, McDeserialize, McDefault, Debug, Clone, PartialEq)]
pub struct AttributeProperty {
	pub id: VarInt,
	pub value: f64,
	pub modifiers: PrefixedArray<ModifierData>,
}

#[derive(McSerialize, McDeserialize, McDefault, Debug, Clone, PartialEq)]
pub struct ModifierData {
	pub id: String,
	pub amount: f64,
	pub operation: ModifierOperation,
}

#[derive(TypeEnum, McDefault, Debug, Clone, PartialEq)]
#[type_enum(u8)]
pub enum ModifierOperation {
	AddSubtractAmount = 0,
	AddSubtractPercentage = 1,
	MultiplyPercentage = 2,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum ClientStatusAction {
	PerformRespawn = 0,
	RequestStats = 1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EquipmentEntry {
	pub slot: EquipmentSlot,
	pub item: SlotData,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EquipmentList {
	pub entries: Vec<EquipmentEntry>,
}

impl McSerialize for EquipmentList {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		for (i, entry) in self.entries.iter().enumerate() {
			let mut slot_byte = entry.slot.clone() as u8;
			if i < self.entries.len() - 1 {
				slot_byte |= 0x80;
			}
			slot_byte.mc_serialize(serializer)?;
			entry.item.mc_serialize(serializer)?;
		}
		Ok(())
	}
}

impl McDeserialize for EquipmentList {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let mut entries = Vec::new();
		loop {
			let raw_byte = u8::mc_deserialize(deserializer)?;
			let has_next = (raw_byte & 0x80) != 0;
			let slot_byte = raw_byte & 0x7F;

			let slot = EquipmentSlot::from_value(slot_byte)?;
			let item = SlotData::mc_deserialize(deserializer)?;

			entries.push(EquipmentEntry {
				slot,
				item,
			});

			if !has_next {
				break;
			}
		}
		Ok(Self {
			entries,
		})
	}
}

impl McDefault for EquipmentList {
	fn mc_default() -> Self {
		Self {
			entries: vec![EquipmentEntry {
				slot: EquipmentSlot::MainHand,
				item: SlotData::mc_default(),
			}],
		}
	}
}

impl McDefault for EquipmentEntry {
	fn mc_default() -> Self {
		Self {
			slot: EquipmentSlot::MainHand,
			item: SlotData::mc_default(),
		}
	}
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum PlayerCommandAction {
	LeaveBed = 0,
	StartSprinting = 1,
	StopSprinting = 2,
	StartJumpWithHorse = 3,
	StopJumpWithHorse = 4,
	OpenVehicleInventory = 5,
	StartFlyingWithElytra = 6,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum InteractType {
	Interact = 0,
	Attack = 1,
	InteractAt = 2,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum InteractHand {
	Main = 0,
	OffHand = 1,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct CustomReportDetails {
	pub title: String,
	pub description: String,
}

/// A single entry in the Server Links packet. The label is a discriminated union: the
/// `is_built_in` boolean selects between a known [ServerLinkStandardLabel] (VarInt enum) and
/// a custom [TextComponent]. Exactly one of `built_in_label`/`custom_label` is present on the wire.
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ServerLink {
	pub is_built_in: bool,
	#[mc(deserialize_if = is_built_in)]
	pub built_in_label: Option<ServerLinkStandardLabel>,
	#[mc(deserialize_if = !is_built_in)]
	pub custom_label: Option<TextComponent>,
	pub url: String,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum ServerLinkStandardLabel {
	#[doc = "Displayed on connection error screen; included as a comment in the disconnection report."]
	BugReport = 0,
	CommunityGuidelines = 1,
	Support = 2,
	Status = 3,
	Feedback = 4,
	Community = 5,
	Website = 6,
	Forums = 7,
	News = 8,
	Announcements = 9,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum BossBarUpdateAction {
	Add {
		title: TextComponent,
		health: f32,
		color: BossBarColor,
		division: BossBarDivisions,
		flags: BossBarFlags,
	} = 0,
	Remove = 1,
	UpdateHealth {
		health: f32,
	} = 2,
	UpdateTitle {
		title: TextComponent,
	} = 3,
	UpdateStyle {
		color: BossBarColor,
		dividers: BossBarDivisions,
	} = 4,
	UpdateFlags {
		flags: BossBarFlags,
	} = 5,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum BossBarColor {
	Pink = 0,
	Blue = 1,
	Red = 2,
	Green = 3,
	Yellow = 4,
	Purple = 5,
	White = 6,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum BossBarDivisions {
	NoDivision = 0,
	SixNotches = 1,
	TenNotches = 2,
	TwelveNotches = 3,
	TwentyNotches = 4,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ChunkBiomeData {
	pub z: i32,
	pub x: i32,
	/// Chunk data structure, with sections containing only the Biomes field
	pub data: BiomeByteData,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct TooltipMatch {
	pub matc: String,
	pub tooltip: PrefixedOptional<TextComponent>,
}

/// Network representation of chat type.
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ChatTypeNetwork {
	pub chat: ChatTypeEntry,
	pub narration: ChatTypeEntry,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ChatTypeEntry {
	pub translation_key: String,
	parameters: PrefixedArray<ChatTypeParemeter>,
	style: NbtCompound,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum ChatTypeParemeter {
	Sender = 0,
	Target = 1,
	Content = 2,
}
