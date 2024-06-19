//! Testing for the serialization and deserialization of primitive types.

#[cfg(test)]
mod primitive_testing {
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};

	#[test]
	fn test_string_serialization() {
		let mut serializer = McSerializer::new();

		"ABC".to_string().mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!("ABC".to_string(), String::mc_deserialize(&mut deserializer).unwrap());
		assert_eq!(serializer.output, vec![3, 65, 66, 67]);

		serializer.clear();

		"HELLO WORLD 123456789".to_string().mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!("HELLO WORLD 123456789".to_string(), String::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
		
		"".to_string().mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!("".to_string(), String::mc_deserialize(&mut deserializer).unwrap());
	}
	
	#[test]
	fn test_prim_unsigned_serialization() {
		let mut serializer = McSerializer::new();
		
		253u8.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(253u8, u8::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
		
		147u16.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(147u16, u16::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
		
		5678990u32.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(5678990u32, u32::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
		
		5678990878787989798u64.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(5678990878787989798u64, u64::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
	}
	
	#[test]
	fn test_prim_signed_serialization() {
		let mut serializer = McSerializer::new();
		
		89i8.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(89i8, i8::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
		
		147i16.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(147i16, i16::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
		
		5678990i32.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(5678990i32, i32::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
		
		5678990878787989798i64.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(5678990878787989798i64, i64::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();

		(-89i8).mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(-89i8, i8::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();

		(-147i16).mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(-147i16, i16::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();

		(-5678990i32).mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(-5678990i32, i32::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();

		(-5678990878787989798i64).mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(-5678990878787989798i64, i64::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
	}
	
	#[test]
	fn test_boolean_serialization() {
		let mut serializer = McSerializer::new();
		
		true.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(true, bool::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
		
		false.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(false, bool::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
	}
	
	#[test]
	fn test_option_serialzation() {
		let mut serializer = McSerializer::new();
		
		Some(5u8).mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(Some(5u8), Option::<u8>::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
		
		None::<u8>.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(None, Option::<u8>::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
	}
	
	#[test]
	fn test_vec_serialization() {
		let mut serializer = McSerializer::new();
		
		vec![1u8, 2u8, 3u8].mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(vec![1u8, 2u8, 3u8], Vec::<u8>::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
		
		Vec::<u8>::new().mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(Vec::<u8>::new(), Vec::<u8>::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
	}
	
	#[test]
	#[allow(unused_allocation)]
	fn test_box_serialization() {
		let mut serializer = McSerializer::new();
		
		Box::new(5u8).mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(&mut serializer.output);
		assert_eq!(Box::new(5u8), Box::<u8>::mc_deserialize(&mut deserializer).unwrap());
		
		serializer.clear();
	}
}