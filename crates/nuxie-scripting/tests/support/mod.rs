#![allow(dead_code)]

use luaur_rt::{Error, FromLuaMulti, Function, Result, Table, Value};
use nuxie_scripting::vm::ScriptVm;

const MODULE_CACHE_KEY: &str = "rive_scripting_registered_modules";

/// Source helpers used only by baseline conformance tests.
///
/// Production callers intentionally cannot import this module. Editor source
/// compilation and execution live in nuxie-dev; these helpers keep existing
/// binding tests concise without restoring source APIs to `ScriptVm`.
pub trait ScriptVmSourceTestExt {
    fn eval<R: FromLuaMulti>(&self, source: &str) -> Result<R>;
    fn load(&self, name: &str, source: &str) -> Result<Function>;
    fn register_source_module(&self, name: &str, source: &str) -> Result<Value>;
}

impl ScriptVmSourceTestExt for ScriptVm {
    fn eval<R: FromLuaMulti>(&self, source: &str) -> Result<R> {
        self.lua().load(source).eval()
    }

    fn load(&self, name: &str, source: &str) -> Result<Function> {
        self.lua().load(source).set_name(name).into_function()
    }

    fn register_source_module(&self, name: &str, source: &str) -> Result<Value> {
        self.install_rive_globals()?;
        let cache = self.lua().named_registry_value::<Table>(MODULE_CACHE_KEY)?;
        if let value @ (Value::Table(_) | Value::Function(_)) = cache.raw_get::<Value>(name)? {
            return Ok(value);
        }

        let chunk = self.load(name, source)?;
        let environment = self.lua().create_table();
        let metatable = self.lua().create_table();
        metatable.raw_set("__index", self.lua().globals())?;
        metatable.set_readonly(true);
        environment.set_metatable(Some(metatable))?;
        if !chunk.set_environment(environment)? {
            return Err(Error::runtime(format!(
                "module '{name}' could not install its sandbox environment"
            )));
        }
        let result: Value = chunk.call(())?;
        match &result {
            Value::Table(_) | Value::Function(_) => cache.raw_set(name, result.clone())?,
            other => {
                return Err(Error::runtime(format!(
                    "module '{name}' must return a table or function, got {other:?}"
                )));
            }
        }
        Ok(result)
    }
}
