use clap::Parser;
use std::io::Read;

mod common;
mod json;
mod lua;
mod path;
mod script_lib;
mod toml;
mod types;
mod value;
mod yaml;

use crate::common::*;
use crate::types::*;
use crate::value::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    args: Args,
}

#[derive(Clone, clap::Args)]
struct SetArgs {
    value: Vec<String>,
    #[arg(short = 's', long = "string")]
    convert_to_string: bool,
    #[arg(short = 'f', long = "force")]
    force: bool,
}

#[derive(Clone, clap::Args)]
struct DelArgs {
    key: Option<String>,
}

#[derive(Clone, clap::Args)]
struct GetArgs {
    params: Vec<String>,
}

#[derive(Clone, clap::Args, Debug)]
struct Args {
    #[arg(short = 'e', long = "execute")]
    script: Option<String>,
    #[arg(short = 'E', long = "execute-once")]
    script_once: Option<String>,
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

    if cli.args.script.is_some() && cli.args.script_once.is_some() {
        println!("Using both execute and execute-once together is not supported, yet.");
        std::process::exit(1);
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
            Some(&format) => format
                .format
                .from_str(stdin_buffer.as_str())
                .map(|value| (value, format)),
        },
    };

    if value.is_none() {
        log_v("Failed to parse input.");

        std::process::exit(1);
    }

    let (value, format) = value.unwrap();
    let value = Rc::new(RefCell::new(value));

    log_v(&format!("Input format identified as: {}", format.name));

    let output_format = match cli.args.output_format {
        None => format,
        Some(output_format) => *FORMATS
            .iter()
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

    let (script, script_once_mode) = if cli.args.script.is_some() {
        (cli.args.script, false)
    } else if cli.args.script_once.is_some() {
        (cli.args.script_once, true)
    } else {
        (None, false)
    };

    let script = script.map(|script| {
        if script.lines().count() == 1 {
            std::fs::read_to_string(&script).unwrap_or(script)
        } else {
            script
        }
    });

    let lua_instance = Rc::new(RefCell::new(lua::init()));

    if let Some(script) = script.clone()
        && !script_once_mode
    {
        let mut value = value.borrow_mut();

        *value = value.traverse(|key, key_encoded, _value, value_all| {
            let field_name = key.last().map(|entry| match entry {
                crate::path::PathEntry::Field(field_name) => field_name.to_owned(),
                crate::path::PathEntry::Index(index) => format!("{}", index),
                _ => panic!("Not accepted!"),
            });

            log_v(&format!("Processing key '{}'.", key_encoded));

            if let Some(key_filter_regex) = cli.args.key_filter_regex.clone()
                && !regex_test(&key_filter_regex, key_encoded)
            {
                log_v("Key Filter Regex did not pass, skipping.");
                return TraverseAction::Leave;
            }

            if let Some(key_filter_equal) = cli.args.key_filter_equal.clone()
                && key_filter_equal != key_encoded
            {
                log_v("Key Filter did not pass, skipping.");
                return TraverseAction::Leave;
            }

            let new_value = {
                let value = Rc::new(RefCell::new(value_all.clone()));

                if let Err(err) = lua::handle(
                    lua_instance.clone(),
                    &script,
                    value.clone(),
                    field_name.as_deref(),
                    key,
                    key_encoded,
                    false,
                    Box::new(log_v),
                ) {
                    on_lua_failed(&err, log_v);
                }

                value.borrow().clone()
            };

            log_v(&format!(
                "Value was modified to {}",
                format.format.to_str(&new_value, false).unwrap(),
            ));

            TraverseAction::ChangeRoot(new_value)
        });
    } else if let Some(script) = script
        && script_once_mode
    {
        if let Err(err) = lua::handle(
            lua_instance.clone(),
            &script,
            value.clone(),
            None,
            &[],
            "",
            true,
            Box::new(log_v),
        ) {
            on_lua_failed(&err, log_v);
        }
    }

    log_v(&format!(
        "Execution finished. Printing out in '{}' format.",
        output_format.name
    ));

    let mut value = value.borrow_mut();

    if let Some(query) = cli.args.query {
        if let Some(value) = value.change(
            &crate::path::decode(&query).unwrap_or_else(fail("Invalid query/path.")),
            false,
        ) {
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

fn on_lua_failed(err: &str, log_v: impl Fn(&str)) {
    log_v(&format!("Lua script execution failed:\n{}", err));
}

fn fatal(msg: &'static str) -> ! {
    eprintln!("{}", msg);
    std::process::exit(1);
}

fn fail<T>(msg: &'static str) -> impl FnOnce() -> T {
    move || fatal(msg)
}
