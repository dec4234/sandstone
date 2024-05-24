use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeStruct;

pub(crate) mod protocol_details;
pub(crate) mod util;
pub(crate) mod packets;
pub(crate) mod network;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        println!("Hello");
    }
}

/*pub trait ProtocolSerialization: Sized {
    fn proto_serialize(&self, serializer: Serializer) -> Vec<u8>;

    fn proto_deserialize(bytes: Vec<u8>) -> Self;
}*/

pub struct Body {
    name: String,
    num: i32,
}

impl Serialize for Body {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer{
        let mut s = serializer.serialize_struct("Body", 2)?;
        s.serialize_field("name", &self.name);
        s.end()
    }
}

/*impl ProtocolSerialization for Body {
    fn proto_serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn proto_deserialize(bytes: Vec<u8>) -> Self {
        todo!()
    }
}*/

pub enum MyType {
    ALPHA(Body)
}

pub struct McSerializer {
    pub output: String,
}

impl McSerializerTrait for McSerializer {

}

pub trait McSerializerTrait {

}


