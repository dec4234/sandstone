use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::serialize_primitives;

impl McSerialize for String {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		VarInt(self.len() as i32).mc_serialize(serializer)?;
		serializer.serialize_bytes(self.as_bytes());

		Ok(())
	}
}

impl McDeserialize for String {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let var_output = VarInt::mc_deserialize(deserializer)?;
		let bounds: (usize, usize) = (deserializer.index, deserializer.index + var_output.0 as usize);

		if bounds.1 > deserializer.data.len() {
			return Err(SerializingErr::OutOfBounds);
		}

		let s = String::from_utf8(deserializer.data[bounds.0..bounds.1].to_vec())?;

		deserializer.increment(var_output.0 as usize); // length of string
		
		Ok(s)
	}
}

impl McSerialize for bool {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match *self {
			true => {serializer.serialize_u8(1)}
			false => {serializer.serialize_u8(0)}
		}

		Ok(())
	}
}

impl McDeserialize for bool {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, bool> {
		let b = u8::mc_deserialize(deserializer)?;

		match b {
			0 => {Ok(false)},
			1 => {Ok(true)},
			_ => {Err(SerializingErr::UniqueFailure("Byte received does not match any bool value.".to_string()))}
		}
	}
}

serialize_primitives!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, usize, isize);

#[macro_use]
mod macros {
	/// Used to implement McSerialize and McDeserialize for primitive types. These are ultimately
	/// the basis for all other types used in Rust.
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
                fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
                    if deserializer.data.len() == 0 {
                        return Err(SerializingErr::InputEnded);
                    }
					let length = std::mem::size_of::<$t>();

					if deserializer.index + length > deserializer.data.len() {
						return Err(SerializingErr::InputEnded);
					}
					
                    let split = deserializer.data[deserializer.index..].split_at(length);

                    let b = <$t>::from_be_bytes(split.0.try_into()?);
                    deserializer.increment(std::mem::size_of::<$t>());

                    return Ok(b);
                }
            }

            )*
        };
    }
}

impl<T: McSerialize> McSerialize for Vec<T> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> where T: McSerialize {
		for item in self {
			item.mc_serialize(serializer)?;
		}

		Ok(())
	}
}

impl<T: McDeserialize> McDeserialize for Vec<T> {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized, T: McDeserialize {
		let mut vec = vec![];

		while !deserializer.is_at_end() {
			vec.push(T::mc_deserialize(deserializer)?);
		}

		Ok(vec)
	}
}


impl<T: McSerialize> McSerialize for Option<T> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> where T: McSerialize {
		match self {
			Some(item) => {
				item.mc_serialize(serializer)?;
			}
			None => {}
		}

		Ok(())
	}
}

impl<T: McDeserialize> McDeserialize for Option<T> {
	/// Special Deserialization note for Options: The protocol uses Options in some places, usually preceded
	/// by a boolean indicating if the option is present. In order to handle these cases in a packet,
	/// you must make a custom struct for at least the Option field and the accompanying boolean field, and
	/// place them in the correct order in the packet. See [crate::protocol::packets::LoginPropertyElement] as an example.
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized, T: McDeserialize {
		if deserializer.is_at_end() {
			return Ok(None);
		}
		
		Ok(Some(T::mc_deserialize(deserializer)?))
	}
}