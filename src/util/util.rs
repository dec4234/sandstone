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
}