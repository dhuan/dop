use crate::{path::PathEntry, value::*};

pub trait DataFormat {
    fn from_str(&self, s: &str) -> Option<Value>;
    fn to_str(&self, value: &Value, pretty: bool) -> Result<String, ToStrError>;
}

#[derive(Debug)]
pub enum ToStrError {
    #[allow(unused)]
    ParseError(String),
    UnsupportedType((String, Vec<PathEntry>)),
}
