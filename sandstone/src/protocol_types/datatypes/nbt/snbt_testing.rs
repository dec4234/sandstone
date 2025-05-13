#![allow(unused_imports)]

use crate::protocol_types::datatypes::nbt::nbt::{NbtCompound, NbtTag};

#[test]
pub fn simple_test() {
	let _base = "TAG_Compound('hello world'): 1 entry{TAG_String('name'): 'dec4234'}";
	
	let mut nbt = NbtCompound::new(Some("hello world"));
	nbt.add("name", NbtTag::String("dec4234".to_string()));
	
	NbtTag::Compound(nbt);
}

/**
TODO: current issue is that the name parsing either catches too much or not at all
Probably should leave name parsing to the caller...
*/
#[ignore]
#[test]
pub fn regex_testing() {
	let input = "TAG_Float('value'): 0.75TAG_String('name'): 'Hampus'TAG_Double('doubleTest'): 0.49312871321823148TAG_Long(None): 14TAG_Float('floatTest'): 0.49823147058486938\
	TAG_String('name'): 'Compound tag #1'TAG_Byte('byteTest'): 127TAG_Int('SpawnY'): 63TAG_Short('id'): 25";

	// basic types
	let re = regex::Regex::new(r"TAG_(\w+)\((.+?)\): (\d\.\d+|'\w+'|\d+)").unwrap();

	let captures = re.captures_iter(input);

	let cap = re.captures(input).unwrap();
	println!("Type: {}, Name: {}, Value: {}", &cap[1], &cap[2], &cap[3]);

	for cap in captures {
		let _end = if let Some(Some(cap)) = cap.iter().last() { // use this to split input string
			Some(cap.end())
		} else {
			None
		};
		
		println!("Type: {}, Name: {}, Value: {}", &cap[1], &cap[2], &cap[3]);
	}
	
	println!("\nList types\n");
	
	// compound and list
	let input = "TAG_Compound('Level'): 11 entriesTAG_List('listTest (compound)'): 2 entries";
	let re = regex::Regex::new(r"TAG_(\w+)\(('.*?')\): \d+ (entry|entries)").unwrap();
	
	let cap = re.captures_iter(input);
	
	for (_, cap) in cap.enumerate() {
		println!("Type: {}, Name: {}, Value: {}", &cap[1], &cap[2], &cap[3]);
	}
	
	println!("\nArray types\n");
	
	// byte, int, long array
	let input = "TAG_Byte_Array('byteArray'): [1, 2, 3, 4, 5]TAG_Int_Array('intArray'): [1, 2, 3, 4, 5]TAG_Long_Array('longArray'): [1, 2, 3, 4, 5]";
	let re = regex::Regex::new(r"TAG_(\w+_Array)\((.+?)\): \[([\d, ]*)\]").unwrap(); // not redundant
	
	let cap = re.captures_iter(input);
	
	for (_, cap) in cap.enumerate() {
		println!("Type: {}, Name: {}, Value: {}", &cap[1], &cap[2], &cap[3]);
	}
}