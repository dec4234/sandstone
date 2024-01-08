use quartz_nbt::io::Flavor;
use quartz_nbt::{io, NbtCompound};

#[ignore]
#[test]
fn test_serializer_nbt() {
    let mut compound = NbtCompound::new();
    compound.insert("foo", 123);
    compound.insert("bar", -3.6f32);

    let mut binary: Vec<u8> = Vec::new();
    io::write_nbt(&mut binary, Some("root-tag"), &compound, Flavor::Uncompressed).unwrap();

    println!("Out: {:?}", binary);
}