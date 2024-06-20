use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol_types::datatypes::var_types::{VarInt, VarLong};

/*
The purpose of this file is to test random nesting of enums and structs for serialization and deserialization
 */

enum Group {
	VarI(VarIntMix),
	StrM(StringMix),
}

impl Group {
	pub fn _id(&self) -> u8{
		match self {
			Group::VarI(_) => {0}
			Group::StrM(_) => {1}
		}
	}
}

impl McSerialize for Group {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			Group::VarI(b) => {b.mc_serialize(serializer)?}
			Group::StrM(b) => {b.mc_serialize(serializer)?}
		}

		Ok(())
	}
}

impl McDeserialize for Group {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let a = StringMix::mc_deserialize(deserializer);

		if let Ok(a) = a {
			return Ok(Group::StrM(a));
		}

		deserializer.reset();

		drop(a);

		let a = VarIntMix::mc_deserialize(deserializer);

		if let Ok(a) = a {
			return Ok(Group::VarI(a));
		}

		deserializer.reset();

		return Err(SerializingErr::UniqueFailure("Reached end".to_string()));
	}
}

struct VarIntMix {
	one: VarInt,
	two: String,
	three: u32,
	four: VarLong,
}

impl McSerialize for VarIntMix {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.one.mc_serialize(serializer)?;
		self.two.mc_serialize(serializer)?;
		self.three.mc_serialize(serializer)?;
		self.four.mc_serialize(serializer)?;

		Ok(())
	}
}

impl McDeserialize for VarIntMix {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let varmix = Self {
			one: VarInt::mc_deserialize(deserializer)?,
			two: String::mc_deserialize(deserializer)?,
			three: u32::mc_deserialize(deserializer)?,
			four: VarLong::mc_deserialize(deserializer)?,
		};

		if !deserializer.is_at_end() {
			return Err(SerializingErr::LeftoverInput);
		}

		Ok(varmix)
	}
}

struct StringMix {
	first: u8,
	second: String,
	third: String,
	fourth: u8,
	fifth: String
}

impl McSerialize for StringMix {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.first.mc_serialize(serializer).unwrap();
		self.second.mc_serialize(serializer).unwrap();
		self.third.mc_serialize(serializer).unwrap();
		self.fourth.mc_serialize(serializer).unwrap();
		self.fifth.mc_serialize(serializer).unwrap();

		Ok(())
	}
}

impl McDeserialize for StringMix {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let testing = Self {
			first: u8::mc_deserialize(deserializer)?,
			second: String::mc_deserialize(deserializer)?,
			third: String::mc_deserialize(deserializer)?,
			fourth: u8::mc_deserialize(deserializer)?,
			fifth: String::mc_deserialize(deserializer)?,
		};

		if !deserializer.is_at_end() {
			return Err(SerializingErr::LeftoverInput);
		}

		Ok(testing)
	}
}

#[cfg(test)]
mod tests {
	use crate::protocol::packet_definer::{PacketDirection, PacketState};
	use crate::protocol::packets::{HandshakingBody, Packet};
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
	use crate::protocol::serialization::serializer_testing::{Group, StringMix, VarIntMix};
	use crate::protocol_types::datatypes::var_types::{VarInt, VarLong};

	#[test]
	fn struct_serialization() {
		let a = StringMix {
			first: 16,
			second: "hello".to_string(),
			third: "abcd".to_string(),
			fourth: 99,
			fifth: "zxy".to_string()
		};

		let mut serializer = McSerializer::new();
		a.mc_serialize(&mut serializer).unwrap();
		println!("Serialized: {:?}", serializer.output);

		let mut deserializer = McDeserializer::new(&serializer.output);

		let b = StringMix::mc_deserialize(&mut deserializer).unwrap();
		assert_eq!(16, b.first);
		assert_eq!("hello", b.second);
		assert_eq!("abcd", b.third);
		assert_eq!(99, b.fourth);
		assert_eq!("zxy", b.fifth);
		assert_eq!(deserializer.data, vec![16, 5, 104, 101, 108, 108, 111, 4, 97, 98, 99, 100, 99, 3, 122, 120, 121]);
	}

	#[test]
	fn mixed_serialization() {
		let mut serializer = McSerializer::new();

		let varmix = VarIntMix {
			one: VarInt(3),
			two: "abcd".to_string(),
			three: 98,
			four: VarLong(17),
		};

		varmix.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = McDeserializer::new(&serializer.output);
		let test = StringMix::mc_deserialize(&mut deserializer);

		if test.is_ok() {
			panic!("Deserialization should fail because types are mixed");
		}
	}

	#[test]
	fn test_enum_serialization() {
		let mut serializer = McSerializer::new();

		let e = Group::StrM(StringMix {
			first: 78,
			second: "a".to_string(),
			third: "b".to_string(),
			fourth: 7,
			fifth: "c".to_string(),
		});

		e.mc_serialize(&mut serializer).unwrap();

		println!("{:?}", serializer.output);

		let mut deserializer = McDeserializer::new(&serializer.output);
		let group = Group::mc_deserialize(&mut deserializer).unwrap();

		match group {
			Group::VarI(b) => {println!("VarI: {}", b.two)}
			Group::StrM(b) => {println!("StrM: {}", b.fifth)}
		}
	}

	#[test]
	fn try_direct_deserialize() {
		let mut serializer = McSerializer::new();

		let p = Packet::Handshaking(HandshakingBody {
			protocol_version: VarInt(3),
			server_address: "".to_string(),
			port: 0,
			next_state: VarInt(6),
		});

		p.mc_serialize(&mut serializer).unwrap();

		println!("{:?}", serializer.output);
		let mut deserializer = McDeserializer::new(&serializer.output);
		let out = Packet::deserialize_state(&mut deserializer, PacketState::HANDSHAKING, PacketDirection::SERVER).unwrap();

		println!("{:?}", out);
	}

	#[test]
	fn test_subslicing() {
		let v: Vec<u8> = vec![3, 5, 6, 7, 8, 9, 10];
		let mut deserializer = McDeserializer::new(&v);

		assert_eq!([3, 5, 6], deserializer.slice(3));
		assert_eq!([7, 8, 9], deserializer.slice(3));
		assert_eq!([10], deserializer.slice(3));
	}
}