//! A bit field is a fixed length primitive unsigned or signed integer that packs its data into
//! individual bits.

use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use std::ops::{BitAnd, BitOr, Not, Shl, Shr, Sub};

/// A simple bit field internally represented by any primitive signed or unsigned integer.
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct BitField<T: BitFieldInteger + McSerialize + McDeserialize> {
	bits: T,
}

impl<T: BitFieldInteger + McSerialize + McDeserialize> BitField<T> {
	/// Create a new BitField with the given starting value.
	pub fn new(start: T) -> Self {
		Self {
			bits: start,
		}
	}

	/// Get the bit at a paritcular index in the BitField
	pub fn get_bit(&self, index: usize) -> bool {
		let mask = T::one() << index;
		(self.bits & mask) != T::zero()
	}

	/// Set the bit at a particular location.
	///
	/// # Parameters
	/// - value: true to set the bit to 1, false to set the bit to 0
	pub fn set_bit(&mut self, index: usize, value: bool) {
		let mask = T::one() << index;
		if value {
			self.bits = self.bits | mask;
		} else {
			self.bits = self.bits & !mask;
		}
	}

	/// Set all bits in the bitfield to 1.
	pub fn set_all(&mut self) {
		self.bits = T::max_value();
	}

	/// Set all bits in the bitfield to 0.
	pub fn clear_all(&mut self) {
		self.bits = T::zero();
	}

	/// Flip all bits in the bitfield to their opposite value.
	pub fn flip(&mut self) {
		self.bits = !self.bits;
	}

	/// Given a particular start and end index, slice the bitfield into a sub-bitfield.
	///
	/// # Parameters
	/// - start: The starting index (inclusive)
	/// - end: The ending index (exclusive)
	pub fn slice(&self, start: usize, end: usize) -> BitField<T> {
		let width = end - start;
		let mask = ((T::one() << width) - T::one()) << start;
		let bits = (self.bits & mask) >> start;
		BitField::new(bits)
	}
}

impl<T: BitFieldInteger + McSerialize + McDeserialize> McSerialize for BitField<T> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.bits.mc_serialize(serializer)?;

		Ok(())
	}
}

impl<'a, T: BitFieldInteger + McSerialize + McDeserialize> McDeserialize for BitField<T> {
	fn mc_deserialize(deserializer: &mut McDeserializer) -> SerializingResult<'a, BitField<T>> {
		let bits = T::mc_deserialize(deserializer)?;

		Ok(Self {
			bits,
		})
	}
}

impl<T: BitFieldInteger + McSerialize + McDeserialize + McDefault> McDefault for BitField<T> {
	fn mc_default() -> Self {
		BitField::new(T::mc_default())
	}
}

/// Define a struct backed by a [`BitField`] from just a list of flag names.
///
/// Each flag is assigned a bit index in declaration order (the first flag is bit 0).
/// The macro generates the struct, a `new(...)` constructor taking each flag as a `bool`,
/// a getter `flag(&self) -> bool` and a setter `set_flag(&mut self, value: bool)` per flag.
///
/// The backing integer type may be specified explicitly; it defaults to `u8`.
///
/// # Examples
/// ```ignore
/// bitflag!(PlayerInputFlags: u8 {
///     forward, backward, left, right, jumping, sneaking, sprinting
/// });
/// ```
#[macro_export]
macro_rules! bitflag {
	($name:ident { $($flag:ident),* $(,)? }) => {
		$crate::bitflag!($name: u8 { $($flag),* });
	};
	($name:ident: $repr:ty { $($flag:ident),* $(,)? }) => {
		#[derive(Default, ::sandstone_derive::McSerialize, ::sandstone_derive::McDeserialize, Debug, Clone, PartialEq)]
		pub struct $name {
			pub flags: $crate::util::java::bitfield::BitField<$repr>,
		}

		impl $name {
			#[allow(clippy::too_many_arguments)]
			pub fn new($($flag: bool),*) -> Self {
				let mut flags = $crate::util::java::bitfield::BitField::new(0);
				let mut index = 0usize;
				$(
					flags.set_bit(index, $flag);
					index += 1;
				)*
				let _ = index;
				Self { flags }
			}

			$crate::bitflag!(@methods 0usize; $($flag),*);
		}

		impl $crate::protocol::testing::McDefault for $name {
			fn mc_default() -> Self {
				Self {
					flags: $crate::util::java::bitfield::BitField::new(0),
				}
			}
		}
	};
	(@methods $index:expr; ) => {};
	(@methods $index:expr; $flag:ident $(, $rest:ident)*) => {
		paste::paste! {
			pub fn $flag(&self) -> bool {
				self.flags.get_bit($index)
			}

			pub fn [<set_ $flag>](&mut self, value: bool) {
				self.flags.set_bit($index, value);
			}
		}

		$crate::bitflag!(@methods $index + 1usize; $($rest),*);
	};
}

/// A number that can be used for a fixed-length bit field.
pub trait BitFieldInteger: Copy + PartialEq + BitAnd<Output = Self> + BitOr<Output = Self> + Not<Output = Self> + Shl<usize, Output = Self> + Shr<usize, Output = Self> + Sub<Output = Self> {
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

	// The generated getters/setters must map each flag to its declaration-order bit index;
	// a wrong index would silently corrupt the wire format, so assert the raw bits directly.
	#[test]
	fn test_bitflag_macro() {
		use crate::protocol::serialization::serializer_error::SerializingErr;
		use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
		crate::bitflag!(TestFlags: u8 { a, b, c });

		// `new` assigns flags to bits 0, 1, 2 in order.
		let f = TestFlags::new(true, false, true);
		assert_eq!(f.flags.get_bit(0), true);
		assert_eq!(f.flags.get_bit(1), false);
		assert_eq!(f.flags.get_bit(2), true);

		assert_eq!(f.a(), true);
		assert_eq!(f.b(), false);
		assert_eq!(f.c(), true);

		// Setters target the matching bit.
		let mut f = TestFlags::new(false, false, false);
		f.set_b(true);
		assert_eq!(f.flags.get_bit(1), true);
		assert_eq!(f.b(), true);
		assert_eq!(f.a(), false);
		assert_eq!(f.c(), false);
	}
}
