use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol_details::datatypes::var_types::VarInt;
use crate::serialize_primitives;

impl McSerialize for String {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
        VarInt(self.len() as i32).mc_serialize(serializer)?;
        serializer.serialize_bytes(self.as_bytes());

        Ok(())
    }
}

impl McDeserialize for String {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
        let var_output = VarInt::mc_deserialize(deserializer)?;
        let bounds: (usize, usize) = (deserializer.index, deserializer.index + var_output.0 as usize);

        if bounds.1 > deserializer.data.len() {
            return Err(SerializingErr::CouldNotDeserializeString);
        }

        let s = String::from_utf8(deserializer.data[bounds.0..bounds.1].to_vec());

        deserializer.increment(var_output.0 as usize); // length of string

        if let Ok(s) = s {
            Ok(s)
        } else {
            Err(SerializingErr::CouldNotDeserializeString)
        }
    }
}

impl McSerialize for bool {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
        match *self {
            true => {serializer.serialize_u8(1)}
            false => {serializer.serialize_u8(0)}
        }

        Ok(())
    }
}

impl McDeserialize for bool {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, bool> {
        let b = u8::mc_deserialize(deserializer)?;

        match b {
            0 => {Ok(false)},
            1 => {Ok(true)},
            _ => {Err(SerializingErr::UniqueFailure("Byte received does not match any bool value.".to_string()))}
        }
    }
}

serialize_primitives!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! serialize_primitives {
        ($($t: ty),*) => {
            $(
            impl McSerialize for $t {
                fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
                    for b in self.to_be_bytes() {
                        serializer.serialize_u8(b);
                    }

                    Ok(())
                }
            }

            impl McDeserialize for $t {
                fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
                    if deserializer.data.len() == 0 {
                        return Err(SerializingErr::InputEnded);
                    }

                    let split = deserializer.data[deserializer.index..].split_at(std::mem::size_of::<$t>());

                    let b = <$t>::from_be_bytes(split.0.try_into().unwrap());
                    deserializer.increment(std::mem::size_of::<$t>());

                    return Ok(b);
                }
            }

            )*
        };
    }
}