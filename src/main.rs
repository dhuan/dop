use clap::Parser;
use std::io::Read;

mod common;
mod json;
mod types;
mod yaml;

use crate::common::*;
use crate::types::{DataFormat, TraverseAction, Value};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    script: String,
    #[arg(short, long)]
    output_format: Option<String>,
}

#[derive(Debug, PartialEq)]
enum ValueType {
    Auto,
    String,
    Number,
}

struct FormatConfig {
    name: &'static str,
    format: &'static dyn DataFormat,
}

const FORMATS: &[&FormatConfig] = &[
    &FormatConfig {
        name: "json",
        format: &json::Json {},
    },
    &FormatConfig {
        name: "yaml",
        format: &yaml::Yaml {},
    },
];

fn main() {
    let args = Args::parse();

    let available_formats = FORMATS
        .iter()
        .map(|format| format.name)
        .collect::<Vec<&str>>();
    let mut stdin_buffer = String::new();

    std::io::stdin()
        .read_to_string(&mut stdin_buffer)
        .expect("Failed to read stdin!!!");

    let value = guess_value(stdin_buffer.as_str());
    if let None = value {
        println!("Failed to parse input.");

        std::process::exit(1);
    }

    let (value, format) = value.unwrap();

    let output_format = args
        .output_format
        .clone()
        .unwrap_or(format.name.to_string());

    let output_format = FORMATS
        .into_iter()
        .find(|&format| format.name == output_format)
        .unwrap_or_else(|| {
            println!(
                "Format not supported: {}\n\nAvailable formats: {}",
                output_format,
                available_formats.join(",")
            );

            std::process::exit(1);
        });

    let tmp_file_value = mktemp().expect("failed to create tmp file!");
    let tmp_file_value_string = mktemp().expect("failed to create tmp file!");
    let tmp_file_value_number = mktemp().expect("failed to create tmp file!");

    let value = value.traverse(|key, value| {
        let tmp_file_modified_time = get_modified_time(&tmp_file_value).unwrap();
        let tmp_file_string_modified_time = get_modified_time(&tmp_file_value_string).unwrap();
        let tmp_file_number_modified_time = get_modified_time(&tmp_file_value_number).unwrap();

        let exit_ok = exec(
            args.script.as_str(),
            &vec![
                ("KEY", key.as_str()),
                ("VALUE", unquote(value.to_string().as_str())),
                ("SET_VALUE", tmp_file_value.as_str()),
                ("SET_VALUE_STRING", tmp_file_value_string.as_str()),
                ("SET_VALUE_NUMBER", tmp_file_value_number.as_str()),
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
            } else if file_has_been_modified(&tmp_file_value_number, &tmp_file_number_modified_time)
                .unwrap()
            {
                (tmp_file_value_number.clone(), Some(ValueType::Number))
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
            format.format,
        );

        if let Value::String(s) = value_modified {
            value_modified = Value::String(trim_new_line(&s).to_string());
        }

        TraverseAction::Change(value_modified)
    });

    println!("{}", output_format.format.to_str(&value).unwrap());
}

fn resolve_value(value: &str, t: ValueType, format: &dyn DataFormat) -> Value {
    let value = trim_new_line(value);

    if t == ValueType::String {
        return Value::String(value.to_string());
    }

    if t == ValueType::Number {
        return Value::Number(value.parse::<i64>().unwrap());
    }

    if value == "true" {
        return Value::Bool(true);
    }

    if value == "false" {
        return Value::Bool(false);
    }

    if let Ok(num) = value.parse::<i64>() {
        return Value::Number(num);
    }

    if value.starts_with("[") || value.starts_with("{") {
        if let Some(value) = format.from_str(value) {
            return value;
        }
    }

    return Value::String(value.to_string());
}

fn guess_value(stdin: &str) -> Option<(Value, &'static FormatConfig)> {
    for &format in FORMATS {
        if let Some(value) = format.format.from_str(stdin) {
            return Some((value, format));
        }
    }

    None
}
