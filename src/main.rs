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

fn main() {
    let args = Args::parse();

    let mut stdin_buffer = String::new();

    std::io::stdin()
        .read_to_string(&mut stdin_buffer)
        .expect("Failed to read stdin!!!");

    let tmp_file = mktemp().expect("failed to create tmp file!");

    let value: serde_json::Value =
        serde_json::from_str(stdin_buffer.as_str()).expect("Failed to parse json!!!");

    let json_new = traverse(&value, move |key, value| {
        std::fs::write(tmp_file.clone(), to_value_for_mod(value))
            .expect("Failed to write to file!");

        let tmp_file_modified_time = std::fs::metadata(tmp_file.clone())
            .unwrap()
            .modified()
            .unwrap();

        let exit_ok = exec(
            args.script.as_str(),
            &vec![
                ("KEY", key.as_str()),
                ("VALUE", value.to_string().as_str()),
                ("SET_VALUE", tmp_file.as_str()),
            ],
        )
        .expect("command failed!");

        if !exit_ok {
            return TraverseAction::Remove;
        }

        if !file_has_been_modified(&tmp_file, &tmp_file_modified_time).unwrap() {
            return TraverseAction::Leave;
        }

        let mut value_modified = resolve_value(
            String::from_utf8(
                std::fs::read(tmp_file.clone()).expect("Failed to read tmp file after executing."),
            )
            .expect("Failed to parse to string.")
            .as_str(),
        );

        if value_modified.is_string() {
            value_modified = Value::from(trim_new_line(value_modified.as_str().unwrap()));
        }

        TraverseAction::Change(value_modified)
    });

    println!("{json_new}");
}

fn to_value_for_mod(value: &Value) -> String {
    if value.is_string() {
        return value.to_string().replace(r#"""#, "");
    }

    value.to_string()
}

fn resolve_value(value: &str) -> Value {
    let value = trim_new_line(value);

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
