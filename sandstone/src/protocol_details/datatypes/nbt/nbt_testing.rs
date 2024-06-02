use crate::packets::serialization::serializer_handler::{McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol_details::datatypes::nbt::nbt::{NbtByteArray, NbtCompound, NbtIntArray, NbtList, NbtLongArray, NbtTag};

#[test]
fn test_compound_serialization() {
	let mut compound = NbtCompound::new(Some("A"));
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
	
	let mut compound2 = NbtCompound::new(Some("AB"));
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

#[test]
fn test_compounds_in_compounds() {
	let mut outer = NbtCompound::new(Some("outer"));
	let mut mid1 = NbtCompound::new(Some("mid1"));
	let mut mid2 = NbtCompound::new(Some("mid2"));
	let mut inner1 = NbtCompound::new(Some("inner1"));
	let mut inner2 = NbtCompound::new(Some("inner2"));
	let mut inner3 = NbtCompound::new(Some("inner3"));
	
	inner1.add("i8", 123i8);
	inner1.add("i16", 1234i16);
	
	inner2.add("i8", 123i8);
	inner2.add("str", "hello");
	
	inner3.add("byte_list", NbtByteArray::new(vec![8, 3, 9, 0, 2, 1]));
	inner3.add("int_list", NbtIntArray::new(vec![97197, 288, -28238, -89, 8373]));
	
	mid1.add("inner1", inner1);
	mid1.add("i32", 12345i32);
	
	mid2.add("inner2", inner2);
	mid2.add("i32", 187585i32);
	mid2.add("inner3", inner3);
	mid2.add("long_list", NbtLongArray::new(vec![8766897606966, 78287698760875, 786876876732, 125435138536]));
	
	outer.add("mid1", mid1);
	outer.add("mid2", mid2);
	
	let mut serializer = McSerializer::new();
	outer.mc_serialize(&mut serializer).unwrap();
	
	println!("Out: {:?}", serializer.output);
	
	let mut deserializer = McDeserializer::new(&serializer.output);
	let deserialized = NbtTag::mc_deserialize(&mut deserializer).unwrap();
	
	match deserialized {
		NbtTag::Compound(deserialized) => {
			assert_eq!(outer, deserialized);
			assert_eq!(outer["mid1"], deserialized["mid1"]);
			assert_eq!(outer["mid2"], deserialized["mid2"]);
		},
		_ => panic!("Expected compound")
	}
}