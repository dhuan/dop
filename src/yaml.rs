use crate::common::*;
use crate::types::{DataFormat, ToStrError};
use crate::value::*;
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;

pub struct Yaml {}

impl DataFormat for Yaml {
    fn from_str(&self, s: &str) -> Option<Value> {
        to_value(&serde_yaml::from_str::<YamlValue>(s).ok()?)
    }
    fn to_str(&self, value: &Value, _: bool) -> Result<String, ToStrError> {
        Ok(
            serde_yaml::to_string(&to_yaml_value(value).ok_or(ToStrError::ParseError(
                "Failed to convert to JSON value".to_string(),
            ))?)
            .map_err(to_parse_error)?
            .to_string(),
        )
    }
}

fn to_yaml_value(value: &Value) -> Option<YamlValue> {
    match value {
        Value::String(value) => Some(YamlValue::from(value.to_owned())),
        Value::Int(value) => Some(YamlValue::from(*value)),
        Value::Float(value) => Some(YamlValue::from(*value)),
        Value::Bool(value) => Some(YamlValue::from(*value)),
        Value::List(list) => Some(YamlValue::Sequence(
            list.iter()
                .map(|value| to_yaml_value(value).unwrap())
                .collect(),
        )),
        Value::Object(value) => {
            let mut obj = serde_yaml::Mapping::new();

            let mut keys = value
                .keys()
                .map(|key| {
                    to_yaml_value(
                        &(match key.clone() {
                            Key::String(s) => Value::String(s),
                            Key::Int(num) => Value::Int(num),
                        }),
                    )
                    .unwrap()
                })
                .collect::<Vec<YamlValue>>();

            keys.sort_by(|a, b| {
                serde_yaml::to_string(a)
                    .unwrap()
                    .cmp(&serde_yaml::to_string(b).unwrap())
            });

            for key in keys {
                let value =
                    to_yaml_value(value.get(&yaml_key_to_value_key(&key)).unwrap()).unwrap();

                obj.insert(
                    key,
                    value,
                    /*
                    to_yaml_value(try_get_from_value_object(value, &key).unwrap())
                        .unwrap_or(YamlValue::Null),
                    */
                );
            }

            Some(YamlValue::from(obj))
        }
        Value::Null => Some(YamlValue::Null),
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

    if value.is_mapping() {
        let mut obj: HashMap<Key, Value> = std::collections::HashMap::new();

        for (key, value) in value.as_mapping().unwrap().iter() {
            obj.insert(yaml_key_to_value_key(key), to_value(value).unwrap());
        }

        return Some(Value::Object(obj));
    }

    None
}

fn yaml_key_to_value_key(key: &YamlValue) -> Key {
    match key {
        YamlValue::String(s) => Key::String(s.to_owned()),
        YamlValue::Number(num) => Key::Int(num.as_i64().unwrap()),
        _ => panic!("Not supported yet."),
    }
}
