//! An NBT implementation without support for sNBT (string NBT). See
//! [here](https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/NBT) for more information.

#![allow(clippy::from_over_into)]

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::nbt::nbt_error::NbtError;
use crate::protocol_types::datatypes::nbt::NbtTag::List;
use crate::{list_nbtvalue, primvalue_nbtvalue};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Index;

pub mod nbt_error;
pub mod nbt_utils;
mod snbt_testing;

primvalue_nbtvalue!((i8, Byte), (i16, Short), (i32, Int), (i64, Long), (f32, Float), (f64, Double));

list_nbtvalue!((i8, ByteArray, NbtByteArray, 7), (i32, IntArray, NbtIntArray, 11), (i64, LongArray, NbtLongArray, 12));

/// # NBT Tag (Protocol Type)
/// A tag is a component of an NBT compound/map. Each type of tag represents a different primitive datatype or list type.
/// Also check out [NbtCompound]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum NbtTag {
	/// Used to mark the end of a compound or list
	End,
	Byte(i8),
	Short(i16),
	Int(i32),
	Long(i64),
	Float(f32),
	Double(f64),
	ByteArray(NbtByteArray),
	String(String),
	List(NbtList),
	Compound(NbtCompound),
	IntArray(NbtIntArray),
	LongArray(NbtLongArray),
}

impl NbtTag {
	/// Returns the type ID of the NBT tag, used for serialization and deserialization
	pub fn get_type_id(&self) -> u8 {
		match self {
			NbtTag::End => 0,
			NbtTag::Byte(_) => 1,
			NbtTag::Short(_) => 2,
			NbtTag::Int(_) => 3,
			NbtTag::Long(_) => 4,
			NbtTag::Float(_) => 5,
			NbtTag::Double(_) => 6,
			NbtTag::ByteArray(_) => 7,
			NbtTag::String(_) => 8,
			NbtTag::List(_) => 9,
			NbtTag::Compound(_) => 10,
			NbtTag::IntArray(_) => 11,
			NbtTag::LongArray(_) => 12,
		}
	}

	/// Given the type ID, deserialize the corresponding NbtTag.
	pub fn deserialize_specific<'a>(deserializer: &mut McDeserializer, ty: u8) -> SerializingResult<'a, Self> {
		match ty {
			// Primitives
			0 => Ok(NbtTag::End),
			1 => Ok(NbtTag::Byte(i8::mc_deserialize(deserializer)?)),
			2 => Ok(NbtTag::Short(i16::mc_deserialize(deserializer)?)),
			3 => Ok(NbtTag::Int(i32::mc_deserialize(deserializer)?)),
			4 => Ok(NbtTag::Long(i64::mc_deserialize(deserializer)?)),
			5 => Ok(NbtTag::Float(f32::mc_deserialize(deserializer)?)),
			6 => Ok(NbtTag::Double(f64::mc_deserialize(deserializer)?)),
			8 => {
				// String
				let len = u16::mc_deserialize(deserializer)?;
				let bytes = deserializer.slice(len as usize);

				Ok(NbtTag::String(String::from_utf8_lossy(bytes).to_string()))
			}
			7 => {
				// Byte array
				Ok(NbtTag::ByteArray(NbtByteArray::mc_deserialize(deserializer)?))
			}
			11 => {
				// Int Array
				Ok(NbtTag::IntArray(NbtIntArray::mc_deserialize(deserializer)?))
			}
			12 => {
				// Int Array
				Ok(NbtTag::LongArray(NbtLongArray::mc_deserialize(deserializer)?))
			}
			9 => {
				// List
				Ok(NbtTag::List(NbtList::mc_deserialize(deserializer)?))
			}
			10 => {
				// compound
				Ok(NbtTag::Compound(NbtCompound::from_no_tag(deserializer)?))
			}
			_ => Err(SerializingErr::UniqueFailure("Could not identify tag type".to_string())),
		}
	}
}

impl McSerialize for NbtTag {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		// do not include type id here - list and compound tags will include it themselves
		match self {
			// stuff with special cases
			NbtTag::End => {}
			NbtTag::String(s) => {
				// not the same as regular string serialization (no varint)
				(s.len() as u16).mc_serialize(serializer)?;
				serializer.serialize_bytes(s.as_bytes());
			}
			NbtTag::Byte(i) => {
				serializer.serialize_bytes(i.to_be_bytes().as_slice());
			}
			NbtTag::Short(i) => {
				serializer.serialize_bytes(i.to_be_bytes().as_slice());
			}
			NbtTag::Int(i) => {
				serializer.serialize_bytes(i.to_be_bytes().as_slice());
			}
			NbtTag::Long(i) => {
				serializer.serialize_bytes(i.to_be_bytes().as_slice());
			}
			NbtTag::Float(f) => {
				serializer.serialize_bytes(f.to_be_bytes().as_slice());
			}
			NbtTag::Double(f) => {
				serializer.serialize_bytes(f.to_be_bytes().as_slice());
			}
			NbtTag::ByteArray(b) => b.mc_serialize(serializer)?,
			NbtTag::IntArray(b) => b.mc_serialize(serializer)?,
			NbtTag::LongArray(b) => b.mc_serialize(serializer)?,
			NbtTag::List(b) => b.mc_serialize(serializer)?,
			NbtTag::Compound(c) => c.serialize_no_tag(serializer)?,
		}

		Ok(())
	}
}

impl McDeserialize for NbtTag {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, NbtTag> {
		let ty = u8::mc_deserialize(deserializer)?;

		NbtTag::deserialize_specific(deserializer, ty)
	}
}

impl McDefault for NbtTag {
	fn mc_default() -> Self {
		NbtTag::String("default".to_string())
	}
}

impl From<&str> for NbtTag {
	fn from(value: &str) -> Self {
		NbtTag::String(value.to_string())
	}
}

impl From<String> for NbtTag {
	fn from(value: String) -> Self {
		NbtTag::String(value)
	}
}

impl TryFrom<NbtTag> for String {
	type Error = NbtError;

	fn try_from(value: NbtTag) -> Result<Self, Self::Error> {
		match value {
			NbtTag::String(s) => Ok(s),
			_ => Err(NbtError::InvalidType),
		}
	}
}

impl From<bool> for NbtTag {
	fn from(value: bool) -> Self {
		NbtTag::Byte(if value {
			1
		} else {
			0
		})
	}
}

impl TryFrom<NbtTag> for bool {
	type Error = NbtError;

	fn try_from(value: NbtTag) -> Result<Self, Self::Error> {
		match value {
			NbtTag::Byte(b) => Ok(b != 0),
			_ => Err(NbtError::InvalidType),
		}
	}
}

// this is replicated for the other types in the primvalue_nbtvalue! macro
impl From<NbtTag> for Option<String> {
	fn from(value: NbtTag) -> Self {
		match value {
			NbtTag::String(s) => Some(s),
			_ => None, // any other type is not convertible to Option<String>
		}
	}
}

impl<T: Into<NbtTag>> From<Vec<T>> for NbtTag {
	fn from(value: Vec<T>) -> Self {
		let tags: Vec<NbtTag> = value.into_iter().map(Into::into).collect();
		match NbtList::from_vec(tags) {
			Ok(list) => List(list),
			// Fall back to an empty list if mixed types are in a list
			Err(_) => List(NbtList::new()),
		}
	}
}

impl<T: TryFrom<NbtTag>> TryFrom<NbtTag> for Vec<T>
where
	NbtError: From<<T as TryFrom<NbtTag>>::Error>,
{
	type Error = NbtError;

	fn try_from(value: NbtTag) -> Result<Self, Self::Error> {
		match value {
			NbtTag::ByteArray(list) => {
				let mut vec = vec![];
				for b in list.list {
					vec.push(T::try_from(NbtTag::Byte(b))?);
				}
				Ok(vec)
			}
			List(list) => {
				let mut vec = vec![];
				for tag in list.list {
					if tag == NbtTag::End {
						continue;
					}
					vec.push(T::try_from(tag)?);
				}
				Ok(vec)
			}
			NbtTag::IntArray(list) => {
				let mut vec = vec![];
				for i in list.list {
					vec.push(T::try_from(NbtTag::Int(i))?);
				}
				Ok(vec)
			}
			NbtTag::LongArray(list) => {
				let mut vec = vec![];
				for l in list.list {
					vec.push(T::try_from(NbtTag::Long(l))?);
				}
				Ok(vec)
			}
			_ => Err(NbtError::InvalidType),
		}
	}
}

/// # NBT Compound (Protocol Type)
/// This is the NBT type referred to and used by the protocol everywhere.
/// A compound is a map of String keys to NBT Tags.
///
/// Note that network compounds don't have a root name. To create a network compound, use [NbtCompound::new_no_name()]
///
/// https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/NBT
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NbtCompound {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub root_name: Option<String>,
	pub(crate) map: HashMap<String, NbtTag>,
}

impl NbtCompound {
	/// Root name is only present if it is not the root compound of network NBT (1.20.2+).
	pub fn new<T: Into<String>>(root_name: Option<T>) -> Self {
		let option: Option<String> = root_name.map(|root_name| root_name.into());

		Self {
			map: HashMap::new(),
			root_name: option,
		}
	}

	/// Create a new NbtCompound without a root name, usually for network nbt or a compound inside a compound.
	pub fn new_no_name() -> Self {
		Self::new::<String>(None)
	}

	/// Add a tag to the compound with a String key.
	#[inline]
	pub fn add<K: Into<String>, V: Into<NbtTag>>(&mut self, name: K, tag: V) {
		let tag = tag.into();

		if tag == NbtTag::End {
			return; // do not add End tag
		}

		self.map.insert(name.into(), tag);
	}

	/// Move every entry of `other` into this compound, overwriting any existing entries with the
	/// same name. Used to flatten a nested compound's fields into a parent (e.g. `#[nbt(flatten)]`).
	pub fn merge(&mut self, other: NbtCompound) {
		for (name, tag) in other.map {
			self.add(name, tag);
		}
	}

	/// Get the tag mapped to the given name. Returns 'None' if the name does not exist in the compound.
	pub fn get<T: Into<String>>(&self, name: T) -> Option<&NbtTag> {
		self.map.get(&name.into())
	}

	/// Remove a NBT Tag from the compound map
	///
	/// ## Parameters
	/// - `name` = The String key of the NBT Tag that needs to be removed
	#[inline]
	pub fn remove<T: Into<String>>(&mut self, name: T) {
		self.map.remove(&name.into());
	}

	/// Deserialize compounds with a root name
	pub fn from_root<'a>(deserializer: &mut McDeserializer) -> SerializingResult<'a, NbtCompound> {
		let t = u8::mc_deserialize(deserializer)?;

		if t != 10 {
			return Err(SerializingErr::UniqueFailure(format!("Expected compound tag id, got {t} instead")));
		}

		let name_length = u16::mc_deserialize(deserializer)?;
		let name = String::from_utf8_lossy(deserializer.slice(name_length as usize)).to_string();
		let mut compound = NbtCompound::new(Some(name));

		loop {
			let tag = deserializer.pop();

			if tag.is_none() || tag.unwrap() == 0 {
				// END Tag
				break;
			}

			let name_length = u16::mc_deserialize(deserializer)?;
			let name = String::from_utf8_lossy(deserializer.slice(name_length as usize)).to_string();

			let tag = NbtTag::deserialize_specific(deserializer, tag.unwrap())?;
			compound.add(name, tag);
		}

		Ok(compound)
	}

	/// Deserialize a compound's contents immediately without looking for a root name or tag id.
	fn from_no_tag<'a>(deserializer: &mut McDeserializer) -> SerializingResult<'a, NbtCompound> {
		let mut compound = NbtCompound::new_no_name();

		loop {
			let tag = deserializer.pop();

			if tag.is_none() || tag.unwrap() == 0 {
				// END Tag
				break;
			}

			let name_length = u16::mc_deserialize(deserializer)?;
			let name = String::from_utf8_lossy(deserializer.slice(name_length as usize)).to_string();

			let tag = NbtTag::deserialize_specific(deserializer, tag.unwrap())?;
			compound.add(name, tag);
		}

		Ok(compound)
	}

	/// Serialize the compound without a leading tag byte.
	fn serialize_no_tag(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		// only serialize root name if present (non-network compound tag or pre 1.20.2)
		if let Some(root_name) = &self.root_name {
			(root_name.len() as u16).mc_serialize(serializer)?;
			serializer.serialize_bytes(root_name.as_bytes());
		}

		self.serialize_tags(serializer)?;
		Ok(())
	}

	/// Serialize only the tags mapped inside of a compound.
	fn serialize_tags(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		for (name, tag) in self.map.iter() {
			// End marks the end of the compound so cancel iter here if it is End
			if *tag != NbtTag::End {
				serializer.serialize_u8(tag.get_type_id());
				(name.len() as u16).mc_serialize(serializer)?;
				serializer.serialize_bytes(name.as_bytes());
				tag.mc_serialize(serializer)?;
			}
		}
		serializer.serialize_u8(0); // end tag
		Ok(())
	}

	/// Get the boolean value of a tag, where 0 is false and any other value is true. Returns 'None' if the tag is not a byte or the tag is not present.
	pub fn get_bool<T: Into<String>>(&self, name: T) -> Option<bool> {
		match self.get(name) {
			Some(NbtTag::Byte(b)) => Some(*b != 0),
			_ => None,
		}
	}

	/// Get the string value of a tag. Returns 'None' if the tag is not a string or the tag is not present.
	pub fn get_string<T: Into<String>>(&self, name: T) -> Option<String> {
		match self.get(name) {
			Some(NbtTag::String(s)) => Some(s.clone()),
			_ => None,
		}
	}
}

impl Index<&str> for NbtCompound {
	type Output = NbtTag;

	/// Returns a reference to the value inside of the HashMap mapped to the given key.
	/// Returns `NbtTag::End` (which `add` never stores) as an absent-key sentinel if the key does
	/// not exist; use [NbtCompound::get] for an `Option` instead.
	fn index(&self, index: &str) -> &Self::Output {
		if !self.map.contains_key(index) {
			return &NbtTag::End;
		}

		&self.map[index]
	}
}

impl McSerialize for NbtCompound {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		if self.map.is_empty() && self.root_name.is_none() {
			0u8.mc_serialize(serializer)?;
			return Ok(());
		}
		10u8.mc_serialize(serializer)?;

		self.serialize_no_tag(serializer)
	}
}

impl McDeserialize for NbtCompound {
	/// Deserialize a compound without a root name, such as network NBT or compounds in compounds.
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let t = u8::mc_deserialize(deserializer)?;

		if t == 0 {
			return Ok(Self::new_no_name());
		}

		if t != 10 {
			debug!("Nearby bytes: {:?}", deserializer.subset(50, 15));
			return Err(SerializingErr::UniqueFailure(format!("Expected compound tag id, got {t} instead")));
		}

		Self::from_no_tag(deserializer)
	}
}

impl McDefault for NbtCompound {
	fn mc_default() -> Self {
		let mut compound = NbtCompound::new_no_name();

		compound.add("default_string", "default_value");
		compound.add("default_int", 42);
		compound.add("default_float", 3.19f32);

		compound
	}
}

impl Into<NbtTag> for NbtCompound {
	fn into(self) -> NbtTag {
		NbtTag::Compound(self)
	}
}

impl TryFrom<NbtTag> for NbtCompound {
	type Error = NbtError;

	fn try_from(tag: NbtTag) -> Result<Self, Self::Error> {
		match tag {
			NbtTag::Compound(c) => Ok(c),
			_ => Err(NbtError::InvalidType),
		}
	}
}

impl From<NbtTag> for Option<NbtCompound> {
	fn from(tag: NbtTag) -> Self {
		match tag {
			NbtTag::Compound(c) => Some(c),
			_ => None,
		}
	}
}

impl<T> From<Box<T>> for NbtCompound
where
	T: Into<NbtCompound>,
{
	fn from(boxed: Box<T>) -> Self {
		(*boxed).into()
	}
}

impl PartialEq for NbtCompound {
	fn eq(&self, other: &Self) -> bool {
		self.map == other.map && self.root_name == other.root_name
	}
}

impl Eq for NbtCompound {}

/// A list of NbtTags held sequentially.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct NbtList {
	pub type_id: u8,
	pub list: Vec<NbtTag>,
}

impl NbtList {
	pub fn new() -> Self {
		Self {
			type_id: 0, // set to END by default
			list: vec![],
		}
	}

	pub fn from_vec(vec: Vec<NbtTag>) -> Result<Self, NbtError> {
		let mut list = NbtList::new();

		for tag in vec {
			list.add_tag(tag)?;
		}

		Ok(list)
	}

	#[inline]
	pub fn add<T: Into<NbtTag>>(&mut self, tag: T) -> Result<(), NbtError> {
		let tag = tag.into();

		if tag.get_type_id() == 0 {
			return Err(NbtError::EndTagNotAllowedInList);
		}

		self.add_tag(tag)
	}

	#[inline]
	pub fn add_tag(&mut self, tag: NbtTag) -> Result<(), NbtError> {
		if self.type_id == 0 {
			self.type_id = tag.get_type_id();
		} else if tag.get_type_id() != self.type_id {
			return Err(NbtError::IncompatibleTypes);
		}

		self.list.push(tag);

		Ok(())
	}
}

impl IntoIterator for NbtList {
	type Item = NbtTag;
	type IntoIter = std::vec::IntoIter<NbtTag>;

	fn into_iter(self) -> Self::IntoIter {
		self.list.into_iter()
	}
}

impl<'a> IntoIterator for &'a NbtList {
	type Item = &'a NbtTag;
	type IntoIter = std::slice::Iter<'a, NbtTag>;

	fn into_iter(self) -> Self::IntoIter {
		self.list.iter()
	}
}

impl McSerialize for NbtList {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.type_id.mc_serialize(serializer)?;
		(self.list.len() as i32).mc_serialize(serializer)?;
		for tag in &self.list {
			tag.mc_serialize(serializer)?;
		}
		Ok(())
	}
}

impl McDeserialize for NbtList {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, NbtList> {
		let t = u8::mc_deserialize(deserializer)?;
		let length = i32::mc_deserialize(deserializer)?;

		if t == 0 && length > 0 {
			return Err(SerializingErr::UniqueFailure("Type cannot be END when length is positive".to_string()));
		}

		let mut list = NbtList::new();

		for _ in 0..length {
			let tag = NbtTag::deserialize_specific(deserializer, t)?;

			if tag.get_type_id() != t {
				return Err(SerializingErr::UniqueFailure("Type must be the same as the type for the list".to_string()));
			}

			if list.add_tag(tag).is_err() {
				return Err(SerializingErr::UniqueFailure("Could not push tag to list".to_string()));
			}
		}

		Ok(list)
	}
}

impl TryFrom<NbtTag> for NbtList {
	type Error = NbtError;

	fn try_from(tag: NbtTag) -> Result<Self, Self::Error> {
		match tag {
			List(list) => Ok(list),
			_ => Err(NbtError::InvalidType),
		}
	}
}

impl Into<NbtTag> for NbtList {
	fn into(self) -> NbtTag {
		List(self)
	}
}

impl Default for NbtList {
	fn default() -> Self {
		NbtList::new()
	}
}

impl McDefault for NbtList {
	fn mc_default() -> Self {
		let mut list = NbtList::new();

		list.add_tag(NbtTag::mc_default()).expect("Mixed types in NbtList default");

		list
	}
}

#[cfg(test)]
mod test {
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
	use crate::protocol_types::datatypes::nbt::NbtError;
	use crate::protocol_types::datatypes::nbt::{NbtByteArray, NbtCompound, NbtIntArray, NbtList, NbtLongArray, NbtTag};
	use sandstone_derive::{AsNbt, FromNbt};

	/// Test standard serialization of a NbtCompound.
	#[test]
	fn test_compound_serialization() {
		let mut compound = NbtCompound::new(Some("A"));
		compound.add("i8", 123i8);
		compound.add("i16", 1234i16);
		compound.add("i32", 12345i32);
		compound.add("f32", -3.6f32);
		compound.add("f64", -3.6789f64);
		compound.add("str", "hello");
		compound.add("byte_array", NbtByteArray::new(vec![1, 2, 3, 4, 5]));
		compound.add("int_array", NbtIntArray::new(vec![1, 2, 3, 4, 5]));
		compound.add("long_array", NbtLongArray::new(vec![1, 2, 3, 4, 5]));
		compound.add("list", NbtList::from_vec(vec![NbtTag::Int(1), NbtTag::Int(2), NbtTag::Int(3)]).unwrap());

		let mut compound2 = NbtCompound::new::<String>(None);
		compound2.add("byte", 13i8);
		compound.add("compound", compound2);

		let mut serializer = McSerializer::new();
		compound.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = McDeserializer::new(&serializer.output);
		let deserialized = NbtCompound::from_root(&mut deserializer).unwrap();

		assert_eq!(compound["i8"], deserialized["i8"]);
		assert_eq!(compound["i16"], deserialized["i16"]);
		assert_eq!(compound["i32"], deserialized["i32"]);
		assert_eq!(compound["f32"], deserialized["f32"]);
		assert_eq!(compound["f64"], deserialized["f64"]);
		assert_eq!(compound["str"], deserialized["str"]);
		assert_eq!(compound["byte_array"], deserialized["byte_array"]);
		assert_eq!(compound["int_array"], deserialized["int_array"]);
		assert_eq!(compound["long_array"], deserialized["long_array"]);
		assert_eq!(compound["list"], deserialized["list"]);

		assert_eq!(compound["compound"], deserialized["compound"]);

		assert_eq!(compound.root_name, deserialized.root_name);
	}

	/// Test compounds within compounds and their serialization.
	#[test]
	fn test_compounds_in_compounds() {
		let mut outer = NbtCompound::new(Some("outer"));
		let mut mid1 = NbtCompound::new::<String>(None);
		let mut mid2 = NbtCompound::new::<String>(None);
		let mut inner1 = NbtCompound::new::<String>(None);
		let mut inner2 = NbtCompound::new::<String>(None);
		let mut inner3 = NbtCompound::new::<String>(None);

		inner1.add("i8", 123i8);
		inner1.add("i16", 1234i16);

		inner2.add("i8", 123i8);
		inner2.add("str", "hello");

		inner3.add("byte_list", NbtByteArray::new(vec![8, 3, 9, 0, 2, 1]));
		inner3.add("int_list", NbtIntArray::new(vec![97197, 288, -28238, -89, 8373]));

		mid1.add("inner1", inner1);
		mid1.add("i32", 12345i32);

		mid2.add("inner2", inner2);
		mid2.add("i32", 187585i32);
		mid2.add("inner3", inner3);
		mid2.add("long_list", NbtLongArray::new(vec![8766897606966, 78287698760875, 786876876732, 125435138536]));

		outer.add("mid1", mid1);
		outer.add("mid2", mid2);

		let mut serializer = McSerializer::new();
		outer.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = McDeserializer::new(&serializer.output);
		let deserialized = NbtCompound::from_root(&mut deserializer).unwrap();

		assert_eq!(outer, deserialized);
		assert_eq!(outer["mid1"], deserialized["mid1"]);
		assert_eq!(outer["mid2"], deserialized["mid2"]);
	}

	/// Test network compound deserialization.
	#[test]
	fn test_network() {
		let mut compound = NbtCompound::new_no_name();
		compound.add("i8", 123i8);
		compound.add("i16", 1234i16);
		compound.add("i32", 12345i32);
		compound.add("f32", -3.6f32);
		compound.add("f64", -3.6789f64);
		compound.add("str", "hello");

		let mut serializer = McSerializer::new();
		compound.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = McDeserializer::new(&serializer.output);
		let deserialized = NbtCompound::mc_deserialize(&mut deserializer).unwrap();

		assert_eq!(deserialized["i8"], NbtTag::Byte(123i8));
		assert_eq!(deserialized["i16"], NbtTag::Short(1234i16));
		assert_eq!(deserialized["i32"], NbtTag::Int(12345i32));
		assert_eq!(deserialized["f32"], NbtTag::Float(-3.6f32));
		assert_eq!(deserialized["f64"], NbtTag::Double(-3.6789f64));
		assert_eq!(deserialized["str"], NbtTag::String("hello".to_string()));
	}

	/// With `NbtTag::None` removed, absence is modeled by key-absence rather than a sentinel value.
	/// A missing key must not be in the map, `get` must return `None`, and `Index` returns the `End`
	/// absent-key sentinel. Crucially, an absent key must round-trip as still-absent.
	#[test]
	fn test_absent_key() {
		// No root name: `NbtTag::mc_deserialize` reads a network (unnamed) root compound.
		let mut compound = NbtCompound::new_no_name();
		compound.add("present", 1i32);

		assert!(compound.get("absent").is_none());
		assert_eq!(compound["absent"], NbtTag::End); // absent-key sentinel
		assert_eq!(compound["present"], NbtTag::Int(1));

		let mut serializer = McSerializer::new();
		compound.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = McDeserializer::new(&serializer.output);
		match NbtTag::mc_deserialize(&mut deserializer) {
			Ok(NbtTag::Compound(c)) => {
				assert!(c.get("absent").is_none());
				assert_eq!(c["present"], NbtTag::Int(1));
			}
			_ => panic!("Expected NbtTag::Compound"),
		}
	}

	/// Test struct for conversion to/from NbtCompound.
	#[derive(AsNbt, FromNbt, Debug, PartialEq)]
	struct TestStruct {
		a: i32,
		b: String,
		c: f64,
	}

	/// Confirm that the `as_nbt` and `from_nbt` methods work the same.
	#[test]
	fn test_into_vs_as() {
		let test = TestStruct {
			a: 42,
			b: "Hello".to_string(),
			c: 3.19,
		};

		let as_test = test.as_nbt();
		let nbt: NbtCompound = test.into();

		assert_eq!(as_test, nbt);
	}

	/// Test that a struct can be converted into an NbtCompound using the `as_nbt` function.
	#[test]
	fn test_as_nbt() {
		let test = TestStruct {
			a: 42,
			b: "Hello".to_string(),
			c: 3.19,
		};

		let nbt: NbtCompound = test.into();

		assert_eq!(nbt["a"], NbtTag::Int(42));
		assert_eq!(nbt["b"], NbtTag::String("Hello".to_string()));
		assert_eq!(nbt["c"], NbtTag::Double(3.19));
	}

	/// Test that an NbtCompound can be converted into a struct using the `from_nbt` function.
	#[test]
	fn test_from_nbt() {
		let mut nbt = NbtCompound::new(Some("Test"));

		nbt.add("a", NbtTag::Int(42));
		nbt.add("b", NbtTag::String("Hello".to_string()));
		nbt.add("c", NbtTag::Double(3.19));

		let test: TestStruct = nbt.try_into().unwrap();

		assert_eq!(test.a, 42);
		assert_eq!(test.b, "Hello");
		assert_eq!(test.c, 3.19);
	}

	#[derive(AsNbt, FromNbt, Debug, PartialEq, Clone)]
	struct OptionTestStruct {
		a: i32,
		b: Option<String>,
		c: Option<f64>,
	}

	/// Test struct with None values to and from NBT.
	#[test]
	fn test_simple_none_nbt() {
		let test = OptionTestStruct {
			a: 0,
			b: None,
			c: None,
		};

		let nbt: NbtCompound = test.clone().into();
		assert_eq!(nbt["a"], NbtTag::Int(0));
		assert!(nbt.get("b").is_none()); // None Option fields are simply absent from the compound
		assert!(nbt.get("c").is_none());

		let test2: OptionTestStruct = nbt.try_into().unwrap();
		assert_eq!(test, test2);
	}

	/// Test for struct with Option fields to and from NBT
	#[test]
	fn test_simple_option_nbt() {
		let test = OptionTestStruct {
			a: 0,
			b: Some("Hello".to_string()),
			c: Some(2.6),
		};

		let nbt: NbtCompound = test.clone().into();
		assert_eq!(nbt["a"], NbtTag::Int(0));
		assert_eq!(nbt["b"], NbtTag::String("Hello".to_string()));
		assert_eq!(nbt["c"], NbtTag::Double(2.6));

		let test2: OptionTestStruct = nbt.try_into().unwrap();
		assert_eq!(test, test2);
	}

	/// Test how bytes are deserialized directly from NbtTag. Previously, the NbtTag and NbtCompound would both
	/// try to pull the compound id (10) and then deserialize the tags, which would cause an off by-one error.
	#[test]
	fn test_deserialize_compound_via_nbttag() {
		let mut deserializer = McDeserializer::new(&[10, 0, 11, 100, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110, 8, 0, 9, 116, 114, 97, 110, 115, 108, 97, 116, 101, 0]);
		NbtTag::mc_deserialize(&mut deserializer).expect("Failed to deserialize NBT from bytes");

		let mut compound = NbtCompound::new_no_name();
		compound.add("translate", "translate".to_string());
		let mut compound2 = NbtCompound::new_no_name();
		compound2.add("some_key", "some_value".to_string());
		compound.add("compound", compound2);

		let mut serializer = McSerializer::new();
		compound.mc_serialize(&mut serializer).expect("Failed to serialize NBT to bytes");
		let mut deserializer = McDeserializer::new(&serializer.output);
		// NbtTag assumes that the root compound has no name
		let deserialized = NbtTag::mc_deserialize(&mut deserializer).expect("Failed to deserialize NBT from bytes");
		assert_eq!(deserialized, NbtTag::Compound(compound));
	}

	/// Test that NbtList can be converted to and from Vec<T> types.
	#[test]
	fn test_list_conversions() {
		let v = vec![1i8, 2, 3, 4, 5];
		let nbt_list = NbtTag::from(v.clone());
		let deserialized: Vec<i8> = nbt_list.try_into().unwrap();
		assert_eq!(deserialized, v);

		let v = vec![13, 42, 99];
		let nbt_list = NbtTag::from(v.clone());
		let deserialized: Vec<i32> = nbt_list.try_into().unwrap();
		assert_eq!(deserialized, v);

		let v = vec![1i64, 2, 3, 4, 5];
		let nbt_list = NbtTag::from(v.clone());
		let deserialized: Vec<i64> = nbt_list.try_into().unwrap();
		assert_eq!(deserialized, v);

		let v = vec![
			ListTestStruct {
				i: 1,
				str: "one".to_string(),
			},
			ListTestStruct {
				i: 2,
				str: "two".to_string(),
			},
		];
		let nbt_list = NbtTag::from(v.clone());
		let deserialized: Vec<ListTestStruct> = nbt_list.try_into().unwrap();
		assert_eq!(deserialized, v);
	}

	#[derive(FromNbt, AsNbt, Debug, PartialEq, Clone)]
	pub struct ListTestStruct {
		i: i32,
		str: String,
	}

	#[derive(AsNbt, FromNbt, Debug, PartialEq, Clone)]
	struct NoticeVariant {
		message: String,
	}

	#[derive(AsNbt, FromNbt, Debug, PartialEq, Clone)]
	struct ConfirmVariant {
		yes: String,
		no: String,
	}

	/// An internally-tagged enum: each variant is written as a compound with a discriminant entry
	/// (here under the `kind` key) alongside the variant's own fields.
	#[derive(AsNbt, FromNbt, Debug, PartialEq, Clone)]
	#[nbt(tag = "kind")]
	enum TaggedEnum {
		#[nbt(rename = "minecraft:notice")]
		Notice(NoticeVariant),
		#[nbt(rename = "minecraft:confirmation")]
		Confirm(ConfirmVariant),
	}

	/// The derived enum support must write the discriminant under the configured tag key and read it
	/// back to the same variant. If the tag were dropped or mis-keyed, a receiver could not tell the
	/// variants apart — the whole point of an internally-tagged union.
	#[test]
	fn test_enum_internally_tagged_round_trip() {
		let notice = TaggedEnum::Notice(NoticeVariant {
			message: "hello".to_string(),
		});
		let nbt: NbtCompound = notice.clone().into();
		assert_eq!(nbt["kind"], NbtTag::String("minecraft:notice".to_string()));
		assert_eq!(nbt["message"], NbtTag::String("hello".to_string()));
		assert_eq!(TaggedEnum::try_from(nbt).unwrap(), notice);

		let confirm = TaggedEnum::Confirm(ConfirmVariant {
			yes: "y".to_string(),
			no: "n".to_string(),
		});
		let nbt: NbtCompound = confirm.clone().into();
		assert_eq!(nbt["kind"], NbtTag::String("minecraft:confirmation".to_string()));
		assert_eq!(TaggedEnum::try_from(nbt).unwrap(), confirm);
	}

	#[derive(AsNbt, FromNbt, Debug, PartialEq, Clone)]
	struct FlattenParent {
		title: String,
		#[nbt(flatten)]
		inner: TaggedEnum,
	}

	/// A `#[nbt(flatten)]` field merges its compound into the parent rather than nesting it. The
	/// flattened enum's discriminant and fields must sit at the same level as the parent's own
	/// fields, and survive the round trip, since that flat layout is what the wire format requires.
	#[test]
	fn test_flatten_round_trip() {
		let parent = FlattenParent {
			title: "t".to_string(),
			inner: TaggedEnum::Notice(NoticeVariant {
				message: "m".to_string(),
			}),
		};
		let nbt: NbtCompound = parent.clone().into();
		assert_eq!(nbt["title"], NbtTag::String("t".to_string()));
		assert_eq!(nbt["kind"], NbtTag::String("minecraft:notice".to_string()));
		assert_eq!(nbt["message"], NbtTag::String("m".to_string()));
		assert_eq!(FlattenParent::try_from(nbt).unwrap(), parent);
	}

	/// Tests that NbtCompound can be serialized to and deserialized from JSON.
	#[test]
	fn test_nbt_json() {
		let mut compound = NbtCompound::new(Some("TestCompound"));
		compound.add("i8", 123i8);
		compound.add("i16", 1234i16);
		compound.add("bool", true);
		compound.add("str", "hello".to_string());
		let json = serde_json::to_string(&compound).expect("Failed to serialize NBT to JSON");
		let deserialized: NbtCompound = serde_json::from_str(&json).expect("Failed to deserialize NBT from JSON");
		assert_eq!(compound, deserialized);
	}
}
