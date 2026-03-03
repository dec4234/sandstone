use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::McDefault;

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

#[cfg(test)]
mod test {
	use crate::protocol_types::datatypes::command::{NodeFlags, NodeType};
	use crate::protocol_types::datatypes::internal_types::PackedEntries;

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
}