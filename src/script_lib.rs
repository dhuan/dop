use crate::common::*;
use crate::path;
use crate::types::*;
use crate::value::*;

pub fn key_match(
    env: &ScriptEnv,
    arg: Option<&[&str]>,
    _format: &dyn DataFormat,
) -> (Option<String>, bool) {
    if arg.is_none() {
        return (None, false);
    }

    (None, regex_test(arg.unwrap().first().unwrap(), &env.key))
}

pub fn is_null(
    env: &ScriptEnv,
    _: Option<&[&str]>,
    _format: &dyn DataFormat,
) -> (Option<String>, bool) {
    (None, env.value_type == "null")
}

pub fn is_string(
    env: &ScriptEnv,
    _: Option<&[&str]>,
    _format: &dyn DataFormat,
) -> (Option<String>, bool) {
    (None, env.value_type == "string")
}

pub fn is_number(
    env: &ScriptEnv,
    _: Option<&[&str]>,
    _format: &dyn DataFormat,
) -> (Option<String>, bool) {
    (None, env.value_type == "int" || env.value_type == "float")
}

pub fn is_bool(
    env: &ScriptEnv,
    _: Option<&[&str]>,
    _format: &dyn DataFormat,
) -> (Option<String>, bool) {
    (None, env.value_type == "bool")
}

pub fn is_list(
    env: &ScriptEnv,
    _: Option<&[&str]>,
    _format: &dyn DataFormat,
) -> (Option<String>, bool) {
    (None, env.value_type == "list")
}

pub fn is_object(
    env: &ScriptEnv,
    _: Option<&[&str]>,
    _format: &dyn DataFormat,
) -> (Option<String>, bool) {
    (None, env.value_type == "object")
}

pub fn set(
    value_type: ValueType,
) -> impl Fn(&ScriptEnv, Option<&[&str]>, &dyn DataFormat) -> (Option<String>, bool) {
    move |env: &ScriptEnv, args: Option<&[&str]>, format: &dyn DataFormat| {
        let args = args.unwrap();
        if args.is_empty() {
            println!("Set expects at least one parameter.");

            std::process::exit(1);
        }

        let is_changing_outside = args.len() > 1;
        let (key, value) = match is_changing_outside {
            true => (args[0].to_string(), args[1].to_string()),
            false => (env.key.to_string(), args[0].to_string()),
        };

        let mut current_value = format
            .from_str(&std::fs::read_to_string(&env.file_set_value).unwrap())
            .unwrap();

        let value_to_be_modified = current_value.change(&path::decode(&key).unwrap());
        if value_to_be_modified.is_none() {
            return (None, true);
        }

        *(value_to_be_modified.unwrap()) = match value_type {
            ValueType::String => Value::String(value.to_string()),
            _ => format
                .from_str(&value)
                .unwrap_or(Value::String(value.to_string())),
        };

        std::fs::write(
            &env.file_set_value,
            current_value.to_string(|value, pretty| format.to_str(value, pretty), false),
        )
        .unwrap();

        (None, true)
    }
}

pub fn del(
    delete_key: Option<String>,
) -> impl Fn(&ScriptEnv, Option<&[&str]>, &dyn DataFormat) -> (Option<String>, bool) {
    move |env: &ScriptEnv, _args: Option<&[&str]>, format: &dyn DataFormat| {
        let mut current_value = format
            .from_str(&std::fs::read_to_string(&env.file_set_value).unwrap())
            .unwrap();

        let delete_key = match delete_key.clone() {
            None => path::decode(&env.key.clone()).unwrap(),
            Some(key) => path::decode(key.as_str()).unwrap(),
        };

        current_value.remove(&delete_key);

        std::fs::write(
            &env.file_set_value,
            current_value.to_string(|value, pretty| format.to_str(value, pretty), false),
        )
        .unwrap();

        (None, true)
    }
}

pub fn parse_script_env() -> Option<ScriptEnv> {
    Some(ScriptEnv {
        value_type: std::env::var("VALUE_TYPE").ok()?,
        file_set_value: std::env::var("VALUE_ALL").ok()?,
        key: std::env::var("KEY").ok()?,
        format_name: std::env::var("VALUE_FORMAT").ok()?,
    })
}
