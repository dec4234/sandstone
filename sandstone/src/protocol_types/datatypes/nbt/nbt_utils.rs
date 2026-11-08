//! Macro utils for NBT.

#[macro_use]
mod macros {
	/// Create an [`NbtCompound`](crate::protocol_types::datatypes::nbt::nbt::NbtCompound) without a
	/// root name, analogous to how `map!` creates a `HashMap`. Keys are anything `Into<String>` and
	/// values are anything `Into<NbtTag>`.
	#[macro_export]
	macro_rules! nbt_compound {
        ($($k:expr => $v:expr),* $(,)?) => {
            {
                let mut compound = $crate::protocol_types::datatypes::nbt::NbtCompound::new_no_name();
                $(compound.add($k, $v);)*
                compound
            }
        };
    }

	/// Used to generate the NbtValue trait for primitive types
	#[macro_export]
	macro_rules! primvalue_nbtvalue {
        ($(($t: ty, $name: ident)),*) => {
            $(
            impl Into<NbtTag> for $t {
                fn into(self) -> NbtTag {
                    NbtTag::$name(self)
                }
            }

            impl TryFrom<NbtTag> for $t {
                type Error = $crate::protocol_types::datatypes::nbt::nbt_error::NbtError;

                fn try_from(tag: NbtTag) -> Result<Self, Self::Error> {
                    match tag {
                        NbtTag::$name(val) => Ok(val),
                        _ => Err($crate::protocol_types::datatypes::nbt::nbt_error::NbtError::InvalidType)
                    }
                }
            }

            impl TryFrom<NbtTag> for Option<$t> {
                type Error = $crate::protocol_types::datatypes::nbt::nbt_error::NbtError;

                fn try_from(tag: NbtTag) -> Result<Self, Self::Error> {
                    match tag {
                        NbtTag::$name(val) => Ok(Some(val)),
                        _ => Ok(None)
                    }
                }
            }
        )*
        };
}
	/// Used to generate the NbtValue trait for list types such as bytearray, intarray, and longarray
	#[macro_export]
	macro_rules! list_nbtvalue {
        ($(($t: ty, $name: ident, $fancyname: ident, $num: literal)),*) => {
            $(
                #[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
                pub struct $fancyname {
                    pub list: Vec<$t>,
                }

                impl $fancyname {
                    pub fn new(list: Vec<$t>) -> Self {
                        Self {
                            list,
                        }
                    }
                }

                impl IntoIterator for $fancyname {
                    type Item = $t;
                    type IntoIter = std::vec::IntoIter<$t>;

                    fn into_iter(self) -> Self::IntoIter {
                        self.list.into_iter()
                    }
                }

                impl<'a> IntoIterator for &'a $fancyname {
                    type Item = &'a $t;
                    type IntoIter = std::slice::Iter<'a, $t>;

                    fn into_iter(self) -> Self::IntoIter {
                        self.list.iter()
                    }
                }

                impl McSerialize for $fancyname {
                    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
                        (self.list.len() as u32).mc_serialize(serializer)?;
                        for tag in &self.list {
                            tag.mc_serialize(serializer)?;
                        }
                        Ok(())
                    }
                }

                impl McDeserialize for $fancyname {
                    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, $fancyname> {
                        let length = i32::mc_deserialize(deserializer)?;
                        let mut bytes = vec![];

                        for _ in 0..length {
                            let u = <$t>::mc_deserialize(deserializer);

                            if let Ok(u) = u {
                                bytes.push(u);
                            } else {
                                return Err(SerializingErr::UniqueFailure("Could not find expected element to deserialize".to_string()));
                            }
                        }

                        return Ok($fancyname::new(bytes));
                    }
                }

                impl Into<NbtTag> for $fancyname {
                    fn into(self) -> NbtTag {
                        NbtTag::$name(self)
                    }
                }

                impl TryFrom<NbtTag> for $fancyname {
                    type Error = $crate::protocol_types::datatypes::nbt::nbt_error::NbtError;

                    fn try_from(tag: NbtTag) -> Result<Self, Self::Error> {
                        match tag {
                            NbtTag::$name(val) => Ok(val),
                            _ => Err($crate::protocol_types::datatypes::nbt::nbt_error::NbtError::InvalidType)
                        }
                    }
                }

                impl From<NbtTag> for Option<$fancyname> {
                    fn from(tag: NbtTag) -> Self {
                        match tag {
                            NbtTag::$name(val) => Some(val),
                            _ => None
                        }
                    }
                }
            )*
        };
    }
}
#[cfg(test)]
mod tests {
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
	use crate::protocol_types::datatypes::nbt::nbt_error::NbtError;
	use crate::protocol_types::datatypes::nbt::{NbtByteArray, NbtCompound, NbtIntArray, NbtLongArray, NbtTag};

	/// nbt_compound! must be equal to a compound built by hand with add(), and accept a trailing comma.
	#[test]
	fn nbt_compound_macro_matches_manual_build() {
		let from_macro = nbt_compound! {
			"a" => 1i32,
			"b".to_string() => "text",
		};
		let mut manual = NbtCompound::new_no_name();
		manual.add("a", 1i32);
		manual.add("b", "text");
		assert_eq!(from_macro, manual);
		assert_eq!(from_macro.root_name, None);
	}

	/// Each primitive maps to exactly one tag type. A strict conversion from the wrong tag must fail.
	#[test]
	fn primitive_tag_conversions() {
		let cases: Vec<(NbtTag, u8)> = vec![(5i8.into(), 1), (5i16.into(), 2), (5i32.into(), 3), (5i64.into(), 4), (5f32.into(), 5), (5f64.into(), 6)];
		for (tag, id) in cases {
			assert_eq!(tag.get_type_id(), id);
		}

		assert_eq!(i32::try_from(NbtTag::Int(7)).unwrap(), 7);
		assert_eq!(f64::try_from(NbtTag::Double(1.5)).unwrap(), 1.5);
		assert!(matches!(i32::try_from(NbtTag::Long(7)), Err(NbtError::InvalidType)));
		assert!(matches!(i8::try_from(NbtTag::Short(7)), Err(NbtError::InvalidType)));
	}

	/// The Option conversions are lenient: a wrong tag type is "absent", not an error.
	#[test]
	fn optional_primitive_tag_conversions() {
		assert_eq!(Option::<i16>::try_from(NbtTag::Short(3)).unwrap(), Some(3));
		assert_eq!(Option::<i16>::try_from(NbtTag::Int(3)).unwrap(), None);
		assert_eq!(Option::<f32>::try_from(NbtTag::String("x".into())).unwrap(), None);
	}

	#[test]
	fn array_tag_conversions() {
		let bytes = NbtByteArray::new(vec![1, -2]);
		let ints = NbtIntArray::new(vec![3]);
		let longs = NbtLongArray::new(vec![4]);

		let byte_tag: NbtTag = bytes.clone().into();
		let int_tag: NbtTag = ints.clone().into();
		let long_tag: NbtTag = longs.clone().into();
		assert_eq!(byte_tag.get_type_id(), 7);
		assert_eq!(int_tag.get_type_id(), 11);
		assert_eq!(long_tag.get_type_id(), 12);

		assert_eq!(NbtByteArray::try_from(byte_tag.clone()).unwrap(), bytes);
		assert!(matches!(NbtIntArray::try_from(long_tag.clone()), Err(NbtError::InvalidType)));

		assert_eq!(Option::<NbtLongArray>::from(long_tag), Some(longs));
		assert_eq!(Option::<NbtIntArray>::from(byte_tag), None);
	}

	#[test]
	fn array_iteration() {
		let ints = NbtIntArray::new(vec![1, 2, 3]);
		let borrowed: Vec<&i32> = (&ints).into_iter().collect();
		assert_eq!(borrowed, vec![&1, &2, &3]);
		let owned: Vec<i32> = ints.into_iter().collect();
		assert_eq!(owned, vec![1, 2, 3]);
	}

	/// NBT arrays are a big-endian 4-byte length followed by big-endian elements.
	#[test]
	fn array_wire_format_round_trip() {
		let arr = NbtIntArray::new(vec![1, -1]);
		let mut serializer = McSerializer::new();
		arr.mc_serialize(&mut serializer).unwrap();
		assert_eq!(serializer.as_bytes(), &[0, 0, 0, 2, 0, 0, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF]);

		let mut de = McDeserializer::new(serializer.as_bytes());
		assert_eq!(NbtIntArray::mc_deserialize(&mut de).unwrap(), arr);
		assert!(de.is_at_end());

		let empty = NbtByteArray::new(vec![]);
		let mut serializer = McSerializer::new();
		empty.mc_serialize(&mut serializer).unwrap();
		assert_eq!(serializer.as_bytes(), &[0, 0, 0, 0]);
	}

	/// A length larger than the data must error and not return a short array.
	#[test]
	fn array_length_exceeds_input() {
		let data = [0u8, 0, 0, 3, 1, 2];
		let mut de = McDeserializer::new(&data);
		assert!(NbtByteArray::mc_deserialize(&mut de).is_err());
	}
}
