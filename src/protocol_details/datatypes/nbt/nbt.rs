use std::any::Any;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::{list_nbtvalue, primvalue_nbtvalue};
use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NbtCompound<T: NbtValue + McSerialize + Clone + Debug> {
    list: Vec<T>
}

impl<T: NbtValue + McSerialize + Clone + Debug> NbtCompound<T> {
    pub fn new() -> Self {
        Self {
            list: vec![]
        }
    }

    pub fn add(&mut self, tag: T) {
        self.list.push(tag);
    }
}

impl<T: NbtValue + McSerialize + Clone + Debug> McSerialize for NbtCompound<T> {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
        for tag in &self.list {
            serializer.serialize_u8(tag.get_type_id());
            serializer.serialize_str(&tag.get_name());

            if let Some(payload_size) = tag.get_payload_size() {
                serializer.serialize_u8(payload_size);
            }

            // TODO: look into FROM/TO
            match tag.get_type_id() {
                0 => { // END
                    return Err(SerializingErr::UniqueFailure("END Tag not allowed in compound.".to_string()));
                },
                8 => { // STRING
                    let s = tag.get_name();
                    serializer.serialize_str(&s);
                },
                _ => { // anything with default / individual serialization
                    tag.mc_serialize(serializer)?;
                }
            }
        }

        EndTag {}.mc_serialize(serializer)?;

        Ok(())
    }
}

pub trait NbtValue {
    fn get_type_id(&self) -> u8;
    fn get_payload_size(&self) -> Option<u8>;
    fn get_name(&self) -> String;
}

primvalue_nbtvalue!(
    (i8, 1, 1, "TAG_Byte"),
    (i16, 2, 2, "TAG_Short"),
    (i32, 3, 4, "TAG_Int"),
    (i64, 4, 8, "TAG_Long"),
    (f32, 5, 4, "TAG_Float"),
    (f64, 6, 8, "TAG_Double"),
    (String, 8, 0, "TAG_String")
);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EndTag {}

impl NbtValue for EndTag {
    fn get_type_id(&self) -> u8 {
        0
    }

    fn get_payload_size(&self) -> Option<u8> {
        Some(0)
    }

    fn get_name(&self) -> String {
        "TAG_End".to_string()
    }
}

impl McSerialize for EndTag {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
        serializer.serialize_u8(0);
        
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NbtList<T: NbtValue + McSerialize + Clone + Debug> {
    pub type_id: u8,
    pub list: Vec<T>,
    count: u32, // used for iterator
}

impl <T: NbtValue + McSerialize + Clone + Debug> NbtList<T> {
    pub fn new() -> Self {
        Self {
            type_id: 0, // set to END by default
            list: vec![],
            count: 0
        }
    }

    pub fn add(&mut self, tag: T) -> Result<()> {
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

impl<T: NbtValue + McSerialize + Clone + Debug> Iterator for NbtList<T> {
    type Item = T;

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

impl<T: NbtValue + McSerialize + Clone + Debug> NbtValue for NbtList<T> {
    fn get_type_id(&self) -> u8 {
        9
    }

    fn get_payload_size(&self) -> Option<u8> {
        None
    }

    fn get_name(&self) -> String {
        "TAG_List".to_string()
    }
}

impl<T: NbtValue + McSerialize + Clone + Debug> McSerialize for NbtList<T> {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
        serializer.serialize_u8(self.type_id);
        (self.list.len() as i32).mc_serialize(serializer)?;

        for tag in &self.list {
            tag.mc_serialize(serializer)?;
        }

        Ok(())
    }
}

list_nbtvalue!(
    (i8, 7, "TAG_Byte", NbtByteList), 
    (i32, 11, "TAG_Int", NbtIntList), 
    (i64, 12, "TAG_Long", NbtLongList)
);