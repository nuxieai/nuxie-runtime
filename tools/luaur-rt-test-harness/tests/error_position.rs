use luaur_rt::{Error, Function, Lua, Result};

#[test]
fn translated_lua_l_error_has_exact_caller_position() -> Result<()> {
    let lua = Lua::new();
    let fail = lua.create_function(|_, ()| -> Result<()> {
        Err(Error::lua_l_runtime("translated callback failed."))
    })?;
    lua.globals().set("fail", fail)?;

    let function: Function = lua
        .load("return function()\n  fail()\nend")
        .set_name("callbackChunk")
        .eval()?;
    let error = function.call::<()>(()).unwrap_err().to_string();

    assert_eq!(
        error,
        "runtime error: =callbackChunk:2: translated callback failed."
    );
    Ok(())
}
