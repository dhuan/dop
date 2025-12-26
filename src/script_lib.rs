use crate::common::*;
use crate::types::*;

pub fn key_match(env: &ScriptEnv, arg: Option<&String>) -> (Option<String>, bool) {
    if let None = arg {
        return (None, false);
    }

    (None, regex_test(arg.unwrap(), &env.key))
}

pub fn parse_script_env() -> Option<ScriptEnv> {
    let value = std::env::var("VALUE").ok()?;
    let key = std::env::var("KEY").ok()?;

    Some(ScriptEnv {
        value: value,
        key: key,
    })
}
