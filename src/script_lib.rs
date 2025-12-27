use crate::common::*;
use crate::types::*;

pub fn key_match(env: &ScriptEnv, arg: Option<&String>) -> (Option<String>, bool) {
    if let None = arg {
        return (None, false);
    }

    (None, regex_test(arg.unwrap(), &env.key))
}

pub fn is_string(env: &ScriptEnv, _: Option<&String>) -> (Option<String>, bool) {
    (None, env.value_type == "string")
}

pub fn is_number(env: &ScriptEnv, _: Option<&String>) -> (Option<String>, bool) {
    (None, env.value_type == "number")
}

pub fn parse_script_env() -> Option<ScriptEnv> {
    let value = std::env::var("VALUE").ok()?;
    let value_type = std::env::var("VALUE_TYPE").ok()?;
    let key = std::env::var("KEY").ok()?;

    Some(ScriptEnv {
        value: value,
        value_type: value_type,
        key: key,
    })
}
