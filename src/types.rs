use crate::path::PathEntry;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
}

impl Value {
    pub fn change<'a>(&'a mut self, path: &[PathEntry]) -> Option<&'a mut Value> {
        let mut current = self;

        if path.len() == 0 {
            return Some(current);
        }

        for visit_item in path {
            current = match (current, visit_item) {
                (Value::List(list), PathEntry::Index(index)) => list.get_mut(*index)?,
                (Value::Object(obj), PathEntry::Field(field_name)) => obj.get_mut(field_name)?,
                _ => {
                    return None;
                }
            };
        }

        Some(current)
    }

    pub fn remove<'a>(&'a mut self, path: &[PathEntry]) {
        let (parent, key) = match path.len() {
            0 => {
                return;
            }
            1 => (self, &path[0]),
            _ => (
                self.change(&path[0..(path.len() - 1)]).unwrap(),
                &path[path.len() - 1],
            ),
        };

        match (parent, key) {
            (Value::List(list), PathEntry::Index(index)) => {
                list.remove(*index);
            }
            (Value::Object(obj), PathEntry::Field(field_name)) => {
                obj.remove(field_name);
            }
            _ => {}
        }
    }

    pub fn to_string(&self, format: &dyn DataFormat, pretty: bool) -> String {
        match self {
            Value::String(value) => value.to_owned(),
            Value::Bool(value) => value.to_string(),
            Value::Int(value) => format!("{value}"),
            Value::Float(value) => format!("{value}"),
            Value::Object(value) => format
                .to_str(&Value::Object(value.clone()), pretty)
                .unwrap(),
            Value::List(value) => format.to_str(&Value::List(value.clone()), pretty).unwrap(),
            Value::Null => "null".to_string(),
        }
    }

    pub fn type_encoded(&self) -> String {
        (match *self {
            Value::String(_) => "string",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Object(_) => "object",
            Value::List(_) => "list",
            Value::Null => "null",
        })
        .to_string()
    }

    pub fn traverse<T>(&self, f: T) -> Value
    where
        T: Fn(&[PathEntry], &str, &Value) -> TraverseAction,
    {
        let mut value = self.clone();
        let mut visit: VecDeque<Vec<PathEntry>> = VecDeque::new();
        for key in get_keys(&value).unwrap_or(vec![]) {
            visit.push_back(vec![key]);
        }

        while let Some(path_base) = visit.pop_front() {
            let value_current = get_nested(&mut value, &path_base);

            if let None = value_current {
                continue;
            }

            let value_current = value_current.unwrap();

            match f(&path_base, &crate::path::encode(&path_base), &value_current) {
                TraverseAction::Change(value_changed) => {
                    if let Some(value_ptr) = value.change(&path_base) {
                        *value_ptr = value_changed;
                    }
                }
                TraverseAction::Remove => {
                    if let Some(Value::List(list)) =
                        get_nested(&mut value, &path_base[0..(path_base.len() - 1)])
                    {
                        let last_index = match path_base.last().unwrap() {
                            PathEntry::Index(index) => *index,
                            _ => panic!("Something went wrong"),
                        };

                        if last_index != (list.len() - 1) {
                            visit.push_front(path_base.clone());
                        }
                    }

                    value.remove(&path_base);
                }
                TraverseAction::Leave => {}
            }

            let keys = get_keys(&value_current);
            if let Some(keys) = keys {
                for key in keys.iter() {
                    let mut new_entry = path_base.clone();
                    new_entry.push(key.clone());

                    visit.push_back(new_entry);
                }
            }
        }

        value
    }
}

pub enum TraverseAction {
    Leave,
    Remove,
    Change(Value),
}

fn get_keys(value: &Value) -> Option<Vec<PathEntry>> {
    if let Value::Object(obj) = value {
        return Some(
            obj.keys()
                .into_iter()
                .map(|key| PathEntry::Field(key.to_owned()))
                .collect(),
        );
    }

    if let Value::List(list) = value {
        return Some(
            vec![0; list.len()]
                .iter()
                .enumerate()
                .map(|(i, _)| PathEntry::Index(i))
                .collect::<Vec<PathEntry>>(),
        );
    }

    None
}

fn get_nested(value: &mut Value, path: &[PathEntry]) -> Option<Value> {
    Some(value.change(&path)?.to_owned())
}

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
