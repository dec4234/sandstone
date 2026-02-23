use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::mc;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};

/// Represents a packed i64 (long) that contains block or biome data. See
/// https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format#Data_Array_format for more info. This
/// matches the spec for packed data after 1.16
#[derive(McDefault, Debug, Clone, Hash, PartialEq)]
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

/// A Node used for representing graphs
#[derive(McSerialize, McDeserialize, McDefault, Debug, Clone, Hash, PartialEq)]
pub struct Node {
	pub flags: NodeFlags,
	pub children_count: VarInt,
	pub children: Vec<VarInt>,
	pub redirect_node: Option<VarInt>,
	pub name: Option<String>,
	pub parser_id: Option<VarInt>,
	pub properties: Option<String>, // todo: ambiguity in spec
	#[mc(deserialize_if = flags.has_suggestions)]
	pub suggestions: Option<String>
}

/// Internal node flags represented as a byte with masking
#[derive(McDefault, Debug, Clone, Hash, PartialEq)]
pub struct NodeFlags {
	pub typ: NodeType,
	pub is_executable: bool,
	pub has_redirect: bool,
	pub has_suggestions: bool,
	pub is_restricted: bool
}

impl NodeFlags {
	pub fn from_byte<'a>(byte: u8) -> SerializingResult<'a, NodeFlags> {
		Ok(Self {
			typ: match byte & 0x03 {
				0 => NodeType::Root,
				1 => NodeType::Literal,
				2 => NodeType::Argument,
				_ => panic!("Invalid node type in flags byte")
			},
			is_executable: (byte & 0x04) != 0,
			has_redirect: (byte & 0x08) != 0,
			has_suggestions: (byte & 0x10) != 0,
			is_restricted: (byte & 0x20) != 0
		})
	}

	pub fn to_byte(&self) -> u8 {
		let mut byte = match self.typ {
			NodeType::Root => 0,
			NodeType::Literal => 1,
			NodeType::Argument => 2
		};
		if self.is_executable {
			byte |= 0x04;
		}
		if self.has_redirect {
			byte |= 0x08;
		}
		if self.has_suggestions {
			byte |= 0x10;
		}
		if self.is_restricted {
			byte |= 0x20;
		}
		byte
	}
}

impl McSerialize for NodeFlags {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.to_byte().mc_serialize(serializer)
	}
}

impl McDeserialize for NodeFlags {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let byte = u8::mc_deserialize(deserializer)?;
		Self::from_byte(byte)
	}
}

/// Type of node in a graph
#[derive(McDefault, Debug, Clone, Hash, PartialEq)]
pub enum NodeType {
	Root = 0,
	Literal = 1,
	Argument = 2,
}

#[derive(McDefault, Debug, Clone, Hash, PartialEq)]
pub struct IDSet {
	pub typ: VarInt,
	pub tag_name: Option<String>,
	pub ids: Option<Vec<VarInt>>
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

			if let Some(ids) = &self.ids {
				ids.mc_serialize(serializer)?;
			} else {
				return Err(SerializingErr::MissingField("IDSet with type 0 must have an ID list".to_string()));
			}
		}
		Ok(())
	}
}

impl McDeserialize for IDSet {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?;
		if typ.0 == 0 {
			let tag_name = String::mc_deserialize(deserializer)?;
			let ids = Vec::<VarInt>::mc_deserialize(deserializer)?;
			Ok(Self {
				typ,
				tag_name: Some(tag_name),
				ids: Some(ids)
			})
		} else {
			Ok(Self {
				typ,
				tag_name: None,
				ids: None
			})
		}
	}
}

#[cfg(test)]
mod test {
	use crate::protocol_types::datatypes::internal_types::{NodeFlags, NodeType, PackedEntries};

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