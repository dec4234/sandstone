//! Defines the data structure describing a single Minecraft block, modeled after the
//! `blocks.json` schema from minecraft-data. See the schema for field semantics.
//!
//! Field names intentionally match the JSON (camelCase) so the structs deserialize directly.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// The bounding box of a block. Either occupies a full block or nothing.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum BoundingBox {
	#[default]
	Block,
	Empty,
}

/// The type of a block state property.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum StateType {
	Enum,
	Bool,
	Int,
	Direction,
}

/// A single state property of a block (e.g. `facing`, `powered`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockState {
	/// The name of the property.
	pub name: String,
	/// The type of the property.
	pub r#type: StateType,
	/// The possible values of the property. Absent for `bool`/`int` types.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub values: Option<Vec<String>>,
	/// The number of possible values.
	pub num_values: u32,
}

/// A metadata-based variation of a block.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct Variation {
	pub metadata: u32,
	pub displayName: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
}

/// The concrete item a block drops. Either a bare item id, or an id paired with metadata.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum DropItem {
	Id(u32),
	Detailed { id: u32, metadata: u32 },
}

/// A detailed drop entry, carrying an optional count/chance range.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct DetailedDrop {
	/// Minimum number or chance, default: 1.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub minCount: Option<f64>,
	/// Maximum number or chance, default: minCount.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub maxCount: Option<f64>,
	pub drop: DropItem,
}

/// An entry in a block's drop list. Either a bare item id, or a detailed drop with counts.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Drop {
	Item(u32),
	Detailed(DetailedDrop),
}

/// Describes a single Minecraft block. Mirrors the `blocks.json` schema; fields not marked
/// required in the schema are represented as `Option<T>`.
///
/// Note: `hardness` and `resistance` are `Option` because they are nullable in the schema
/// (`hardness` is required-but-nullable, `resistance` is optional-and-nullable).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[allow(non_snake_case)]
pub struct BlockInfo {
	/// The unique identifier for a block.
	pub id: i32,
	/// The display name of a block.
	pub displayName: String,
	/// The name of a block.
	pub name: String,
	/// Hardness of a block. `None` if null.
	pub hardness: Option<f32>,
	/// Stack size for a block.
	pub stackSize: u32,
	/// True if a block is diggable.
	pub diggable: bool,
	/// BoundingBox of a block.
	pub boundingBox: BoundingBox,
	/// Material of a block.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub material: Option<String>,
	/// Tools that harvest a block without a time penalty, keyed by item id.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub harvestTools: Option<HashMap<u32, bool>>,
	/// Metadata-based variations of a block.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub variations: Option<Vec<Variation>>,
	/// The state properties of a block.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub states: Option<Vec<BlockState>>,
	/// The items a block drops.
	pub drops: Vec<Drop>,
	/// True if a block is transparent.
	pub transparent: bool,
	/// Light emitted by that block (0-15).
	pub emitLight: u8,
	/// Light filtered by that block (0-15).
	pub filterLight: u8,
	/// Minimum state id. Always present in the schema.
	pub minStateId: u32,
	/// Maximum state id. Always present in the schema.
	pub maxStateId: u32,
	/// Default state id.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub defaultState: Option<u32>,
	/// Blast resistance. `None` if null.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub resistance: Option<f32>,
}
