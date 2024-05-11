use quartz_nbt::{io, NbtCompound};
use quartz_nbt::io::Flavor;

use crate::packets::serialization::serializer_handler::{McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol_details::datatypes::nbt::nbt::{NbtByteArray, NbtIntArray, NbtList, NbtLongArray, NbtTag};

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
	let mut compound = crate::protocol_details::datatypes::nbt::nbt::NbtCompound::new(Some("A"));
	compound.add("i8", 123i8);
	compound.add("i16", 1234i16);
	compound.add("i32", 12345i32);
	compound.add("f32", -3.6f32);
	compound.add("f64", -3.6789f64);
	compound.add("str", "hello");
	compound.add("byte_array", NbtByteArray::new(vec![1, 2, 3, 4, 5]));
	compound.add("int_array", NbtIntArray::new(vec![1, 2, 3, 4, 5]));
	compound.add("long_array", NbtLongArray::new(vec![1, 2, 3, 4, 5]));
	compound.add("list", NbtList::from_vec(vec![NbtTag::Int(1), NbtTag::Int(2), NbtTag::Int(3)]).unwrap());
	
	let mut compound2 = crate::protocol_details::datatypes::nbt::nbt::NbtCompound::new(Some("AB"));
	compound2.add("byte", 13i8);
	compound.add("compound", compound2);


	let mut serializer = McSerializer::new();
	//serializer.serialize_u8(10);
	compound.mc_serialize(&mut serializer).unwrap();

	println!("Out: {:?}", serializer.output);

	let mut deserializer = McDeserializer::new(&serializer.output);
	let deserialized = NbtTag::mc_deserialize(&mut deserializer).unwrap();
	
	match deserialized {
		NbtTag::Compound(deserialized) => {
			assert_eq!(compound["i8"], deserialized["i8"]);
			assert_eq!(compound["i16"], deserialized["i16"]);
			assert_eq!(compound["i32"], deserialized["i32"]);
			assert_eq!(compound["f32"], deserialized["f32"]);
			assert_eq!(compound["f64"], deserialized["f64"]);
			assert_eq!(compound["str"], deserialized["str"]);
			assert_eq!(compound["byte_array"], deserialized["byte_array"]);
			assert_eq!(compound["int_array"], deserialized["int_array"]);
			assert_eq!(compound["long_array"], deserialized["long_array"]);
			assert_eq!(compound["list"], deserialized["list"]);
			
			assert_eq!(compound["compound"], deserialized["compound"]);
		},
		_ => panic!("Expected compound")
	}
}