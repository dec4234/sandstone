use crate::as_item;

pub trait NbtType {

}

#[macro_export]
macro_rules! nbt_type {
        ($header: ident => {
            $($name: ident, $name_body: ident, $packetID: literal, $payload_size: literal => {
                $($field: ident: $t: ty),*
            }),*
        }) => {
            $(
                #[derive(Debug, Copy, Clone, Deserialize, Serialize)]
                pub struct $name_body {
                    $($field: ty),*
                }

                impl $name_body {
                    pub fn payload_size() -> u8 {
                        return $payload_size;
                    }
                }
            ),*

            $crate::as_item!(
                pub enum $header {
                    $($name($name_body)),*
                }
            );
        };
    }