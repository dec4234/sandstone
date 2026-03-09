use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};

/// Represents a packed i64 (long) that contains block or biome data. See
/// https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format#Data_Array_format for more info. This
/// matches the spec for packed data after 1.16
#[derive(McDefault, Debug, Clone, Hash, PartialEq, Eq)]
pub struct PackedEntries {
	data: i64,
	/// The number of bits allocated to each entry
	bpe: u8,
}

impl PackedEntries {
	pub fn new(bpe: u8) -> Self {
		Self {
			data: 0,
			bpe,
		}
	}

	/// Get the entry by the index from the packed i64. The first entry occupies the least significant bits
	pub fn get_entry(&self, index: u8) -> u64 {
		let mask = (1 << self.bpe) - 1;
		let shift = index * self.bpe;
		((self.data >> shift) & mask as i64) as u64
	}

	pub fn set_entry(&mut self, index: u8, value: u64) {
		let mask = (1 << self.bpe) - 1;
		let shift = index * self.bpe;
		self.data &= !(mask << shift);
		self.data |= ((value & mask as u64) << shift ) as i64;
	}

	/// A nonstandard deserializer that utilizes bits per entry
	pub(crate) fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer, bpe: u8) -> SerializingResult<'a, Self> where Self: Sized {
		let data = i64::mc_deserialize(deserializer)?;

		Ok(Self {
			data,
			bpe
		})
	}
}

impl McSerialize for PackedEntries {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.data.mc_serialize(serializer)?;
		Ok(())
	}
}

/// ID set used for representing a set of ids in a registry either directly enumerated or indirectly via tag name
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IDSet {
	pub typ: VarInt,
	pub tag_name: Option<String>,
	pub ids: Option<Vec<VarInt>>
}

impl McDefault for IDSet {
	fn mc_default() -> Self {
		Self {
			typ: VarInt(1),
			tag_name: None,
			ids: Some(vec![]),
		}
	}
}

impl McSerialize for IDSet {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.typ.mc_serialize(serializer)?;

		if self.typ.0 == 0 {
			if let Some(tag_name) = &self.tag_name {
				tag_name.mc_serialize(serializer)?;
			} else {
				return Err(SerializingErr::MissingField("IDSet with type 0 must have a tag name".to_string()));
			}
		} else if let Some(ids) = &self.ids { // ids only serialized when type != 0
			if ids.len() != (self.typ.0 - 1) as usize {
				return Err(SerializingErr::InconsistentField(format!("IDSet with type {} must have {} IDs, but {} were provided", self.typ.0, self.typ.0 - 1, ids.len())));
			}
			
			ids.mc_serialize(serializer)?;
		} else {
			return Err(SerializingErr::MissingField("IDSet with type 0 must have an ID list".to_string()));
		}
		Ok(())
	}
}

impl McDeserialize for IDSet {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?;
		if typ.0 == 0 {
			let tag_name = String::mc_deserialize(deserializer)?;
			Ok(Self {
				typ,
				tag_name: Some(tag_name),
				ids: None
			})
		} else {
			let size = typ.0 - 1;

			let mut ids = Vec::new();

			for _ in 0..size {
				ids.push(VarInt::mc_deserialize(deserializer)?);
			}

			Ok(Self {
				typ,
				tag_name: None,
				ids: Some(ids)
			})
		}
	}
}

/// Used when representing a data record of type T or by reference to a registry.
///
/// [Doc Link](https://minecraft.wiki/w/Java_Edition_protocol/Packets#ID_or_X)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IDorX<T: McSerialize + McDeserialize + Clone + PartialEq> {
	Registry(VarInt),
	Inline(T),
}

impl<T: McSerialize + McDeserialize + Clone + PartialEq> McSerialize for IDorX<T> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			IDorX::Registry(id) => {
				VarInt(id.0 + 1).mc_serialize(serializer)?;
			}
			IDorX::Inline(val) => {
				VarInt(0).mc_serialize(serializer)?;
				val.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl<T: McSerialize + McDeserialize + Clone + PartialEq> McDeserialize for IDorX<T> {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		if typ == 0 {
			Ok(IDorX::Inline(T::mc_deserialize(deserializer)?))
		} else {
			Ok(IDorX::Registry(VarInt(typ - 1)))
		}
	}
}

impl<T: McSerialize + McDeserialize + Clone + PartialEq> McDefault for IDorX<T> {
	fn mc_default() -> Self {
		IDorX::Registry(VarInt(0))
	}
}

/// A rotation angle in steps of 1/256 of a full turn
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Angle {
	pub angle: u8
}

/// Compressed / low-precision 3-component vector (1.21.9+). Used for entity velocity and other
/// small motion vectors. Wire format is 1 byte for zero vectors, 6-7+ bytes otherwise.
#[derive(Debug, Clone, PartialEq)]
pub struct LpVec3 {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

impl McDefault for LpVec3 {
	fn mc_default() -> Self {
		Self { x: 0.0, y: 0.0, z: 0.0 }
	}
}

impl McSerialize for LpVec3 {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		if self.x == 0.0 && self.y == 0.0 && self.z == 0.0 {
			serializer.serialize_u8(0x00);
			return Ok(());
		}

		let scale = self.x.abs().max(self.y.abs()).max(self.z.abs()).ceil() as u32;
		let scale = scale.max(1);

		let x_raw = Self::encode_raw(self.x, scale);
		let y_raw = Self::encode_raw(self.y, scale);
		let z_raw = Self::encode_raw(self.z, scale);

		let scale_low = (scale & 0x03) as u8;
		let scale_high = scale >> 2;
		let continuation: u8 = if scale_high > 0 { 1 } else { 0 };

		let byte0: u8 = scale_low | (continuation << 2) | (((x_raw & 0x1F) as u8) << 3);
		let byte1: u8 = ((x_raw >> 5) & 0xFF) as u8;
		let packed: u32 = (((x_raw >> 13) as u32 & 0x03) << 30)
			| ((y_raw as u32) << 15)
			| (z_raw as u32);

		serializer.serialize_u8(byte0);
		serializer.serialize_u8(byte1);
		packed.mc_serialize(serializer)?;

		if continuation != 0 {
			VarInt(scale_high as i32).mc_serialize(serializer)?;
		}

		Ok(())
	}
}

impl McDeserialize for LpVec3 {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let byte0 = u8::mc_deserialize(deserializer)?;

		if byte0 == 0x00 {
			return Ok(Self { x: 0.0, y: 0.0, z: 0.0 });
		}

		let scale_low = (byte0 & 0x03) as u32;
		let continuation = (byte0 >> 2) & 1;
		let x_low = ((byte0 >> 3) & 0x1F) as u16;

		let byte1 = u8::mc_deserialize(deserializer)?;
		let x_mid = (byte1 as u16) << 5;

		let packed = u32::mc_deserialize(deserializer)?;
		let x_high = ((packed >> 30) & 0x03) as u16;
		let y_raw = ((packed >> 15) & 0x7FFF) as u16;
		let z_raw = (packed & 0x7FFF) as u16;

		let x_raw = x_low | x_mid | (x_high << 13);

		let full_scale = if continuation != 0 {
			let scale_high = VarInt::mc_deserialize(deserializer)?.0 as u32;
			scale_low | (scale_high << 2)
		} else {
			scale_low
		};

		Ok(Self {
			x: Self::from_raw(x_raw, full_scale),
			y: Self::from_raw(y_raw, full_scale),
			z: Self::from_raw(z_raw, full_scale),
		})
	}
}

impl LpVec3 {
	fn encode_raw(component: f64, scale: u32) -> u16 {
		let normalized = component / scale as f64;
		((normalized + 1.0) / 2.0 * 32766.0).round().clamp(0.0, 32766.0) as u16
	}

	fn from_raw(raw: u16, scale: u32) -> f64 {
		let normalized = 2.0 * raw as f64 / 32766.0 - 1.0;
		normalized * scale as f64
	}
}

#[cfg(test)]
mod test {
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
	use crate::protocol_types::datatypes::command::{NodeFlags, NodeType};
	use crate::protocol_types::datatypes::internal_types::{IDSet, PackedEntries};
	use crate::protocol_types::datatypes::var_types::VarInt;

	#[test]
	fn test_idset_tag_round_trip() {
		let id_set = IDSet {
			typ: VarInt(0),
			tag_name: Some("minecraft:planks".to_string()),
			ids: None,
		};
		let mut serializer = McSerializer::new();
		id_set.mc_serialize(&mut serializer).unwrap();
		let bytes: Vec<u8> = serializer.into();
		let mut deserializer = McDeserializer::new(&bytes);
		let result = IDSet::mc_deserialize(&mut deserializer).unwrap();
		assert_eq!(id_set, result);
		assert!(deserializer.is_at_end());
	}

	#[test]
	fn test_idset_inline_round_trip() {
		let id_set = IDSet {
			typ: VarInt(4),
			tag_name: None,
			ids: Some(vec![VarInt(10), VarInt(20), VarInt(30)]),
		};
		let mut serializer = McSerializer::new();
		id_set.mc_serialize(&mut serializer).unwrap();
		let bytes: Vec<u8> = serializer.into();
		let mut deserializer = McDeserializer::new(&bytes);
		let result = IDSet::mc_deserialize(&mut deserializer).unwrap();
		assert_eq!(id_set, result);
		assert!(deserializer.is_at_end());
	}

	#[test]
	fn test_idset_followed_by_data() {
		let id_set = IDSet {
			typ: VarInt(0),
			tag_name: Some("minecraft:stone".to_string()),
			ids: None,
		};
		let mut serializer = McSerializer::new();
		id_set.mc_serialize(&mut serializer).unwrap();
		VarInt(42).mc_serialize(&mut serializer).unwrap();
		let bytes: Vec<u8> = serializer.into();
		let mut deserializer = McDeserializer::new(&bytes);
		let result = IDSet::mc_deserialize(&mut deserializer).unwrap();
		assert_eq!(id_set, result);
		let next = VarInt::mc_deserialize(&mut deserializer).unwrap();
		assert_eq!(next, VarInt(42));
		assert!(deserializer.is_at_end());
	}

	#[test]
	fn test_stonecutter_recipe_round_trip() {
		use crate::protocol::packets::packet_component::StonecutterRecipe;
		use crate::protocol::game::info::inventory::slots::SlotDisplay;

		let recipe = StonecutterRecipe {
			id_set: IDSet {
				typ: VarInt(0),
				tag_name: Some("minecraft:stone_crafting_materials".to_string()),
				ids: None,
			},
			slot_display: SlotDisplay::Item(VarInt(45)),
		};
		let mut serializer = McSerializer::new();
		recipe.mc_serialize(&mut serializer).unwrap();
		let bytes: Vec<u8> = serializer.into();
		let mut deserializer = McDeserializer::new(&bytes);
		let result = StonecutterRecipe::mc_deserialize(&mut deserializer).unwrap();
		assert_eq!(recipe, result);
		assert!(deserializer.is_at_end());
	}

	#[test]
	fn test_stonecutter_recipe_inline_ids_round_trip() {
		use crate::protocol::packets::packet_component::StonecutterRecipe;
		use crate::protocol::game::info::inventory::slots::SlotDisplay;

		let recipe = StonecutterRecipe {
			id_set: IDSet {
				typ: VarInt(5),
				tag_name: None,
				ids: Some(vec![VarInt(10), VarInt(20), VarInt(30), VarInt(40)]),
			},
			slot_display: SlotDisplay::Item(VarInt(100)),
		};
		let mut serializer = McSerializer::new();
		recipe.mc_serialize(&mut serializer).unwrap();
		let bytes: Vec<u8> = serializer.into();
		let mut deserializer = McDeserializer::new(&bytes);
		let result = StonecutterRecipe::mc_deserialize(&mut deserializer).unwrap();
		assert_eq!(recipe, result);
		assert!(deserializer.is_at_end());
	}

	#[test]
	fn basic_packed_entries_test() {
		let mut packed = PackedEntries::new(4);
		packed.set_entry(0, 1);
		packed.set_entry(1, 2);
		packed.set_entry(2, 3);
		packed.set_entry(3, 4);

		assert_eq!(packed.get_entry(0), 1);
		assert_eq!(packed.get_entry(1), 2);
		assert_eq!(packed.get_entry(2), 3);
		assert_eq!(packed.get_entry(3), 4);
	}

	#[test]
	fn extract_from_hex() {
		let packed = PackedEntries {
			data: 0x0020863148418841,
			bpe: 5
		};

		assert_eq!(packed.get_entry(0), 1);
		assert_eq!(packed.get_entry(1), 2);
		assert_eq!(packed.get_entry(2), 2);
		assert_eq!(packed.get_entry(3), 3);
		assert_eq!(packed.get_entry(4), 4);
		assert_eq!(packed.get_entry(5), 4);
		assert_eq!(packed.get_entry(6), 5);
		assert_eq!(packed.get_entry(7), 6);
		assert_eq!(packed.get_entry(8), 6);
		assert_eq!(packed.get_entry(9), 4);
		assert_eq!(packed.get_entry(10), 8);

		let packed = PackedEntries {
			data: 0x01018A7260F68C87,
			bpe: 5
		};

		assert_eq!(packed.get_entry(0), 7);
		assert_eq!(packed.get_entry(1), 4);
		assert_eq!(packed.get_entry(2), 3);
		assert_eq!(packed.get_entry(3), 13);
		assert_eq!(packed.get_entry(4), 15);
		assert_eq!(packed.get_entry(5), 16);
		assert_eq!(packed.get_entry(6), 9);
		assert_eq!(packed.get_entry(7), 14);
		assert_eq!(packed.get_entry(8), 10);
		assert_eq!(packed.get_entry(9), 12);
		assert_eq!(packed.get_entry(10), 0);
		assert_eq!(packed.get_entry(11), 2);
	}

	#[test]
	fn test_node_flags() {
		let flags = NodeFlags {
			typ: NodeType::Argument,
			is_executable: true,
			has_redirect: false,
			has_suggestions: true,
			is_restricted: false
		};

		let byte = flags.to_byte();
		assert_eq!(byte, 0b00010110);

		let deserialized_flags = NodeFlags::from_byte(byte).unwrap();
		assert_eq!(deserialized_flags.typ as u8, flags.typ as u8);
		assert_eq!(deserialized_flags.is_executable, flags.is_executable);
		assert_eq!(deserialized_flags.has_redirect, flags.has_redirect);
		assert_eq!(deserialized_flags.has_suggestions, flags.has_suggestions);
		assert_eq!(deserialized_flags.is_restricted, flags.is_restricted);
	}

	#[test]
	fn test_lpvec3_zero_round_trip() {
		use crate::protocol_types::datatypes::internal_types::LpVec3;

		let vec = LpVec3 { x: 0.0, y: 0.0, z: 0.0 };
		let mut serializer = McSerializer::new();
		vec.mc_serialize(&mut serializer).unwrap();
		let bytes: Vec<u8> = serializer.into();
		assert_eq!(bytes.len(), 1);
		assert_eq!(bytes[0], 0x00);
		let mut deserializer = McDeserializer::new(&bytes);
		let result = LpVec3::mc_deserialize(&mut deserializer).unwrap();
		assert_eq!(result.x, 0.0);
		assert_eq!(result.y, 0.0);
		assert_eq!(result.z, 0.0);
		assert!(deserializer.is_at_end());
	}

	#[test]
	fn test_lpvec3_nonzero_round_trip() {
		use crate::protocol_types::datatypes::internal_types::LpVec3;

		let vec = LpVec3 { x: 0.5, y: -0.3, z: 0.8 };
		let mut serializer = McSerializer::new();
		vec.mc_serialize(&mut serializer).unwrap();
		let bytes: Vec<u8> = serializer.into();
		assert!(bytes.len() >= 6);
		let mut deserializer = McDeserializer::new(&bytes);
		let result = LpVec3::mc_deserialize(&mut deserializer).unwrap();
		assert!((result.x - 0.5).abs() < 0.001, "x: expected ~0.5, got {}", result.x);
		assert!((result.y - (-0.3)).abs() < 0.001, "y: expected ~-0.3, got {}", result.y);
		assert!((result.z - 0.8).abs() < 0.001, "z: expected ~0.8, got {}", result.z);
		assert!(deserializer.is_at_end());
	}

	#[test]
	fn test_lpvec3_large_scale_round_trip() {
		use crate::protocol_types::datatypes::internal_types::LpVec3;

		let vec = LpVec3 { x: 5.0, y: -3.0, z: 8.0 };
		let mut serializer = McSerializer::new();
		vec.mc_serialize(&mut serializer).unwrap();
		let bytes: Vec<u8> = serializer.into();
		assert!(bytes.len() > 6, "large scale should need continuation VarInt");
		let mut deserializer = McDeserializer::new(&bytes);
		let result = LpVec3::mc_deserialize(&mut deserializer).unwrap();
		assert!((result.x - 5.0).abs() < 0.01, "x: expected ~5.0, got {}", result.x);
		assert!((result.y - (-3.0)).abs() < 0.01, "y: expected ~-3.0, got {}", result.y);
		assert!((result.z - 8.0).abs() < 0.01, "z: expected ~8.0, got {}", result.z);
		assert!(deserializer.is_at_end());
	}
}

/// Map a string identifier to a given value of type T.
#[derive(Debug, Clone, PartialEq)]
pub struct Mapping<T> {
	pub key: String,
	pub value: T,
}

impl<T: McSerialize> McSerialize for Mapping<T> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.key.mc_serialize(serializer)?;
		self.value.mc_serialize(serializer)
	}
}

impl<T: McDeserialize> McDeserialize for Mapping<T> {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let key = String::mc_deserialize(deserializer)?;
		let value = T::mc_deserialize(deserializer)?;
		Ok(Self { key, value })
	}
}

impl<T: McDefault> McDefault for Mapping<T> {
	fn mc_default() -> Self {
		Self { key: McDefault::mc_default(), value: T::mc_default() }
	}
}