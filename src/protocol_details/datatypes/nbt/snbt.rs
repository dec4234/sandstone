use anyhow::Result;

use crate::{list_snbt, primtype_snbt};
use crate::protocol_details::datatypes::nbt::nbt::{NbtCompound, NbtTag};

/*
This file handles SNBT (String NBT) and all serialization/deserialization involved.

TODO: implement deserializer - chances are it has to be split from SNBT trait

Remaining Questions:
1. Does SNBT actually implement new lines? Or is it just for the examples?
2. What is the relationship between a compounds root name and its provided tag name. 
   I suppose that the root-name is what is actually used
*/

// https://wiki.vg/NBT
pub trait SNBT {
	fn to_snbt(&self, name: Option<String>) -> String;
	fn from_snbt(snbt: &str) -> Result<(Self, String)> where Self: Sized;
}

impl SNBT for NbtCompound {
	fn to_snbt(&self, name: Option<String>) -> String {
		let mut s = String::new();

		let name = match &self.root_name {
			Some(name) => name,
			None => ""
		};

		let e = if self.map.len() == 1 {
			"entry"
		} else {
			"entries"
		};

		s.push_str(format!("TAG_Compound('{}'): {} {}", name, self.map.len(), e).as_str());
		s.push_str("{");

		for (name, tag) in &self.map {
			match tag {
				NbtTag::Compound(c) => {
					s.push_str(&c.to_snbt(Some(name.to_string())));
				}
				NbtTag::Byte(b) => {
					s.push_str(&b.to_snbt(Some(name.to_string())));
				}
				NbtTag::Short(b) => {
					s.push_str(format!("TAG_Short('{}'): {}", name, b).as_str());
				}
				NbtTag::Int(i) => {
					s.push_str(format!("TAG_Int('{}'): {}", name, i).as_str());
				}
				NbtTag::Long(i) => {
					s.push_str(format!("TAG_Long('{}'): {}", name, i).as_str());
				}
				NbtTag::Float(f) => {
					s.push_str(format!("TAG_Float('{}'): {}", name, f).as_str());
				}
				NbtTag::Double(f) => {
					s.push_str(format!("TAG_Double('{}'): {}", name, f).as_str());
				}
				NbtTag::ByteArray(b) => {
					s.push_str(&b.list.to_snbt(Some(name.to_string())));
				}
				NbtTag::IntArray(i) => {
					s.push_str(&i.list.to_snbt(Some(name.to_string())));
				}
				NbtTag::LongArray(i) => {
					s.push_str(&i.list.to_snbt(Some(name.to_string())));
				}
				NbtTag::String(str) => {
					s.push_str(format!("TAG_String('{}'): '{}'", name, str).as_str());
				}
				NbtTag::List(l) => {
					s.push_str(format!("TAG_List('{}'): {} entries {{", name, l.list.len()).as_str());

					for tag in &l.list {
						s.push_str(tag.to_snbt(None).as_str());
					}

					s.push_str("}");
				}
				_ => {}
			}
		}

		s.push_str("}");
		s
	}

	fn from_snbt(snbt: &str) -> Result<(Self, String)> where Self: Sized {
		todo!()
	}
}

impl SNBT for NbtTag {
	fn to_snbt(&self, name: Option<String>) -> String {
		match self {
			NbtTag::Compound(c) => {
				c.to_snbt(name)
			}
			NbtTag::Byte(b) => {
				b.to_snbt(name)
			}
			NbtTag::Short(b) => {
				format!("TAG_Short('{}'): {}", name.unwrap_or("None".to_string()), b)
			}
			NbtTag::Int(i) => {
				format!("TAG_Int('{}'): {}", name.unwrap_or("None".to_string()), i)
			}
			NbtTag::Long(i) => {
				format!("TAG_Long('{}'): {}", name.unwrap_or("None".to_string()), i)
			}
			NbtTag::Float(f) => {
				format!("TAG_Float('{}'): {}", name.unwrap_or("None".to_string()), f)
			}
			NbtTag::Double(f) => {
				format!("TAG_Double('{}'): {}", name.unwrap_or("None".to_string()), f)
			}
			NbtTag::ByteArray(b) => {
				b.list.to_snbt(name)
			}
			NbtTag::IntArray(i) => {
				i.list.to_snbt(name)
			}
			NbtTag::LongArray(i) => {
				i.list.to_snbt(name)
			}
			NbtTag::String(str) => {
				format!("TAG_String('{}'): '{}'", name.unwrap_or("None".to_string()), str)
			}
			NbtTag::List(l) => {
				let e = if l.list.len() == 1 {
					"entry"
				} else {
					"entries"
				};

				format!("TAG_List('{}'): {} {} {{", name.unwrap_or("None".to_string()), l.list.len(), e)
			}
			_ => {
				"".to_string()
			}
		}
	}

	fn from_snbt(snbt: &str) -> Result<(Self, String)> where Self: Sized {
		todo!()
	}
}

primtype_snbt!("Byte", i8);
primtype_snbt!("Short", i16);
primtype_snbt!("Int", i32);
primtype_snbt!("Long", i64);
primtype_snbt!("Float", f32);

list_snbt!("ByteArray", i8);
list_snbt!("IntArray", i32);
list_snbt!("LongArray", i64);