use crate::common::*;
use crate::path;
use crate::types::*;
use crate::value::*;

pub fn key_match(
    env: &ScriptEnv,
    arg: Option<&[String]>,
    _format: &dyn DataFormat,
) -> Result<Option<String>, Option<String>> {
    if arg.is_none() {
        return Err(None);
    }

    if regex_test(arg.unwrap().first().unwrap(), &env.key) {
        Ok(None)
    } else {
        Err(None)
    }
}

pub fn is_null(
    env: &ScriptEnv,
    _: Option<&[String]>,
    _format: &dyn DataFormat,
) -> Result<Option<String>, Option<String>> {
    is_value_type(env, "null")
}

pub fn is_string(
    env: &ScriptEnv,
    _: Option<&[String]>,
    _format: &dyn DataFormat,
) -> Result<Option<String>, Option<String>> {
    is_value_type(env, "string")
}

pub fn is_number(
    env: &ScriptEnv,
    _: Option<&[String]>,
    _format: &dyn DataFormat,
) -> Result<Option<String>, Option<String>> {
    if env.value_type == "int" || env.value_type == "float" {
        Ok(None)
    } else {
        Err(None)
    }
}

pub fn is_bool(
    env: &ScriptEnv,
    _: Option<&[String]>,
    _format: &dyn DataFormat,
) -> Result<Option<String>, Option<String>> {
    is_value_type(env, "bool")
}

pub fn is_list(
    env: &ScriptEnv,
    _: Option<&[String]>,
    _format: &dyn DataFormat,
) -> Result<Option<String>, Option<String>> {
    is_value_type(env, "list")
}

pub fn is_value_type(env: &ScriptEnv, type_check: &str) -> Result<Option<String>, Option<String>> {
    if env.value_type == type_check {
        Ok(None)
    } else {
        Err(None)
    }
}

pub fn is_object(
    env: &ScriptEnv,
    _: Option<&[String]>,
    _format: &dyn DataFormat,
) -> Result<Option<String>, Option<String>> {
    is_value_type(env, "object")
}

pub fn get(
    env: &ScriptEnv,
    args: Option<&[String]>,
    format: &dyn DataFormat,
) -> Result<Option<String>, Option<String>> {
    let args = args.unwrap_or_default();
    let argsc = args.len();

    if env.is_script_once && argsc == 0 {
        return Err(Some(
            "'get' during execute-once must receive a key.".to_string(),
        ));
    }

    let key = if env.is_script_once {
        Some(args.iter().nth(0).unwrap().to_owned())
    } else if argsc == 0 {
        Some(env.key.clone())
    } else if args.len() > 0 {
        Some(args.iter().nth(0).unwrap().to_owned())
    } else {
        None
    };

    if key.is_none() {
        return Err(Some("Failed to parse.".to_string()));
    }

    let value = format
        .from_str(&std::fs::read_to_string(&env.file_set_value).unwrap())
        .unwrap();

    let key = match path::decode(&key.ok_or(None)?) {
        Some(key) => key,
        None => {
            return Err(None);
        }
    };

    let value = match value.get(&key) {
        Some(value) => value.clone(),
        None => {
            return Err(None);
        }
    };

    Ok(Some(value.to_string(
        |value, pretty| format.to_str(value, pretty),
        false,
    )))
}

pub fn set(
    value_type: ValueType,
    force: bool,
) -> impl Fn(&ScriptEnv, Option<&[String]>, &dyn DataFormat) -> Result<Option<String>, Option<String>>
{
    move |env: &ScriptEnv, args: Option<&[String]>, format: &dyn DataFormat| {
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

        if !is_changing_outside && env.is_script_once {
            std::process::exit(1);
        }

        let mut current_value = format
            .from_str(&std::fs::read_to_string(&env.file_set_value).unwrap())
            .unwrap();

        let value_to_be_modified = current_value.change(&path::decode(&key).unwrap(), force);
        if value_to_be_modified.is_none() {
            return Ok(None);
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

        Ok(None)
    }
}

pub fn del(
    env: &ScriptEnv,
    args: Option<&[String]>,
    format: &dyn DataFormat,
) -> Result<Option<String>, Option<String>> {
    let delete_key = match args {
        None => None,
        Some(args) => {
            if args.len() == 0 {
                None
            } else {
                Some(args.iter().nth(0).unwrap())
            }
        }
    };

    let mut current_value = format
        .from_str(&std::fs::read_to_string(&env.file_set_value).unwrap())
        .unwrap();

    let delete_key = match delete_key.clone() {
        None => path::decode(&env.key.clone()).unwrap(),
        Some(key) => path::decode(key).unwrap(),
    };

    current_value.remove(&delete_key);

    std::fs::write(
        &env.file_set_value,
        current_value.to_string(|value, pretty| format.to_str(value, pretty), false),
    )
    .unwrap();

    Ok(None)
}

pub fn parse_script_env() -> Option<ScriptEnv> {
    Some(ScriptEnv {
        value_type: std::env::var("VALUE_TYPE").ok()?,
        file_set_value: std::env::var("VALUE_ALL").ok()?,
        key: std::env::var("KEY").ok()?,
        format_name: std::env::var("VALUE_FORMAT").ok()?,
        is_script_once: std::env::var("IS_SCRIPT_ONCE")
            .ok()
            .map(|is_script_once| is_script_once == "true")
            .unwrap_or(false),
    })
}
