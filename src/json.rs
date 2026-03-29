use crate::types::DataFormat;
use crate::value::*;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub struct Json {}

impl DataFormat for Json {
    fn from_str(&self, s: &str) -> Option<Value> {
        to_value(&serde_json::from_str::<JsonValue>(s).ok()?)
    }
    fn to_str(&self, value: &Value, pretty: bool) -> Option<String> {
        if pretty {
            return serde_json::to_string_pretty(&to_json_value(value)?).ok();
        }

        Some(format!("{}", to_json_value(value)?))
    }
}

pub fn to_json_value(value: &Value) -> Option<JsonValue> {
    match value {
        Value::String(value) => Some(JsonValue::from(value.to_owned())),
        Value::Int(value) => Some(JsonValue::from(*value)),
        Value::Float(value) => Some(JsonValue::from(*value)),
        Value::Bool(value) => Some(JsonValue::from(*value)),
        Value::Null => Some(JsonValue::Null),
        Value::List(list) => Some(JsonValue::Array(
            list.iter()
                .map(|value| to_json_value(value).unwrap())
                .collect(),
        )),
        Value::Object(value) => {
            let mut obj = serde_json::Map::new();

            for (key, value) in value {
                obj.insert(
                    key_to_string(key),
                    to_json_value(value).unwrap_or(JsonValue::Null),
                );
            }

            Some(JsonValue::from(obj))
        }
    }
}

pub fn to_value(value: &JsonValue) -> Option<Value> {
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

    if value.is_boolean() {
        return Some(Value::Bool(value.as_bool().unwrap()));
    }

    if value.is_number() {
        if let Some(num) = value.as_i64() {
            return Some(Value::Int(num));
        }

        if let Some(num) = value.as_f64() {
            return Some(Value::Float(num));
        }

        return None;
    }

    if value.is_string() {
        return Some(Value::String(value.as_str().unwrap().to_owned()));
    }

    if value.is_null() {
        return Some(Value::Null);
    }

    if value.is_object() {
        let mut obj: HashMap<Key, Value> = std::collections::HashMap::new();

        for (key, value) in value.as_object().unwrap().iter() {
            obj.insert(to_key(key), to_value(value).unwrap());
        }

        return Some(Value::Object(obj));
    }

    None
}
