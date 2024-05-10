use quartz_nbt::{io, NbtCompound};
use quartz_nbt::io::Flavor;

use crate::packets::serialization::serializer_handler::{McDeserialize, McDeserializer, McSerialize, McSerializer};

#[ignore]
#[test]
fn test_serializer_nbt() {
	let mut compound = NbtCompound::new();
	compound.insert("foo", 123);
	compound.insert("bar", -3.6f32);

	let mut binary: Vec<u8> = Vec::new();
	io::write_nbt(&mut binary, None, &compound, Flavor::Uncompressed).unwrap();

	println!("Out: {:?}", binary);

	let mut compound = crate::protocol_details::datatypes::nbt::nbt::NbtCompound::new(Some(""));
	compound.add("foo", 123);
	compound.add("bar", -3.6f32);

	let mut serializer = McSerializer::new();
	compound.mc_serialize(&mut serializer).unwrap();

	println!("Out: {:?}", serializer.output);
	println!("foo: {:?}", compound["foo"]);

	//         String(root name)                                       String (tag name)    i32?                                                   f32
	// type    u16      data                                     type  u16                  data                  type   u16     String            data             END?
	// [10,    0, 8,    114, 111, 111, 116, 45, 116, 97, 103,    5,    0, 3, 98, 97, 114,   192, 102, 102, 102,   3,     0, 3,   102, 111, 111,    0, 0, 0, 123,    0]
}

#[test]
fn test_compound_serialization() {
	let mut compound = crate::protocol_details::datatypes::nbt::nbt::NbtCompound::new(Some("root-tag"));
	compound.add("foo", 123);
	compound.add("bar", -3.6f32);
	compound.add("baz", "hello");


	let mut serializer = McSerializer::new();
	compound.mc_serialize(&mut serializer).unwrap();

	println!("Out: {:?}", serializer.output);

	let mut deserializer = McDeserializer::new(&serializer.output);
	let deserialized = crate::protocol_details::datatypes::nbt::nbt::NbtTag::mc_deserialize(&mut deserializer).unwrap();
	
	match deserialized {
		crate::protocol_details::datatypes::nbt::nbt::NbtTag::Compound(deserialized) => {
			assert_eq!(compound["foo"], deserialized["foo"]);
			assert_eq!(compound["bar"], deserialized["bar"]);
			assert_eq!(compound["baz"], deserialized["baz"]);
		},
		_ => panic!("Expected compound")
	}
}