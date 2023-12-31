use crate::packets::serialization::serialize_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol_details::datatypes::var_types::VarInt;


impl McSerialize for String {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
        VarInt(self.len() as i32).mc_serialize(serializer)?;
        serializer.serialize_bytes(self.as_bytes());

        Ok(())
    }
}

impl McDeserialize for String {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, String> {
        let var_output = VarInt::mc_deserialize(deserializer)?;
        let bounds: (usize, usize) = (deserializer.index + 1, deserializer.index + var_output.0 as usize + 1);
        let s = String::from_utf8(deserializer.data[bounds.0..bounds.1].to_vec());

        deserializer.increment(var_output.0 as usize + 1); //

        if let Ok(s) = s {
            Ok(s)
        } else {
            Err(SerializingErr::CouldNotDeserializeString)
        }
    }
}

impl McSerialize for u8 {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
        serializer.serialize_u8(*self);

        Ok(())
    }
}

impl McDeserialize for u8 {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, u8> {
        if deserializer.data.len() == 0 {
            return Err(SerializingErr::InputEnded);
        }

        let b = deserializer.data[deserializer.index];
        deserializer.increment(1);

        return Ok(b);
    }
}

struct Testing {
    first: u8,
    second: String,
    third: String,
    fourth: u8,
    fifth: String
}

impl McSerialize for Testing {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
        self.first.mc_serialize(serializer).unwrap();
        self.second.mc_serialize(serializer).unwrap();
        self.third.mc_serialize(serializer).unwrap();
        self.fourth.mc_serialize(serializer).unwrap();
        self.fifth.mc_serialize(serializer).unwrap();

        Ok(())
    }
}

impl McDeserialize for Testing {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> where Self: Sized {
        let testing = Testing {
            first: u8::mc_deserialize(deserializer)?,
            second: String::mc_deserialize(deserializer)?,
            third: String::mc_deserialize(deserializer)?,
            fourth: u8::mc_deserialize(deserializer)?,
            fifth: String::mc_deserialize(deserializer)?,
        };

        Ok(testing)
    }
}

#[test]
fn testing2() {
    let a = Testing {
        first: 16,
        second: "hello".to_string(),
        third: "abcd".to_string(),
        fourth: 13,
        fifth: "zxy".to_string()
    };

    let mut serializer = McSerializer::new();
    a.mc_serialize(&mut serializer).unwrap();
    println!("Serialized: {:?}", serializer.output);

    let mut deserializer = McDeserializer::new(&serializer.output);

    let b = Testing::mc_deserialize(&mut deserializer).unwrap();
    println!("First: {}", b.first);
    println!("Second: \"{}\"", b.second);
    println!("Third: \"{}\"", b.third);
    println!("Fourth: {}", b.fourth);
    println!("Fifth: \"{}\"", b.fifth);
    println!("Data: {:?}", deserializer.data);
}