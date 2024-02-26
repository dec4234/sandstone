mod nbt;
mod nbt_testing;
mod nbt_error;

#[macro_use]
mod macros {
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
}