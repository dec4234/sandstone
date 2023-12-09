use crate::as_item;

pub trait NbtType {

}

#[macro_export]
macro_rules! nbt_type {
        ($header: ident => {
            $($name: ident, $name_body: ident, $packetID: literal => {
                $($field: ident: $t: ty),*
            }),*
        }) => {
            $(
                $crate::as_item!(
                    #[derive(Debug, Copy, Clone, Deserialize, Serialize)]
                    pub struct $name_body {
                        $($field: $t),*
                    }
                );
            )*

            $crate::as_item!(
                pub enum $header {
                    $($name($name_body)),*
                }
            );
        };
    }