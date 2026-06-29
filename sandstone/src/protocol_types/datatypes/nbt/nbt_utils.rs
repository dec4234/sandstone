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