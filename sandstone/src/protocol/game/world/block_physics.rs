//! The hardness and the collision shape of each block state, from the tables in
//! [block_state_table](crate::protocol::game::world::block_state_table).

use crate::protocol::game::world::block_state_table::{BLOCKS, BlockInfo, COLLISION_SHAPES, STATE_COLLISION_SHAPES};

/// The block that owns the global state ID `state_id`. Returns `None` if no block has that ID.
pub fn block_by_id(state_id: i32) -> Option<&'static BlockInfo> {
	// BLOCKS is sorted by first_id and the ID ranges have no gaps.
	let next = BLOCKS.partition_point(|block| block.first_id <= state_id);
	let block = BLOCKS.get(next.checked_sub(1)?)?;
	let count: usize = block.properties.iter().map(|(_, values)| values.len()).product();
	(state_id < block.first_id + count as i32).then_some(block)
}

/// The hardness (destroy time) of the block of a state. 0 breaks at once in survival, -1 cannot be
/// broken. Returns `None` if no block has that ID.
pub fn hardness(state_id: i32) -> Option<f32> {
	block_by_id(state_id).map(|block| block.hardness)
}

/// The collision boxes of a state, as `[min_x, min_y, min_z, max_x, max_y, max_z]` relative to the
/// block position. An empty slice means no collision (air, flowers). Returns `None` if no block has
/// that ID.
///
/// If [BlockInfo::dynamic_shape] is true for the block, this is only a default shape.
pub fn collision_shape(state_id: i32) -> Option<&'static [[f64; 6]]> {
	let index = *STATE_COLLISION_SHAPES.get(usize::try_from(state_id).ok()?)?;
	Some(COLLISION_SHAPES[index as usize])
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::protocol::game::world::block_state_table::BLOCK_STATE_COUNT;
    use crate::protocol::game::world::block_states::{AIR, BEDROCK, OAK_FENCE, SHORT_GRASS, STONE};

    /// The values are from the vanilla 26.3 server. Survival break time comes from the hardness, so a
	/// wrong value makes blocks break too fast or too slow.
	#[test]
	fn hardness_matches_vanilla() {
		assert_eq!(hardness(STONE), Some(1.5));
		// Grass breaks at once in survival.
		assert_eq!(hardness(SHORT_GRASS), Some(0.0));
		// Bedrock cannot be broken.
		assert_eq!(hardness(BEDROCK), Some(-1.0));
		assert_eq!(hardness(BLOCK_STATE_COUNT), None);
		assert_eq!(hardness(-1), None);
	}

	/// A player must not be put inside a block, and must be able to walk through air and grass.
	#[test]
	fn collision_shapes_match_vanilla() {
		assert_eq!(collision_shape(AIR), Some(&[][..]));
		assert_eq!(collision_shape(SHORT_GRASS), Some(&[][..]));
		assert_eq!(collision_shape(STONE), Some(&[[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]][..]));
		// A fence with no connections is a post 1.5 blocks high, so a player cannot jump over it.
		assert_eq!(collision_shape(OAK_FENCE), Some(&[[0.375, 0.0, 0.375, 0.625, 1.5, 0.625]][..]));
		assert_eq!(collision_shape(BLOCK_STATE_COUNT), None);
		assert_eq!(collision_shape(-1), None);
	}

	/// Each state must have a shape, or a lookup for the last states panics or gives the wrong shape.
	#[test]
	fn every_state_has_a_shape() {
		assert_eq!(STATE_COLLISION_SHAPES.len(), BLOCK_STATE_COUNT as usize);
		for id in 0..BLOCK_STATE_COUNT {
			assert!(collision_shape(id).is_some(), "{id}");
		}
	}

	#[test]
	fn slab_shape_follows_the_state() {
		// The bottom slab (default) fills the lower half, the top slab the upper half.
		let slab = BLOCKS.iter().find(|b| b.name == "minecraft:oak_slab").unwrap();
		assert_eq!(collision_shape(slab.default_id), Some(&[[0.0, 0.0, 0.0, 1.0, 0.5, 1.0]][..]));
		// type is the first property: [top, bottom, double], waterlogged [true, false].
		assert_eq!(collision_shape(slab.first_id), Some(&[[0.0, 0.5, 0.0, 1.0, 1.0, 1.0]][..]));
	}
}
