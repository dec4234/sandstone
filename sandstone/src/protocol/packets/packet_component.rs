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
use sandstone_derive::ByteEnum;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};
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

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct Tag {
	pub identifier: String,
	pub entries: PrefixedArray<VarInt>
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

#[derive(ByteEnum, McDefault, Debug, Clone, PartialEq)]
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
	pub modifiers: PrefixedArray<ModifierData>
}

#[derive(McSerialize, McDeserialize, McDefault, Debug, Clone, PartialEq)]
pub struct ModifierData {
	pub id: String,
	pub amount: f64,
	pub operation: ModifierOperation
}

#[derive(ByteEnum, McDefault, Debug, Clone, PartialEq)]
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