use crate::types::DataFormat;
use crate::value::*;
use std::collections::HashMap;
use toml::Value as TomlValue;

pub struct Toml {}

impl DataFormat for Toml {
    fn from_str(&self, s: &str) -> Option<Value> {
        to_value(&toml::from_str(s).ok()?)
    }
    fn to_str(&self, value: &Value, _: bool) -> Option<String> {
        toml::to_string(&to_toml_value(value).unwrap()).ok()
    }
}

fn to_value(value: &TomlValue) -> Option<Value> {
    if value.is_array() {
        return Some(Value::List(
            value
                .as_array()
                .unwrap()
                .iter()
                .map(|value| to_value(value).unwrap())
                .collect(),
        ));
    }

    if value.is_bool() {
        return Some(Value::Bool(value.as_bool().unwrap()));
    }

    if value.is_integer() {
        return Some(Value::Int(value.as_integer().unwrap()));
    }

    if value.is_float() {
        return Some(Value::Float(value.as_float().unwrap()));
    }

    if value.is_str() {
        return Some(Value::String(value.as_str().unwrap().to_owned()));
    }

    if value.is_table() {
        let mut obj: HashMap<String, Value> = std::collections::HashMap::new();

        for (key, value) in value.as_table().unwrap().iter() {
            obj.insert(key.clone(), to_value(value).unwrap());
        }

        return Some(Value::Object(obj));
    }

    None
}

fn to_toml_value(value: &Value) -> Option<TomlValue> {
    match value {
        Value::String(value) => Some(TomlValue::from(value.to_owned())),
        Value::Int(value) => Some(TomlValue::from(*value)),
        Value::Float(value) => Some(TomlValue::from(*value)),
        Value::Bool(value) => Some(TomlValue::from(*value)),
        Value::List(list) => Some(TomlValue::Array(
            list.iter()
                .map(|value| to_toml_value(value).unwrap())
                .collect(),
        )),
        Value::Object(value) => {
            let mut obj = toml::Table::new();

            for (key, value) in value {
                obj.insert(
                    key.to_owned(),
                    to_toml_value(value).unwrap_or(TomlValue::from("null")),
                );
            }

            Some(TomlValue::from(obj))
        }
        Value::Null => Some(TomlValue::String("null".to_string())),
    }
}
