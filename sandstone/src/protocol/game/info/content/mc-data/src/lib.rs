//! A compile-time wrapper around the PrismarineJS
//! [minecraft-data](https://github.com/PrismarineJS/minecraft-data) repository, focused on
//! blocks and items.
//!
//! Block/item data is fetched from the repo at build time for the requested Minecraft
//! version (see `build.rs`), embedded into the binary, and exposed two ways:
//!
//! - The generated [`Block`] / [`Item`] namespaces give a zero-sized, type-safe handle for
//!   every entry with `const` accessors:
//!   ```
//!   # use mc_data::Block;
//!   let id = Block::Bedrock::get_id();
//!   let name = Block::Bedrock::get_name();
//!   let full = Block::Bedrock::info(); // &'static BlockInfo
//!   ```
//! - The [`blocks`] / [`items`] data tables expose the full [`BlockInfo`] / [`ItemInfo`]
//!   metadata, with `*_by_id` / `*_by_name` lookups.
//!
//! The version actually built in is available as [`RESOLVED_VERSION`].

pub mod blocks;
pub mod items;

use std::collections::HashMap;
use std::sync::LazyLock;

pub use blocks::BlockInfo;
pub use items::ItemInfo;

/// The minecraft-data version (fancy name, e.g. `1.21.11`) the embedded data was built from.
pub const RESOLVED_VERSION: &str = env!("SANDSTONE_MC_VERSION_RESOLVED");

static BLOCKS_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/blocks.json"));
static ITEMS_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/items.json"));

static BLOCKS_DATA: LazyLock<Vec<BlockInfo>> =
	LazyLock::new(|| serde_json::from_str(BLOCKS_JSON).expect("embedded blocks.json is valid"));
static ITEMS_DATA: LazyLock<Vec<ItemInfo>> =
	LazyLock::new(|| serde_json::from_str(ITEMS_JSON).expect("embedded items.json is valid"));

static BLOCKS_BY_ID: LazyLock<HashMap<i32, usize>> =
	LazyLock::new(|| BLOCKS_DATA.iter().enumerate().map(|(i, b)| (b.id, i)).collect());
static BLOCKS_BY_NAME: LazyLock<HashMap<&'static str, usize>> =
	LazyLock::new(|| BLOCKS_DATA.iter().enumerate().map(|(i, b)| (b.name.as_str(), i)).collect());
static ITEMS_BY_ID: LazyLock<HashMap<i32, usize>> =
	LazyLock::new(|| ITEMS_DATA.iter().enumerate().map(|(i, it)| (it.id, i)).collect());
static ITEMS_BY_NAME: LazyLock<HashMap<&'static str, usize>> =
	LazyLock::new(|| ITEMS_DATA.iter().enumerate().map(|(i, it)| (it.name.as_str(), i)).collect());

/// All blocks for the resolved version, in id order.
pub fn blocks() -> &'static [BlockInfo] {
	&BLOCKS_DATA
}

/// All items for the resolved version, in id order.
pub fn items() -> &'static [ItemInfo] {
	&ITEMS_DATA
}

/// Looks up the full metadata for a block by its numeric id.
pub fn block_by_id(id: i32) -> Option<&'static BlockInfo> {
	BLOCKS_BY_ID.get(&id).map(|&i| &BLOCKS_DATA[i])
}

/// Looks up the full metadata for a block by its registry name (e.g. `stone`).
pub fn block_by_name(name: &str) -> Option<&'static BlockInfo> {
	BLOCKS_BY_NAME.get(name).map(|&i| &BLOCKS_DATA[i])
}

/// Looks up the full metadata for an item by its numeric id.
pub fn item_by_id(id: i32) -> Option<&'static ItemInfo> {
	ITEMS_BY_ID.get(&id).map(|&i| &ITEMS_DATA[i])
}

/// Looks up the full metadata for an item by its registry name (e.g. `stick`).
pub fn item_by_name(name: &str) -> Option<&'static ItemInfo> {
	ITEMS_BY_NAME.get(name).map(|&i| &ITEMS_DATA[i])
}

// Generated `Block` / `Item` namespaces (one unit struct per entry). Their `info()` methods
// rely on the lookups above.
include!(concat!(env!("OUT_DIR"), "/content_generated.rs"));

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn data_tables_non_empty() {
		assert!(!blocks().is_empty(), "blocks table should not be empty");
		assert!(!items().is_empty(), "items table should not be empty");
	}

	#[test]
	fn block_lookups_agree() {
		let by_name = block_by_name("stone").expect("stone block exists");
		let by_id = block_by_id(by_name.id).expect("stone block id resolves");
		assert_eq!(by_name, by_id);
		assert_eq!(by_name.name, "stone");
	}

	#[test]
	fn namespace_accessors_match_data() {
		// Air is id 0 in every modern version, for both blocks and items.
		assert_eq!(Block::Air::get_id(), 0);
		assert_eq!(Block::Air::get_name(), "air");
		assert_eq!(Item::Air::get_id(), 0);

		// `get_*` accessors must agree with the full metadata table.
		let info = Block::Stone::info();
		assert_eq!(info.id, Block::Stone::get_id());
		assert_eq!(info.name, Block::Stone::get_name());
		assert_eq!(info.displayName, Block::Stone::get_display_name());
		assert_eq!(info.minStateId, Block::Stone::get_min_id());
		assert_eq!(info.maxStateId, Block::Stone::get_max_id());

		// The headline ergonomic the namespace exists for.
		assert_eq!(block_by_name("bedrock").unwrap().id, Block::Bedrock::get_id());
	}
}
