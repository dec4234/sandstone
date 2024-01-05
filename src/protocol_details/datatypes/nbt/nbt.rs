use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub enum NbtParseError {
    UnmatchedBrace,
}

impl Debug for NbtParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())?;
        Ok(())
    }
}

impl Display for NbtParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            _ => f.write_str("Unknown Error"),
        }
    }
}

impl Error for NbtParseError {}

pub struct NamedTag {
    name: String,
    tag: NbtTag,
}



pub enum NbtTag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    List((u8, i32, Vec<NbtTag>)),
    Compound((String, Vec<NbtTag>)),
    Byte_Array((i32, Vec<i8>)),
    Int_Array((i32, Vec<i32>)),
    Long_Array((i32, Vec<i64>)),
}

impl NbtTag {
    pub fn type_id(&self) -> u8 {
        match self {
            NbtTag::End => {0}
            NbtTag::Byte(_) => {1}
            NbtTag::Short(_) => {2}
            NbtTag::Int(_) => {3}
            NbtTag::Long(_) => {4}
            NbtTag::Float(_) => {5}
            NbtTag::Double(_) => {6}
            NbtTag::String(_) => {8}
            NbtTag::List(_) => {9}
            NbtTag::Compound(_) => {10}
            NbtTag::Byte_Array(_) => {7}
            NbtTag::Int_Array(_) => {11}
            NbtTag::Long_Array(_) => {12}
        }
    }

    pub fn get_payload_size(&self) -> Option<u8> {
        match self {
            NbtTag::End => {Some(0)}
            NbtTag::Byte(_) => {Some(1)}
            NbtTag::Short(_) => {Some(2)}
            NbtTag::Int(_) => {Some(4)}
            NbtTag::Long(_) => {Some(8)}
            NbtTag::Float(_) => {Some(4)}
            NbtTag::Double(_) => {Some(8)}
            NbtTag::String(_) => {None}
            NbtTag::List(_) => {None}
            NbtTag::Compound(_) => {None}
            NbtTag::Byte_Array(_) => {None}
            NbtTag::Int_Array(_) => {None}
            NbtTag::Long_Array(_) => {None}
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            NbtTag::End => {"TAG_End".to_string()}
            NbtTag::Byte(_) => {"TAG_Byte".to_string()}
            NbtTag::Short(_) => {"TAG_Short".to_string()}
            NbtTag::Int(_) => {"TAG_Int".to_string()}
            NbtTag::Long(_) => {"TAG_Long".to_string()}
            NbtTag::Float(_) => {"TAG_Float".to_string()}
            NbtTag::Double(_) => {"TAG_Double".to_string()}
            NbtTag::String(_) => {"TAG_String".to_string()}
            NbtTag::List(_) => {"TAG_List".to_string()}
            NbtTag::Compound(_) => {"TAG_Compound".to_string()}
            NbtTag::Byte_Array(_) => {"TAG_Byte_Array".to_string()}
            NbtTag::Int_Array(_) => {"TAG_Int_Array".to_string()}
            NbtTag::Long_Array(_) => {"TAG_Long_Array".to_string()}
        }
    }
}