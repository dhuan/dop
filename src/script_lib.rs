use crate::common::*;
use crate::types::*;

pub fn key_match(env: &ScriptEnv, arg: Option<&[&str]>) -> (Option<String>, bool) {
    if let None = arg {
        return (None, false);
    }

    (
        None,
        regex_test(arg.unwrap().into_iter().nth(0).unwrap(), &env.key),
    )
}

pub fn is_string(env: &ScriptEnv, _: Option<&[&str]>) -> (Option<String>, bool) {
    (None, env.value_type == "string")
}

pub fn is_number(env: &ScriptEnv, _: Option<&[&str]>) -> (Option<String>, bool) {
    (None, env.value_type == "number")
}

pub fn is_bool(env: &ScriptEnv, _: Option<&[&str]>) -> (Option<String>, bool) {
    (None, env.value_type == "bool")
}

pub fn is_list(env: &ScriptEnv, _: Option<&[&str]>) -> (Option<String>, bool) {
    (None, env.value_type == "list")
}

pub fn is_object(env: &ScriptEnv, _: Option<&[&str]>) -> (Option<String>, bool) {
    (None, env.value_type == "object")
}

pub fn set(env: &ScriptEnv, args: Option<&[&str]>) -> (Option<String>, bool) {
    let args = args.unwrap();
    let len = args.len();

    if len == 0 {
        println!("Set expects at least one parameter.");

        std::process::exit(1);
    }

    if len > 1 {
        println!("Not supported yet.");

        std::process::exit(1);
    }

    let value = args.iter().nth(0).unwrap();

    if let Err(err) = std::fs::write(&env.file_set_value, value) {
        return (Some(err.to_string()), false);
    }

    (None, true)
}

pub fn parse_script_env() -> Option<ScriptEnv> {
    Some(ScriptEnv {
        value_type: std::env::var("VALUE_TYPE").ok()?,
        file_set_value: std::env::var("SET_VALUE").ok()?,
        key: std::env::var("KEY").ok()?,
    })
}
