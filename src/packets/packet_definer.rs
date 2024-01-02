pub enum PacketDirection {
    SERVER,
    CLIENT,
    BILATERAL
}

pub enum PacketState {
    STATUS,
    HANDSHAKING,
    LOGIN,
    PLAY
}

pub trait PacketTrait {
    fn packet_id() -> u8;

    fn state() -> PacketState;
}

pub trait PacketVersionDefinition {

}

pub trait PacketDirectionTrait {
    fn get_direction() -> PacketDirection;
}

#[macro_use]
pub mod macros {
    #[macro_export]
    macro_rules! protocol {
        ($nice_name: ident, $version_number: literal => {
            $($name: ident, $name_body: ident, $packetID: literal, $state: ident => {
                $($field: ident: $t: ty),*
            }),*
        }) => {
            $(
                #[derive(Debug, Clone)]
                pub struct $name_body {
                    $(pub(crate) $field: $t),*
                }

                impl PacketTrait for $name_body {
                    fn packet_id() -> u8 {
                        return $packetID;
                    }

                    fn state() -> PacketState {
                        return PacketState::$state;
                    }
                }

                impl McSerialize for $name_body {
                     fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
                         $(self.$field.mc_serialize(serializer)?;)*

                         Ok(())
                     }
                }

                impl McDeserialize for $name_body {
                    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
                        let s = Self {
                            $($field: <$t>::mc_deserialize(deserializer)?,)*
                        };

                        if !deserializer.isAtEnd() {
                            return Err(SerializingErr::LeftoverInput);
                        }

                        Ok(s)
                    }
                }
            )*

            $crate::as_item!(
                #[derive(Debug, Clone)]
                pub enum $nice_name {
                    $($name($name_body)),*
                }
            );

            impl PacketVersionDefinition for $nice_name {

            }

            impl McSerialize for $nice_name {
                fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {


                     Ok(())
                }
            }
        };
    }
}