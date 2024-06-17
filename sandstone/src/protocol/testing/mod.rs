pub mod packet_testing;
mod primitive_serialization_testing;
mod benchmarking;

// TODO: derive macro
/// A trait that defines the default value for a type. This is used for testing packet serialization.
/// This is different from the typical `Default` trait in Rust, as it returns more random and meaningful
/// values.
pub trait McDefault {
	fn default() -> Self;
}

impl McDefault for String {
	fn default() -> Self {
		"h14og3rob38g*@Hge3bp`2*GD".to_string()
	}
}

impl McDefault for u8 {
	fn default() -> Self {
		35
	}
}

impl McDefault for u16 {
	fn default() -> Self {
		1234
	}
}

impl McDefault for u32 {
	fn default() -> Self {
		1343487356
	}
}

impl McDefault for u64 {
	fn default() -> Self {
		1234567890123456789
	}
}

impl McDefault for u128 {
	fn default() -> Self {
		1234567890123456789012345601234567890
	}
}

impl McDefault for i8 {
	fn default() -> Self {
		-35
	}
}

impl McDefault for i16 {
	fn default() -> Self {
		2394
	}
}

impl McDefault for i32 {
	fn default() -> Self {
		-1234567890
	}
}

impl McDefault for i64 {
	fn default() -> Self {
		-1234567890123456789
	}
}

impl McDefault for i128 {
	fn default() -> Self {
		-1234567890123456789012345601234567890
	}
}

impl McDefault for f32 {
	fn default() -> Self {
		1387137.48347
	}
}

impl McDefault for f64 {
	fn default() -> Self {
		7.34873493946394
	}
}

impl McDefault for usize {
	fn default() -> Self {
		1234567890123456789
	}
}

impl McDefault for isize {
	fn default() -> Self {
		-1234567890123456789
	}
}

impl<T: McDefault> McDefault for Vec<T> {
	fn default() -> Self {
		vec![T::default(), T::default(), T::default()]
	}
}

impl<T: McDefault> McDefault for Option<T> {
	fn default() -> Self {
		Some(T::default())
	}
}

impl McDefault for bool {
	fn default() -> Self {
		true
	}
}

// TODO: serialization impls for Box?
impl<T: McDefault> McDefault for Box<T> {
	fn default() -> Self {
		Box::new(T::default())
	}
}

