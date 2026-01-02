use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(i64),
    Bool(bool),
    List(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
}

impl Value {
    pub fn change<'a>(&'a mut self, path: &[&str]) -> Option<&'a mut Value> {
        let mut current = self;

        if path.len() == 0 {
            return Some(current);
        }

        for &visit_item in path {
            current = match current {
                Value::List(list) => {
                    let index = visit_item.parse::<usize>().ok()?;

                    list.get_mut(index)?
                }
                Value::Object(obj) => obj.get_mut(visit_item)?,
                _ => {
                    return None;
                }
            };
        }

        Some(current)
    }

    pub fn remove<'a>(&'a mut self, path: &[&str]) {
        let (parent, key) = match path.len() {
            0 => {
                return;
            }
            1 => (self, path[0]),
            _ => (
                self.change(&path[0..(path.len() - 1)]).unwrap(),
                path[path.len() - 1],
            ),
        };

        if let Value::List(list) = parent {
            let index = key.parse::<usize>();
            if let Err(_) = index {
                return;
            }

            let index = index.unwrap();

            list.remove(index);
        }

        if let Value::Object(obj) = parent {
            obj.remove(key);
        }
    }

    pub fn to_string(&self, format: &dyn DataFormat, pretty: bool) -> String {
        match self {
            Value::String(value) => value.to_owned(),
            Value::Bool(value) => value.to_string(),
            Value::Number(value) => format!("{value}"),
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
            Value::Number(_) => "number",
            Value::Bool(_) => "bool",
            Value::Object(_) => "object",
            Value::List(_) => "list",
            Value::Null => "null",
        })
        .to_string()
    }

    pub fn traverse<T>(&self, f: T) -> Value
    where
        T: Fn(String, &Value) -> TraverseAction,
    {
        let mut value = self.clone();
        let mut visit: VecDeque<String> = VecDeque::new();
        for key in get_keys(&value).unwrap_or(vec![]) {
            visit.push_back(key);
        }

        while let Some(path_base) = visit.pop_front() {
            let path = path_base.split(".").collect::<Vec<&str>>();

            let value_current = get_nested(&mut value, &path);

            if let None = value_current {
                continue;
            }

            let value_current = value_current.unwrap();

            match f(path.join("."), &value_current) {
                TraverseAction::Change(value_changed) => {
                    if let Some(value_ptr) = value.change(&path) {
                        *value_ptr = value_changed;
                    }
                }
                TraverseAction::Remove => {
                    if let Some(Value::List(list)) =
                        get_nested(&mut value, &path[0..(path.len() - 1)])
                    {
                        if path.last().unwrap().parse::<usize>().unwrap() != (list.len() - 1) {
                            visit.push_front(path_base.clone());
                        }
                    }

                    value.remove(&path);
                }
                TraverseAction::Leave => {}
            }

            let keys = get_keys(&value_current);
            if let Some(keys) = keys {
                let path = path.join(".");
                for key in keys.iter() {
                    visit.push_back(format!("{}.{}", path, key));
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

fn get_keys(value: &Value) -> Option<Vec<String>> {
    if let Value::Object(obj) = value {
        return Some(obj.keys().into_iter().map(|key| key.to_owned()).collect());
    }

    if let Value::List(list) = value {
        return Some(
            vec![0; list.len()]
                .iter()
                .enumerate()
                .map(|(i, _)| i.to_string())
                .collect::<Vec<String>>(),
        );
    }

    None
}

fn get_nested(value: &mut Value, path: &[&str]) -> Option<Value> {
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
    Number,
}

impl ValueType {
    pub fn to_string(&self) -> &str {
        match self {
            ValueType::String => "string",
            ValueType::Number => "number",
            ValueType::Auto => "auto",
        }
    }
}
