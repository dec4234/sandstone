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
	TRANSFER, // is this actually a distinct state or is it just login
}

impl PacketState {
	/// Converts an u8 to a PacketState. Returns None if the id is unknown.
	pub fn from_id(id: u8) -> Option<PacketState> {
		match id {
			1 => Some(PacketState::STATUS),
			2 => Some(PacketState::LOGIN),
			3 => Some(PacketState::TRANSFER),
			_ => None, // others are unknown at this time
		}
	}

	/// Gets the ID of the packet state. Returns None if the state is unknown.
	pub fn get_id(&self) -> Option<u8> {
		match self {
			PacketState::STATUS => Some(1),
			PacketState::LOGIN => Some(2),
			PacketState::TRANSFER => Some(3),
			_ => None,
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
                   $($name: ident, $packetID: literal $(#[$struct_meta: meta])* => {
                        $(
                            $(#[$field_meta:meta])*
                            $field: ident: $t: ty
                        ),*
                    }),*
                }),*
            }),*
        }) => {
            paste::paste! {
            $(
                $(
                    $(
                        #[derive(Debug, Clone, PartialEq, sandstone_derive::McDeserialize, sandstone_derive::McSerialize, sandstone_derive::McDefault)]
                        $(#[$struct_meta])*
                        pub struct [<$name Packet>] { // The body struct of the packet
                            $(
                                $(#[$field_meta])*
                                pub $field: $t
                            ),*
                        }

                        impl [<$name Packet>] {
                            pub fn new($($field: $t),*) -> Self {
                                Self {
                                    $($field),*
                                }
                            }
                        }

                        impl From<[<$name Packet>]> for Packet {
                            fn from(p: [<$name Packet>]) -> Self {
                                Packet::$name(p)
                            }
                        }

                        impl From<Packet> for [<$name Packet>] {
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
                    $($($($name([<$name Packet>]),)*)*)*
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
                                                let a = [<$name Packet>]::mc_deserialize(&mut sub);

                                                if let Ok(a) = a {
                                                    return Ok(Packet::$name(a));
                                                } else if let Err(e) = a {
                                                    return Err(SerializingErr::DeserializationError(format!("Failed to deserialize packet {:?} with id '0x{:X}' with message: '{}'", stringify!($name), packet_id.0, e)));
                                                }
                                            }
                                        )*

                                            _ => {}
                                    }
                                }
                            )*
                        }
                    )*

                    return Err(SerializingErr::NoKnownPacket(format!("Could not find matching packet for destination {:?} and state {:?} with packet id '0x{:X}'", packet_direction, state, packet_id.0)));
                }
            }

            /// One test per packet: build it from `McDefault`, write the full frame, read it back
            /// through `deserialize_state` with the packet's own state and direction, and compare.
            /// This catches field order or type mistakes, a wrong id, and a wrong state or direction.
            #[cfg(test)]
            mod generated_packet_round_trips {
                use super::*;

                $($($(
                    #[test]
                    fn [<round_trip_ $name:snake>]() {
                        let packet = Packet::$name([<$name Packet>]::mc_default());
                        assert_eq!(packet.state(), PacketState::$state);
                        assert_eq!(packet.direction(), PacketDirection::$direction);

                        let mut serializer = McSerializer::new();
                        packet.mc_serialize(&mut serializer).expect("serialize failed");

                        let mut deserializer = McDeserializer::new(&serializer.output);
                        let decoded = Packet::deserialize_state(&mut deserializer, PacketState::$state, PacketDirection::$direction)
                            .expect("deserialize failed");
                        assert_eq!(packet, decoded);
                        assert!(deserializer.is_at_end(), "bytes left over after the frame");
                    }
                )*)*)*
            }
            } // end paste
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

#[cfg(test)]
mod tests {
	use crate::protocol::packets::packet_definer::{PacketDirection, PacketState};
	use crate::protocol::packets::{Packet, PingRequestPacket, PingResponsePacket, StatusRequestPacket};
	use crate::protocol::serialization::serializer_error::SerializingErr;
	use crate::protocol::serialization::{McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
	use crate::protocol_types::datatypes::var_types::VarInt;

	/// The handshake `next_state` field uses 1 = STATUS, 2 = LOGIN, 3 = TRANSFER.
	#[test]
	fn packet_state_handshake_ids() {
		for (id, state) in [(1u8, PacketState::STATUS), (2, PacketState::LOGIN), (3, PacketState::TRANSFER)] {
			assert_eq!(PacketState::from_id(id), Some(state));
			assert_eq!(state.get_id(), Some(id));
		}
		assert_eq!(PacketState::from_id(0), None);
		assert_eq!(PacketState::from_id(4), None);
		for state in [PacketState::HANDSHAKING, PacketState::CONFIGURATION, PacketState::PLAY] {
			assert_eq!(state.get_id(), None, "{state:?} is not a handshake intent");
		}
	}

	fn to_bytes(packet: &Packet) -> Vec<u8> {
		let mut serializer = McSerializer::new();
		packet.mc_serialize(&mut serializer).unwrap();
		serializer.output
	}

	/// Uncompressed frame: VarInt length of (id + body), VarInt packet id, then the body.
	#[test]
	fn packet_frame_format() {
		let packet: Packet = PingRequestPacket::new(5).into();
		assert_eq!(to_bytes(&packet), vec![9, 0x01, 0, 0, 0, 0, 0, 0, 0, 5]);

		let empty: Packet = StatusRequestPacket::new().into();
		assert_eq!(to_bytes(&empty), vec![1, 0x00]);
	}

	#[test]
	fn packet_metadata() {
		let packet: Packet = PingRequestPacket::new(5).into();
		assert_eq!(packet.packet_id(), VarInt(0x01));
		assert_eq!(packet.state(), PacketState::STATUS);
		assert_eq!(packet.direction(), PacketDirection::SERVER);

		let packet: Packet = PingResponsePacket::new(5).into();
		assert_eq!(packet.direction(), PacketDirection::CLIENT);
	}

	#[test]
	fn packet_enum_conversions() {
		let body = PingRequestPacket::new(42);
		let packet: Packet = body.clone().into();
		assert_eq!(packet, Packet::PingRequest(body.clone()));
		assert_eq!(PingRequestPacket::from(packet), body);
	}

	#[test]
	#[should_panic(expected = "Invalid conversion")]
	fn packet_enum_wrong_conversion_panics() {
		let packet: Packet = StatusRequestPacket::new().into();
		let _ = PingRequestPacket::from(packet);
	}

	/// The same id means different packets in different directions. Direction must pick the packet.
	#[test]
	fn deserialize_uses_direction() {
		let bytes = to_bytes(&PingRequestPacket::new(5).into());

		let mut de = McDeserializer::new(&bytes);
		let server = Packet::deserialize_state(&mut de, PacketState::STATUS, PacketDirection::SERVER).unwrap();
		assert_eq!(server, Packet::PingRequest(PingRequestPacket::new(5)));

		let mut de = McDeserializer::new(&bytes);
		let client = Packet::deserialize_state(&mut de, PacketState::STATUS, PacketDirection::CLIENT).unwrap();
		assert_eq!(client, Packet::PingResponse(PingResponsePacket::new(5)));
	}

	/// Two frames in one buffer: the first read must stop at its length so the second one is intact.
	#[test]
	fn deserialize_back_to_back_frames() {
		let mut bytes = to_bytes(&PingRequestPacket::new(1).into());
		bytes.extend(to_bytes(&StatusRequestPacket::new().into()));

		let mut de = McDeserializer::new(&bytes);
		let first = Packet::deserialize_state(&mut de, PacketState::STATUS, PacketDirection::SERVER).unwrap();
		let second = Packet::deserialize_state(&mut de, PacketState::STATUS, PacketDirection::SERVER).unwrap();
		assert_eq!(first, Packet::PingRequest(PingRequestPacket::new(1)));
		assert_eq!(second, Packet::StatusRequest(StatusRequestPacket::new()));
		assert!(de.is_at_end());
	}

	#[test]
	fn deserialize_unknown_id() {
		let bytes = [1u8, 0x7F];
		let mut de = McDeserializer::new(&bytes);
		let err = Packet::deserialize_state(&mut de, PacketState::STATUS, PacketDirection::SERVER).unwrap_err();
		assert!(matches!(err, SerializingErr::NoKnownPacket(_)), "{err:?}");
	}

	/// A known id with a body that is too short must give an error that names the packet.
	#[test]
	fn deserialize_bad_body_names_packet() {
		let bytes = [3u8, 0x01, 0, 0];
		let mut de = McDeserializer::new(&bytes);
		match Packet::deserialize_state(&mut de, PacketState::STATUS, PacketDirection::SERVER) {
			Err(SerializingErr::DeserializationError(msg)) => assert!(msg.contains("PingRequest"), "{msg}"),
			other => panic!("expected DeserializationError, got {other:?}"),
		}
	}

	/// A length prefix longer than the buffer means a partial frame. It must error, not read garbage.
	#[test]
	fn deserialize_truncated_frame() {
		let bytes = [9u8, 0x01, 0, 0];
		let mut de = McDeserializer::new(&bytes);
		assert!(Packet::deserialize_state(&mut de, PacketState::STATUS, PacketDirection::SERVER).is_err());
	}
}
