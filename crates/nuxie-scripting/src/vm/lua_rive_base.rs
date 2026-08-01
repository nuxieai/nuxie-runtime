//! Rive's `_G.print` replacement, mirroring `lua_rive_base.cpp`.

use luaur_rt::{Function, Lua, LuaString, MultiValue, Result};

use super::logging_scripting_context::LoggingScriptingContext;

/// Install the host-routed `print` before the VM globals are sandboxed.
pub(super) fn install_host_print(lua: &Lua, logging: LoggingScriptingContext) -> Result<()> {
    let tostring: Function = lua.globals().get("tostring")?;
    let print = lua.create_function(move |_, args: MultiValue| {
        if args.is_empty() {
            return Ok(());
        }

        logging.begin_line();
        for value in args {
            let value: LuaString = tostring.call(value)?;
            logging.append(&value.as_bytes());
        }
        logging.end_line();
        Ok(())
    })?;
    lua.globals().set("print", print)
}
