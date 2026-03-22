use crate::path::*;
use crate::script_lib;
use crate::types::*;
use crate::value::*;
use mlua::{LuaSerdeExt, prelude::*};
use std::cell::RefCell;
use std::rc::Rc;

fn add_global_func<F, A, R>(lua: Rc<Lua>, func_name: &str, func: F) -> Result<(), String>
where
    F: FnMut(&Lua, A) -> mlua::Result<R> + mlua::MaybeSend + 'static,
    A: FromLuaMulti,
    R: IntoLuaMulti,
{
    lua.globals()
        .set(
            func_name,
            lua.create_function_mut(func)
                .map_err(|err| err.to_string())?,
        )
        .unwrap();

    Ok(())
}

pub struct LibContext {
    pub lua: Rc<Lua>,
    pub env: Rc<ScriptEnv>,
    pub value: Rc<RefCell<Value>>,
    pub key: Vec<PathEntry>,
}

pub fn handle(
    script: &str,
    env: &ScriptEnv,
    value: Rc<RefCell<Value>>,
    field_name: Option<&str>,
    key: &[PathEntry],
    key_encoded: &str,
    script_once_mode: bool,
    log: Box<impl Fn(&str) + 'static>,
) -> Result<(), String> {
    let lua = Rc::new(Lua::new());
    let lib_ctx = Rc::new(LibContext {
        lua: lua.clone(),
        env: Rc::new(env.clone()),
        value: value.clone(),
        key: key.to_vec(),
    });

    add_global_func(lua.clone(), "log", move |_: &Lua, value: String| {
        log(&value);

        Ok(())
    })?;
    add_global_func(lua.clone(), "set", script_lib::set(lib_ctx.clone()))?;
    add_global_func(lua.clone(), "unset", script_lib::unset(lib_ctx.clone()))?;
    add_global_func(lua.clone(), "get", script_lib::get(lib_ctx.clone()))?;
    add_global_func(
        lua.clone(),
        "is_string",
        script_lib::is_string(lib_ctx.clone()),
    )?;

    lua.globals()
        .set("FIELD_NAME", field_name.unwrap_or_default())
        .unwrap();

    let value_lua = match if script_once_mode {
        value.borrow().clone()
    } else {
        value.borrow().get(key).unwrap().clone()
    } {
        Value::Null => lua.globals().get("null").unwrap(),
        value => to_lua_value(&lua, &value.clone()),
    };

    lua.globals().set("KEY", key_encoded).unwrap();

    lua.globals().set("VALUE", value_lua).unwrap();

    lua.load(script).exec().map_err(|err| err.to_string())?;

    Ok(())
}

fn to_lua_value(lua: &Lua, value: &Value) -> LuaValue {
    lua.to_value(&crate::json::to_json_value(value).unwrap())
        .unwrap()
}
