use std::any::Any;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::primvalue_nbtvalue;

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

            match tag.get_type_id() {
                0 => { // END
                    return Err(SerializingErr::UniqueFailure("END Tag not allowed in compound.".to_string()));
                },
                8 => { // STRING
                    let s = tag.get_name();
                    serializer.serialize_str(&s);
                },
                9 => { // LIST
                    // TODO: cast to list?
                    // list.mc_serialize(serializer)?; // TODO: implement serialize on all lists
                },
                // TODO: Rest of the lists here
                _ => { // primitives
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
pub struct NbtList<T: NbtValue> {
    list: Vec<T>
}

impl<T: NbtValue> NbtValue for NbtList<T> {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NbtByteArray {
    list: Vec<i8>
}

impl NbtValue for NbtByteArray {
    fn get_type_id(&self) -> u8 {
        7
    }

    fn get_payload_size(&self) -> Option<u8> {
        None
    }

    fn get_name(&self) -> String {
        "TAG_Byte_Array".to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NbtIntArray {
    list: Vec<i32>
}

impl NbtValue for NbtIntArray {
    fn get_type_id(&self) -> u8 {
        11
    }

    fn get_payload_size(&self) -> Option<u8> {
        None
    }

    fn get_name(&self) -> String {
        "TAG_Int_Array".to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NbtLongArray {
    list: Vec<i64>
}

impl NbtValue for NbtLongArray {
    fn get_type_id(&self) -> u8 {
        12
    }

    fn get_payload_size(&self) -> Option<u8> {
        None
    }

    fn get_name(&self) -> String {
        "TAG_Long_Array".to_string()
    }
}