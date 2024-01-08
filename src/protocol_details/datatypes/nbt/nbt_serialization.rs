use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol_details::datatypes::nbt::nbt::NbtTag;

impl McSerialize for NbtTag {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
        serializer.serialize_u8(self.type_id()); // All start by serializing their type id

        match self {
            NbtTag::End => {}
            NbtTag::Byte(_) => {}
            NbtTag::Short(_) => {}
            NbtTag::Int(_) => {}
            NbtTag::Long(_) => {}
            NbtTag::Float(_) => {}
            NbtTag::Double(_) => {}
            NbtTag::String(_) => {}
            NbtTag::List(_) => {}
            NbtTag::Compound(_) => {}
            NbtTag::Byte_Array(_) => {}
            NbtTag::Int_Array(_) => {}
            NbtTag::Long_Array(_) => {}
        }

        Ok(())
    }
}

impl McDeserialize for NbtTag {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
        todo!()
    }
}