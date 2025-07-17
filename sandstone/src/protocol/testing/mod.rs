#![allow(unused_imports)]

use crate::protocol;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol_types::datatypes::var_types::{VarInt, VarLong};
use uuid::Uuid;

pub mod packet_testing;
mod primitive_serialization_testing;
mod benchmarking;
mod derive_testing;

// TODO: derive macro
/// A trait that defines the default value for a type. This is used for testing packet serialization.
/// This is different from the typical `Default` trait in Rust, as it returns more random and meaningful
/// values.
pub trait McDefault {
	fn mc_default() -> Self;
}

impl McDefault for String {
	fn mc_default() -> Self {
		"h14og3rob38g*@Hge3bp`2*GD".to_string()
	}
}

impl McDefault for u8 {
	fn mc_default() -> Self {
		35
	}
}

impl McDefault for u16 {
	fn mc_default() -> Self {
		1234
	}
}

impl McDefault for u32 {
	fn mc_default() -> Self {
		1343487356
	}
}

impl McDefault for u64 {
	fn mc_default() -> Self {
		1234567890123456789
	}
}

impl McDefault for u128 {
	fn mc_default() -> Self {
		1234567890123456789012345601234567890
	}
}

impl McDefault for i8 {
	fn mc_default() -> Self {
		-35
	}
}

impl McDefault for i16 {
	fn mc_default() -> Self {
		2394
	}
}

impl McDefault for i32 {
	fn mc_default() -> Self {
		-1234567890
	}
}

impl McDefault for i64 {
	fn mc_default() -> Self {
		-1234567890123456789
	}
}

impl McDefault for i128 {
	fn mc_default() -> Self {
		-1234567890123456789012345601234567890
	}
}

impl McDefault for f32 {
	fn mc_default() -> Self {
		1387137.48347
	}
}

impl McDefault for f64 {
	fn mc_default() -> Self {
		7.34873493946394
	}
}

impl McDefault for usize {
	fn mc_default() -> Self {
		1234567890123456789
	}
}

impl McDefault for isize {
	fn mc_default() -> Self {
		-1234567890123456789
	}
}

impl<T: McDefault> McDefault for Vec<T> {
	fn mc_default() -> Self {
		vec![T::mc_default(), T::mc_default(), T::mc_default()]
	}
}

impl<T: McDefault> McDefault for Option<T> {
	fn mc_default() -> Self {
		Some(T::mc_default())
	}
}

impl McDefault for bool {
	fn mc_default() -> Self {
		true
	}
}

impl<T: McDefault> McDefault for Box<T> {
	fn mc_default() -> Self {
		Box::new(T::mc_default())
	}
}

impl McDefault for VarInt {
	fn mc_default() -> Self {
		VarInt(1234567890)
	}
}

impl McDefault for VarLong {
	fn mc_default() -> Self {
		VarLong(1234567890123456789)
	}
}

impl McDefault for Uuid {
	fn mc_default() -> Self {
		Uuid::from_u128(12345678901234567890123456789012)
	}
}

impl<T: McDefault + protocol::serialization::McDeserialize + protocol::serialization::McSerialize> McDefault for PrefixedArray<T> {
	fn mc_default() -> Self {
		PrefixedArray::new(vec![T::mc_default(), T::mc_default(), T::mc_default()])
	}
}