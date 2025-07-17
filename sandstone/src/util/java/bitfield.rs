//! A bit field is a fixed length primitive unsigned or signed integer that packs its data into 
//! individual bits.

use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use std::ops::{BitAnd, BitOr, Not, Shl, Shr, Sub};

/// A simple bit field internally represented as any primitive signed or unsigned integer.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BitField<T: BitFieldInteger + McSerialize + McDeserialize> {
	bits: T,
}

impl<T: BitFieldInteger + McSerialize + McDeserialize> BitField<T> {
	pub fn new(start: T) -> Self {
		Self { bits: start }
	}

	pub fn get_bit(&self, index: usize) -> bool {
		let mask = T::one() << index;
		(self.bits & mask) != T::zero()
	}

	pub fn set_bit(&mut self, index: usize, value: bool) {
		let mask = T::one() << index;
		if value {
			self.bits = self.bits | mask;
		} else {
			self.bits = self.bits & !mask;
		}
	}

	pub fn set_all(&mut self) {
		self.bits = T::max_value();
	}

	pub fn clear_all(&mut self) {
		self.bits = T::zero();
	}

	pub fn flip(&mut self) {
		self.bits = !self.bits;
	}

	pub fn slice(&self, start: usize, end: usize) -> BitField<T> {
		let width = end - start;
		let mask = ((T::one() << width) - T::one()) << start;
		let bits = (self.bits & mask) >> start;
		BitField::new(bits)
	}
}

impl <T: BitFieldInteger + McSerialize + McDeserialize> McSerialize for BitField<T> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.bits.mc_serialize(serializer)?;
		
		Ok(())
	}
}

impl <'a, T: BitFieldInteger + McSerialize + McDeserialize> McDeserialize for BitField<T> {
	fn mc_deserialize(deserializer: &mut McDeserializer) -> SerializingResult<'a, BitField<T>> {
		let bits = T::mc_deserialize(deserializer)?;
		
		Ok(Self { bits })
	}
}

impl <T: BitFieldInteger + McSerialize + McDeserialize + McDefault> McDefault for BitField<T> {
	fn mc_default() -> Self {
		BitField::new(T::mc_default())
	}
}

/// A number that can be used for a fixed-length bit field.
pub trait BitFieldInteger:
Copy
+ PartialEq
+ BitAnd<Output = Self>
+ BitOr<Output = Self>
+ Not<Output = Self>
+ Shl<usize, Output = Self>
+ Shr<usize, Output = Self>
+ Sub<Output = Self>
{
	fn zero() -> Self;
	fn one() -> Self;
	fn max_value() -> Self;
}

macro_rules! impl_bitfield_integer {
    ($($t:ty),*) => {
        $(
            impl BitFieldInteger for $t {
                fn zero() -> Self { 0 }
                fn one() -> Self { 1 }
                fn max_value() -> Self { <$t>::MAX }
            }
        )*
    };
}

impl_bitfield_integer!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

#[cfg(test)]
mod test {
	use crate::util::java::bitfield::BitField;

	#[test]
	fn test_basic_bitfield() {
		let mut bitfield = BitField::new(0);

		bitfield.set_bit(0, true);
		assert_eq!(bitfield.get_bit(0), true);

		bitfield.set_bit(1, true);
		assert_eq!(bitfield.get_bit(1), true);

		bitfield.set_bit(0, false);
		assert_eq!(bitfield.get_bit(0), false);

		bitfield.set_all();
		assert_eq!(bitfield.get_bit(0), true);
		assert_eq!(bitfield.get_bit(1), true);

		bitfield.clear_all();
		assert_eq!(bitfield.get_bit(0), false);
		assert_eq!(bitfield.get_bit(1), false);

		bitfield.flip();
		assert_eq!(bitfield.get_bit(0), true);
		assert_eq!(bitfield.get_bit(1), true);
	}
}



