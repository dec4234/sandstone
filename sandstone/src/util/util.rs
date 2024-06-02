/*
Useful utilities for the library such as macro helpers and enum builders
 */

#[macro_use]
pub mod macros {
    /// Create A Hashmap
    #[macro_export]
    macro_rules! map {
        ($($k:expr => $v:expr),+) => {
            {
                let mut map = HashMap::new();
                $(map.insert($k, $v);)+
                map
            }
        }
    }

    #[macro_export]
    macro_rules! as_item {
        ($i:item) => {
            $i
        };
    }

    #[macro_export]
    macro_rules! enumize {
        ($name: ident, $y: ty => {
                $($na: ident, $lit: expr),*
            }
        )  => {
            $crate::as_item!{
                #[derive(Clone, Copy, PartialEq)]
                pub enum $name {
                    $($na),*,
                }
            }

            impl $name {
                pub fn get_all() -> Vec<$name> {
                    vec![$($name::$na),*,]
                }

                pub fn from(code: $y) -> Option<$name> where $y: PartialEq {
                    for n in $name::get_all() {
                        if n.get() == code {
                            return Some(n);
                        }
                    }

                    None
                }

                pub fn get(&self) -> $y {
                    match self {
                        $($name::$na => $lit),*
                    }
                }
            }
        };
    }
    
    // Should only be used in protocol_version.rs
    #[macro_export]
    macro_rules! versions {
        ($name: ident, $y: ty => {
                $($na: ident, $lit: expr, $fancy: literal),*
            }
        )  => {
            $crate::as_item!{
                /// Protocol version describes each major version of Minecraft: Java Edition since 1.8.9 <br>
                /// For each major version (ie. 1.8, 1.9, etc) the last released sub-version is used, since there
                /// is no conceivable reason to use any of the previous sub-versions.<br>
                /// Provided is also the protocol number associated with the last sub-version for that major version,
                /// as well as the name typically associated with that version.
                #[derive(Clone, Copy, PartialEq)]
                #[allow(non_snake_case)]
                pub enum $name {
                    $($na),*,
                }
            }

            impl $name {
                pub fn get_all() -> Vec<$name> {
                    vec![$($name::$na),*,]
                }

                pub fn from(code: $y) -> Option<$name> where $y: PartialEq {
                    for n in $name::get_all() {
                        if n.get_version_number() == code {
                            return Some(n);
                        }
                    }

                    None
                }

                pub fn get_version_number(&self) -> $y {
                    match self {
                        $($name::$na => $lit),*
                    }
                }
				
				pub fn get_fancy_name(&self) -> String {
					match self {
						$($name::$na => $fancy),*
					}.to_string()
				}
            }
        };
    }
}