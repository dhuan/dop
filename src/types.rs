use crate::value::*;

pub trait DataFormat {
    fn from_str(&self, s: &str) -> Option<Value>;
    fn to_str(&self, value: &Value, pretty: bool) -> Option<String>;
}

pub struct ScriptEnv {
    pub value_type: String,
    pub file_set_value: String,
    pub file_set_value_string: String,
    pub key: String,
}

pub type ScriptLibFn = dyn Fn(&ScriptEnv, Option<&[&str]>) -> (Option<String>, bool);

#[derive(Debug, PartialEq)]
pub enum ValueType {
    Auto,
    String,
    Int,
    Float,
}

impl ValueType {
    pub fn to_string(&self) -> &str {
        match self {
            ValueType::String => "string",
            ValueType::Int => "int",
            ValueType::Float => "float",
            ValueType::Auto => "auto",
        }
    }
}
