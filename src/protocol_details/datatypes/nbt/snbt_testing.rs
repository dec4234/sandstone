use crate::protocol_details::datatypes::nbt::nbt::{NbtCompound, NbtTag};
use crate::protocol_details::datatypes::nbt::snbt::SNBT;

#[test]
pub fn simple_test() {
	let base = "TAG_Compound('hello world'): 1 entry{TAG_String('name'): 'dec4234'}";
	
	let mut nbt = NbtCompound::new(Some("hello world"));
	nbt.add("name", NbtTag::String("dec4234".to_string()));
	
	let nbt = NbtTag::Compound(nbt);
	
	assert_eq!(nbt.to_snbt(None), base);
}