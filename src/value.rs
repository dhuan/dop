use crate::path::*;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, PartialEq)]
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
    pub fn has(&self, path: &[PathEntry]) -> bool {
        self.get(path).is_some()
    }

    pub fn get(&self, path: &[PathEntry]) -> Option<&Value> {
        if path.len() == 0 {
            return None;
        }

        if path.len() == 1 {
            return match (self, path[0].clone()) {
                (Value::List(list), PathEntry::Index(index)) => match list.len() {
                    0 => None,
                    _ => list.iter().nth(index),
                },
                (Value::Object(obj), PathEntry::Field(field_name)) => obj.get(&field_name),
                _ => None,
            };
        }

        match (self, path[0].clone()) {
            (Value::List(list), PathEntry::Index(index)) => match list.iter().nth(index) {
                None => None,
                Some(value) => value.get(&path[1..]),
            },
            (Value::Object(obj), PathEntry::Field(field_name)) => match obj.get(&field_name) {
                None => None,
                Some(value) => value.get(&path[1..]),
            },
            _ => None,
        }
    }

    pub fn add<'a>(&'a mut self, path: &[PathEntry], new_value: &Value) -> Option<Vec<PathEntry>> {
        if path.len() == 0 {
            return None;
        }

        let mut value = self;

        for i in 0..(path.len()) {
            if i == (path.len() - 1) {
                match (value, &path[i]) {
                    (Value::Object(obj), PathEntry::Field(field_name)) => {
                        obj.insert(field_name.clone(), new_value.clone());

                        return Some(path.to_vec());
                    }
                    (Value::List(list), PathEntry::IndexNew) => {
                        list.push(new_value.clone());

                        let mut path_new = path.to_vec();
                        let last = path_new.last_mut().unwrap();
                        *last = PathEntry::Index(list.len() - 1);

                        return Some(path_new);
                    }
                    _ => {
                        return None;
                    }
                };
            } else {
                value = match (value, &path[i]) {
                    (Value::Object(obj), PathEntry::Field(field_name)) => {
                        obj.get_mut(field_name)?
                    }
                    (Value::List(list), PathEntry::Index(index)) => list.iter_mut().nth(*index)?,
                    _ => {
                        return None;
                    }
                }
            }
        }

        None
    }

    pub fn change<'a>(&'a mut self, path: &[PathEntry]) -> Option<&'a mut Value> {
        let path = match self.has(path) {
            true => path,
            false => match self.add(path, &Value::Null) {
                None => {
                    return None;
                }
                Some(path_new) => &path_new.clone(),
            },
        };

        if !self.has(path) {
            self.add(path, &Value::Null);
        }

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

    #[allow(unused)]
    pub fn diff(&self, compare: &Value) -> Option<Vec<(Vec<PathEntry>, Value)>> {
        let mut result: Vec<(Vec<PathEntry>, Value)> = vec![];
        let mut ignore: Vec<String> = vec![];

        compare.traverse(|path, path_encoded, value, _| {
            if ignore.len() > 0 {
                let path = path.to_vec();
                for i in 0..(path.len()) {
                    let path_check = crate::path::encode(&path[0..i].to_vec());

                    if ignore.contains(&path_check) {
                        return TraverseAction::Leave;
                    }
                }
            }

            let should_add_to_result = match self.get(path) {
                None => true,
                Some(self_value) => {
                    !type_equals(self_value, value)
                        || match (self_value, value) {
                            (Value::String(a), Value::String(b)) => a != b,
                            (Value::Int(a), Value::Int(b)) => a != b,
                            (Value::Float(a), Value::Float(b)) => a != b,
                            (Value::Bool(a), Value::Bool(b)) => a != b,
                            _ => false,
                        }
                }
            };

            if should_add_to_result {
                result.push((path.to_vec(), value.clone()));

                let is_object_or_list = match value {
                    Value::Object(_) => true,
                    Value::List(_) => true,
                    _ => false,
                };

                if is_object_or_list {
                    ignore.push(path_encoded.to_string());
                }
            }

            TraverseAction::Leave
        });

        match result.len() {
            0 => None,
            _ => Some(result),
        }
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

    pub fn to_string<T>(&self, format: T, pretty: bool) -> String
    where
        T: Fn(&Value, bool) -> Option<String>,
    {
        match self {
            Value::String(value) => value.to_owned(),
            Value::Bool(value) => value.to_string(),
            Value::Int(value) => format!("{value}"),
            Value::Float(value) => format!("{value}"),
            Value::Object(value) => format(&Value::Object(value.clone()), pretty).unwrap(),
            Value::List(value) => format(&Value::List(value.clone()), pretty).unwrap(),
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

    pub fn traverse<T>(&self, mut f: T) -> Value
    where
        T: FnMut(&[PathEntry], &str, &Value, &Value) -> TraverseAction,
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

            match f(
                &path_base,
                &crate::path::encode(&path_base),
                &value_current,
                &value,
            ) {
                TraverseAction::ChangeRoot(value_new) => {
                    let value_previous = value.clone();
                    value = value_new;

                    if let Some(parent) = path_parent(&path_base) {
                        let value_parent = value.get(&parent);
                        let value_parent_previous = value_previous.get(&parent);

                        match (value_parent, value_parent_previous) {
                            (Some(Value::List(list_new)), Some(Value::List(list_previous))) => {
                                if list_previous.len() > list_new.len() {
                                    visit.push_front(path_base.clone());
                                }
                            }
                            _ => (),
                        }
                    }
                }
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
    #[allow(unused)]
    Remove,
    #[allow(unused)]
    Change(Value),
    ChangeRoot(Value),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DataFormat;
    use serde_json::Value as JsonValue;

    #[test]
    fn test_traverse_change_one_value() {
        let value_new = from_json("[1,2,3,4]").traverse(|_path, key_encoded, value, _value_all| {
            if key_encoded == "[1]" {
                if let Value::Int(num) = value {
                    return TraverseAction::Change(Value::Int(num * 2));
                }
            }

            TraverseAction::Leave
        });

        assert_eq!(value_new, from_json("[1,4,3,4]"));
    }

    #[test]
    fn test_traverse_change_root() {
        let value_new =
            from_json("[1,2,3,4]").traverse(|_path, _key_encoded, _value, _value_all| {
                TraverseAction::ChangeRoot(Value::String("changed!".to_string()))
            });

        assert_eq!(value_new, Value::String("changed!".to_string()));
    }

    #[test]
    fn test_change_existing_value() {
        let mut value = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);

        let value2 = value.change(&vec![PathEntry::Index(1)]).unwrap();
        *value2 = Value::String("changed".to_string());

        assert_eq!(to_json_str(&value, false).unwrap(), r#"[1,"changed",3,4]"#,);
    }

    #[test]
    fn test_traverse_delete_all_from_array() {
        let value_new =
            from_json(r#"{"list":[1,2,3]}"#).traverse(|path, _key_encoded, value, value_all| {
                match value {
                    Value::Int(_) => {
                        let mut value_all = value_all.clone();
                        value_all.remove(path);

                        TraverseAction::ChangeRoot(value_all)
                    }
                    _ => TraverseAction::Leave,
                }
            });

        assert_eq!(to_json_str(&value_new, false).unwrap(), r#"{"list":[]}"#,);
    }

    #[test]
    fn test_add_new_value_with_list() {
        test_add(
            r#"[1,2,3,4]"#,
            &vec![PathEntry::IndexNew],
            &Value::String("new value!".to_string()),
            r#"[1,2,3,4,"new value!"]"#,
        );
    }

    #[test]
    fn test_add_new_value_with_list_nested() {
        test_add(
            r#"{"foo":[1,2,3,4]}"#,
            &vec![PathEntry::Field("foo".to_string()), PathEntry::IndexNew],
            &Value::String("new value!".to_string()),
            r#"{"foo":[1,2,3,4,"new value!"]}"#,
        );
    }

    #[test]
    fn test_add_new_value_with_object_at_the_root() {
        test_add(
            r#"{"foo":[1,2,3,4]}"#,
            &vec![PathEntry::Field("bar".to_string())],
            &Value::String("new value!".to_string()),
            r#"{"bar":"new value!","foo":[1,2,3,4]}"#,
        );
    }

    #[test]
    fn test_add_new_value_with_object_nested() {
        test_add(
            r#"{"foo":{"bar":{"key_a": 1}}}"#,
            &vec![
                PathEntry::Field("foo".to_string()),
                PathEntry::Field("bar".to_string()),
                PathEntry::Field("key_b".to_string()),
            ],
            &Value::Int(2),
            r#"{"foo":{"bar":{"key_a":1,"key_b":2}}}"#,
        );
    }

    #[test]
    fn test_has() {
        let cases: Vec<(&str, &str, bool)> = vec![
            (r#"{"foo":{"bar":"some value"}}"#, "foo.bar", true),
            (r#"{"foo":{"bar":[1,2,3]}}"#, "foo.bar[1]", true),
            (r#"{"foo":{"bar":[1,2,3]}}"#, "foo.bar[3]", false),
            (r#"{"foo":{"bar":[{"a":"b"}]}}"#, "foo.bar[0].a", true),
            (r#"{"foo":{"bar":[{"a":"b"}]}}"#, "foo.bar[0].b", false),
        ];

        let json = crate::json::Json {};
        for (input, path, expect) in cases {
            let value = json.from_str(input).unwrap();

            assert_eq!(value.has(&crate::path::decode(path).unwrap()), expect);
        }
    }

    fn test_add(value: &str, path: &[PathEntry], value_to_add: &Value, expect: &str) {
        let mut value = crate::json::Json {}.from_str(value).unwrap();

        let value2 = value.change(path).unwrap();
        *value2 = value_to_add.clone();

        assert_eq!(
            value.to_string(
                |value, pretty| crate::json::Json {}.to_str(value, pretty),
                false
            ),
            expect,
        );
    }

    #[test]
    fn test_diff() {
        let cases = vec![
            (
                r#"{"foo":"bar"}"#,
                r#"{"foo":"bar","hello":"world"}"#,
                vec![(
                    vec![PathEntry::Field("hello".to_string())],
                    crate::json::Json {}.from_str(r#""world""#).unwrap(),
                )],
            ),
            (
                r#"{"foo":"bar"}"#,
                r#"{"foo":"bar","some_obj":{"a":{"b":{"c":"some value"}}}}"#,
                vec![(
                    vec![PathEntry::Field("some_obj".to_string())],
                    crate::json::Json {}
                        .from_str(r#"{"a":{"b":{"c":"some value"}}}"#)
                        .unwrap(),
                )],
            ),
            (
                r#"{"foo":"bar"}"#,
                r#"{"foo":"bar2"}"#,
                vec![(
                    vec![PathEntry::Field("foo".to_string())],
                    crate::json::Json {}.from_str(r#""bar2""#).unwrap(),
                )],
            ),
        ];

        for (input_a, input_b, expected) in cases {
            let value1 = crate::json::Json {}.from_str(input_a).unwrap();
            let value2 = crate::json::Json {}.from_str(input_b).unwrap();

            let result = value1.diff(&value2).unwrap();

            assert_eq!(result.len(), expected.len());

            for i in 0..result.len() {
                assert_eq!(result[i].1, expected[i].1);

                let path_a = &result[i].0;
                let path_b = &expected[i].0;

                assert_eq!(path_a.len(), path_b.len());

                for i in 0..path_a.len() {
                    assert_eq!(path_a[i], path_b[i]);
                }
            }
        }
    }

    fn from_json(json_encoded: &str) -> Value {
        crate::json::Json {}.from_str(json_encoded).unwrap()
    }

    fn to_json_str(value: &Value, pretty: bool) -> Option<String> {
        Some(match pretty {
            true => serde_json::to_string_pretty(&to_json_value(value)?).ok()?,
            false => serde_json::to_string(&to_json_value(value)?).ok()?,
        })
    }

    fn to_json_value(value: &Value) -> Option<JsonValue> {
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
                        key.to_owned(),
                        to_json_value(value).unwrap_or(JsonValue::Null),
                    );
                }

                Some(JsonValue::from(obj))
            }
        }
    }
}

fn type_equals(a: &Value, b: &Value) -> bool {
    a.type_encoded() == b.type_encoded()
}

fn path_parent(path: &[PathEntry]) -> Option<Vec<PathEntry>> {
    if path.len() < 2 {
        return None;
    }

    Some(path[0..(path.len() - 1)].to_vec())
}
