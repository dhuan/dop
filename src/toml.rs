use crate::common::*;
use crate::path::*;
use crate::types::{DataFormat, ToStrError};
use crate::value::*;
use std::collections::HashMap;
use toml::Value as TomlValue;

pub struct Toml {}

impl DataFormat for Toml {
    fn from_str(&self, s: &str) -> Option<Value> {
        to_value(&toml::from_str(s).ok()?)
    }
    fn to_str(&self, value: &Value, _: bool) -> Result<String, ToStrError> {
        Ok(
            toml::to_string(&to_toml_value(value, &[]).map_err(|err| match err {
                ToTomlValueError::UnsupportedType((value, path)) => {
                    ToStrError::UnsupportedType((value.type_encoded(), path))
                }
            })?)
            .map_err(to_parse_error)?,
        )
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
        let mut obj: HashMap<Key, Value> = std::collections::HashMap::new();

        for (key, value) in value.as_table().unwrap().iter() {
            obj.insert(to_key(key), to_value(value).unwrap());
        }

        return Some(Value::Object(obj));
    }

    None
}

#[derive(Debug)]
enum ToTomlValueError {
    UnsupportedType((Value, Vec<PathEntry>)),
}

fn to_toml_value(value: &Value, path: &[PathEntry]) -> Result<TomlValue, ToTomlValueError> {
    match value {
        Value::String(value) => Ok(TomlValue::from(value.to_owned())),
        Value::Int(value) => Ok(TomlValue::from(*value)),
        Value::Float(value) => Ok(TomlValue::from(*value)),
        Value::Bool(value) => Ok(TomlValue::from(*value)),
        Value::List(list) => Ok(TomlValue::Array(
            list.iter()
                .enumerate()
                .map(|(i, value)| {
                    to_toml_value(value, &{
                        let mut path_new = path.to_owned();
                        path_new.push(PathEntry::Index(i));

                        path_new
                    })
                })
                .collect::<Result<Vec<TomlValue>, ToTomlValueError>>()?,
        )),
        Value::Object(value) => {
            let mut obj = toml::Table::new();

            for (key, value) in value {
                if let Value::Null = value {
                    return Err(ToTomlValueError::UnsupportedType((Value::Null, {
                        let mut path_new = path.to_owned();
                        path_new.push(PathEntry::Field(if let Key::String(s) = key {
                            s.to_owned()
                        } else {
                            panic!("Failed to parse key");
                        }));

                        path_new
                    })));
                }

                obj.insert(
                    key_to_string(key),
                    to_toml_value(value, &{
                        let mut path_new = path.to_owned();
                        path_new.push(match key {
                            Key::String(s) => PathEntry::Field(s.to_owned()),
                            Key::Int(i) => PathEntry::Index((*i).try_into().unwrap()),
                        });

                        path_new
                    })?,
                );
            }

            Ok(TomlValue::from(obj))
        }
        Value::Null => Err(ToTomlValueError::UnsupportedType((
            Value::Null,
            path.to_vec(),
        ))),
    }
}
