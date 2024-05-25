pub enum PacketDirection {
	SERVER,
	CLIENT,
	BIDIRECTIONAL
}

/// Used to help discern the type of packet being received. Note that different states could have
/// packets with the same ids. 
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PacketState {
	STATUS,
	HANDSHAKING,
	LOGIN,
    CONFIGURATION,
	PLAY
}

impl PacketState {
    pub fn from_id(id: u8) -> Option<PacketState> {
        match id {
            1 => Some(PacketState::STATUS),
            2 => Some(PacketState::LOGIN),
            _ => None
        }
    }
    
    pub fn get_id(&self) -> Option<u8> {
        match self {
            PacketState::STATUS => Some(1),
            PacketState::LOGIN => Some(2),
            _ => None
        }
    }
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
mod macros {
    #[macro_export]
    macro_rules! packets {
        ($ref_ver: ident => {
            $($name: ident, $name_body: ident, $packetID: literal, $state: ident => {
                $($field: ident: $t: ty),*
            }),*
        }) => {
            $(
                #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                pub struct $name_body { // The body struct of the packet
                    $(pub(crate) $field: $t),*
                }
            
                impl McDeserialize for $name_body {
                    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
                        let s = Self {
                            $($field: <$t>::mc_deserialize(deserializer)?,)*
                        };

                        Ok(s)
                    }
                }
            
                impl McSerialize for $name_body {
                    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
                        $(self.$field.mc_serialize(serializer)?;)*

                        Ok(())
                    }
                }
            
                impl From<$name_body> for Packet {
                    fn from(p: $name_body) -> Self {
                        Packet::$name(p)
                    }
                }
            
                impl Into<$name_body> for Packet {
                    fn into(self) -> $name_body {
                        match self {
                            Packet::$name(p) => p,
                            _ => panic!("Invalid conversion")
                        }
                    }
                }
            )*
            
            $crate::as_item!(
                #[derive(Debug, Clone)]
                pub enum Packet {
                    $($name($name_body)),*
                }
            );
            
            impl McSerialize for Packet {
                fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
                    match self {
                        $(Packet::$name(b) => {b.mc_serialize(serializer)?}),*
                    }

                     Ok(())
                }
            }
            
            impl StateBasedDeserializer for Packet {
                fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: &PacketState) -> DeserializeResult<'a, Self> {
                    let length = VarInt::mc_deserialize(deserializer)?;
                    let packet_id = VarInt::mc_deserialize(deserializer)?;
                    
                    $(
                    if(state == &PacketState::$state && packet_id.0 == $packetID) {
                        let a = $name_body::mc_deserialize(deserializer);

                        if let Ok(a) = a {
                            return Ok(Packet::$name(a));
                        }
                    }
                    )*
                    
                    return Err(SerializingErr::UniqueFailure("Could not find matching type.".to_string()));
                }
            }
        };
    }
    
    #[macro_export]
    macro_rules! componenet_struct {
        ($name: ident => {
            $($field: ident: $t: ty),*
        }) => {
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct $name { // The body struct of the packet
                $($field: $t),*
            }
        
            impl McDeserialize for $name {
                fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
                    let s = Self {
                        $($field: <$t>::mc_deserialize(deserializer)?,)*
                    };

                    Ok(s)
                }
            }
        
            impl McSerialize for $name {
                fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
                    $(self.$field.mc_serialize(serializer)?;)*

                    Ok(())
                }
            }
        };
    }
}