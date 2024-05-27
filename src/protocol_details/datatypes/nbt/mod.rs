pub mod nbt;
pub mod snbt;
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
        
            impl From<NbtTag> for $t {
                fn from(tag: NbtTag) -> Self {
                    match tag {
                        NbtTag::$name(val) => val,
                        _ => panic!("Invalid conversion")
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
                #[derive(Debug, Clone, PartialEq)]
                pub struct $fancyname {
                    pub list: Vec<$t>,
                    pub count: u32, // iterator
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
                    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, $fancyname> {
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
            
                impl From<NbtTag> for $fancyname {
                    fn from(tag: NbtTag) -> Self {
                        match tag {
                            NbtTag::$name(val) => val,
                            _ => panic!("Invalid conversion")
                        }
                    }
                }
            )*
        };
    }

	#[macro_export]
	macro_rules! primtype_snbt {
        ($name: literal, $t: ty) => {
            impl SNBT for $t {
                fn to_snbt(&self, name: Option<String>) -> String {
                    let mut s = String::new();
                    
                    s.push_str(format!("TAG_{}(", $name).as_str());
                    
                    if let Some(name) = name {
                        s.push_str(format!("'{name}')").as_str());
                    } else {
                        s.push_str(format!("None)").as_str());
                    }
                    
                    s.push_str(format!(": {}", *self).as_str());
                    
                    s
                }
            
                fn from_snbt(snbt: &str) -> Result<Self> where Self: Sized {
                    todo!()
                }
            }
        };
    }

	#[macro_export]
	macro_rules! list_snbt {
        ($name: literal, $t: ty) => {
            impl SNBT for Vec<$t> {
                fn to_snbt(&self, name: Option<String>) -> String {
                    let mut s = String::new();
                    
                    s.push_str(format!("TAG_{}(", $name).as_str());
                    
                    if let Some(name) = name {
                        s.push_str(format!("'{name}')").as_str());
                    } else {
                        s.push_str(format!("None)").as_str());
                    }
                    
                    s.push_str(": [");
                    
                    for b in self {
                        s.push_str(format!("{}, ", b).as_str());
                    }
                    
                    if self.len() > 0 { // if the vec has any elements
                        s.pop(); // remove final comman and space
                        s.pop();
                    }
                    
                    s.push_str("]");
                    
                    s
                }
            
                fn from_snbt(snbt: &str) -> Result<Self> where Self: Sized {
                    todo!()
                }
            }
        };
    }
}