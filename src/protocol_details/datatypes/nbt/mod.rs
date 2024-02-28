mod nbt;
mod nbt_testing;
mod nbt_error;

#[macro_use]
mod macros {
    use quartz_nbt::NbtTag;
    /// Used to generate the NbtValue trait for primitive types
    #[macro_export]
    macro_rules! primvalue_nbtvalue  {
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
                        serializer.serialize_u8($num);
                        (self.list.len() as u32).mc_serialize(serializer)?;
                        for tag in &self.list {
                            tag.mc_serialize(serializer)?;
                        }
                        Ok(())
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
}