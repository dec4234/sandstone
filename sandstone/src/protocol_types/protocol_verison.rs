//! Defines the protocol version numbers for the final patch of each Minecraft version. This is important
//! for verifying client protocol versions.

use crate::versions;

/// Internal Only. Creates an enum of Minecraft versions with their protocol numbers and fancy names.
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
            ///
            /// Please keep in mind that while protocol numbers are provided back all the way to 1.8.9,
            /// the library typically only supports the latest version of Minecraft: Java Edition.
            #[derive(Clone, Copy, PartialEq, Debug)]
            #[allow(non_snake_case)]
            pub enum $name {
                $($na),*,
            }
        }

        impl $name {
            pub fn get_all() -> Vec<$name> {
                vec![$($name::$na),*,]
            }

            /// Get the protocol version number such as 47, 110, etc.
            pub fn get_version_number(&self) -> $y {
                match self {
                    $($name::$na => $lit),*
                }
            }

            /// Get the fancy name associated with a version such as "1.8.9" or "1.19.4"
            pub fn get_fancy_name(&self) -> String {
                match self {
                    $($name::$na => $fancy),*
                }.to_string()
            }
        }

        impl TryFrom<i16> for $name {
            type Error = String;

            fn try_from(value: i16) -> Result<Self, Self::Error> {
                match value {
                    $($lit => Ok($name::$na),)*
                    _ => Err(format!("Unknown protocol version: {}", value)),
                }
            }
        }
    };
}

// https://wiki.vg/Protocol_History
versions!(ProtocolVerison, i16 => {
    V1_8, 47, "1.8.9",
    V1_9, 110, "1.9.4",
    V1_10, 210, "1.10.2",
    V1_11, 316, "1.11.2",
    V1_12, 340, "1.12.2",
    V1_13, 404, "1.13.2",
    V1_14, 498, "1.14.4",
    V1_15, 578, "1.15.2",
    V1_16, 754, "1.16.5",
    V1_17, 756, "1.17.1",
    V1_18, 758, "1.18.2",
    V1_19, 762, "1.19.4",
    V1_20, 766, "1.20.6",
    V1_21, 772, "1.21.8"
});

impl ProtocolVerison {
    /// Returns the most recent protocol version supported by the library.
    pub fn latest() -> ProtocolVerison {
        ProtocolVerison::V1_21
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tryfrom_i16() {
        use crate::protocol_types::protocol_verison::ProtocolVerison;

        assert_eq!(ProtocolVerison::try_from(47).unwrap(), ProtocolVerison::V1_8);
        assert_eq!(ProtocolVerison::try_from(110).unwrap(), ProtocolVerison::V1_9);
        assert_eq!(ProtocolVerison::try_from(772).unwrap(), ProtocolVerison::V1_21);
        assert!(ProtocolVerison::try_from(999).is_err());
    }

    #[test]
    fn get_fancy_name() {
        use crate::protocol_types::protocol_verison::ProtocolVerison;

        assert_eq!(ProtocolVerison::V1_8.get_fancy_name(), "1.8.9");
        assert_eq!(ProtocolVerison::V1_9.get_fancy_name(), "1.9.4");
        assert_eq!(ProtocolVerison::V1_20.get_fancy_name(), "1.20.6");
    }
}