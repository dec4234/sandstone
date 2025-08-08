//! Important utility functions and macros used throughout the library.

pub(crate) mod threadpool;
pub mod encryption;
pub mod java;

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

    /// Some neat trick to convert a block of code into an item for macros?
    /// Source is [mcproto-rs](https://github.com/Twister915/mcproto-rs)
    #[macro_export]
    macro_rules! as_item {
        ($i:item) => {
            $i
        };
    }

    /// Create an enum that provides a getter for the value associated with each variant.
    /// Perhaps this is not meant to be done with the Rust enum system, but it's a useful abstraction
    /// from the Java enums.
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
}