//! Luau VM boot + execution on `luaur` (pure-Rust Luau).
//!
//! Mirrors the shape of C++ `rive::ScriptingVM`
//! (`include/rive/lua/scripting_vm.hpp`, `src/lua/rive_lua_libs.cpp`):
//! the runtime never compiles Luau source — the editor ships precompiled
//! Luau bytecode inside `ScriptAsset`, and `ScriptingVM::loadModule` feeds
//! it straight to `luau_load`. [`ScriptVm::load_bytecode`] is the Rust
//! equivalent, built on `luaur_rt::Lua::exec_raw` + `luaur_vm::luau_load`.
//!
//! Source compilation ([`ScriptVm::eval`] / [`ScriptVm::load`]) is also
//! exposed because luaur ships the Luau *compiler* too, which the C++
//! runtime does not embed — useful for tests and future editor-style flows.

mod bytecode;

use std::ffi::CString;

use bytecode::validate_luau_bytecode;
use luaur_rt::ffi::lua_error;
use luaur_rt::{
    FromLuaMulti, Function, IntoLuaMulti, Lua, MultiValue, Table, Value, Vector as LuaVector,
};
use luaur_vm::functions::luau_load::luau_load;
use rive_runtime::{
    ScriptError, ScriptHost, ScriptInstance, ScriptMethod, ScriptValue,
    ScriptingVm as RuntimeScriptingVm,
};

use crate::envelope::SignedContent;

pub use luaur_rt::{Error, Result};

/// Registry key for the require cache (C++: `registeredCacheTableKey` in
/// `src/lua/rive_lua_libs.cpp`).
const MODULE_CACHE_KEY: &str = "rive_scripting_registered_modules";

/// A booted Luau VM.
///
/// Thin wrapper over [`luaur_rt::Lua`] with the Rive-specific entry points;
/// [`ScriptVm::lua`] exposes the full mlua-style API for binding work.
pub struct ScriptVm {
    lua: Lua,
}

/// A luaur-backed scripted object instance table.
pub struct LuaScriptInstance {
    table: Table,
}

impl LuaScriptInstance {
    pub fn new(table: Table) -> Self {
        Self { table }
    }

    pub fn table(&self) -> &Table {
        &self.table
    }
}

impl Default for ScriptVm {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptVm {
    /// Boot a VM with the Luau standard libraries open.
    pub fn new() -> Self {
        Self { lua: Lua::new() }
    }

    /// The underlying mlua-style handle (globals, create_function, userdata).
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Install the Rive globals that Luau bytecode resolves with GETIMPORT.
    ///
    /// This mirrors the relevant early part of C++ `ScriptingVM::init`:
    /// globals are installed before any script/module bytecode is loaded.
    /// Full sandboxing is still a follow-up because luaur does not yet expose
    /// `luaL_sandbox`; the seam keeps that work localized here.
    pub fn install_rive_globals(&self) -> Result<()> {
        let vector =
            self.lua
                .create_function(|_, (x, y, z): (Option<f32>, Option<f32>, Option<f32>)| {
                    Ok(LuaVector::new(
                        x.unwrap_or(0.0),
                        y.unwrap_or(0.0),
                        z.unwrap_or(0.0),
                    ))
                })?;
        self.lua.globals().set("Vector", vector)?;

        let late = self
            .lua
            .create_function(|_, _: MultiValue| Ok(Value::Nil))?;
        self.lua.globals().set("late", late)?;
        Ok(())
    }

    /// Compile and evaluate Luau *source*, returning the chunk's results.
    pub fn eval<R: FromLuaMulti>(&self, source: &str) -> Result<R> {
        self.lua.load(source).eval()
    }

    /// Compile Luau *source* into a callable function without running it.
    pub fn load(&self, name: &str, source: &str) -> Result<Function> {
        self.lua.load(source).set_name(name).into_function()
    }

    /// Load precompiled Luau *bytecode* (the payload `.riv` files carry)
    /// into an unexecuted closure — the Rust twin of C++
    /// `ScriptingVM::loadModule`'s `luau_load` call.
    ///
    /// Structural bytecode errors are rejected before reaching `luau_load`.
    /// The pinned luaur loader mirrors C++ pointer-heavy deserialization, so
    /// hostile `.riv` payloads get a safe Rust preflight before the raw VM call.
    pub fn load_bytecode(&self, chunk_name: &str, bytecode: &[u8]) -> Result<Function> {
        validate_luau_bytecode(bytecode).map_err(|e| {
            Error::runtime(format!(
                "ScriptAsset '{chunk_name}': malformed Luau bytecode: {e}"
            ))
        })?;
        let name = CString::new(format!("={chunk_name}"))
            .unwrap_or_else(|_| CString::new("=script").expect("static"));
        unsafe {
            self.lua.exec_raw((), |state| {
                let rc = luau_load(
                    state,
                    name.as_ptr(),
                    bytecode.as_ptr() as *const core::ffi::c_char,
                    bytecode.len(),
                    0,
                );
                if rc != 0 {
                    // luau_load left its error message on the stack; raise it
                    // so exec_raw's protected call surfaces it as Error.
                    lua_error(state);
                }
                // Success: the loaded closure is on the stack and becomes
                // exec_raw's result.
            })
        }
    }

    /// Load and *execute* a script/module payload, returning what the chunk
    /// returns (protocol scripts return their generator function; utility
    /// modules return a table) — the spike-sized twin of C++
    /// `ScriptingVM::registerScript` / `registerModule`
    /// (`src/lua/rive_lua_libs.cpp`), minus the per-module sandboxed thread
    /// and require-cache bookkeeping.
    pub fn run_bytecode<R: FromLuaMulti>(&self, chunk_name: &str, bytecode: &[u8]) -> Result<R> {
        self.load_bytecode(chunk_name, bytecode)?.call(())
    }

    /// Load a raw `ScriptAsset` payload as it appears in a `.riv` file:
    /// strip the signed-content envelope, then load the inner Luau bytecode.
    /// Signature *verification* is out of scope for the spike (unsigned
    /// in-band bytecode is the corpus norm; C++ merely marks unverified).
    pub fn load_script_asset_payload(&self, name: &str, payload: &[u8]) -> Result<Function> {
        let envelope = SignedContent::parse(payload)
            .map_err(|e| Error::runtime(format!("ScriptAsset '{name}': {e}")))?;
        self.load_bytecode(name, envelope.content)
    }

    /// The registered-module cache table (stored in the Lua named registry,
    /// mirroring C++ `registeredCacheTableKey`). First use installs a
    /// `require` global that resolves plain module names against it — the
    /// contract Rive scripts rely on (`require("Transform2")`), implemented
    /// in C++ by `ScriptingVM::executeModule`'s cache plus Rive's custom
    /// `require`.
    fn module_cache(&self) -> Result<Table> {
        if let Ok(cache) = self.lua.named_registry_value::<Table>(MODULE_CACHE_KEY) {
            return Ok(cache);
        }
        let cache = self.lua.create_table();
        self.lua
            .set_named_registry_value(MODULE_CACHE_KEY, &cache)?;
        let lookup = cache.clone();
        let require = self.lua.create_function(move |_, name: String| {
            match lookup.get::<Value>(name.as_str())? {
                Value::Nil => Err(Error::runtime(format!("module '{name}' not found"))),
                value => Ok(value),
            }
        })?;
        self.lua.globals().set("require", require)?;
        Ok(cache)
    }

    /// The module previously registered under `name`, if any.
    pub fn registered_module(&self, name: &str) -> Result<Value> {
        self.module_cache()?.get(name)
    }

    /// Execute a `ScriptAsset` payload (envelope + Luau bytecode) and cache
    /// its result under `name` so scripts can `require` it — the twin of
    /// C++ `ScriptingVM::registerModule`. Idempotent per name.
    pub fn register_module(&self, name: &str, payload: &[u8]) -> Result<Value> {
        let cache = self.module_cache()?;
        if let value @ (Value::Table(_) | Value::Function(_)) = cache.get::<Value>(name)? {
            return Ok(value);
        }
        let result: Value = self.load_script_asset_payload(name, payload)?.call(())?;
        match &result {
            Value::Table(_) | Value::Function(_) => {}
            other => {
                return Err(Error::runtime(format!(
                    "module '{name}' must return a table or function, got {other:?}"
                )));
            }
        }
        cache.set(name, result.clone())?;
        Ok(result)
    }

    /// Register a batch of modules, retrying until a pass makes no progress
    /// — the spike-sized twin of C++ `ScriptingContext::performRegistration`,
    /// which retries modules whose dependencies had not registered yet.
    /// Returns the names that still failed, with their last error.
    pub fn perform_registration<'a>(
        &self,
        modules: impl IntoIterator<Item = (&'a str, &'a [u8])>,
    ) -> Vec<(&'a str, Error)> {
        let mut pending: Vec<(&str, &[u8])> = modules.into_iter().collect();
        loop {
            let mut failures = Vec::new();
            let before = pending.len();
            for (name, payload) in pending {
                if let Err(error) = self.register_module(name, payload) {
                    failures.push((name, payload, error));
                }
            }
            if failures.is_empty() {
                return Vec::new();
            }
            if failures.len() == before {
                // No progress this pass: report what is left.
                return failures
                    .into_iter()
                    .map(|(name, _, error)| (name, error))
                    .collect();
            }
            pending = failures
                .into_iter()
                .map(|(name, payload, _)| (name, payload))
                .collect();
        }
    }

    /// Call a global function by name.
    pub fn call_global<R: FromLuaMulti>(&self, name: &str, args: impl IntoLuaMulti) -> Result<R> {
        let function: Function = self.lua.globals().get(name)?;
        function.call(args)
    }

    /// Read a global value.
    pub fn global(&self, name: &str) -> Result<Value> {
        self.lua.globals().get(name)
    }
}

impl RuntimeScriptingVm for ScriptVm {
    fn install_rive_globals(&mut self) -> std::result::Result<(), ScriptError> {
        ScriptVm::install_rive_globals(self).map_err(script_error)
    }

    fn register_module(
        &mut self,
        name: &str,
        payload: &[u8],
    ) -> std::result::Result<(), ScriptError> {
        ScriptVm::install_rive_globals(self).map_err(script_error)?;
        ScriptVm::register_module(self, name, payload)
            .map(|_| ())
            .map_err(script_error)
    }

    fn instantiate_script(
        &mut self,
        name: &str,
        payload: &[u8],
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<Box<dyn ScriptInstance>, ScriptError> {
        ScriptVm::install_rive_globals(self).map_err(script_error)?;
        let generator: Function = self
            .load_script_asset_payload(name, payload)
            .map_err(script_error)?
            .call(())
            .map_err(script_error)?;
        let context = self.lua.create_table();
        let instance: Table = generator.call(context).map_err(script_error)?;
        Ok(Box::new(LuaScriptInstance::new(instance)))
    }
}

impl ScriptInstance for LuaScriptInstance {
    fn has_method(&self, method: ScriptMethod) -> std::result::Result<bool, ScriptError> {
        let value: Value = self.table.get(method.as_str()).map_err(script_error)?;
        Ok(matches!(value, Value::Function(_)))
    }

    fn call_method(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<ScriptValue, ScriptError> {
        let value: Value = self.table.get(method.as_str()).map_err(script_error)?;
        let Value::Function(function) = value else {
            return match value {
                Value::Nil => Ok(ScriptValue::Nil),
                other => Err(ScriptError::new(format!(
                    "script method '{}' is {}, not function",
                    method.as_str(),
                    other.type_name()
                ))),
            };
        };

        let lua = self.table.lua();
        let mut call_args = MultiValue::with_capacity(args.len() + 1);
        call_args.push_back(Value::Table(self.table.clone()));
        for arg in args {
            call_args.push_back(script_value_to_lua(&lua, arg));
        }
        let value: Value = function.call(call_args).map_err(script_error)?;
        Ok(script_value_from_lua(value).map_err(script_error)?)
    }

    fn get_input(&self, name: &str) -> std::result::Result<ScriptValue, ScriptError> {
        let value: Value = self.table.get(name).map_err(script_error)?;
        Ok(script_value_from_lua(value).map_err(script_error)?)
    }

    fn set_input(
        &mut self,
        name: &str,
        value: ScriptValue,
    ) -> std::result::Result<(), ScriptError> {
        let lua = self.table.lua();
        self.table
            .set(name, script_value_to_lua(&lua, &value))
            .map_err(script_error)
    }
}

fn script_error(error: Error) -> ScriptError {
    ScriptError::new(error.to_string())
}

fn script_value_to_lua(lua: &Lua, value: &ScriptValue) -> Value {
    match value {
        ScriptValue::Nil => Value::Nil,
        ScriptValue::Bool(value) => Value::Boolean(*value),
        ScriptValue::Number(value) => Value::Number(*value),
        ScriptValue::String(value) => Value::String(lua.create_string(value)),
        ScriptValue::Vec2 { x, y } => Value::Vector(LuaVector::new(*x, *y, 0.0)),
        ScriptValue::Vec3 { x, y, z } => Value::Vector(LuaVector::new(*x, *y, *z)),
    }
}

fn script_value_from_lua(value: Value) -> Result<ScriptValue> {
    Ok(match value {
        Value::Nil => ScriptValue::Nil,
        Value::Boolean(value) => ScriptValue::Bool(value),
        Value::Integer(value) => ScriptValue::Number(value as f64),
        Value::Number(value) => ScriptValue::Number(value),
        Value::String(value) => ScriptValue::String(value.to_str()?),
        Value::Vector(value) => ScriptValue::Vec3 {
            x: value.x(),
            y: value.y(),
            z: value.z(),
        },
        other => {
            return Err(Error::runtime(format!(
                "cannot convert Lua {} to runtime ScriptValue",
                other.type_name()
            )));
        }
    })
}
