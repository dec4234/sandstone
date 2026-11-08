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
		if var_output.0 < 0 {
			return Err(SerializingErr::OutOfBounds(format!("String length prefix of {} is negative", var_output.0)));
		}
		let bounds: (usize, usize) = (deserializer.index, deserializer.index + var_output.0 as usize);

		if bounds.1 > deserializer.data.len() {
			return Err(SerializingErr::OutOfBounds(format!(
				"String length prefix of {} exceeds remaining input length of {}",
				var_output.0,
				deserializer.data.len() - deserializer.index
			)));
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
			true => serializer.serialize_u8(1),
			false => serializer.serialize_u8(0),
		}

		Ok(())
	}
}

impl McDeserialize for bool {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, bool> {
		let b = u8::mc_deserialize(deserializer)?;

		match b {
			0 => Ok(false),
			1 => Ok(true),
			_ => Err(SerializingErr::UniqueFailure("Byte received does not match any bool value.".to_string())),
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
	/// Serialize the contents of the Vec<T> to the serializer without any length prefix.
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()>
	where
		T: McSerialize,
	{
		for item in self {
			item.mc_serialize(serializer)?;
		}

		Ok(())
	}
}

impl<T: McDeserialize> McDeserialize for Vec<T> {
	/// Deserializes a Vec<T> from the deserializer until the end of the input is reached.
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
		T: McDeserialize,
	{
		let mut vec = vec![];

		while !deserializer.is_at_end() {
			vec.push(T::mc_deserialize(deserializer)?);
		}

		Ok(vec)
	}
}

impl<T: McSerialize> McSerialize for Option<T> {
	/// Ensure that you mean to use this rather than a [PrefixedOptional<T>] This is a simple serialization
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()>
	where
		T: McSerialize,
	{
		if let Some(item) = self {
			item.mc_serialize(serializer)?;
		}

		Ok(())
	}
}

impl<T: McDeserialize> McDeserialize for Option<T> {
	/// Ensure that you mean to use this rather than a [PrefixedOptional<T>].
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
		T: McDeserialize,
	{
		if deserializer.is_at_end() {
			return Ok(None);
		}

		Ok(Some(T::mc_deserialize(deserializer)?))
	}
}

// not sure if this will ever be needed, but it's nice to have
impl<T: McSerialize> McSerialize for Box<T> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()>
	where
		T: McSerialize,
	{
		(**self).mc_serialize(serializer)
	}
}

impl<T: McDeserialize> McDeserialize for Box<T> {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
		T: McDeserialize,
	{
		Ok(Box::new(T::mc_deserialize(deserializer)?))
	}
}

impl<const N: usize> McSerialize for [u8; N] {
	/// Serialize a fixed-size byte array as-is, with no length prefix.
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		serializer.serialize_bytes(self);
		Ok(())
	}
}

impl<const N: usize> McDeserialize for [u8; N] {
	/// Deserialize exactly `N` bytes into a fixed-size byte array. Errors if fewer than `N` bytes remain.
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		match deserializer.slice_option(N) {
			Some(slice) => Ok(slice.try_into()?),
			None => Err(SerializingErr::InputEnded),
		}
	}
}

/// # PrefixedArray (Protocol Type)
/// A PrefixedArray is a Vec<T> with a VarInt prefix indicating the length of the array.
#[derive(Default, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct PrefixedArray<T: McSerialize + McDeserialize> {
	pub(crate) vec: Vec<T>,
}

impl<T: McDeserialize + McSerialize> PrefixedArray<T> {
	pub fn slice(&self) -> &[T] {
		&self.vec
	}
}

impl<T: McSerialize + McDeserialize> PrefixedArray<T> {
	pub fn new(vec: Vec<T>) -> Self {
		Self {
			vec,
		}
	}
}

impl<T: McSerialize + McDeserialize> McSerialize for PrefixedArray<T> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()>
	where
		T: McSerialize,
	{
		VarInt(self.vec.len() as i32).mc_serialize(serializer)?;
		for item in &self.vec {
			item.mc_serialize(serializer)?;
		}

		Ok(())
	}
}

impl<T: McSerialize + McDeserialize> McDeserialize for PrefixedArray<T> {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
		T: McDeserialize,
	{
		let var_output = VarInt::mc_deserialize(deserializer)?;
		if var_output.0 < 0 {
			return Err(SerializingErr::OutOfBounds(format!("PrefixedArray length prefix of {} is negative", var_output.0)));
		}
		let mut vec = Vec::with_capacity(var_output.0 as usize);

		for _ in 0..var_output.0 {
			vec.push(T::mc_deserialize(deserializer)?);
		}

		Ok(PrefixedArray::new(vec))
	}
}

impl<T: McSerialize + McDeserialize> From<Vec<T>> for PrefixedArray<T> {
	fn from(vec: Vec<T>) -> Self {
		PrefixedArray::new(vec)
	}
}

impl<T: McSerialize + McDeserialize> From<PrefixedArray<T>> for Vec<T> {
	fn from(prefixed_array: PrefixedArray<T>) -> Self {
		prefixed_array.vec
	}
}

impl<T: McSerialize + McDeserialize> From<Box<[T]>> for PrefixedArray<T> {
	fn from(boxed_slice: Box<[T]>) -> Self {
		PrefixedArray::new(boxed_slice.into_vec())
	}
}

impl<T: McSerialize + McDeserialize> From<PrefixedArray<T>> for Box<[T]> {
	fn from(prefixed_array: PrefixedArray<T>) -> Self {
		prefixed_array.vec.into_boxed_slice()
	}
}

/// # PrefixedOptional (Protocol Type)
/// An Optional<T> with a bool prefix indicating if the value is present.
/// This has started to replace most occurrences of Option<T> in the protocol, as it is more explicit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixedOptional<T: McSerialize + McDeserialize> {
	pub(crate) is_present: bool,
	pub(crate) value: Option<T>,
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
			value,
		}
	}
}

impl<T: McSerialize + McDeserialize> McSerialize for PrefixedOptional<T> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()>
	where
		T: McSerialize,
	{
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
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
		T: McDeserialize,
	{
		let is_present = u8::mc_deserialize(deserializer)? == 1;

		let value = if is_present { Some(T::mc_deserialize(deserializer)?) } else { None };

		Ok(PrefixedOptional {
			is_present,
			value,
		})
	}
}

impl<T: McSerialize + McDeserialize + McDefault> McDefault for PrefixedOptional<T> {
	fn mc_default() -> Self {
		PrefixedOptional {
			is_present: true,
			value: Some(T::mc_default()),
		}
	}
}

impl<T: McSerialize + McDeserialize> From<Option<T>> for PrefixedOptional<T> {
	fn from(option: Option<T>) -> Self {
		PrefixedOptional::new(option)
	}
}

impl<T: McSerialize + McDeserialize> From<PrefixedOptional<T>> for Option<T> {
	fn from(prefixed_optional: PrefixedOptional<T>) -> Self {
		prefixed_optional.value
	}
}

#[cfg(test)]
mod tests {
	use crate::protocol::serialization::serializer_error::SerializingErr;
	use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
	use crate::protocol::testing::McDefault;
	use crate::protocol_types::datatypes::var_types::VarInt;

	fn to_bytes<T: McSerialize>(value: &T) -> Vec<u8> {
		let mut serializer = McSerializer::new();
		value.mc_serialize(&mut serializer).unwrap();
		serializer.output
	}

	/// Protocol strings are a VarInt byte length followed by UTF-8 bytes.
	#[test]
	fn string_wire_format() {
		assert_eq!(to_bytes(&"abc".to_string()), vec![3, b'a', b'b', b'c']);
		assert_eq!(to_bytes(&"abc"), to_bytes(&"abc".to_string()), "&str must match String on the wire");

		// The prefix counts bytes, not chars.
		let s = "é".to_string();
		assert_eq!(to_bytes(&s), vec![2, 0xC3, 0xA9]);
		let bytes = to_bytes(&s);
		let mut de = McDeserializer::new(&bytes);
		assert_eq!(String::mc_deserialize(&mut de).unwrap(), s);
		assert!(de.is_at_end());
	}

	/// A length prefix larger than the input must error, not panic or read past the buffer.
	#[test]
	fn string_length_exceeds_input() {
		let data = [5u8, b'a', b'b'];
		let mut de = McDeserializer::new(&data);
		assert!(matches!(String::mc_deserialize(&mut de), Err(SerializingErr::OutOfBounds(_))));
	}

	/// Regression: a negative length prefix from a peer used to panic (add overflow). It must be an error.
	#[test]
	fn string_negative_length_errors() {
		let bytes = to_bytes(&VarInt(-1));
		let mut de = McDeserializer::new(&bytes);
		assert!(matches!(String::mc_deserialize(&mut de), Err(SerializingErr::OutOfBounds(_))));

		let bytes = to_bytes(&VarInt(i32::MIN));
		let mut de = McDeserializer::new(&bytes);
		assert!(matches!(String::mc_deserialize(&mut de), Err(SerializingErr::OutOfBounds(_))));
	}

	#[test]
	fn string_invalid_utf8() {
		let data = [2u8, 0xFF, 0xFE];
		let mut de = McDeserializer::new(&data);
		assert!(matches!(String::mc_deserialize(&mut de), Err(SerializingErr::CouldNotDeserializeString(_))));
	}

	/// The protocol only allows 0x00 and 0x01 for booleans.
	#[test]
	fn bool_wire_format_and_invalid_byte() {
		assert_eq!(to_bytes(&true), vec![1]);
		assert_eq!(to_bytes(&false), vec![0]);

		let data = [0u8, 1, 2];
		let mut de = McDeserializer::new(&data);
		assert!(!bool::mc_deserialize(&mut de).unwrap());
		assert!(bool::mc_deserialize(&mut de).unwrap());
		assert!(matches!(bool::mc_deserialize(&mut de), Err(SerializingErr::UniqueFailure(_))));
	}

	/// All fixed-size numbers in the protocol are big-endian.
	#[test]
	fn primitives_are_big_endian() {
		assert_eq!(to_bytes(&0x0102u16), vec![1, 2]);
		assert_eq!(to_bytes(&-2i32), vec![0xFF, 0xFF, 0xFF, 0xFE]);
		assert_eq!(to_bytes(&1.0f32), vec![0x3F, 0x80, 0, 0]);

		let data = [0u8, 0, 0, 0, 0, 0, 0, 0x2A];
		let mut de = McDeserializer::new(&data);
		assert_eq!(i64::mc_deserialize(&mut de).unwrap(), 42);
		assert!(de.is_at_end());
	}

	#[test]
	fn primitives_short_input_errors() {
		let empty: [u8; 0] = [];
		let mut de = McDeserializer::new(&empty);
		assert!(matches!(u8::mc_deserialize(&mut de), Err(SerializingErr::InputEnded)));

		let data = [1u8, 2, 3];
		let mut de = McDeserializer::new(&data);
		assert!(matches!(u32::mc_deserialize(&mut de), Err(SerializingErr::InputEnded)));
		assert_eq!(de.index, 0, "a failed read must not consume input");
	}

	/// A bare Vec<T> has no prefix and reads until the input ends. It must only be used as the last field.
	#[test]
	fn vec_has_no_prefix_and_reads_to_end() {
		let v: Vec<u16> = vec![1, 2, 3];
		let bytes = to_bytes(&v);
		assert_eq!(bytes, vec![0, 1, 0, 2, 0, 3]);
		let mut de = McDeserializer::new(&bytes);
		assert_eq!(Vec::<u16>::mc_deserialize(&mut de).unwrap(), v);

		// A trailing partial element is an error, not silently dropped.
		let partial = [0u8, 1, 0];
		let mut de = McDeserializer::new(&partial);
		assert!(Vec::<u16>::mc_deserialize(&mut de).is_err());
	}

	/// A bare Option<T> writes nothing for None and reads None only at the end of input.
	#[test]
	fn option_has_no_prefix() {
		assert!(to_bytes(&Option::<u8>::None).is_empty());
		assert_eq!(to_bytes(&Some(7u8)), vec![7]);

		let empty: [u8; 0] = [];
		let mut de = McDeserializer::new(&empty);
		assert_eq!(Option::<u8>::mc_deserialize(&mut de).unwrap(), None);

		let data = [7u8];
		let mut de = McDeserializer::new(&data);
		assert_eq!(Option::<u8>::mc_deserialize(&mut de).unwrap(), Some(7));
	}

	#[test]
	fn box_is_transparent() {
		let boxed = Box::new(0x1234u16);
		let bytes = to_bytes(&boxed);
		assert_eq!(bytes, to_bytes(&0x1234u16));
		let mut de = McDeserializer::new(&bytes);
		assert_eq!(Box::<u16>::mc_deserialize(&mut de).unwrap(), boxed);
	}

	#[test]
	fn fixed_byte_array() {
		let arr = [1u8, 2, 3, 4];
		assert_eq!(to_bytes(&arr), vec![1, 2, 3, 4]);

		let data = [1u8, 2, 3, 4, 5];
		let mut de = McDeserializer::new(&data);
		assert_eq!(<[u8; 4]>::mc_deserialize(&mut de).unwrap(), arr);
		assert_eq!(de.index, 4);
		assert!(matches!(<[u8; 4]>::mc_deserialize(&mut de), Err(SerializingErr::InputEnded)));
	}

	/// PrefixedArray is a VarInt element count followed by the elements.
	#[test]
	fn prefixed_array_wire_format() {
		let arr = PrefixedArray::new(vec![1u16, 2]);
		let bytes = to_bytes(&arr);
		assert_eq!(bytes, vec![2, 0, 1, 0, 2]);

		// Trailing bytes after the counted elements belong to the next field and must not be consumed.
		let mut with_trailing = bytes.clone();
		with_trailing.push(0xEE);
		let mut de = McDeserializer::new(&with_trailing);
		assert_eq!(PrefixedArray::<u16>::mc_deserialize(&mut de).unwrap(), arr);
		assert_eq!(de.collect_remaining(), &[0xEE]);

		assert_eq!(to_bytes(&PrefixedArray::<u16>::new(vec![])), vec![0]);
	}

	#[test]
	fn prefixed_array_count_exceeds_input() {
		let data = [3u8, 0, 1];
		let mut de = McDeserializer::new(&data);
		assert!(PrefixedArray::<u16>::mc_deserialize(&mut de).is_err());
	}

	/// Regression: a negative count from a peer used to panic in Vec::with_capacity (capacity overflow),
	/// also in release builds. It must be an error.
	#[test]
	fn prefixed_array_negative_count_errors() {
		let bytes = to_bytes(&VarInt(-1));
		let mut de = McDeserializer::new(&bytes);
		assert!(matches!(PrefixedArray::<u8>::mc_deserialize(&mut de), Err(SerializingErr::OutOfBounds(_))));

		let bytes = to_bytes(&VarInt(i32::MIN));
		let mut de = McDeserializer::new(&bytes);
		assert!(matches!(PrefixedArray::<u8>::mc_deserialize(&mut de), Err(SerializingErr::OutOfBounds(_))));
	}

	#[test]
	fn prefixed_array_conversions() {
		let arr: PrefixedArray<u8> = vec![1, 2].into();
		assert_eq!(arr.slice(), &[1, 2]);
		let boxed: Box<[u8]> = arr.clone().into();
		assert_eq!(&*boxed, &[1, 2]);
		let back: PrefixedArray<u8> = boxed.into();
		assert_eq!(back, arr);
		let v: Vec<u8> = back.into();
		assert_eq!(v, vec![1, 2]);
	}

	/// PrefixedOptional is a bool presence byte, then the value only if present.
	#[test]
	fn prefixed_optional_wire_format() {
		let some = PrefixedOptional::new(Some(9u8));
		let none = PrefixedOptional::<u8>::new(None);
		assert!(some.is_present());
		assert!(!none.is_present());
		assert_eq!(some.value(), Some(&9));
		assert_eq!(none.value(), None);

		assert_eq!(to_bytes(&some), vec![1, 9]);
		assert_eq!(to_bytes(&none), vec![0]);

		let data = [0u8, 1, 9];
		let mut de = McDeserializer::new(&data);
		assert_eq!(PrefixedOptional::<u8>::mc_deserialize(&mut de).unwrap(), none);
		assert_eq!(PrefixedOptional::<u8>::mc_deserialize(&mut de).unwrap(), some);
		assert!(de.is_at_end());
	}

	#[test]
	fn prefixed_optional_present_but_missing_value() {
		let data = [1u8];
		let mut de = McDeserializer::new(&data);
		assert!(PrefixedOptional::<u8>::mc_deserialize(&mut de).is_err());
	}

	/// mc_default must produce a present value so round-trip tests exercise the inner type.
	#[test]
	fn prefixed_optional_default_and_conversions() {
		let def = PrefixedOptional::<u8>::mc_default();
		assert!(def.is_present());
		assert!(def.value().is_some());

		let from: PrefixedOptional<u8> = Some(3u8).into();
		assert_eq!(from, PrefixedOptional::new(Some(3)));
		let back: Option<u8> = from.into();
		assert_eq!(back, Some(3));
	}
}
