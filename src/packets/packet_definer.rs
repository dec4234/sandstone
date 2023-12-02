#[macro_use]
pub mod macros {

    #[macro_export]
    macro_rules! define_packet {
        ($name: ident, $traitname: ident => {
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
                #[derive(Copy, Clone, PartialEq)]
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