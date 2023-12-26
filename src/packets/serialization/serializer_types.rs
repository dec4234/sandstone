use crate::packets::serialization::serialize_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McSerialize, McSerializer};
use crate::protocol_details::datatypes::var_types::VarInt;


impl McSerialize for String {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
        VarInt(self.len() as i32).mc_serialize(serializer)?;
        serializer.serialize_bytes(self.as_bytes());

        Ok(())
    }
}

impl McDeserialize for String {
    fn mc_deserialize(input: &mut [u8]) -> DeserializeResult<String> {
        let var_output = VarInt::mc_deserialize(input)?;
        let bounds: (usize, usize) = (var_output.0.to_bytes().len(), var_output.0.0 as usize + 1);
        let s = String::from_utf8(input[bounds.0..bounds.1].to_vec());

        if let Ok(s) = s {
            Ok((s, &input[bounds.1..]))
        } else {
            Err(SerializingErr::CouldNotDeserializeString)
        }
    }
}

impl McSerialize for u8 {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
        serializer.serialize_u8(*self);

        Ok(())
    }
}

impl McDeserialize for u8 {
    fn mc_deserialize(input: &mut [u8]) -> DeserializeResult<u8> {
        if input.len() == 0 {
            return Err(SerializingErr::InputEnded);
        }

        return Ok((input[0], &input[1..]));
    }
}