use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol_details::datatypes::nbt::nbt::NbtTag;

//TODO: Same for NamedTag
impl McSerialize for NbtTag {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
        match self {
            NbtTag::End => {}
            NbtTag::Byte(b) => {}
            NbtTag::Short(s) => {}
            NbtTag::Int(i) => {}
            NbtTag::Long(l) => {}
            NbtTag::Float(f) => {}
            NbtTag::Double(d) => {}
            NbtTag::String(s) => {}
            NbtTag::List((ty, len, list)) => {}
            NbtTag::Compound((name, list)) => {}
            NbtTag::Byte_Array((len, list)) => {}
            NbtTag::Int_Array((len, list)) => {}
            NbtTag::Long_Array((len, list)) => {}
        }

        Ok(())
    }
}

impl McDeserialize for NbtTag {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
        todo!()
    }
}