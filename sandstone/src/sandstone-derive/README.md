# sandstone-derive
This is a support package for my other project, [sandstone](github.com/dec4234/sandstone). It provides a procedural macro for deriving the necessary traits for the sandstone library.

This package enables derives for the `McSerialize` and `McDeserialize` traits from the sandstone library.

## Example
```rust
#[derive(McSerialize, McDeserialize)]
pub struct TestStruct {
    pub field1: i32,
    pub field2: String,
}
```

This will create mc_serialize and mc_deserialize implementations for the struct `TestStruct`. This allows it to be sent over
the minecraft protocol.

This package is meant to be used in conjunction with the sandstone library, and is not intended to be used on its own.