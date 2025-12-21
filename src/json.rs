use crate::types::{DataFormat, Value};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub struct Json {}

impl DataFormat for Json {
    fn from_str(&self, s: &str) -> Option<Value> {
        to_value(&serde_json::from_str::<JsonValue>(s).ok()?)
    }
    fn to_str(&self, value: &Value) -> Option<String> {
        Some(format!("{}", to_json_value(value)?))
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_all_values() {
        test_json(
            r#"[1,2,3]"#,
            action_change(|num: i64| num * 2),
            r#"[2,4,6]"#,
        );
    }

    #[test]
    fn map_some_values() {
        test_json(
            r#"[1,2,3]"#,
            if_key_equals_then_action_change("1", |num: i64| num * 2),
            r#"[1,4,3]"#,
        );
    }

    #[test]
    fn map_with_objects() {
        test_json(
            r#"{"foo":{"bar":10,"some_key":"this will be intact"}}"#,
            if_key_equals_then_action_change("foo.bar", |num: i64| num * 2),
            r#"{"foo":{"bar":20,"some_key":"this will be intact"}}"#,
        );
    }

    #[test]
    fn remove_one_value() {
        test_json(
            r#"[1,2,3]"#,
            if_key_equals_then_action_remove(&vec!["1"]),
            r#"[1,3]"#,
        );
    }

    #[test]
    fn remove_one_value_from_object() {
        test_json(
            r#"{"a":{"b":{"c":"ok","d":"ok"}}}"#,
            if_key_equals_then_action_remove(&vec!["a.b.d"]),
            r#"{"a":{"b":{"c":"ok"}}}"#,
        );
    }

    #[test]
    fn remove_all() {
        test_json(
            r#"{"a":{"b":{"c":"ok","d":"ok"}}}"#,
            |_, _| TraverseAction::Remove,
            r#"{}"#,
        );
    }

    #[test]
    fn remove_one_value_from_object_list() {
        test_json(
            r#"{"countries":[{"continent":"Europe","name":"Germany"},{"continent":"South America","name":"Brazil"}]}"#,
            if_key_equals_then_action_remove(&vec!["countries.0.continent"]),
            r#"{"countries":[{"name":"Germany"},{"continent":"South America","name":"Brazil"}]}"#,
        );
    }

    fn test_json<T>(json_str: &str, f: T, expected_result: &str)
    where
        T: Fn(String, &Value) -> TraverseAction,
    {
        let value: Value = serde_json::from_str(json_str).expect("Failed to parse json!");

        let result = traverse(&value, f);

        assert_eq!(format!("{}", result), expected_result);
    }

    pub fn action_change<T, U>(mapper: impl Fn(T) -> U) -> impl Fn(String, &Value) -> TraverseAction
    where
        T: DeserializeOwned + Serialize,
        U: DeserializeOwned + Serialize,
    {
        move |_, value| {
            let typed_value: T =
                serde_json::from_value(value.clone()).expect("Failed to deserialize value");

            let mapped_value = mapper(typed_value);

            let new_value =
                serde_json::to_value(mapped_value).expect("Failed to serialize mapped value");

            TraverseAction::Change(new_value)
        }
    }

    pub fn if_key_equals_then_action_change<T, U>(
        key_search: &str,
        mapper: impl Fn(T) -> U,
    ) -> impl Fn(String, &Value) -> TraverseAction
    where
        T: DeserializeOwned + Serialize,
        U: DeserializeOwned + Serialize,
    {
        move |key, value| {
            if key_search != key {
                return TraverseAction::Leave;
            }

            action_change(&mapper)(key, value)
        }
    }

    pub fn if_key_equals_then_action_remove(
        key_search: &[&str],
    ) -> impl Fn(String, &Value) -> TraverseAction {
        |key, _| {
            if key_search.contains(&key.as_str()) {
                return TraverseAction::Remove;
            }

            TraverseAction::Leave
        }
    }
}
*/

fn to_json_value(value: &Value) -> Option<JsonValue> {
    match value {
        Value::String(value) => Some(JsonValue::from(value.to_owned())),
        Value::Number(value) => Some(JsonValue::from(*value)),
        Value::Bool(value) => Some(JsonValue::from(*value)),
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

fn to_value(value: &JsonValue) -> Option<Value> {
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
        return Some(Value::Number(value.as_i64().unwrap()));
    }

    if value.is_string() {
        return Some(Value::String(value.as_str().unwrap().to_owned()));
    }

    if value.is_object() {
        let mut obj: HashMap<String, Value> = std::collections::HashMap::new();

        for (key, value) in value.as_object().unwrap().iter() {
            obj.insert(key.clone(), to_value(value).unwrap());
        }

        return Some(Value::Object(obj));
    }

    None
}
