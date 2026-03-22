use crate::value::*;

pub trait DataFormat {
    fn from_str(&self, s: &str) -> Option<Value>;
    fn to_str(&self, value: &Value, pretty: bool) -> Option<String>;
}
