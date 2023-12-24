use std::any::{Any, TypeId};
use std::env::var;
use std::fmt::{Debug, Display, Error, Formatter};
use serde::{Deserialize, Deserializer, ser, Serialize, Serializer};
use anyhow::{anyhow, Result};
use bincode::config;
use serde::de::{Error as SerdeError, Visitor};
use serde::de::Unexpected::StructVariant;
use crate::packets::versions::v1_20;
use crate::protocol_details::datatypes::var_types;
use crate::protocol_details::datatypes::var_types::VarInt;

pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    let mut serializer = McSerializer {
        output: vec![]
    };

    value.serialize(&mut serializer)?;
    Ok(String::from_utf8(serializer.output)?)
}

// https://serde.rs/impl-serializer.html
pub struct McSerializer {
    pub output: Vec<u8>,
}

impl McSerializer {
    pub fn new() -> Self {
        McSerializer {
            output: vec![]
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.output.as_slice()
    }

    fn add_bytes(&mut self, vec: Vec<u8>) {
        for b in vec {
            self.output.push(b)
        }
    }

    fn add_byte_slice(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.output.push(*b);
        }
    }

    fn serialize_varint(&mut self, var_int: VarInt) -> Result<(), Error> {
        var_int.serialize(self)
    }
}

impl <'a> Serializer for &'a mut McSerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.output.push(v as u8);

        Ok(())
    }

    // Note that all data sent must be big Endian
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(v.to_be_bytes().as_slice())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(v.to_be_bytes().as_slice())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(v.to_be_bytes().as_slice())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.output.push(v);

        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(v.to_be_bytes().as_slice())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(v.to_be_bytes().as_slice())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_varint(VarInt(v.len() as i32))?;
        self.add_byte_slice(v.as_bytes());

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error> where T: Serialize {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<Self::Ok, Self::Error> where T: Serialize {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(self, name: &'static str, variant_index: u32, variant: &'static str, value: &T) -> Result<Self::Ok, Self::Error> where T: Serialize {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
        variant.serialize(&mut *self)?;
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        variant.serialize(&mut *self)?;
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut McSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error> where T: ?Sized + Serialize {
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for &'a mut McSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error> where T: ?Sized + Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut McSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error> where T: ?Sized + Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut McSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error> where T: ?Sized + Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut McSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error> where T: ?Sized + Serialize {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Error> where T: ?Sized + Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut McSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error> where T: ?Sized + Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut McSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error> where T: ?Sized + Serialize {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct McDeserializer<'de> {
    byte_slice: &'de [u8],
    index: usize
}

impl<'de> McDeserializer<'de> {
    pub fn new(byte_slice: &'de[u8]) -> Self {
        Self {
            byte_slice,
            index: 0
        }
    }

    pub fn parse_u8(&mut self) -> Result<u8, serde::de::value::Error> {
        if self.index >= self.byte_slice.len() {
            return Err(serde::de::value::Error::custom("Index out of bounds for incoming bytes"));
        }

        self.index += 1;

        Ok(self.byte_slice[self.index - 1])
    }

    pub fn parse_varint(&mut self) -> Result<VarInt, serde::de::value::Error> {
        if self.byte_slice.len() == 0 {
            return Err(serde::de::value::Error::custom("No data to parse from"));
        }

        let mut bytes: Vec<u8> = vec![];
        let mut i = 0;

        while self.byte_slice[self.index] & var_types::CONTINUE_BYTE == 1 && i < 5 {
            bytes.push(self.byte_slice[self.index]);

            i += 1;
            self.increment()?;
        }

        // push the last byte or only byte
        bytes.push(self.byte_slice[self.index]);
        self.increment()?;

        return if let Ok(v) = VarInt::new_from_bytes(bytes) {
            Ok(v)
        } else {
            Err(serde::de::value::Error::custom("Could not parse VarInt"))
        }
    }

    pub fn parse_string(&mut self) -> Result<String, serde::de::value::Error> {
        let len = self.parse_varint()?; // no need to separately increment

        let mut i = 0;

        let mut bytes = vec![];

        while i < len.0 && self.index < self.byte_slice.len() {
            bytes.push(self.byte_slice[self.index]);

            i += 1;
            let _ = self.increment(); // ignore result, we check for out of bounds
        }

        return if let Ok(s) = String::from_utf8(bytes) {
            Ok(s)
        } else {
            Err(serde::de::value::Error::custom("Could not parse String"))
        }
    }

    pub fn parse_struct<V: Visitor<'de>>(&mut self) -> Result<V::Value, serde::de::value::Error>
        where <V as Visitor<'de>>::Value: Deserialize<'de> {

        if let Ok(val) = bincode::deserialize::<V::Value>(self.byte_slice) {
            Ok(val)
        } else {
            Err(serde::de::value::Error::custom("Could not deserialize struct"))
        }
    }

    fn increment(&mut self) -> Result<(), serde::de::value::Error> {
        self.index += 1;

        if self.index >= self.byte_slice.len() {
            Err(serde::de::value::Error::custom("Index out of bounds while deserializing"))
        } else {
            Ok(())
        }
    }

    fn remaining(&self) -> Result<usize, serde::de::value::Error>{
        return Ok(self.byte_slice.len() - self.index);
    }
}

impl<'de, 'a> Deserializer<'de> for &'a mut McDeserializer<'de> {
    type Error = serde::de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_i16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_i64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        visitor.visit_u8(self.parse_u8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        let b: [u8; 2] = [self.byte_slice[0], self.byte_slice[1]];

        visitor.visit_u16(u16::from_be_bytes(b))
    }

    fn deserialize_u32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_f32<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_char<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        visitor.visit_str(self.parse_string()?.as_str())
    }

    fn deserialize_string<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        visitor.visit_string(self.parse_string()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_unit_struct<V>(self, name: &'de str, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_newtype_struct<V>(self, name: &'de str, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_tuple_struct<V>(self, name: &'de str, len: usize, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_struct<V>(self, name: &'de str, fields: &'de [&'de str], visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        panic!("Should not attempt to deserialize struct");
    }

    fn deserialize_enum<V>(self, name: &'de str, variants: &'de [&'de str], visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        println!("{:?}", variants);

        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error> where V: Visitor<'de> {
        todo!()
    }
}


#[derive(Serialize)]
struct Testing {
    s: u16
}

impl <'a> Deserialize<'a> for Testing {
    fn deserialize<D>(d: D) -> Result<Self, <D as Deserializer<'a>>::Error> where D: Deserializer<'a> {
        Ok(Self {
            s: u16::deserialize(d)?,
        })
    }
}

#[derive(Serialize)]
enum TestEnum {
    ALPHA(Testing)
}

impl <'de> Deserialize<'de> for TestEnum {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {
        let test= Testing::deserialize(deserializer);

        if let Ok(test) = test {
            return Ok(TestEnum::ALPHA(test));
        } else {
            return Err(test.err().unwrap());
        }
    }
}

pub struct MyVisitor {

}

impl Visitor<'_> for MyVisitor {
    type Value = u16;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        todo!()
    }

    fn visit_u16<E>(self, v: u16) -> std::result::Result<Self::Value, E> where E: SerdeError {
        Ok(v)
    }
}

#[test]
pub fn test_serialize() {
    let mut serializer = McSerializer::new();
    let test = Testing {
        s: 35,
    };

    test.serialize(&mut serializer).unwrap();

    println!("{:?}", serializer.output);

    let mut deserializer = McDeserializer::new(serializer.as_bytes());
    /*let out = Testing::deserialize(&mut deserializer).unwrap();
    println!("{}", out.s);*/

    let out = TestEnum::deserialize(&mut deserializer).unwrap();

    match out {
        TestEnum::ALPHA(b) => {println!("Inner: {}", b.s)}
    }
}

#[test]
fn serialize_handshake() {
    let handshake = v1_20::HandshakingBody {
        protocol_version: VarInt(758),
        server_address: "localhost".to_string(),
        port: 25565,
        next_state: VarInt(1),
    };

    let mut serializer = McSerializer::new();

    handshake.serialize(&mut serializer).unwrap();
    println!("{:?}", serializer.output);

    // length, id      protocol      Address                                          port         next state
    // [16, 0,         246, 5,       9, 108, 111, 99, 97, 108, 104, 111, 115, 116,    99, 221,     1]
}