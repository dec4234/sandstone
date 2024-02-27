use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer};
use anyhow::{anyhow, Result};
use crate::{list_nbtvalue, primvalue_nbtvalue};


#[derive(Debug, Clone, PartialEq)]
pub enum NbtTag {
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
    LongArray(NbtLongArray)
}

impl NbtTag {
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
            NbtTag::LongArray(_) => 12
        }
    }

    pub fn get_payload_size(&self) -> Option<u8> {
        match self {
            NbtTag::End => Some(0),
            NbtTag::Byte(_) => Some(1),
            NbtTag::Short(_) => Some(2),
            NbtTag::Int(_) => Some(4),
            NbtTag::Long(_) => Some(8),
            NbtTag::Float(_) => Some(4),
            NbtTag::Double(_) => Some(8),
            NbtTag::ByteArray(b) => None,
            NbtTag::String(s) => None,
            NbtTag::List(l) => None,
            NbtTag::Compound(c) => None,
            NbtTag::IntArray(i) => None,
            NbtTag::LongArray(l) => None,
        }
    }

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
            NbtTag::LongArray(_) => "TAG_Long_Array".to_string()
        }
    }
}

impl McSerialize for NbtTag {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> std::result::Result<(), SerializingErr> {
        match self {
            NbtTag::End => {serializer.serialize_u8(0)}
            NbtTag::Byte(b) => {b.mc_serialize(serializer)?}
            NbtTag::Short(b) => {b.mc_serialize(serializer)?}
            NbtTag::Int(b) => {b.mc_serialize(serializer)?}
            NbtTag::Long(b) => {b.mc_serialize(serializer)?}
            NbtTag::Float(b) => {b.mc_serialize(serializer)?}
            NbtTag::Double(b) => {b.mc_serialize(serializer)?}
            NbtTag::String(s) => {
                (s.len() as u16).mc_serialize(serializer)?;
                serializer.serialize_bytes(s.as_bytes());
            }
            NbtTag::List(l) => {}
            NbtTag::Compound(c) => {}
            NbtTag::ByteArray(ba) => {}
            NbtTag::IntArray(ia) => {}
            NbtTag::LongArray(la) => {}
        }
        
        Ok(())
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
    (i8, ByteArray, NbtByteArray), 
    (i32, IntArray, NbtIntArray), 
    (i64, LongArray, NbtLongArray)
);



#[derive(Debug, Clone, PartialEq)]
pub struct NbtCompound {
    map: HashMap<String, NbtTag>
}

impl NbtCompound {
    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }

    #[inline]
    pub fn add<K: Into<String>, V: Into<NbtTag>>(&mut self, name: K, tag: V) {
        self.map.insert(name.into(), tag.into());
    }
}

#[derive(Debug, Clone, PartialEq)]
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

    pub fn add<T: Into<NbtTag>>(&mut self, tag: T) -> Result<()> {
        let tag = tag.into();
        
        if tag.get_type_id() == 0 {
            return Err(anyhow!("END Tag not allowed in NbtList"));
        }
        
        if self.type_id == 0 {
            self.type_id = tag.get_type_id();
        } else if self.type_id != tag.get_type_id() {
            return Err(anyhow!("Type mismatch in NbtList"));
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