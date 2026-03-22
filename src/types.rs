use crate::value::*;

pub trait DataFormat {
    fn from_str(&self, s: &str) -> Option<Value>;
    fn to_str(&self, value: &Value, pretty: bool) -> Option<String>;
}

#[derive(Clone)]
pub struct ScriptEnv {
    pub file_set_value: String,
    pub key: String,
    pub is_script_once: bool,
}
