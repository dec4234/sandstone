pub trait NbtTagSerialization {
    fn serialize(&self) -> String;
    fn deserialize(input: String) -> Option<Self> where Self: Sized;
}

pub trait NbtTypeBody {

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
                    #[derive(Debug, Clone, Deserialize, Serialize)]
                    pub struct $name_body {
                        $($field: $t),*
                    }
                );

                impl $name_body {
                    pub fn packet_id() -> u16 {
                        return $packetID;
                    }
                }
            )*

            $crate::as_item!(
                #[derive(Debug, Clone, Deserialize, Serialize)]
                pub enum $header {
                    $($name($name_body)),*
                }
            );
        };
    }