//! Create macros used to generate NbtTag trait implementations for primitive types and list types.

#![allow(clippy::from_over_into)]

pub mod nbt;
mod nbt_testing;
mod snbt_testing;
pub mod nbt_error;

#[macro_use]
mod macros {
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
                    #[serde(skip_serializing)]
                    pub count: u32, // used for iterator
                    pub list: Vec<$t>,
                }
                
                impl $fancyname {
                    pub fn new(list: Vec<$t>) -> Self {
                        Self {
                            list,
                            count: 0,
                        }
                    }
                }
            
                impl Iterator for $fancyname {
                    type Item = $t;
                
                    fn next(&mut self) -> Option<Self::Item> {
                        if self.count < self.list.len() as u32 {
                            let tag = self.list[self.count as usize];
                            self.count += 1;
                            Some(tag)
                        } else {
                            None
                        }
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