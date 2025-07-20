//! Defines key macros, traits and enums used to describe packets.

/// Defines the DESTINATION of the packet. So a packet that is C -> S would be `PacketDirection::SERVER`.
///
/// In the context of initiating a 'CraftConnection', this is the type of client that is being created.
/// So if you are creating a client that connects to a server, you would use `PacketDirection::CLIENT`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PacketDirection {
	SERVER,
	CLIENT,
}

/// Used to help discern the type of packet being received. Note that different states could have
/// packets with the same ids. 
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PacketState {
	STATUS,
	HANDSHAKING,
	LOGIN,
    CONFIGURATION,
	PLAY,
    TRANSFER // is this actually a distinct state or is it just login
}

impl PacketState {
    /// Converts an u8 to a PacketState. Returns None if the id is unknown.
    pub fn from_id(id: u8) -> Option<PacketState> {
        match id {
            1 => Some(PacketState::STATUS),
            2 => Some(PacketState::LOGIN),
            3 => Some(PacketState::TRANSFER),
            _ => None // others are unknown at this time
        }
    }
    
    /// Gets the ID of the packet state. Returns None if the state is unknown.
    pub fn get_id(&self) -> Option<u8> {
        match self {
            PacketState::STATUS => Some(1),
            PacketState::LOGIN => Some(2),
            PacketState::TRANSFER => Some(3),
            _ => None
        }
    }
}

#[macro_use]
mod macros {
    /// Internal Only. This is the complex macro used to define every packet in the game. First, we it define the packet with all of its fields,
    /// then it adds it to a central enum. This enum is used to deserialize the raw incoming packets from a connection since otherwise
    /// we can only determine the packet based on the id and current state of the connection.
    /// 
    /// Generally, this is an internal macro, but you may need to work on it in order to change packets around based on
    /// different game versions. Needless to say, this packet is only used for a single Minecraft version at a time.
    #[macro_export]
    macro_rules! packets {
        ($ref_ver: ident => {
            // These are split into multiple levels to allow for more efficient deserialization 
            $($state: ident => {
                $($direction: ident => {
                   $($name: ident, $name_body: ident, $packetID: literal $(#[$struct_meta: meta])* => {
                        $(
                            $(#[$field_meta:meta])*
                            $field: ident: $t: ty
                        ),*
                    }),* 
                }),*
            }),*
        }) => {
            $(
                $(
                    $(
                        #[derive(Debug, Clone, PartialEq, sandstone_derive::McDeserialize, sandstone_derive::McSerialize, sandstone_derive::McDefault)]
                        $(#[$struct_meta])*
                        pub struct $name_body { // The body struct of the packet
                            $(
                                $(#[$field_meta])*
                                pub $field: $t
                            ),*
                        }
                        
                        impl $name_body {
                            pub fn new($($field: $t),*) -> Self {
                                Self {
                                    $($field),*
                                }
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
                )*
            )*
            
            $crate::as_item!( // weird workaround from mcproto-rs
                #[derive(Debug, Clone, PartialEq)]
                pub enum Packet {
                    $($($($name($name_body),)*)*)*
                }
            );
            
            impl Packet {
                pub fn packet_id(&self) -> VarInt {
                    match self {
                        $($($(Packet::$name(_) => VarInt($packetID as i32),)*)*)*
                    }
                }
                
                pub fn state(&self) -> PacketState {
                    match self {
                        $($($(Packet::$name(_) => PacketState::$state,)*)*)*
                    }
                }
                
                pub fn direction(&self) -> PacketDirection {
                    match self {
                        $($($(Packet::$name(_) => PacketDirection::$direction,)*)*)*
                    }
                }
            }
            
            impl McSerialize for Packet {
                fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
                    let mut length_serializer = McSerializer::new();
                    match self {
                        $($($(Packet::$name(b) => {b.mc_serialize(&mut length_serializer)?}),*)*)*
                    }
                    
                    let packet_id = self.packet_id();
                    
                    let bytes = packet_id.to_bytes(); // getting the bytes is kind of expensive, so cache it
                    
                    VarInt(length_serializer.output.len() as i32 + bytes.len() as i32).mc_serialize(serializer)?;
                    bytes.mc_serialize(serializer)?;
                    serializer.merge(length_serializer);
                    
            
                    Ok(())
                }
            }
            
            impl StateBasedDeserializer for Packet {
                /// Deserialize a packet from a byte buffer, given the state and direction of the packet.
                /// The byte buffer should include the raw packet details such as the packet length and id.
                fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: PacketState, packet_direction: PacketDirection) -> SerializingResult<'a, Self> {
                    let length = VarInt::mc_deserialize(deserializer)?;

                    let mut sub = deserializer.sub_deserializer_length(length.0 as usize)?;
                    
                    let packet_id = VarInt::mc_deserialize(&mut sub)?;
                    
                    $(
                        if state == PacketState::$state {
                            $(
                                if packet_direction == PacketDirection::$direction {
                                    match packet_id.0 {
                                        $(
                                            $packetID => {
                                                let a = $name_body::mc_deserialize(&mut sub);

                                                if let Ok(a) = a {
                                                    return Ok(Packet::$name(a));
                                                }
                                            }
                                        )*
                                        
                                            _ => {}
                                    }
                                }
                            )*
                        }
                    )*
                    
                    return Err(SerializingErr::UniqueFailure(format!("Could not find matching packet for direction {:?} and state {:?} with packet id '0x{:X}'.", packet_direction, state, packet_id.0)));
                }
            }
        };
    }
    
    #[macro_export]
    macro_rules! pac {
        ($stru: ident => {
            ($state: ident) => {
                $($name: ident, $name_body: ident, $packetID: literal => {
                    $($field: ident: $t: ty),*
                }),* 
            },*
        }) => {
            $(
                $(
                pub struct $name_body { // The body struct of the packet
                    $(pub(crate) $field: $t),*
                }
                )*
            )*
            
            pub enum stru {
                $(
                    $(
                        $name($name_body)
                    )*
                )*
            }
            
            impl stru {
                pub fn here() {
                    
                }
            }
        }
    }

    /// Defines the structs for some fields for packets. This is most frequently used for nested
    /// fields without the use of Optional<T>
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
                fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
                    let s = Self {
                        $($field: <$t>::mc_deserialize(deserializer)?,)*
                    };

                    Ok(s)
                }
            }

            impl McSerialize for $name {
                fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
                    $(self.$field.mc_serialize(serializer)?;)*

                    Ok(())
                }
            }
        };
    }
}