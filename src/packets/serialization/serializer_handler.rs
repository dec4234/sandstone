use std::fmt::{Debug, Display};

use crate::packets::serialization::serializer_error::SerializingErr;

pub type DeserializeResult<'a, T> = Result<T, SerializingErr>;

pub struct McSerializer {
    pub output: Vec<u8>
}

impl McSerializer {
    pub fn new() -> Self {
        Self {
            output: vec![]
        }
    }

    pub fn clear(&mut self) {
        self.output.clear();
    }

    pub fn serialize_bytes(&mut self, input: &[u8]) {
        let mut i = self.output.len();
        self.output.resize(self.output.len() + input.len(), 1); // maybe this is helpful?

        for b in input {
            self.output[i] = *b;
            i += 1;
        }
    }

    pub fn serialize_vec(&mut self, vec: Vec<u8>) {
        self.serialize_bytes(vec.as_slice());
    }

    pub fn serialize_u8(&mut self, b: u8) {
        self.output.push(b);
    }

    pub fn serialize_str(&mut self, s: &str) {
        self.serialize_bytes(s.as_bytes());
    }
}

pub struct McDeserializer<'a> {
    pub data: &'a [u8],
    pub index: usize
}

impl <'a> McDeserializer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            index: 0
        }
    }

    pub fn collect_remaining(&self) -> &[u8] {
        &self.data[self.index..]
    }

    pub fn remainder(&self) -> &'a [u8] {
        &self.data[self.index..]
    }

    pub fn increment(&mut self, amount: usize) {
        self.index += amount;
    }

    pub fn increment_by_diff(&mut self, other: usize) {
        if other > self.index {
            self.increment(other - self.index);
        }
    }

    pub fn isAtEnd(&self) -> bool {
        self.index >= self.data.len()
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn create_sub_deserializer(&self) -> McDeserializer {
        McDeserializer::new(&self.data[self.index..])
    }
}

pub trait McDeserialize {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> where Self: Sized;
}

pub trait McSerialize {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr>;
}