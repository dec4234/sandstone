//! An NBT implementation without support for sNBT (string NBT). See
//! [here](https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/NBT) for more information.

#![allow(clippy::from_over_into)]

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::nbt::NbtTag::List;
use crate::protocol_types::datatypes::nbt::nbt_error::NbtError;
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

/// Serialize a string using Java's UTF format prefixed with a u16 of the length.
fn serialize_nbt_string<'a>(serializer: &mut McSerializer, s: &str) -> SerializingResult<'a, ()> {
	let encoded = cesu8::to_java_cesu8(s);
	(encoded.len() as u16).mc_serialize(serializer)?;
	serializer.serialize_bytes(&encoded);
	Ok(())
}

/// Deserialize a string written by [serialize_nbt_string]: a `u16` byte length followed by that many
/// bytes of Java modified UTF-8.
fn deserialize_nbt_string<'a>(deserializer: &mut McDeserializer) -> SerializingResult<'a, String> {
	let len = u16::mc_deserialize(deserializer)?;
	let bytes = deserializer.slice(len as usize);
	let decoded = cesu8::from_java_cesu8(bytes).map_err(|e| SerializingErr::UniqueFailure(format!("Invalid modified UTF-8 NBT string: {e}")))?;
	Ok(decoded.into_owned())
}

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

	/// Remove the `{"": item}` wrapper that `NbtList::from_vec_wrapping` (and vanilla) puts on an
	/// item of a list with mixed types. Any other tag is returned unchanged.
	pub fn unwrap_list_item(self) -> NbtTag {
		match self {
			NbtTag::Compound(mut compound) if compound.is_list_wrapper() => compound.map.remove("").expect("a list wrapper has the empty key"),
			other => other,
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
			8 => Ok(NbtTag::String(deserialize_nbt_string(deserializer)?)),
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
	/// Writes the type id, then the payload. This is network NBT with no root name, and it is the
	/// same format that `mc_deserialize` reads.
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		serializer.serialize_u8(self.get_type_id());
		self.serialize_payload(serializer)
	}
}

impl NbtTag {
	/// Writes only the payload, with no type id. Lists and compounds write the type id themselves.
	pub(crate) fn serialize_payload(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			// stuff with special cases
			NbtTag::End => {}
			NbtTag::String(s) => {
				serialize_nbt_string(serializer, s)?;
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
		// End means "no value" in this crate: `NbtCompound::add` skips it and `TryFrom<NbtTag> for Vec<T>`
		// skips it. A list cannot hold End, so the End items are skipped here too, and the other items stay.
		let tags: Vec<NbtTag> = value.into_iter().map(Into::into).filter(|tag| *tag != NbtTag::End).collect();
		List(NbtList::from_vec_wrapping(tags).expect("End items are removed, and mixed types are wrapped"))
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
					vec.push(T::try_from(tag.unwrap_list_item())?);
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

	/// True if this compound has only the empty key `""`. Vanilla uses this shape to hold one item
	/// of a list with mixed item types. See `NbtList::from_vec_wrapping`.
	pub fn is_list_wrapper(&self) -> bool {
		self.map.len() == 1 && self.map.contains_key("")
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

		let name = deserialize_nbt_string(deserializer)?;
		let mut compound = NbtCompound::new(Some(name));

		loop {
			let tag = deserializer.pop();

			if tag.is_none() || tag.unwrap() == 0 {
				// END Tag
				break;
			}

			let name = deserialize_nbt_string(deserializer)?;

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

			let name = deserialize_nbt_string(deserializer)?;

			let tag = NbtTag::deserialize_specific(deserializer, tag.unwrap())?;
			compound.add(name, tag);
		}

		Ok(compound)
	}

	/// Serialize the compound without a leading tag byte.
	fn serialize_no_tag(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		// only serialize root name if present (non-network compound tag or pre 1.20.2)
		if let Some(root_name) = &self.root_name {
			serialize_nbt_string(serializer, root_name)?;
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
				serialize_nbt_string(serializer, name)?;
				tag.serialize_payload(serializer)?;
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

	/// Like `from_vec`, but a list with mixed item types does not fail. Vanilla puts each item into a
	/// compound with one empty key (`"x"` becomes `{"": "x"}`), so the list holds only compounds.
	/// A compound stays as it is, except one that already has the wrapper shape, which is wrapped
	/// again so that a reader does not unwrap it by mistake. This is also done in a list of only
	/// compounds. A list with one item type and no wrapper-shaped compound is not changed.
	///
	/// <https://minecraft.wiki/w/NBT_format>
	pub fn from_vec_wrapping(vec: Vec<NbtTag>) -> Result<Self, NbtError> {
		if vec.iter().any(|tag| tag.get_type_id() == 0) {
			return Err(NbtError::EndTagNotAllowedInList);
		}

		let first = vec.first().map(NbtTag::get_type_id);
		let mixed = !vec.iter().all(|tag| Some(tag.get_type_id()) == first);
		let has_wrapper_shape = vec.iter().any(|tag| matches!(tag, NbtTag::Compound(compound) if compound.is_list_wrapper()));
		if !mixed && !has_wrapper_shape {
			return Self::from_vec(vec);
		}

		let wrapped = vec
			.into_iter()
			.map(|tag| match tag {
				NbtTag::Compound(compound) if !compound.is_list_wrapper() => NbtTag::Compound(compound),
				other => {
					let mut wrapper = NbtCompound::new_no_name();
					wrapper.add("", other);
					NbtTag::Compound(wrapper)
				}
			})
			.collect();

		Self::from_vec(wrapped)
	}

	#[inline]
	pub fn add<T: Into<NbtTag>>(&mut self, tag: T) -> Result<(), NbtError> {
		self.add_tag(tag.into())
	}

	/// End (type 0) is only the element type of an empty list, never an element itself.
	///
	/// <https://minecraft.wiki/w/NBT_format>
	#[inline]
	pub fn add_tag(&mut self, tag: NbtTag) -> Result<(), NbtError> {
		if tag.get_type_id() == 0 {
			return Err(NbtError::EndTagNotAllowedInList);
		}

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
			tag.serialize_payload(serializer)?;
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

	/// Regression: a bare `NbtTag` (Block Entity Data, Tag Query Response, the Lock component) is
	/// network NBT, so it starts with a type id. Serialize used to write only the payload, while
	/// deserialize read a type id. The two sides did not agree, and peers read the wrong bytes.
	#[test]
	fn bare_tag_writes_type_id_and_round_trips() {
		let mut compound = NbtCompound::new_no_name();
		compound.add("x", 1i32);
		for tag in [NbtTag::End, NbtTag::String("hi".to_string()), NbtTag::Int(7), NbtTag::Compound(compound)] {
			let mut serializer = McSerializer::new();
			tag.mc_serialize(&mut serializer).unwrap();
			assert_eq!(serializer.output[0], tag.get_type_id(), "{tag:?}");

			let mut deserializer = McDeserializer::new(&serializer.output);
			assert_eq!(NbtTag::mc_deserialize(&mut deserializer).unwrap(), tag);
			assert!(deserializer.is_at_end());
		}

		// The string payload is a u16 length and the bytes, after the type id 8.
		let mut serializer = McSerializer::new();
		NbtTag::String("hi".to_string()).mc_serialize(&mut serializer).unwrap();
		assert_eq!(serializer.output, vec![8, 0, 2, b'h', b'i']);
	}

	/// An NBT list has one element type for all items, and it is written once in the header. An item of
	/// a different type would be written as if it had the list type, so the list must refuse it.
	#[test]
	fn nbt_list_type_checks() {
		let mut list = NbtList::new();
		assert_eq!(list.type_id, 0);
		list.add(1i32).unwrap();
		assert_eq!(list.type_id, 3, "the first item sets the list type");
		list.add_tag(NbtTag::Int(2)).unwrap();

		assert_eq!(list.add("x"), Err(NbtError::IncompatibleTypes));
		assert_eq!(list.add_tag(NbtTag::Long(3)), Err(NbtError::IncompatibleTypes));
		assert_eq!(list.add(NbtTag::End), Err(NbtError::EndTagNotAllowedInList));
		assert_eq!(list.list, vec![NbtTag::Int(1), NbtTag::Int(2)], "a refused item must not be added");

		assert_eq!(NbtList::from_vec(vec![NbtTag::Byte(1), NbtTag::Short(1)]), Err(NbtError::IncompatibleTypes));
	}

	/// End is only the type of an empty list. If add_tag took an End item, the list type would stay 0,
	/// the next item would set a new type, and the list would hold mixed types. Both entry points must refuse it.
	#[test]
	fn nbt_list_refuses_end_on_every_path() {
		let mut list = NbtList::new();
		assert_eq!(list.add_tag(NbtTag::End), Err(NbtError::EndTagNotAllowedInList));
		assert!(list.list.is_empty());
		assert_eq!(list.type_id, 0);

		assert_eq!(NbtList::from_vec(vec![NbtTag::End, NbtTag::Int(1)]), Err(NbtError::EndTagNotAllowedInList));

		// An empty list of type End is still valid on the wire.
		let bytes = [0u8, 0, 0, 0, 0];
		let empty = NbtList::mc_deserialize(&mut McDeserializer::new(&bytes)).unwrap();
		assert_eq!(empty, NbtList::new());
	}

	/// The three array tags and a list all convert to a Vec of the element type. The element type must
	/// match exactly (an IntArray is not a Vec<i64>), and a non-list tag is an error.
	#[test]
	fn nbt_tag_to_vec_conversions() {
		let bytes: Vec<i8> = NbtTag::ByteArray(NbtByteArray::new(vec![1, -2])).try_into().unwrap();
		assert_eq!(bytes, vec![1, -2]);
		let ints: Vec<i32> = NbtTag::IntArray(NbtIntArray::new(vec![3, 4])).try_into().unwrap();
		assert_eq!(ints, vec![3, 4]);
		let longs: Vec<i64> = NbtTag::LongArray(NbtLongArray::new(vec![5])).try_into().unwrap();
		assert_eq!(longs, vec![5]);
		let from_list: Vec<String> = NbtTag::List(NbtList::from_vec(vec![NbtTag::from("a"), NbtTag::from("b")]).unwrap()).try_into().unwrap();
		assert_eq!(from_list, vec!["a".to_string(), "b".to_string()]);

		let wrong_element: Result<Vec<i64>, _> = NbtTag::IntArray(NbtIntArray::new(vec![1])).try_into();
		assert_eq!(wrong_element, Err(NbtError::InvalidType));
		let not_a_list: Result<Vec<i32>, _> = NbtTag::Int(1).try_into();
		assert_eq!(not_a_list, Err(NbtError::InvalidType));
	}

	/// A Vec of one type becomes a List tag of that type, in order.
	#[test]
	fn vec_into_nbt_list() {
		match NbtTag::from(vec![1i32, 2, 3]) {
			NbtTag::List(list) => {
				assert_eq!(list.type_id, 3);
				assert_eq!(list.list, vec![NbtTag::Int(1), NbtTag::Int(2), NbtTag::Int(3)]);
			}
			other => panic!("expected a list, got {other:?}"),
		}
	}

	/// A list with mixed item types must keep every item. Before, `From<Vec<T>>` gave an empty list
	/// with no error, and data was lost. Vanilla wraps each non-compound item as `{"": item}`.
	#[test]
	fn mixed_vec_wraps_items_as_vanilla_does() {
		let mut styled = NbtCompound::new_no_name();
		styled.add("text", "b");
		let mut looks_like_wrapper = NbtCompound::new_no_name();
		looks_like_wrapper.add("", 5i32);

		let wrap = |tag: NbtTag| {
			let mut wrapper = NbtCompound::new_no_name();
			wrapper.add("", tag);
			NbtTag::Compound(wrapper)
		};

		match NbtTag::from(vec![NbtTag::from("a"), NbtTag::Compound(styled.clone()), NbtTag::Compound(looks_like_wrapper.clone())]) {
			NbtTag::List(list) => {
				assert_eq!(list.type_id, 10, "a mixed list must hold only compounds");
				assert_eq!(
					list.list,
					vec![
						wrap(NbtTag::from("a")),
						NbtTag::Compound(styled),
						// Not wrapped again, a reader would unwrap it and get the Int 5, not the compound.
						wrap(NbtTag::Compound(looks_like_wrapper)),
					]
				);
			}
			other => panic!("expected a list, got {other:?}"),
		}

		// One item type: no wrapping, so the wire bytes are the same as before.
		assert_eq!(NbtList::from_vec_wrapping(vec![NbtTag::from("a"), NbtTag::from("b")]), NbtList::from_vec(vec![NbtTag::from("a"), NbtTag::from("b")]));
		// End is never a list item, also when the types are mixed.
		assert_eq!(NbtList::from_vec_wrapping(vec![NbtTag::Int(1), NbtTag::End]), Err(NbtError::EndTagNotAllowedInList));
		// The strict constructor still refuses mixed types.
		assert_eq!(NbtList::from_vec(vec![NbtTag::from("a"), NbtTag::Int(1)]), Err(NbtError::IncompatibleTypes));
	}

	/// An End item means "no value". Before, one End item made `From<Vec<T>>` give an empty list, and
	/// all the other items were lost. Now only the End item is skipped.
	#[test]
	fn vec_with_end_item_keeps_other_items() {
		match NbtTag::from(vec![NbtTag::Int(1), NbtTag::End, NbtTag::Int(2)]) {
			NbtTag::List(list) => assert_eq!(list.list, vec![NbtTag::Int(1), NbtTag::Int(2)]),
			other => panic!("expected a list, got {other:?}"),
		}
	}

	/// Write a mixed list and read it back as a Vec. The reader must remove the `{"": item}` wrapper,
	/// or every wrapped item is read as a compound and is not the value that was written.
	#[test]
	fn vec_read_unwraps_mixed_list_items() {
		let mut wrapped = NbtCompound::new_no_name();
		wrapped.add("", "a");
		let list = NbtTag::List(NbtList::from_vec(vec![NbtTag::Compound(wrapped)]).unwrap());
		let read: Vec<String> = list.try_into().unwrap();
		assert_eq!(read, vec!["a".to_string()], "the empty-key wrapper must be removed");

		// A list of only compounds has one type, but a wrapper-shaped compound must still be wrapped.
		// If not, the reader removes its `""` key and gets the Int 5, not the compound.
		let mut plain = NbtCompound::new_no_name();
		plain.add("k", 1i32);
		let mut wrapper_shape = NbtCompound::new_no_name();
		wrapper_shape.add("", 5i32);
		let compounds = vec![plain, wrapper_shape];
		let read: Vec<NbtCompound> = NbtTag::from(compounds.clone()).try_into().unwrap();
		assert_eq!(read, compounds);
	}

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

	/// NBT strings (both tag names and values) use Java's modified UTF-8, not standard UTF-8.
	#[test]
	fn test_modified_utf8_strings() {
		let mut compound = NbtCompound::new_no_name();
		compound.add("k", "😀"); // U+1F600, a supplementary character

		let mut serializer = McSerializer::new();
		compound.mc_serialize(&mut serializer).unwrap();
		let bytes = &serializer.output;

		let cesu8_surrogate_pair = [0xED, 0xA0, 0xBD, 0xED, 0xB8, 0x80];
		let standard_utf8 = [0xF0, 0x9F, 0x98, 0x80];
		assert!(bytes.windows(6).any(|w| w == cesu8_surrogate_pair), "expected CESU-8 surrogate pair");
		assert!(!bytes.windows(4).any(|w| w == standard_utf8), "must not emit standard 4-byte UTF-8 for a supplementary char");

		let mut deserializer = McDeserializer::new(&serializer.output);
		let deserialized = NbtCompound::mc_deserialize(&mut deserializer).unwrap();
		assert_eq!(deserialized["k"], NbtTag::String("😀".to_string()));

		// NUL is written as 0xC0 0x80, and survives the round trip in both names and values.
		let mut nul_compound = NbtCompound::new_no_name();
		nul_compound.add("a\0b", "x\0y");
		let mut nul_serializer = McSerializer::new();
		nul_compound.mc_serialize(&mut nul_serializer).unwrap();
		assert!(nul_serializer.output.windows(2).any(|w| w == [0xC0, 0x80]), "NUL must be encoded as 0xC0 0x80");

		let mut nul_deserializer = McDeserializer::new(&nul_serializer.output);
		let nul_deserialized = NbtCompound::mc_deserialize(&mut nul_deserializer).unwrap();
		assert_eq!(nul_deserialized["a\0b"], NbtTag::String("x\0y".to_string()));
	}
}
