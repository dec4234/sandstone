#![allow(unused_imports)] // because something is wrong with unused imports and tests

#[cfg(test)]
mod test {
	use crate::protocol::serialization::serializer_error::SerializingErr;
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
	use crate::protocol_types::datatypes::nbt::nbt::{NbtByteArray, NbtCompound, NbtIntArray, NbtList, NbtLongArray, NbtTag};
	use sandstone_derive::{AsNbt, FromNbt};

	/// Test standard serialization of a NbtCompound.
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

		let mut compound2 = NbtCompound::new::<String>(None);
		compound2.add("byte", 13i8);
		compound.add("compound", compound2);
		
		let mut serializer = McSerializer::new();
		compound.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = McDeserializer::new(&serializer.output);
		let deserialized = NbtCompound::from_root(&mut deserializer).unwrap();

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

		assert_eq!(compound.root_name, deserialized.root_name);
	}

	/// Test compounds within compounds and their serialization.
	#[test]
	fn test_compounds_in_compounds() {
		let mut outer = NbtCompound::new(Some("outer"));
		let mut mid1 = NbtCompound::new::<String>(None);
		let mut mid2 = NbtCompound::new::<String>(None);
		let mut inner1 = NbtCompound::new::<String>(None);
		let mut inner2 = NbtCompound::new::<String>(None);
		let mut inner3 = NbtCompound::new::<String>(None);

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

		let mut deserializer = McDeserializer::new(&serializer.output);
		let deserialized = NbtCompound::from_root(&mut deserializer).unwrap();

		assert_eq!(outer, deserialized);
		assert_eq!(outer["mid1"], deserialized["mid1"]);
		assert_eq!(outer["mid2"], deserialized["mid2"]);
	}
	
	/// Test network compound deserialization.
	#[test]
	fn test_network() {
		let mut compound = NbtCompound::new_no_name();
		compound.add("i8", 123i8);
		compound.add("i16", 1234i16);
		compound.add("i32", 12345i32);
		compound.add("f32", -3.6f32);
		compound.add("f64", -3.6789f64);
		compound.add("str", "hello");
		
		let mut serializer = McSerializer::new();
		compound.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = McDeserializer::new(&serializer.output);
		let deserialized = NbtCompound::mc_deserialize(&mut deserializer).unwrap();
		
		assert_eq!(deserialized["i8"], NbtTag::Byte(123i8));
		assert_eq!(deserialized["i16"], NbtTag::Short(1234i16));
		assert_eq!(deserialized["i32"], NbtTag::Int(12345i32));
		assert_eq!(deserialized["f32"], NbtTag::Float(-3.6f32));
		assert_eq!(deserialized["f64"], NbtTag::Double(-3.6789f64));
		assert_eq!(deserialized["str"], NbtTag::String("hello".to_string()));
	}
	
	/// Test how NbtTag::None behaves when serialized and deserialized. It can be present in a compound but
	/// it should not be included in the serialization output.
	#[test]
	fn test_none() {
		let mut compound = NbtCompound::new(Some("A"));
		compound.add("none", NbtTag::None);
		
		assert_eq!(compound["none"], NbtTag::None); // actually mapped to None inside of the compound
		assert_eq!(compound["abc123"], NbtTag::None); // not because 'None' was added to the compound, but because it returns None when a certain key is not found
		
		let mut serializer = McSerializer::new();
		compound.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = McDeserializer::new(&serializer.output);
		match NbtTag::mc_deserialize(&mut deserializer) {
			Ok(NbtTag::Compound(c)) => {
				assert_eq!(c["none"], NbtTag::None);
				assert_eq!(c["abc123"], NbtTag::None);
			},
			_ => panic!("Expected NbtTag::Compound"),
		}
	}

	/// Test struct for conversion to/from NbtCompound.
	#[derive(AsNbt, FromNbt, Debug, PartialEq)]
	struct TestStruct {
		a: i32,
		b: String,
		c: f64,
	}

	/// Confirm that the `as_nbt` and `from_nbt` methods work the same.
	#[test]
	fn test_into_vs_as() {
		let test = TestStruct {
			a: 42,
			b: "Hello".to_string(),
			c: 3.14,
		};

		let as_test = test.as_nbt();
		let nbt: NbtCompound = test.into();
		
		assert_eq!(as_test, nbt);
	}
	
	/// Test that a struct can be converted into an NbtCompound using the `as_nbt` function.
	#[test]
	fn test_as_nbt() {
		let test = TestStruct {
			a: 42,
			b: "Hello".to_string(),
			c: 3.14,
		};

		let nbt: NbtCompound = test.into();

		assert_eq!(nbt["a"], NbtTag::Int(42));
		assert_eq!(nbt["b"], NbtTag::String("Hello".to_string()));
		assert_eq!(nbt["c"], NbtTag::Double(3.14));
	}

	/// Test that an NbtCompound can be converted into a struct using the `from_nbt` function.
	#[test]
	fn test_from_nbt() {
		let mut nbt = NbtCompound::new(Some("Test"));
		
		nbt.add("a", NbtTag::Int(42));
		nbt.add("b", NbtTag::String("Hello".to_string()));
		nbt.add("c", NbtTag::Double(3.14));

		let test: TestStruct = nbt.into();

		assert_eq!(test.a, 42);
		assert_eq!(test.b, "Hello");
		assert_eq!(test.c, 3.14);
	}

	#[derive(AsNbt, FromNbt, Debug, PartialEq, Clone)]
	struct OptionTestStruct {
		a: i32,
		b: Option<String>,
		c: Option<f64>,
	}
	
	/// Test struct with None values to and from NBT.
	#[test]
	fn test_simple_none_nbt() {
		let test = OptionTestStruct {
			a: 0,
			b: None,
			c: None,
		};
		
		let nbt: NbtCompound = test.clone().into();
		assert_eq!(nbt["a"], NbtTag::Int(0));
		assert_eq!(nbt["b"], NbtTag::None);
		assert_eq!(nbt["c"], NbtTag::None);
		
		let test2: OptionTestStruct = nbt.into();
		assert_eq!(test, test2);
	}
	
	/// Test for struct with Option fields to and from NBT
	#[test]
	fn test_simple_option_nbt() {
		let test = OptionTestStruct {
			a: 0,
			b: Some("Hello".to_string()),
			c: Some(2.6),
		};
		
		let nbt: NbtCompound = test.clone().into();
		assert_eq!(nbt["a"], NbtTag::Int(0));
		assert_eq!(nbt["b"], NbtTag::String("Hello".to_string()));
		assert_eq!(nbt["c"], NbtTag::Double(2.6));
		
		let test2: OptionTestStruct = nbt.into();
		assert_eq!(test, test2);
	}

	#[test]
	fn test_deserialize_from_bytes() {
		let mut deserializer = McDeserializer::new(&[10, 0, 11, 100, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110, 8, 0, 9, 116, 114, 97, 110, 115, 108, 97, 116, 101, 0]);
		let compound = NbtTag::mc_deserialize(&mut deserializer).expect("Failed to deserialize NBT from bytes");
		println!("{:?}", compound);
	}
}