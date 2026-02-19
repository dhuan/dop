use clap::{Parser, Subcommand};
use std::io::Read;

mod common;
mod json;
mod path;
mod script_lib;
mod toml;
mod types;
mod value;
mod yaml;

use crate::common::*;
use crate::types::*;
use crate::value::*;

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
    value: Vec<String>,
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
            Some(&args.value.iter().map(|s| s.as_str()).collect::<Vec<&str>>()),
        )),
        None => None,
    };
    if let Some((f, param)) = script_lib_fn {
        match script_lib::parse_script_env() {
            None => {
                println!("Failed to parse script env!");
            }
            Some(env) => {
                let format = FORMATS
                    .iter()
                    .find(|format| format.name == env.format_name)
                    .unwrap();

                let (result, ok) = f(&env, param, format.format);

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

    let mut value = value.traverse(|key, key_encoded, value, value_all| {
        let field_name = match key.last().unwrap() {
            crate::path::PathEntry::Field(field_name) => field_name,
            crate::path::PathEntry::Index(index) => &format!("{}", index),
            _ => panic!("Not accepted!"),
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

        if let Err(err) = std::fs::write(
            &tmp_file_value,
            format.format.to_str(&value_all, true).unwrap(),
        ) {
            eprintln!("Failed to write to temporary files: {}", err.to_string());

            std::process::exit(1);
        }

        let (exit_ok, stdout, stderr) = exec(
            cli.args.script.clone().unwrap().as_str(),
            &vec![
                ("KEY", key_encoded),
                (
                    "VALUE",
                    unquote(
                        value
                            .to_string(
                                |value, pretty| output_format.format.to_str(value, pretty),
                                false,
                            )
                            .as_str(),
                    ),
                ),
                ("VALUE_TYPE", &value.type_encoded()),
                ("VALUE_ALL", tmp_file_value.as_str()),
                ("VALUE_FORMAT", format.name),
                ("FIELD_NAME", field_name),
            ],
        )
        .expect("command failed!");

        log_v(&format!(
            "Output: {}",
            match (stdout.clone(), stderr.clone()) {
                (None, None) => "N/A".to_string(),
                _ => {
                    let mut out = String::new();

                    if let Some(stdout) = stdout {
                        out.push_str(format!("\nstdout: {}", stdout).as_str());
                    }

                    if let Some(stderr) = stderr {
                        out.push_str(format!("\nstderr: {}", stderr).as_str());
                    }

                    out
                }
            }
        ));

        if !exit_ok {
            log_v("Script's exit status was not OK. Removing value.");
            return TraverseAction::Remove;
        }

        let value_modified = match file_has_been_modified(&tmp_file_value).unwrap() {
            false => None,
            true => Some(
                format
                    .format
                    .from_str(
                        &String::from_utf8(
                            std::fs::read(&tmp_file_value).expect("Failed to read tmp file."),
                        )
                        .unwrap(),
                    )
                    .unwrap(),
            ),
        };

        if value_modified.is_none() {
            log_v("The value was not modified, moving on.");

            return TraverseAction::Leave;
        }

        let value_modified = value_modified.unwrap();

        log_v(&format!(
            "Value was modified to {}",
            format.format.to_str(&value_modified, false).unwrap(),
        ));

        TraverseAction::ChangeRoot(value_modified)
    });

    log_v(&format!(
        "Execution finished. Printing out in '{}' format.",
        output_format.name
    ));

    if let Some(query) = cli.args.query {
        if let Some(value) = value.change(&crate::path::decode(&query).unwrap()) {
            println!(
                "{}",
                value.to_string(
                    |value, pretty| output_format.format.to_str(value, pretty),
                    cli.args.pretty
                )
            );
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

fn guess_value(stdin: &str) -> Option<(Value, &'static FormatConfig)> {
    for &format in FORMATS {
        if let Some(value) = format.format.from_str(stdin) {
            return Some((value, format));
        }
    }

    None
}
