use crate::value::*;

pub trait DataFormat {
    fn from_str(&self, s: &str) -> Option<Value>;
    fn to_str(&self, value: &Value, pretty: bool) -> Option<String>;
}

pub struct ScriptEnv {
    pub value_type: String,
    pub file_set_value: String,
    pub key: String,
    pub format_name: String,
    pub is_script_once: bool,
}

pub type ScriptLibFn =
    dyn Fn(&ScriptEnv, Option<&[&str]>, &dyn DataFormat) -> (Option<String>, bool);

#[derive(Debug, PartialEq)]
pub enum ValueType {
    Auto,
    String,
}
