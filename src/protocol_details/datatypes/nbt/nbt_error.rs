use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum NbtParseError {
    InputEndedPrematurely,
    UnknownTypeNumber,
    UnexpectedByte,
    MissingEndTag,
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