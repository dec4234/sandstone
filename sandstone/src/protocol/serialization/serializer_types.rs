//! Implementations of the McSerialize and McDeserialize traits for primitive types and some common Rust types.

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::serialize_primitives;

impl McSerialize for String {
	/// Serializes a String as a VarInt length prefix followed by the UTF-8 bytes of the string.
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

impl McSerialize for &str {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.to_string().mc_serialize(serializer)
	}
}

impl McSerialize for bool {
	/// Serializes a bool as a single byte, 1 for true and 0 for false.
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
	/// Internal Only. Used to implement McSerialize and McDeserialize for primitive types. These are ultimately
	/// the basis for all other types used in Rust.
	#[macro_export]
	macro_rules! serialize_primitives {
        ($($t: ty),*) => {
            $(
            impl McSerialize for $t {
				/// Get the primitive type as a set of big endian bytes and serialize it to the serializer.
                fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
                    for b in self.to_be_bytes() {
                        serializer.serialize_u8(b);
                    }

                    Ok(())
                }
            }

            impl McDeserialize for $t {
				/// Deserialize a primitive type from the deserializer. This assumes that the primitive type is of
				/// fixed size and is in big-endian format.
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
	/// Deserializes a Vec<T> from the deserializer until the end of the input is reached.
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized, T: McDeserialize {
		let mut vec = vec![];

		while !deserializer.is_at_end() {
			vec.push(T::mc_deserialize(deserializer)?);
		}

		Ok(vec)
	}
}


impl<T: McSerialize> McSerialize for Option<T> {
	/// Ensure that you mean to use this rather than a [PrefixedOptional<T>] This is a simple serialization
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
	/// Ensure that you mean to use this rather than a [PrefixedOptional<T>].
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized, T: McDeserialize {
		if deserializer.is_at_end() {
			return Ok(None);
		}
		
		Ok(Some(T::mc_deserialize(deserializer)?))
	}
}

// not sure if this will ever be needed, but it's nice to have
impl<T: McSerialize> McSerialize for Box<T> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> where T: McSerialize {
		(**self).mc_serialize(serializer)
	}
}

impl<T: McDeserialize> McDeserialize for Box<T> {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized, T: McDeserialize {
		Ok(Box::new(T::mc_deserialize(deserializer)?))
	}
}

/// A PrefixedArray is a Vec<T> with a VarInt prefix indicating the length of the array. This is a protocol type.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct PrefixedArray<T: McSerialize + McDeserialize> {
	pub(crate) vec: Vec<T>
}

impl<T: McDeserialize + McSerialize> PrefixedArray<T> {
	pub fn slice(&self) -> &[T] {
		&self.vec
	}
}

impl<T: McSerialize + McDeserialize> PrefixedArray<T> {
	pub fn new(vec: Vec<T>) -> Self {
		Self {
			vec
		}
	}
}

impl<T: McSerialize + McDeserialize> McSerialize for PrefixedArray<T> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> where T: McSerialize {
		VarInt(self.vec.len() as i32).mc_serialize(serializer)?;
		for item in &self.vec {
			item.mc_serialize(serializer)?;
		}

		Ok(())
	}
}

impl<T: McSerialize + McDeserialize> McDeserialize for PrefixedArray<T> {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized, T: McDeserialize {
		let var_output = VarInt::mc_deserialize(deserializer)?;
		let mut vec = Vec::with_capacity(var_output.0 as usize);

		for _ in 0..var_output.0 {
			vec.push(T::mc_deserialize(deserializer)?);
		}

		Ok(PrefixedArray::new(vec))
	}
}

/// An Optional<T> with a bool prefix indicating if the value is present. This is a protocol type.
/// This has started to replace most occurrences of Option<T> in the protocol, as it is more explicit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixedOptional<T: McSerialize + McDeserialize> {
	pub(crate) is_present: bool,
	pub(crate) value: Option<T>
}

impl<T: McDeserialize + McSerialize> PrefixedOptional<T> {
	pub fn is_present(&self) -> bool {
		self.is_present
	}

	/// The equivalent Option<T>
	pub fn value(&self) -> Option<&T> {
		self.value.as_ref()
	}
}

impl<T: McSerialize + McDeserialize> PrefixedOptional<T> {
	pub fn new(value: Option<T>) -> Self {
		let is_present = value.is_some();
		Self {
			is_present,
			value
		}
	}
}

impl<T: McSerialize + McDeserialize> McSerialize for PrefixedOptional<T> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> where T: McSerialize {
		if self.is_present {
			serializer.serialize_u8(1);
			self.value.as_ref().unwrap().mc_serialize(serializer)?;
		} else {
			serializer.serialize_u8(0);
		}

		Ok(())
	}
}

impl<T: McSerialize + McDeserialize> McDeserialize for PrefixedOptional<T> {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized, T: McDeserialize {
		let is_present = u8::mc_deserialize(deserializer)? == 1;

		let value = if is_present {
			Some(T::mc_deserialize(deserializer)?)
		} else {
			None
		};

		Ok(PrefixedOptional {
			is_present,
			value
		})
	}
}

impl <T: McSerialize + McDeserialize + McDefault> McDefault for PrefixedOptional<T> {
	fn mc_default() -> Self {
		PrefixedOptional {
			is_present: true,
			value: Some(T::mc_default())
		}
	}
}