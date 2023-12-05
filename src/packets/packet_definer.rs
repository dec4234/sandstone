pub enum PacketDirection {
    SERVER,
    CLIENT,
    BILATERAL
}

pub trait Packet {

}

pub trait PacketDirectionTrait {
    fn get_direction() -> PacketDirection;
}

#[macro_use]
pub mod macros {
    use serde::{Deserialize, Serialize};
    use crate::packets::packet_definer::Packet;
    #[macro_export]
    macro_rules! protocol {
        ($nice_name: ident, $version_name: ident, $version_number: literal => {
            $($name: ident, $packetID: literal => {
                $($field: ident: $t: ty),*
            }),*
        }) => {
            $(
                #[derive(Debug, Copy, Clone, Deserialize, Serialize)]
                pub struct $name {
                    $($field: ty),*,
                }

                impl $name {
                    pub fn packet_id() -> u8 {
                        return $packetID;
                    }
                }
            ),*

            $crate::as_item!(
                pub enum $nice_name {
                    $($name_ver($name)),*
                }
            );

            impl Packet for $nice_name {

            }
        };
    }

    #[macro_export]
    macro_rules! define_packet {
        ($name: ident, $traitname: ident, $direction: ty => {
            $($name_ver: ident, $id: expr, $lower_version: expr, $upper_version: expr => {
                $($field: ident, $t: ty),*
            }),*
        }) => {
            $crate::as_item!(
                pub trait $traitname {
                    fn get_lower_version() -> u16;
                    fn get_upper_version() -> u16;
                }
            );

            $(
                #[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
                pub struct $name_ver {
                    $($field: $t),*
                }

                impl $traitname for $name_ver {
                    fn get_lower_version() -> u16 {
                        return $lower_version;
                    }

                    fn get_upper_version() -> u16 {
                        return $upper_version;
                    }
                }


            ),*

            $crate::as_item!(
                pub enum $name {
                    $($name_ver($name_ver)),*
                }
            );

            impl $name {
                pub fn get_for_version(version: u16) -> Option<impl $traitname> {
                    None::<PingRequestUniversal>
                }
            }
        };
    }
}