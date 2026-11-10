//! `__index` dispatchers that keep method lookup inside the VM.
//!
//! A Rust `__index` closure makes every `value:method(...)` cross into the
//! host twice: once to find the method and once to call it. These Lua
//! dispatchers answer the keys the VM can answer from a table and hand every
//! other key to the unchanged Rust closure, so scripts observe the same values
//! and errors with one host crossing fewer per method call.

use crate::luau::{Function, Lua, Result, Table};

use super::embedded_chunk::EmbeddedChunk;

const HOST_DISPATCH_KEY: &str = "nuxie.host_dispatch";

fn exports(lua: &Lua) -> Result<Table> {
    if let Some(exports) = lua.named_registry_value::<Option<Table>>(HOST_DISPATCH_KEY)? {
        return Ok(exports);
    }
    let exports: Table = EmbeddedChunk::HostDispatch.load(lua)?.call(())?;
    lua.set_named_registry_value(HOST_DISPATCH_KEY, &exports)?;
    Ok(exports)
}

/// `__index` that returns `methods[key]` when it is present and otherwise
/// calls `fallback(self, key)`. `methods` must hold only keys whose
/// `fallback` answer is that same entry.
pub(super) fn methods_first_index(lua: &Lua, methods: &Table, fallback: Function) -> Result<Function> {
    let make: Function = exports(lua)?.get("methodsFirst")?;
    make.call((methods.clone(), fallback))
}

/// `__index` that calls `numeric(self, key)` for number keys and
/// `fallback(self, key)` for every other key.
pub(super) fn numbers_first_index(
    lua: &Lua,
    numeric: Function,
    fallback: Function,
) -> Result<Function> {
    let make: Function = exports(lua)?.get("numbersFirst")?;
    make.call((numeric, fallback))
}

/// The function's chunk name as `debug.info(f, "s")` reports it: `"[C]"` for
/// a host function. Both VM settings answer this the same way.
#[cfg(all(test, feature = "compiler"))]
pub(super) fn function_source(lua: &Lua, function: &Function) -> String {
    let info: Function = lua
        .load("return function(f) return debug.info(f, 's') end")
        .eval()
        .expect("debug.info helper");
    info.call(function.clone()).expect("debug.info")
}

#[allow(dead_code)]
const HOST_DISPATCH_SOURCE: &str = r#"
return {
    methodsFirst = function(methods, fallback)
        return function(self, key)
            local method = methods[key]
            if method ~= nil then
                return method
            end
            return fallback(self, key)
        end
    end,
    numbersFirst = function(numeric, fallback)
        return function(self, key)
            if type(key) == "number" then
                return numeric(self, key)
            end
            return fallback(self, key)
        end
    end,
}
"#;
