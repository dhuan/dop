#[cfg(test)]
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::VecDeque;

pub enum TraverseAction {
    Leave,
    Remove,
    Change(Value),
}

fn get_nested(value: &Value, path: Vec<&str>) -> Option<Value> {
    let mut result = Some(value.clone());
    for path_item in path {
        if let None = result {
            return None;
        }

        let value = result.clone().unwrap();

        if starts_with_num(path_item) {
            if let Some(value) = value.get(atoi(path_item).unwrap_or(0) as usize) {
                result = Some(value.clone());

                continue;
            } else {
                return None;
            }
        }

        if let Some(value) = value.get(path_item) {
            result = Some(value.clone());
        } else {
            return None;
        }
    }

    result
}

fn starts_with_num(s: &str) -> bool {
    match s.chars().next() {
        None => false,
        Some(c) => c.to_string().parse::<i32>().is_ok(),
    }
}

fn atoi(s: &str) -> Option<i32> {
    match s.to_string().parse::<i32>() {
        Err(_) => None,
        Ok(num) => Some(num),
    }
}

pub fn traverse<T>(value: &Value, f: T) -> Value
where
    T: Fn(String, &Value) -> TraverseAction,
{
    let mut value = value.clone();
    let mut visit: VecDeque<String> = VecDeque::new();
    for key in get_keys(&value).unwrap_or(vec![]) {
        visit.push_back(key);
    }

    while let Some(path) = visit.pop_front() {
        let path = path.split(".").collect::<Vec<&str>>();

        let value_current = get_nested(&value, path.clone());

        if let None = value_current {
            continue;
        }

        let value_current = value_current.unwrap();

        match f(path.join("."), &value_current) {
            TraverseAction::Change(value_changed) => {
                if let Some(value_ptr) = value.pointer_mut(serde_path(&path).as_str()) {
                    *value_ptr = value_changed;
                }
            }
            TraverseAction::Remove => {
                if path.len() == 1 {
                    if value.is_array() {
                        let value = value.as_array_mut().unwrap();

                        value.remove(path[0].parse::<usize>().unwrap());
                    }

                    if value.is_object() {
                        value.as_object_mut().unwrap().remove(path[0]);
                    }
                } else {
                    let (parent_path, field_name_to_remove) = parent_json_path(&path);

                    let parent_path: Vec<&str> = parent_path.iter().map(|s| s.as_str()).collect();

                    let value_mut = value
                        .pointer_mut(serde_path(&parent_path).as_str())
                        .unwrap();

                    if value_mut.is_object() {
                        value_mut
                            .as_object_mut()
                            .unwrap()
                            .remove(field_name_to_remove.as_str())
                            .unwrap();
                    }
                }
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

fn serde_path(path: &[&str]) -> String {
    format!("/{}", path.join("/"))
}

fn get_keys(value: &Value) -> Option<Vec<String>> {
    if value.is_object() {
        return Some(
            value
                .as_object()
                .unwrap()
                .iter()
                .map(|(key, _)| key.clone())
                .collect(),
        );
    }

    if value.is_array() {
        return Some(
            vec![0; value.as_array().unwrap().len()]
                .iter()
                .enumerate()
                .map(|(i, _)| i.to_string())
                .collect::<Vec<String>>(),
        );
    }

    None
}

fn parent_json_path(path: &Vec<&str>) -> (Vec<String>, String) {
    if path.len() < 2 {
        panic!("Failed!!!!!!!!!!!!!!!!!!!");
    }

    let new_path: Vec<String> = (&path[0..=(path.len() - 2)])
        .iter()
        .map(|s| s.to_string())
        .collect();

    let last_key: Vec<String> = (&path[path.len() - 1..])
        .iter()
        .map(|s| s.to_string())
        .collect();

    (new_path, last_key[0].clone())
}

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
