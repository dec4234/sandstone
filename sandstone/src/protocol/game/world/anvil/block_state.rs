//! Block states by name, as region files keep them, and their global state IDs, as chunk packets
//! send them.

use crate::protocol::game::world::anvil::{AnvilError, AnvilResult};
use crate::protocol::game::world::block_physics::block_by_id;
use crate::protocol::game::world::block_state_table::{BLOCKS, BlockInfo};
use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

static BY_NAME: LazyLock<HashMap<&'static str, &'static BlockInfo>> = LazyLock::new(|| BLOCKS.iter().map(|block| (block.name, block)).collect());

/// A block and the values of its properties, for example `minecraft:oak_stairs[facing=north]`.
///
/// A property that is not in `properties` has the value of the block's default state. A block
/// state with no properties is the default state of the block.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockState {
	pub name: String,
	pub properties: BTreeMap<String, String>,
}

impl BlockState {
	/// The default state of the block `name` (for example `minecraft:stone`).
	pub fn new<S: Into<String>>(name: S) -> Self {
		Self {
			name: name.into(),
			properties: BTreeMap::new(),
		}
	}

	/// The state with the global state ID `id`, with every property set. Returns `None` if no block
	/// has that ID.
	pub fn from_id(id: i32) -> Option<Self> {
		let block = block_by_id(id)?;
		let mut index = (id - block.first_id) as usize;
		let mut properties = BTreeMap::new();
		// The last property changes fastest, so take the values from the end.
		for (name, values) in block.properties.iter().rev() {
			properties.insert(name.to_string(), values[index % values.len()].to_string());
			index /= values.len();
		}

		Some(Self {
			name: block.name.to_string(),
			properties,
		})
	}

	/// The global state ID of this state. Errors if the block, a property, or a value is unknown.
	pub fn to_id(&self) -> AnvilResult<i32> {
		let block = *BY_NAME.get(self.name.as_str()).ok_or_else(|| AnvilError::UnknownBlock(self.name.clone()))?;

		for name in self.properties.keys() {
			if !block.properties.iter().any(|(p, _)| p == name) {
				return Err(AnvilError::UnknownBlock(format!("{}: no property `{name}`", self.name)));
			}
		}

		// Start from the default state so that a property that is not given keeps its default value.
		let mut default_index = (block.default_id - block.first_id) as usize;
		let mut positions = vec![0; block.properties.len()];
		for (i, (_, values)) in block.properties.iter().enumerate().rev() {
			positions[i] = default_index % values.len();
			default_index /= values.len();
		}

		let mut index = 0;
		for (i, (name, values)) in block.properties.iter().enumerate() {
			let position = match self.properties.get(*name) {
				Some(value) => values
					.iter()
					.position(|v| v == value)
					.ok_or_else(|| AnvilError::UnknownBlock(format!("{}: `{name}` has no value `{value}`", self.name)))?,
				None => positions[i],
			};
			index = index * values.len() + position;
		}

		Ok(block.first_id + index as i32)
	}

	/// True if this is the default state of its block. Errors if the state is unknown.
	pub fn is_default(&self) -> AnvilResult<bool> {
		Ok(self.to_id()? == BY_NAME[self.name.as_str()].default_id)
	}
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::protocol::game::world::block_state_table::BLOCK_STATE_COUNT;
    use crate::protocol::game::world::block_states::{AIR, GRASS_BLOCK, STONE};

    fn state(name: &str, properties: &[(&str, &str)]) -> BlockState {
		BlockState {
			name: name.to_string(),
			properties: properties.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
		}
	}

	/// Every ID must turn into a state and back into the same ID. If one does not, a chunk read from
	/// a region file shows the wrong blocks.
	#[test]
	fn every_state_round_trips() {
		for id in 0..BLOCK_STATE_COUNT {
			let state = BlockState::from_id(id).unwrap_or_else(|| panic!("no state for {id}"));
			assert_eq!(state.to_id().unwrap(), id, "{state:?}");
		}
		assert!(BlockState::from_id(BLOCK_STATE_COUNT).is_none());
		assert!(BlockState::from_id(-1).is_none());
	}

	/// The IDs must match the vanilla report, not only be self-consistent. These values are from
	/// the 26.3 blocks.json report.
	#[test]
	fn ids_match_the_vanilla_report() {
		assert_eq!(state("minecraft:acacia_button", &[("face", "floor"), ("facing", "north"), ("powered", "true")]).to_id().unwrap(), 12514);
		assert_eq!(
			state("minecraft:acacia_button", &[("face", "floor"), ("facing", "south"), ("powered", "false")]).to_id().unwrap(),
			12517
		);
		assert_eq!(BlockState::new("minecraft:stone").to_id().unwrap(), STONE);
		assert_eq!(BlockState::new("minecraft:air").to_id().unwrap(), AIR);
		assert_eq!(
			BlockState::from_id(12517).unwrap(),
			state("minecraft:acacia_button", &[("face", "floor"), ("facing", "south"), ("powered", "false")])
		);
	}

	/// Region files leave out properties for the default state. A missing property must take the
	/// default value, not the first value (grass_block's first state is snowy=true).
	#[test]
	fn missing_properties_use_the_default() {
		assert_eq!(BlockState::new("minecraft:grass_block").to_id().unwrap(), GRASS_BLOCK);
		assert_eq!(state("minecraft:grass_block", &[("snowy", "true")]).to_id().unwrap(), GRASS_BLOCK - 1);
		assert!(BlockState::new("minecraft:grass_block").is_default().unwrap());
		assert!(!state("minecraft:grass_block", &[("snowy", "true")]).is_default().unwrap());
	}

	#[test]
	fn unknown_names_are_errors() {
		assert!(matches!(BlockState::new("minecraft:not_a_block").to_id(), Err(AnvilError::UnknownBlock(_))));
		assert!(matches!(state("minecraft:stone", &[("facing", "north")]).to_id(), Err(AnvilError::UnknownBlock(_))));
		assert!(matches!(state("minecraft:grass_block", &[("snowy", "maybe")]).to_id(), Err(AnvilError::UnknownBlock(_))));
	}
}
