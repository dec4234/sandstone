//! Defines the data structure describing a single Minecraft item, modeled after the
//! `items.json` schema from minecraft-data.
//!
//! Field names intentionally match the JSON (camelCase) so the structs deserialize directly.

use serde::{Deserialize, Serialize};

/// Describes a single Minecraft item. Mirrors the `items.json` schema; fields not marked
/// required in the schema are represented as `Option<T>`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[allow(non_snake_case)]
pub struct ItemInfo {
	/// The unique identifier for an item.
	pub id: i32,
	/// The name of an item.
	pub name: String,
	/// The display name of an item.
	pub displayName: String,
	/// Stack size for an item.
	pub stackSize: u32,
	/// The enchantment categories this item accepts.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub enchantCategories: Option<Vec<String>>,
	/// Maximum durability before the item breaks. Absent for non-damageable items.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub maxDurability: Option<u32>,
	/// Item names that can repair this item in an anvil.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub repairWith: Option<Vec<String>>,
}
