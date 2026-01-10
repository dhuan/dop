use clap::{Parser, Subcommand};
use std::io::Read;

mod common;
mod json;
mod path;
mod script_lib;
mod toml;
mod types;
mod yaml;

use crate::common::*;
use crate::types::*;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    commands: Option<Commands>,

    #[command(flatten)]
    args: Args,
}

#[derive(Subcommand)]
enum Commands {
    Set(SetArgs),
    KeyMatch { search: String },
    IsString,
    IsNumber,
    IsBool,
    IsList,
    IsObject,
    IsNull,
}

#[derive(Clone, clap::Args)]
struct SetArgs {
    value: String,
    #[arg(short = 's', long = "string")]
    convert_to_string: bool,
}

#[derive(Clone, clap::Args, Debug)]
struct Args {
    #[arg(short = 'e', long = "execute")]
    script: Option<String>,
    #[arg(short, long)]
    query: Option<String>,
    #[arg(short, long = "key-filter")]
    key_filter_regex: Option<String>,
    #[arg(short = 'K', long = "key-equal")]
    key_filter_equal: Option<String>,
    #[arg(short, long)]
    output_format: Option<String>,
    #[arg(short, long)]
    input_format: Option<String>,
    #[arg(short = 'P', long)]
    pretty: bool,
    #[arg(short, long)]
    verbose: bool,
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
    &FormatConfig {
        name: "toml",
        format: &toml::Toml {},
    },
];

fn main() {
    let cli = Cli::parse();

    let script_lib_fn: Option<(Box<ScriptLibFn>, Option<&[&str]>)> = match &cli.commands {
        Some(Commands::KeyMatch { search }) => {
            Some((Box::new(script_lib::key_match), Some(&[search])))
        }
        Some(Commands::IsNull) => Some((Box::new(script_lib::is_null), None)),
        Some(Commands::IsString) => Some((Box::new(script_lib::is_string), None)),
        Some(Commands::IsNumber) => Some((Box::new(script_lib::is_number), None)),
        Some(Commands::IsBool) => Some((Box::new(script_lib::is_bool), None)),
        Some(Commands::IsList) => Some((Box::new(script_lib::is_list), None)),
        Some(Commands::IsObject) => Some((Box::new(script_lib::is_object), None)),
        Some(Commands::Set(args)) => Some((
            Box::new(script_lib::set(match args.convert_to_string {
                false => ValueType::Auto,
                true => ValueType::String,
            })),
            Some(&vec![args.value.as_str()]),
        )),
        None => None,
    };
    if let Some((f, param)) = script_lib_fn {
        match script_lib::parse_script_env() {
            None => {
                println!("Failed to parse script env!");
            }
            Some(env) => {
                let (result, ok) = f(&env, param);

                if let Some(result) = result {
                    println!("{result}");
                }

                if !ok {
                    std::process::exit(1);
                }
            }
        }

        return;
    }

    let available_formats = FORMATS
        .iter()
        .map(|format| format.name)
        .collect::<Vec<&str>>();
    let mut stdin_buffer = String::new();

    std::io::stdin()
        .read_to_string(&mut stdin_buffer)
        .expect("Failed to read stdin!!!");

    let log_v: fn(message: &str) -> () = match cli.args.verbose {
        false => |_| {},
        true => |message| eprintln!("{} {}", chrono::offset::Local::now(), message),
    };

    let value = match cli.args.input_format {
        None => {
            log_v("Input format was not specified. Will try to find out.");

            guess_value(stdin_buffer.as_str())
        }
        Some(input_format) => match FORMATS.iter().find(|&format| format.name == input_format) {
            None => {
                println!("Invalid format!");

                std::process::exit(1);
            }
            Some(&format) => match format.format.from_str(stdin_buffer.as_str()) {
                None => None,
                Some(value) => Some((value, format)),
            },
        },
    };

    if let None = value {
        log_v("Failed to parse input.");

        std::process::exit(1);
    }

    let (value, format) = value.unwrap();

    log_v(&format!("Input format identified as: {}", format.name));

    let output_format = match cli.args.output_format {
        None => format,
        Some(output_format) => *FORMATS
            .into_iter()
            .find(|&format| format.name == output_format)
            .unwrap_or_else(|| {
                println!(
                    "Format not supported: {}\n\nAvailable formats: {}",
                    output_format,
                    available_formats.join(",")
                );

                std::process::exit(1);
            }),
    };

    let tmp_file_value = mktemp().expect("failed to create tmp file!");
    let tmp_file_value_string = mktemp().expect("failed to create tmp file!");
    let tmp_file_value_number = mktemp().expect("failed to create tmp file!");

    let mut value = value.traverse(|key, key_encoded, value| {
        let field_name = match key.last().unwrap() {
            crate::path::PathEntry::Field(field_name) => field_name,
            crate::path::PathEntry::Index(index) => &format!("{}", index),
        };

        if let None = cli.args.script {
            return TraverseAction::Leave;
        }

        log_v(&format!("Processing key '{}'.", key_encoded));

        if let Some(key_filter_regex) = cli.args.key_filter_regex.clone() {
            if !regex_test(&key_filter_regex, &key_encoded) {
                log_v(&format!("Key Filter Regex did not pass, skipping."));
                return TraverseAction::Leave;
            }
        }

        if let Some(key_filter_equal) = cli.args.key_filter_equal.clone() {
            if key_filter_equal != key_encoded {
                log_v(&format!("Key Filter did not pass, skipping."));
                return TraverseAction::Leave;
            }
        }

        for file in vec![
            &tmp_file_value,
            &tmp_file_value_string,
            &tmp_file_value_number,
        ] {
            if let Err(err) = std::fs::write(file, UNCHANGED_CONTENT) {
                eprintln!("Failed to write to temporary files: {}", err.to_string());

                std::process::exit(1);
            }
        }

        let exit_ok = exec(
            cli.args.script.clone().unwrap().as_str(),
            &vec![
                ("KEY", key_encoded),
                (
                    "VALUE",
                    unquote(value.to_string(output_format.format, false).as_str()),
                ),
                ("VALUE_TYPE", &value.type_encoded()),
                ("SET_VALUE", tmp_file_value.as_str()),
                ("SET_VALUE_STRING", tmp_file_value_string.as_str()),
                ("SET_VALUE_NUMBER", tmp_file_value_number.as_str()),
                ("FIELD_NAME", field_name),
            ],
        )
        .expect("command failed!");

        if !exit_ok {
            log_v("Script's exit status was not OK. Removing value.");
            return TraverseAction::Remove;
        }

        let (new_value_file, new_value_type): (&str, Option<ValueType>) =
            if file_has_been_modified(&tmp_file_value).unwrap() {
                (&tmp_file_value, Some(ValueType::Auto))
            } else if file_has_been_modified(&tmp_file_value_string).unwrap() {
                (&tmp_file_value_string, Some(ValueType::String))
            } else if file_has_been_modified(&tmp_file_value_number).unwrap() {
                (&tmp_file_value_number, Some(ValueType::Int))
            } else {
                ("", None)
            };

        if let None = new_value_type {
            log_v("The value was not modified, moving on.");

            return TraverseAction::Leave;
        }

        let new_value_type = new_value_type.unwrap();

        let mut value_modified = resolve_value(
            String::from_utf8(
                std::fs::read(new_value_file).expect("Failed to read tmp file after executing."),
            )
            .expect("Failed to parse to string.")
            .as_str(),
            &new_value_type,
            format.format,
        );

        if let Value::String(s) = value_modified {
            value_modified = Value::String(trim_new_line(&s).to_string());
        }

        log_v(&format!(
            "Value was modified to ({}) {}",
            new_value_type.to_string(),
            value_modified.to_string(format.format, false)
        ));

        TraverseAction::Change(value_modified)
    });

    log_v(&format!(
        "Execution finished. Printing out in '{}' format.",
        output_format.name
    ));

    if let Some(query) = cli.args.query {
        if let Some(value) = value.change(&crate::path::decode(&query).unwrap()) {
            println!("{}", value.to_string(output_format.format, cli.args.pretty));
        }

        return;
    }

    println!(
        "{}",
        output_format
            .format
            .to_str(&value, cli.args.pretty)
            .unwrap()
    );
}

fn resolve_value(value: &str, t: &ValueType, format: &dyn DataFormat) -> Value {
    let value = trim_new_line(value);

    if *t == ValueType::String {
        return Value::String(value.to_string());
    }

    if *t == ValueType::Int {
        return Value::Int(value.parse::<i64>().unwrap());
    }

    if *t == ValueType::Float {
        return Value::Float(value.parse::<f64>().unwrap());
    }

    if *t == ValueType::String {
        return Value::Int(value.parse::<i64>().unwrap());
    }

    if value == "null" && *t == ValueType::Auto {
        return Value::Null;
    }

    if value == "true" {
        return Value::Bool(true);
    }

    if value == "false" {
        return Value::Bool(false);
    }

    if let Ok(num) = value.parse::<i64>() {
        return Value::Int(num);
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
