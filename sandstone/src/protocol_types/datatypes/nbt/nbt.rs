//! An NBT implementation without support for sNBT (string NBT). See 
//! [here](https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/NBT) for more information.

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::nbt::nbt::NbtTag::List;
use crate::protocol_types::datatypes::nbt::nbt_error::NbtError;
use crate::{list_nbtvalue, primvalue_nbtvalue};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Index;

/// The various tags or data types that could be present inside of an NBT compound
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
	/// Used to mark the absence of a tag, like for an Option
	None
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
			NbtTag::None => 255, // not an actual type
		}
	}

	/// Used to assist in deserialization. Returns 'None' if payload size is variable
	pub fn get_payload_size(&self) -> Option<u8> {
		match self {
			NbtTag::End => Some(0),
			NbtTag::Byte(_) => Some(1),
			NbtTag::Short(_) => Some(2),
			NbtTag::Int(_) => Some(4),
			NbtTag::Long(_) => Some(8),
			NbtTag::Float(_) => Some(4),
			NbtTag::Double(_) => Some(8),
			NbtTag::ByteArray(_) => None,
			NbtTag::String(_) => None,
			NbtTag::List(_) => None,
			NbtTag::Compound(_) => None,
			NbtTag::IntArray(_) => None,
			NbtTag::LongArray(_) => None,
			NbtTag::None => Some(0),
		}
	}

	/// Get the string representation of a tag, used for sNBT
	pub fn get_name(&self) -> String {
		match self {
			NbtTag::End => "TAG_End".to_string(),
			NbtTag::Byte(_) => "TAG_Byte".to_string(),
			NbtTag::Short(_) => "TAG_Short".to_string(),
			NbtTag::Int(_) => "TAG_Int".to_string(),
			NbtTag::Long(_) => "TAG_Long".to_string(),
			NbtTag::Float(_) => "TAG_Float".to_string(),
			NbtTag::Double(_) => "TAG_Double".to_string(),
			NbtTag::ByteArray(_) => "TAG_Byte_Array".to_string(),
			NbtTag::String(_) => "TAG_String".to_string(),
			NbtTag::List(_) => "TAG_List".to_string(),
			NbtTag::Compound(_) => "TAG_Compound".to_string(),
			NbtTag::IntArray(_) => "TAG_Int_Array".to_string(),
			NbtTag::LongArray(_) => "TAG_Long_Array".to_string(),
			NbtTag::None => "TAG_None".to_string(),
		}
	}
	
	/// Given the type ID, deserialize the corresponding NbtTag.
	fn deserialize_specific<'a>(deserializer: &mut McDeserializer, ty: u8) -> SerializingResult<'a, Self> {
		match ty {
			// Primitives
			0 => Ok(NbtTag::End),
			1 => Ok(NbtTag::Byte(i8::mc_deserialize(deserializer)?)),
			2 => Ok(NbtTag::Short(i16::mc_deserialize(deserializer)?)),
			3 => Ok(NbtTag::Int(i32::mc_deserialize(deserializer)?)),
			4 => Ok(NbtTag::Long(i64::mc_deserialize(deserializer)?)),
			5 => Ok(NbtTag::Float(f32::mc_deserialize(deserializer)?)),
			6 => Ok(NbtTag::Double(f64::mc_deserialize(deserializer)?)),

			8 => { // String
				let len = u16::mc_deserialize(deserializer)?;
				let bytes = deserializer.slice(len as usize);

				Ok(NbtTag::String(String::from_utf8_lossy(bytes).to_string()))
			},

			7 => { // Byte array
				Ok(NbtTag::ByteArray(NbtByteArray::mc_deserialize(deserializer)?))
			},
			11 => { // Int Array
				Ok(NbtTag::IntArray(NbtIntArray::mc_deserialize(deserializer)?))
			},
			12 => { // Int Array
				Ok(NbtTag::LongArray(NbtLongArray::mc_deserialize(deserializer)?))
			},

			9 => { // List
				Ok(NbtTag::List(NbtList::mc_deserialize(deserializer)?))
			},

			10 => { // compound
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
			NbtTag::String(s) => { // not the same as regular string serialization (no varint)
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
			NbtTag::ByteArray(b) => {
				b.mc_serialize(serializer)?
			}
			NbtTag::IntArray(b) => {
				b.mc_serialize(serializer)?
			}
			NbtTag::LongArray(b) => {
				b.mc_serialize(serializer)?
			}
			NbtTag::List(b) => {
				b.mc_serialize(serializer)?
			}
			NbtTag::Compound(c) => {
				c.serialize_no_tag(serializer)?
			}
			_ => {} // do nothing for None
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

impl From<NbtTag> for String {
	fn from(value: NbtTag) -> Self {
		match value {
			NbtTag::String(s) => s,
			_ => panic!("Cannot convert non-string tag to string"),
		}
	}
}

impl From<bool> for NbtTag {
	fn from(value: bool) -> Self {
		NbtTag::Byte(if value { 1 } else { 0 })
	}
}

impl From<NbtTag> for bool {
	fn from(value: NbtTag) -> Self {
		match value {
			NbtTag::Byte(b) => b != 0,
			_ => panic!("Cannot convert non-byte tag to bool"),
		}
	}
}

impl<T: Into<NbtTag>> From<Option<T>> for NbtTag {
	fn from(value: Option<T>) -> Self {
		if let Some(tag) = value {
			tag.into()
		} else {
			NbtTag::None
		}
	}
}

// this is replicated for the other types in the primvalue_nbtvalue! macro
impl From<NbtTag> for Option<String> {
	fn from(value: NbtTag) -> Self {
		if let NbtTag::String(s) = value {
			Some(s)
		} else {
			None
		}
	}
}

impl<T: Into<NbtTag>> From<Vec<T>> for NbtTag {
	fn from(value: Vec<T>) -> Self {
		let tags: Vec<NbtTag> = value.into_iter().map(Into::into).collect();
		match NbtList::from_vec(tags) {
			Ok(list) => List(list),
			Err(_) => NbtTag::None,
		}
	}
}

impl<T: From<NbtTag>> From<NbtTag> for Vec<T> {
	fn from(value: NbtTag) -> Self {
		match value {
			NbtTag::ByteArray(list) => {
				list.list.into_iter().map(|b| T::from(NbtTag::Byte(b))).collect()
			}
			List(list) => {
				// if its, NbtTag::None, don't add it to the list
				list.list.into_iter().filter_map(|tag| {
					if tag == NbtTag::None || tag == NbtTag::End {
						None
					} else {
						Some(T::from(tag))
					}
				}).collect()
			}
			NbtTag::IntArray(list) => {
				list.list.into_iter().map(|i| T::from(NbtTag::Int(i))).collect()
			}
			NbtTag::LongArray(list) => {
				list.list.into_iter().map(|l| T::from(NbtTag::Long(l))).collect()
			}
			_ => {
				debug!("Cannot convert NbtTag {:?} to Vec<T>", value);
				vec![]
			}
		}
	}
}

primvalue_nbtvalue!(
    (i8, Byte),
    (i16, Short),
    (i32, Int),
    (i64, Long),
    (f32, Float),
    (f64, Double)
);

list_nbtvalue!(
    (i8, ByteArray, NbtByteArray, 7),
    (i32, IntArray, NbtIntArray, 11),
    (i64, LongArray, NbtLongArray, 12)
);

/// Effectively a map of NbtTagLegacys
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NbtCompound {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub(crate) root_name: Option<String>,
	pub(crate) map: HashMap<String, NbtTag>,
}

impl NbtCompound {
	/// Root name is only present if it is not the root compound of network NBT (1.20.2+).
	pub fn new<T: Into<String>>(root_name: Option<T>) -> Self {
		let option: Option<String> = if let Some(root_name) = root_name {
			Some(root_name.into())
		} else {
			None
		};
		
		Self {
			map: HashMap::new(),
			root_name: option
		}
	}
	
	/// Create a new NbtCompound without a root name, usually for network nbt or a compound inside a compound.
	pub fn new_no_name() -> Self {
		Self::new::<String>(None)
	}

	pub fn change_root_name<T: Into<String>>(&mut self, name: T) {
		self.root_name = Some(name.into());
	}

	#[inline]
	pub fn add<K: Into<String>, V: Into<NbtTag>>(&mut self, name: K, tag: V) {
		let tag = tag.into();
		
		if tag == NbtTag::End {
			return; // do not add None tag
		}
		
		self.map.insert(name.into(), tag);
	}

	#[inline]
	pub fn remove<T: Into<String>>(&mut self, name: T) {
		self.map.remove(&name.into());
	}
	
	/// Deserialize compounds with a root name
	pub fn from_root<'a>(deserializer: &mut McDeserializer) -> SerializingResult<'a, NbtCompound> {
		let t = u8::mc_deserialize(deserializer)?;

		if t != 10 {
			return Err(SerializingErr::UniqueFailure(format!("Expected compound tag id, got {} instead", t)));
		}

		let name_length = u16::mc_deserialize(deserializer)?;
		let name = String::from_utf8_lossy(deserializer.slice(name_length as usize)).to_string();
		let mut compound = NbtCompound::new(Some(name));

		loop {
			let tag = deserializer.pop();

			if tag.is_none() || tag.unwrap() == 0 { // END Tag
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

			if tag.is_none() || tag.unwrap() == 0 { // END Tag
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
			if *tag != NbtTag::None && *tag != NbtTag::End {
				serializer.serialize_u8(tag.get_type_id());
				(name.as_bytes().len() as u16).mc_serialize(serializer)?;
				serializer.serialize_bytes(name.as_bytes());
				tag.mc_serialize(serializer)?;
			}
		}
		serializer.serialize_u8(0); // end tag
		Ok(())
	}
}

impl Index<&str> for NbtCompound {
	type Output = NbtTag;

	/// Returns a reference to the value inside of the HashMap mapped to the given key. 
	/// Returns `NbtTag::None` if the key does not exist.
	fn index(&self, index: &str) -> &Self::Output {
		if !self.map.contains_key(index) {
			return &NbtTag::None;
		}
		
		&self.map[index]
	}
}

impl McSerialize for NbtCompound {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		10u8.mc_serialize(serializer)?;
		
		self.serialize_no_tag(serializer)
	}
}

impl McDeserialize for NbtCompound {
	/// Deserialize a compound without a root name, such as network NBT or compounds in compounds.
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let t = u8::mc_deserialize(deserializer)?; // todo: debug here

		if t != 10 {
			debug!("Nearby bytes: {:?}", deserializer.subset(50, 15));
			return Err(SerializingErr::UniqueFailure(format!("Expected compound tag id, got {} instead", t)));
		}
		
		Self::from_no_tag(deserializer)
	}
}

impl McDefault for NbtCompound {
	fn mc_default() -> Self {
		let mut compound = NbtCompound::new_no_name();

		compound.add("default_string", "default_value");
		compound.add("default_int", 42);
		compound.add("default_float", 3.14f32);

		compound
	}
}

impl Into<NbtTag> for NbtCompound {
	fn into(self) -> NbtTag {
		NbtTag::Compound(self)
	}
}

impl From<NbtTag> for NbtCompound {
	fn from(tag: NbtTag) -> Self {
		if let NbtTag::Compound(c) = tag {
			c
		} else {
			panic!("Cannot convert non-compound tag to compound");
		}
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
	count: u32, // used for iterator
}

impl NbtList {
	pub fn new() -> Self {
		Self {
			type_id: 0, // set to END by default
			list: vec![],
			count: 0
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

impl Iterator for NbtList {
	type Item = NbtTag;

	fn next(&mut self) -> Option<Self::Item> {
		if self.count < self.list.len() as u32 {
			let tag = self.list[self.count as usize].clone();
			self.count += 1;
			Some(tag)
		} else {
			None
		}
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
			return Err(SerializingErr::UniqueFailure("Type cannot be END when length is positive".to_string()))
		}

		let mut list = NbtList::new();

		for _ in 0..length {
			let tag = NbtTag::deserialize_specific(deserializer, t)?;

			if tag.get_type_id() != t {
				return Err(SerializingErr::UniqueFailure("Type must be the same as the type for the list".to_string()))
			}

			if let Err(_) = list.add_tag(tag) {
				return Err(SerializingErr::UniqueFailure("Could not push tag to list".to_string()));
			}
		}

		Ok(list)
	}
}

impl From<NbtTag> for NbtList {
	fn from(tag: NbtTag) -> Self {
		if let List(l) = tag {
			l
		} else {
			panic!("Cannot convert non-list tag to list");
		}
	}
}

impl Into<NbtTag> for NbtList {
	fn into(self) -> NbtTag {
		NbtTag::List(self)
	}
}