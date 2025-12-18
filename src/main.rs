use clap::Parser;
use serde_json::Value;
use std::io::Read;

mod common;
mod json;

use crate::common::*;
use crate::json::*;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    script: String,
}

#[derive(Debug, PartialEq)]
enum ValueType {
    Auto,
    String,
}

fn main() {
    let args = Args::parse();

    let mut stdin_buffer = String::new();

    std::io::stdin()
        .read_to_string(&mut stdin_buffer)
        .expect("Failed to read stdin!!!");

    let tmp_file_value = mktemp().expect("failed to create tmp file!");
    let tmp_file_value_string = mktemp().expect("failed to create tmp file!");

    let value: serde_json::Value =
        serde_json::from_str(stdin_buffer.as_str()).expect("Failed to parse json!!!");

    let json_new = traverse(&value, move |key, value| {
        let tmp_file_modified_time = get_modified_time(&tmp_file_value).unwrap();
        let tmp_file_string_modified_time = get_modified_time(&tmp_file_value_string).unwrap();

        let exit_ok = exec(
            args.script.as_str(),
            &vec![
                ("KEY", key.as_str()),
                ("VALUE", value.to_string().as_str()),
                ("SET_VALUE", tmp_file_value.as_str()),
                ("SET_VALUE_STRING", tmp_file_value_string.as_str()),
            ],
        )
        .expect("command failed!");

        if !exit_ok {
            return TraverseAction::Remove;
        }

        let (new_value_file, new_value_type): (String, Option<ValueType>) =
            if file_has_been_modified(&tmp_file_value, &tmp_file_modified_time).unwrap() {
                (tmp_file_value.clone(), Some(ValueType::Auto))
            } else if file_has_been_modified(&tmp_file_value_string, &tmp_file_string_modified_time)
                .unwrap()
            {
                (tmp_file_value_string.clone(), Some(ValueType::String))
            } else {
                ("".to_string(), None)
            };

        if let None = new_value_type {
            return TraverseAction::Leave;
        }

        let new_value_type = new_value_type.unwrap();

        let mut value_modified = resolve_value(
            String::from_utf8(
                std::fs::read(new_value_file).expect("Failed to read tmp file after executing."),
            )
            .expect("Failed to parse to string.")
            .as_str(),
            new_value_type,
        );

        if value_modified.is_string() {
            value_modified = Value::from(trim_new_line(value_modified.as_str().unwrap()));
        }

        TraverseAction::Change(value_modified)
    });

    println!("{json_new}");
}

fn resolve_value(value: &str, t: ValueType) -> Value {
    let value = trim_new_line(value);

    if t == ValueType::String {
        return Value::from(value);
    }

    if value == "true" {
        return Value::from(true);
    }

    if value == "false" {
        return Value::from(false);
    }

    if let Ok(num) = value.parse::<i64>() {
        return Value::from(num);
    }

    if value.starts_with("[") || value.starts_with("{") {
        if let Ok(value) = serde_json::from_str::<Value>(value) {
            return value;
        }
    }

    Value::from(value)
}
