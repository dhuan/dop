use crate::types::{DataFormat, Value};
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;

pub struct Yaml {}

impl DataFormat for Yaml {
    fn from_str(&self, s: &str) -> Option<Value> {
        to_value(&serde_yaml::from_str::<YamlValue>(s).ok()?)
    }
    fn to_str(&self, value: &Value) -> Option<String> {
        Some(format!(
            "{}",
            serde_yaml::to_string(&to_yaml_value(value)?).ok()?
        ))
    }
}

fn to_yaml_value(value: &Value) -> Option<YamlValue> {
    match value {
        Value::String(value) => Some(YamlValue::from(value.to_owned())),
        Value::Number(value) => Some(YamlValue::from(*value)),
        Value::Bool(value) => Some(YamlValue::from(*value)),
        Value::List(list) => Some(YamlValue::Sequence(
            list.iter()
                .map(|value| to_yaml_value(value).unwrap())
                .collect(),
        )),
        Value::Object(value) => {
            let mut obj = serde_yaml::Mapping::new();

            let mut keys = value.keys().collect::<Vec<&String>>();
            keys.sort();

            for key in keys {
                obj.insert(
                    serde_yaml::Value::from(key.to_owned()),
                    to_yaml_value(value.get(key).unwrap()).unwrap_or(YamlValue::Null),
                );
            }

            Some(YamlValue::from(obj))
        }
    }
}

fn to_value(value: &YamlValue) -> Option<Value> {
    if value.is_sequence() {
        return Some(Value::List(
            value
                .as_sequence()
                .unwrap()
                .iter()
                .map(|value| to_value(value).unwrap())
                .collect(),
        ));
    }

    if value.is_bool() {
        return Some(Value::Bool(value.as_bool().unwrap()));
    }

    if value.is_number() {
        return Some(Value::Number(value.as_i64().unwrap()));
    }

    if value.is_string() {
        return Some(Value::String(value.as_str().unwrap().to_owned()));
    }

    if value.is_mapping() {
        let mut obj: HashMap<String, Value> = std::collections::HashMap::new();

        for (key, value) in value.as_mapping().unwrap().iter() {
            obj.insert(key.as_str().unwrap().to_owned(), to_value(value).unwrap());
        }

        return Some(Value::Object(obj));
    }

    None
}
