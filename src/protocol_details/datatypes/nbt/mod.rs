mod nbt;
mod nbt_testing;
mod nbt_error;

#[macro_use]
mod macros {
    /// Used to generate the NbtValue trait for primitive types
    #[macro_export]
    macro_rules! primvalue_nbtvalue  {
    ($(($t: ty, $num: literal, $size: literal, $name: literal)),*) => {
        $(
            impl NbtValue for $t {
                fn get_type_id(&self) -> u8 {
                    $num
                }
                
                fn get_payload_size(&self) -> Option<u8> {
                    Some($size)
                }
                
                fn get_name(&self) -> String {
                    $name.to_string()
                }
            }
        )*
    };
}
    /// Used to generate the NbtValue trait for list types such as bytearray, intarray, and longarray
    #[macro_export]
    macro_rules! list_nbtvalue {
        ($(($t: ty, $num: literal, $name: literal, $fancyname: ident)),*) => {
            $(  
                pub struct $fancyname {
                    pub list: Vec<$t>,
                    pub count: u32, // iterator
                }
            
                impl NbtValue for $fancyname {
                    fn get_type_id(&self) -> u8 {
                        $num
                    }
                    
                    fn get_payload_size(&self) -> Option<u8> {
                        None
                    }
                    
                    fn get_name(&self) -> String {
                        $name.to_string()
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
                        serializer.serialize_u8(self.get_type_id());
                        (self.list.len() as u32).mc_serialize(serializer)?;
                        for tag in &self.list {
                            tag.mc_serialize(serializer)?;
                        }
                        Ok(())
                    }
                }
            )*
        };
        () => {};
    }
}