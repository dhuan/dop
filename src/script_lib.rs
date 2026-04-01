use crate::common::*;
use crate::lua::*;
use crate::path;
use crate::value::*;
use mlua::{
    Lua, LuaSerdeExt, MultiValue,
    prelude::{LuaResult, LuaValue},
};
use serde_json::Value as JsonValue;
use std::rc::Rc;

fn get_internal(
    ctx: Rc<LibContext>,
    args: Option<&[String]>,
) -> Result<Option<Value>, Option<String>> {
    let args = args.unwrap_or_default();
    let argsc = args.len();

    if ctx.script_once_mode && argsc == 0 {
        return Err(Some(
            "'get' during execute-once must receive a key.".to_string(),
        ));
    }

    let key = if ctx.script_once_mode {
        Some(args.iter().nth(0).unwrap().to_owned())
    } else if argsc == 0 {
        Some(ctx.key_encoded.clone())
    } else if args.len() > 0 {
        Some(args.iter().nth(0).unwrap().to_owned())
    } else {
        None
    };

    if key.is_none() {
        return Err(Some("Failed to parse.".to_string()));
    }

    let key = match path::decode(&key.ok_or(None)?) {
        Some(key) => key,
        None => {
            return Err(None);
        }
    };

    let value = ctx.value.borrow().clone();

    let value = match value.get(&key) {
        Some(value) => value.clone(),
        None => {
            return Err(None);
        }
    };

    Ok(Some(value))
}

pub fn get(ctx: Rc<LibContext>) -> impl Fn(&Lua, Option<String>) -> LuaResult<Option<LuaValue>> {
    move |_, params| {
        let args = if let Some(param) = params {
            vec![param]
        } else {
            vec![]
        };

        if let Ok(Some(value)) = get_internal(ctx.clone(), Some(&args)) {
            return Ok(Some(
                ctx.lua
                    .to_value(&crate::json::to_json_value(&value).unwrap())
                    .unwrap(),
            ));
        }

        Ok(None)
    }
}

pub fn exec(ctx: Rc<LibContext>) -> impl Fn(&Lua, String) -> LuaResult<Option<LuaValue>> {
    move |lua, command| {
        let command_result = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;

        let output = ctx
            .lua
            .to_value(
                &String::from_utf8(command_result.stdout)
                    .unwrap()
                    .strip_suffix("\n"),
            )?
            .to_string()?;

        let exec_result = lua.to_value(&lua.create_table_from(map_from_list(&vec![
            ("output".to_string(), lua.to_value(&output).unwrap()),
            (
                "status".to_string(),
                lua.to_value(&command_result.status.code().unwrap_or(-1)).unwrap(),
            ),
        ]))?)?;

        Ok(Some(exec_result))
    }
}

pub fn unset(ctx: Rc<LibContext>) -> impl Fn(&Lua, Option<String>) -> LuaResult<Option<LuaValue>> {
    move |_, delete_key| {
        let delete_key = match delete_key {
            None => ctx.clone().key.clone(),
            Some(key) => match path::decode(&key) {
                Some(key) => key,
                None => return Ok(None),
            },
        };

        ctx.value.borrow_mut().remove(&delete_key);

        Ok(None)
    }
}

pub fn set(ctx: Rc<LibContext>) -> impl Fn(&Lua, MultiValue) -> LuaResult<()> {
    move |_, params| {
        let params = parse_multi_value(params);

        if params.len() == 0 {
            return Ok(());
        }

        let (key, value, force): (Option<&String>, &JsonValue, bool) = {
            let len = params.len();

            if len == 0 {
                return Ok(());
            }

            if len == 1 {
                (None, params.iter().nth(0).unwrap(), false)
            } else if len == 2 {
                if let JsonValue::String(key) = params.iter().nth(0).unwrap() {
                    (Some(key), params.iter().nth(1).unwrap(), false)
                } else {
                    return Ok(());
                }
            } else {
                let force = if let Some(force) = parse_set_options(params.iter().nth(2).unwrap()) {
                    force
                } else {
                    false
                };

                if let JsonValue::String(key) = params.iter().nth(0).unwrap() {
                    (Some(key), params.iter().nth(1).unwrap(), force)
                } else {
                    return Ok(());
                }
            }
        };

        let mut value_mut = ctx.value.borrow_mut();

        if let None = key {
            let value_to_change = match value_mut.change(&ctx.key, force) {
                Some(value) => value,
                None => {
                    return Ok(());
                }
            };

            *value_to_change = crate::json::to_value(&value.clone()).unwrap();
        } else if let Some(key) = key {
            let key = match path::decode(key) {
                Some(key) => key,
                None => {
                    return Ok(());
                }
            };

            if let Some(value_to_change) = value_mut.change(&key, force) {
                *value_to_change = crate::json::to_value(&value.clone()).unwrap();
            }
        }

        Ok(())
    }
}

fn parse_set_options(value: &JsonValue) -> Option<bool> {
    let options = if let JsonValue::Object(obj) = value {
        Some(obj)
    } else {
        return None;
    };

    let force = if let Some(options) = options {
        if let Some(JsonValue::Bool(b)) = options.get("force") {
            *b
        } else {
            false
        }
    } else {
        false
    };

    Some(force)
}

fn parse_multi_value(value: MultiValue) -> Vec<serde_json::Value> {
    value
        .into_vec()
        .iter()
        .map(|v| serde_json::to_value(v).unwrap())
        .collect::<Vec<serde_json::Value>>()
}
