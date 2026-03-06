//! Defines a lot of random components of network packets. This is separate from packet.rs to reduce
//! clutter.

use crate::protocol::game::info::inventory::slots::{RecipeDisplay, SlotDisplay};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::internal_types::IDSet;
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::util::java::bitfield::BitField;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};
use uuid::Uuid;

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
	pub version: String
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct TagArray {
	pub identifier: String,
	pub payload: PrefixedArray<VarInt>
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProtocolPropertyElement {
	pub name: String,
	pub value: String,
	pub signature: PrefixedOptional<String>
}

#[derive(McDefault, Debug, Clone, PartialEq, Eq)]
pub struct PlayerAbilityFlags {
	pub invulnerable: bool,
	pub flying: bool,
	pub allow_flying: bool,
	pub creative_mode: bool
}

impl McSerialize for PlayerAbilityFlags {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let mut field = BitField::new(0u8);
		
		field.set_bit(0, self.invulnerable);
		field.set_bit(1, self.flying);
		field.set_bit(2, self.allow_flying);
		field.set_bit(3, self.creative_mode);
		
		field.mc_serialize(serializer)
	}
}

impl McDeserialize for PlayerAbilityFlags {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized
	{
		let field = BitField::<u8>::mc_deserialize(deserializer)?;
		
		Ok(Self {
			invulnerable: field.get_bit(0),
			flying: field.get_bit(1),
			allow_flying: field.get_bit(2),
			creative_mode: field.get_bit(3)
		})
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PropertySet {
	pub identifier: String,
	pub items: PrefixedArray<VarInt>
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
	pub slot_display: SlotDisplay
}

impl McDefault for StonecutterRecipe {
	fn mc_default() -> Self {
		Self {
			id_set: IDSet {
				typ: VarInt(4),
				tag_name: None,
				ids: Some(vec![VarInt(0), VarInt(1), VarInt(2)]),
			},
			slot_display: SlotDisplay::Empty
		}
	}
}

#[derive(McDefault, Debug, Clone, PartialEq)]
pub enum GameEventType {
	NoRespawnBlockAvailable,
	BeginRaining,
	EndRaining,
	ChangeGameMode,
	WinGame,
	DemoEvent,
	ArrowHitPlayer,
	RainLevelChange,
	ThunderLevelChange,
	PlayPufferfishStingSound,
	PlayElderGuardianAppearance,
	EnableRespawnScreen,
	LimitedCrafting,
	StartWaitingForLevelChunks,
}

impl McSerialize for GameEventType {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let id: u8 = match self {
			GameEventType::NoRespawnBlockAvailable => 0,
			GameEventType::BeginRaining => 1,
			GameEventType::EndRaining => 2,
			GameEventType::ChangeGameMode => 3,
			GameEventType::WinGame => 4,
			GameEventType::DemoEvent => 5,
			GameEventType::ArrowHitPlayer => 6,
			GameEventType::RainLevelChange => 7,
			GameEventType::ThunderLevelChange => 8,
			GameEventType::PlayPufferfishStingSound => 9,
			GameEventType::PlayElderGuardianAppearance => 10,
			GameEventType::EnableRespawnScreen => 11,
			GameEventType::LimitedCrafting => 12,
			GameEventType::StartWaitingForLevelChunks => 13,
		};
		id.mc_serialize(serializer)
	}
}

impl McDeserialize for GameEventType {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let id = u8::mc_deserialize(deserializer)?;
		match id {
			0 => Ok(GameEventType::NoRespawnBlockAvailable),
			1 => Ok(GameEventType::BeginRaining),
			2 => Ok(GameEventType::EndRaining),
			3 => Ok(GameEventType::ChangeGameMode),
			4 => Ok(GameEventType::WinGame),
			5 => Ok(GameEventType::DemoEvent),
			6 => Ok(GameEventType::ArrowHitPlayer),
			7 => Ok(GameEventType::RainLevelChange),
			8 => Ok(GameEventType::ThunderLevelChange),
			9 => Ok(GameEventType::PlayPufferfishStingSound),
			10 => Ok(GameEventType::PlayElderGuardianAppearance),
			11 => Ok(GameEventType::EnableRespawnScreen),
			12 => Ok(GameEventType::LimitedCrafting),
			13 => Ok(GameEventType::StartWaitingForLevelChunks),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid GameEventType id: {}", id))),
		}
	}
}

#[derive(McSerialize, McDeserialize, McDefault, Debug, Clone, PartialEq)]
pub struct AttributeProperty {
	pub id: VarInt,
	pub value: f64,
	pub modifiers: PrefixedArray<ModifierData>
}

#[derive(McSerialize, McDeserialize, McDefault, Debug, Clone, PartialEq)]
pub struct ModifierData {
	pub id: String,
	pub amount: f64,
	pub operation: ModifierOperation
}

#[derive(McDefault, Debug, Clone, PartialEq)]
pub enum ModifierOperation {
	AddSubtractAmount,
	AddSubtractPercentage,
	MultiplyPercentage
}

impl McSerialize for ModifierOperation {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let id: u8 = match self {
			ModifierOperation::AddSubtractAmount => 0,
			ModifierOperation::AddSubtractPercentage => 1,
			ModifierOperation::MultiplyPercentage => 2,
		};
		id.mc_serialize(serializer)
	}
}

impl McDeserialize for ModifierOperation {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let id = u8::mc_deserialize(deserializer)?;
		match id {
			0 => Ok(ModifierOperation::AddSubtractAmount),
			1 => Ok(ModifierOperation::AddSubtractPercentage),
			2 => Ok(ModifierOperation::MultiplyPercentage),
			_ => Err(SerializingErr::OutOfBounds(format!("Invalid ModifierOperation id: {}", id))),
		}
	}
}

