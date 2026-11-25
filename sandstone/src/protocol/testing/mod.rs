#![allow(unused_imports)]

//! Traits and implementations for testing packet serialization and deserialization.

use crate::protocol;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol_types::datatypes::var_types::{VarInt, VarLong};
use uuid::Uuid;

#[cfg(test)]
mod benchmarking;
#[cfg(test)]
mod derive_testing;
#[cfg(test)]
pub mod packet_testing;
mod primitive_serialization_testing;

// TODO: derive macro
/// A trait that defines the default value for a type. This is used for testing packet serialization.
/// This is different from the typical `Default` trait in Rust, as it returns more random and meaningful
/// values.
///
/// `mc_default` exists only in tests or with the `testing` feature. The trait itself always exists, so
/// `#[derive(McDefault)]` and the imports compile in all builds. Each manual impl must also put
/// `#[cfg(any(test, feature = "testing"))]` on its `mc_default`, or it does not compile without the feature.
pub trait McDefault {
	/// Return the default for a particular type, used for testing.
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self;
}

impl McDefault for String {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		"h14og3rob38g*@Hge3bp`2*GD".to_string()
	}
}

impl McDefault for u8 {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		35
	}
}

impl McDefault for u16 {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		1234
	}
}

impl McDefault for u32 {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		1343487356
	}
}

impl McDefault for u64 {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		1234567890123456789
	}
}

impl McDefault for u128 {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		1234567890123456789012345601234567890
	}
}

impl McDefault for i8 {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		-35
	}
}

impl McDefault for i16 {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		2394
	}
}

impl McDefault for i32 {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		-1234567890
	}
}

impl McDefault for i64 {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		-1234567890123456789
	}
}

impl McDefault for i128 {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		-1234567890123456789012345601234567890
	}
}

impl McDefault for f32 {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		1387137.48347
	}
}

impl McDefault for f64 {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		7.34873493946394
	}
}

impl McDefault for usize {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		1234567890123456789
	}
}

impl McDefault for isize {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		-1234567890123456789
	}
}

impl<T: McDefault> McDefault for Vec<T> {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		vec![T::mc_default(), T::mc_default(), T::mc_default()]
	}
}

impl<const N: usize> McDefault for [u8; N] {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		[u8::mc_default(); N]
	}
}

impl<T: McDefault> McDefault for Option<T> {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		Some(T::mc_default())
	}
}

impl McDefault for bool {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		true
	}
}

impl<T: McDefault> McDefault for Box<T> {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		Box::new(T::mc_default())
	}
}

impl McDefault for VarInt {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		VarInt(1234567890)
	}
}

impl McDefault for VarLong {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		VarLong(1234567890123456789)
	}
}

impl McDefault for Uuid {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		Uuid::from_u128(12345678901234567890123456789012)
	}
}

impl<T: McDefault + protocol::serialization::McDeserialize + protocol::serialization::McSerialize> McDefault for PrefixedArray<T> {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		PrefixedArray::new(vec![T::mc_default(), T::mc_default(), T::mc_default()])
	}
}
