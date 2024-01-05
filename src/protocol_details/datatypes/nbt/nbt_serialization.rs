use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol_details::datatypes::nbt::nbt::NbtTag;

//TODO: Same for NamedTag
impl McSerialize for NbtTag {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {

        Ok(())
    }
}

impl McDeserialize for NbtTag {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
        todo!()
    }
}