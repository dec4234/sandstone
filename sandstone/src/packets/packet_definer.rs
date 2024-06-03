/*
Defines key macros, traits and enums used to describe packets.
 */

/// Defines the DESTINATION of the packet. So a packet that is C -> S would be `PacketDirection::SERVER`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PacketDirection {
	SERVER,
	CLIENT,
	BIDIRECTIONAL // are there any?
}

/// Used to help discern the type of packet being received. Note that different states could have
/// packets with the same ids. 
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
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
            _ => None // others are unknown at this time
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

#[macro_use]
mod macros {
    
    /// Used to define the minecraft packet protocol. This includes, the name, packet ID, state and
    /// the respective fields for the packet.
    #[macro_export]
    macro_rules! packets {
        ($ref_ver: ident => {
            $($name: ident, $name_body: ident, $packetID: literal, $state: ident, $direction: ident => {
                $($field: ident: $t: ty),*
            }),*
        }) => {
            $(
                #[derive(Debug, Clone, PartialEq, Eq)]
                pub struct $name_body { // The body struct of the packet
                    $(pub(crate) $field: $t),*
                }
                
                impl $name_body {
                    pub fn new($($field: $t),*) -> Self {
                        Self {
                            $($field),*
                        }
                    }
                }
            
                #[allow(unused)] // incase there's an empty packet
                impl McDeserialize for $name_body {
                    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
                        let s = Self {
                            $($field: <$t>::mc_deserialize(deserializer)?,)*
                        };

                        Ok(s)
                    }
                }
            
                #[allow(unused)] // incase there's an empty packet
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
            
                impl From<Packet> for $name_body {
                    fn from(p: Packet) -> Self {
                        match p {
                            Packet::$name(p) => p,
                            _ => panic!("Invalid conversion")
                        }
                    }
                }
            )*
            
            $crate::as_item!( // weird workaround from mcproto-rs
                #[derive(Debug, Clone, PartialEq, Eq)]
                pub enum Packet {
                    $($name($name_body)),*
                }
            );
            
            impl Packet {
                // TODO: this needs to be a VARINT
                pub fn packet_id(&self) -> u8 {
                    match self {
                        $(Packet::$name(_) => $packetID),*
                    }
                }
                
                pub fn state(&self) -> PacketState {
                    match self {
                        $(Packet::$name(_) => PacketState::$state),*
                    }
                }
                
                pub fn direction(&self) -> PacketDirection {
                    match self {
                        $(Packet::$name(_) => PacketDirection::$direction),*
                    }
                }
            }
            
            impl McSerialize for Packet {
                fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
                    let mut length_serializer = McSerializer::new();
                    match self {
                        $(Packet::$name(b) => {b.mc_serialize(&mut length_serializer)?}),*
                    }
                    
                    let packet_id = VarInt(self.packet_id() as i32);
                    
                    VarInt(length_serializer.output.len() as i32 + packet_id.to_bytes().len() as i32).mc_serialize(serializer)?;
                    packet_id.mc_serialize(serializer)?;
                    serializer.merge(length_serializer);
                    
            
                    Ok(())
                }
            }
            
            impl StateBasedDeserializer for Packet {
                fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: PacketState, packet_direction: PacketDirection) -> DeserializeResult<'a, Self> {
                    let length = VarInt::mc_deserialize(deserializer)?;

                    let sub = deserializer.sub_deserializer_length(length.0 as usize);
                    
                    if let Err(e) = sub {
                        return Err(e);
                    }
                    
                    let mut sub = sub.unwrap();
                    
                    let packet_id = VarInt::mc_deserialize(&mut sub)?;
                    
                    $(
                    if(packet_id.0 == $packetID && state == PacketState::$state && packet_direction == PacketDirection::$direction) {
                        let a = $name_body::mc_deserialize(&mut sub);

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
    macro_rules! component_struct {
        ($name: ident => {
            $($field: ident: $t: ty),*
        }) => {
            #[derive(Debug, Clone, PartialEq, Eq)]
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