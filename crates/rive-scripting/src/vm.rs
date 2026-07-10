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
mod renderer;
mod view_model;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::ffi::CString;
use std::rc::Rc;

use bytecode::validate_luau_bytecode;
use luaur_rt::ffi::lua_error;
use luaur_rt::{
    AnyUserData, FromLuaMulti, Function, IntoLuaMulti, Lua, MultiValue, Table, Value,
    Vector as LuaVector,
};
use luaur_vm::functions::luau_load::luau_load;
use renderer::RendererBindings;
use rive_render_api::{Factory as RenderFactory, Renderer};
use rive_runtime::{
    ScriptArtboard, ScriptError, ScriptHost, ScriptInstance, ScriptMethod, ScriptValue,
    ScriptViewModel, ScriptingVm as RuntimeScriptingVm,
};
use view_model::{ScriptedContext, create_scripted_view_model};

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
    rive_globals_installed: Cell<bool>,
    renderer_bindings: RendererBindings,
    view_models: BTreeMap<String, ScriptViewModel>,
    default_context_view_model: Option<ScriptViewModel>,
    default_context_parent_view_models: Vec<ScriptViewModel>,
}

/// A luaur-backed scripted object instance table.
pub struct LuaScriptInstance {
    table: Table,
    renderer_bindings: RendererBindings,
    context_view_model: Rc<RefCell<Option<ScriptViewModel>>>,
    context: Option<AnyUserData>,
}

impl LuaScriptInstance {
    pub fn new(table: Table) -> Self {
        Self {
            table,
            renderer_bindings: RendererBindings::default(),
            context_view_model: Rc::new(RefCell::new(None)),
            context: None,
        }
    }

    fn with_renderer_bindings(
        table: Table,
        renderer_bindings: RendererBindings,
        context_view_model: Rc<RefCell<Option<ScriptViewModel>>>,
        context: Option<AnyUserData>,
    ) -> Self {
        Self {
            table,
            renderer_bindings,
            context_view_model,
            context,
        }
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
    pub fn instantiate_script_with_factory(
        &mut self,
        name: &str,
        payload: &[u8],
        host: &mut dyn ScriptHost,
        factory: &mut dyn RenderFactory,
    ) -> std::result::Result<Box<dyn ScriptInstance>, ScriptError> {
        let bindings = self.renderer_bindings.clone();
        bindings.with_factory_context(factory, || {
            <Self as RuntimeScriptingVm>::instantiate_script(self, name, payload, host)
        })
    }

    /// Boot a VM with the Luau standard libraries open.
    pub fn new() -> Self {
        Self {
            lua: Lua::new(),
            rive_globals_installed: Cell::new(false),
            renderer_bindings: RendererBindings::default(),
            view_models: BTreeMap::new(),
            default_context_view_model: None,
            default_context_parent_view_models: Vec::new(),
        }
    }

    pub fn set_view_models(&mut self, view_models: BTreeMap<String, ScriptViewModel>) {
        self.view_models = view_models;
    }

    pub fn set_default_context_view_model(&mut self, view_model: Option<ScriptViewModel>) {
        self.default_context_view_model = view_model;
        self.default_context_parent_view_models.clear();
    }

    pub fn set_default_context_view_model_chain(
        &mut self,
        view_model: Option<ScriptViewModel>,
        parents: Vec<ScriptViewModel>,
    ) {
        self.default_context_view_model = view_model;
        self.default_context_parent_view_models = parents;
    }

    /// The underlying mlua-style handle (globals, create_function, userdata).
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Install the Rive globals that Luau bytecode resolves with GETIMPORT.
    ///
    /// This mirrors the relevant early part of C++ `ScriptingVM::init`:
    /// globals are installed before any script/module bytecode is loaded, then
    /// the VM applies `luaL_sandbox` and `luaL_sandboxthread` via luaur.
    pub fn install_rive_globals(&self) -> Result<()> {
        if self.rive_globals_installed.get() {
            return Ok(());
        }

        install_vector_global(&self.lua)?;

        let late = self
            .lua
            .create_function(|_, _: MultiValue| Ok(Value::Nil))?;
        self.lua.globals().set("late", late)?;

        let cache = self.ensure_module_cache()?;
        self.install_require_global(cache)?;
        self.renderer_bindings.install(&self.lua)?;
        view_model::install_data_global(&self.lua, &self.view_models)?;

        self.lua.sandbox(true)?;
        self.rive_globals_installed.set(true);
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
    /// mirroring C++ `registeredCacheTableKey`).
    fn ensure_module_cache(&self) -> Result<Table> {
        if let Ok(cache) = self.lua.named_registry_value::<Table>(MODULE_CACHE_KEY) {
            return Ok(cache);
        }
        let cache = self.lua.create_table();
        self.lua
            .set_named_registry_value(MODULE_CACHE_KEY, &cache)?;
        Ok(cache)
    }

    /// Install Rive's custom `require`, which resolves plain module names
    /// against the registered-module cache (`require("Transform2")`).
    fn install_require_global(&self, cache: Table) -> Result<()> {
        let lookup = cache.clone();
        let require = self.lua.create_function(move |_, name: String| {
            match lookup.get::<Value>(name.as_str())? {
                Value::Nil => Err(Error::runtime(format!("module '{name}' not found"))),
                value => Ok(value),
            }
        })?;
        self.lua.globals().set("require", require)?;
        Ok(())
    }

    fn module_cache(&self) -> Result<Table> {
        self.ensure_module_cache()
    }

    /// The module previously registered under `name`, if any.
    pub fn registered_module(&self, name: &str) -> Result<Value> {
        self.module_cache()?.get(name)
    }

    /// Execute a `ScriptAsset` payload (envelope + Luau bytecode) and cache
    /// its result under `name` so scripts can `require` it — the twin of
    /// C++ `ScriptingVM::registerModule`. Idempotent per name.
    pub fn register_module(&self, name: &str, payload: &[u8]) -> Result<Value> {
        self.install_rive_globals()?;
        self.register_module_after_init(name, payload)
    }

    fn register_module_after_init(&self, name: &str, payload: &[u8]) -> Result<Value> {
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

    pub fn script_instance_from_table(&self, table: Table) -> LuaScriptInstance {
        LuaScriptInstance::with_renderer_bindings(
            table,
            self.renderer_bindings.clone(),
            Rc::new(RefCell::new(None)),
            None,
        )
    }
}

// Coarsely translated from rive-runtime/src/lua/math/lua_vec2d.cpp. The C++
// API is a static-method table; __call retains the constructor shape exposed
// by the initial Rust scripting seam.
fn install_vector_global(lua: &Lua) -> Result<()> {
    let vector = lua.create_table();
    vector.set(
        "distance",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| {
            let x = lhs.x() - rhs.x();
            let y = lhs.y() - rhs.y();
            Ok((x * x + y * y).sqrt())
        })?,
    )?;
    vector.set(
        "distanceSquared",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| {
            let x = lhs.x() - rhs.x();
            let y = lhs.y() - rhs.y();
            Ok(x * x + y * y)
        })?,
    )?;
    vector.set(
        "dot",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| {
            Ok(lhs.x() * rhs.x() + lhs.y() * rhs.y())
        })?,
    )?;
    vector.set(
        "cross",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| {
            Ok(lhs.x() * rhs.y() - lhs.y() * rhs.x())
        })?,
    )?;
    vector.set(
        "scaleAndAdd",
        lua.create_function(|_, (a, b, scale): (LuaVector, LuaVector, f32)| {
            Ok(LuaVector::new(
                a.x() + b.x() * scale,
                a.y() + b.y() * scale,
                0.0,
            ))
        })?,
    )?;
    vector.set(
        "scaleAndSub",
        lua.create_function(|_, (a, b, scale): (LuaVector, LuaVector, f32)| {
            Ok(LuaVector::new(
                a.x() - b.x() * scale,
                a.y() - b.y() * scale,
                0.0,
            ))
        })?,
    )?;
    vector.set(
        "lerp",
        lua.create_function(|_, (lhs, rhs, factor): (LuaVector, LuaVector, f32)| {
            Ok(LuaVector::new(
                lhs.x() + (rhs.x() - lhs.x()) * factor,
                lhs.y() + (rhs.y() - lhs.y()) * factor,
                0.0,
            ))
        })?,
    )?;
    vector.set(
        "xy",
        lua.create_function(|_, (x, y): (f32, f32)| Ok(LuaVector::new(x, y, 0.0)))?,
    )?;
    vector.set(
        "origin",
        lua.create_function(|_, ()| Ok(LuaVector::zero()))?,
    )?;
    vector.set(
        "length",
        lua.create_function(|_, value: LuaVector| {
            Ok((value.x() * value.x() + value.y() * value.y()).sqrt())
        })?,
    )?;
    vector.set(
        "lengthSquared",
        lua.create_function(|_, value: LuaVector| {
            Ok(value.x() * value.x() + value.y() * value.y())
        })?,
    )?;
    vector.set(
        "normalized",
        lua.create_function(|_, value: LuaVector| {
            let length_squared = value.x() * value.x() + value.y() * value.y();
            let scale = if length_squared > 0.0 {
                1.0 / length_squared.sqrt()
            } else {
                1.0
            };
            Ok(LuaVector::new(value.x() * scale, value.y() * scale, 0.0))
        })?,
    )?;

    let metatable = lua.create_table();
    metatable.set(
        "__call",
        lua.create_function(
            |_, (_table, x, y, z): (Table, Option<f32>, Option<f32>, Option<f32>)| {
                Ok(LuaVector::new(
                    x.unwrap_or(0.0),
                    y.unwrap_or(0.0),
                    z.unwrap_or(0.0),
                ))
            },
        )?,
    )?;
    vector.set_metatable(Some(metatable))?;
    vector.set_readonly(true);
    lua.globals().set("Vector", vector)
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
        let context_view_model = Rc::new(RefCell::new(self.default_context_view_model.clone()));
        let context = self
            .lua
            .create_userdata(ScriptedContext::new(
                Rc::clone(&context_view_model),
                self.default_context_parent_view_models.clone(),
            ))
            .map_err(script_error)?;
        let instance: Table = generator.call(context.clone()).map_err(script_error)?;
        Ok(Box::new(LuaScriptInstance::with_renderer_bindings(
            instance,
            self.renderer_bindings.clone(),
            context_view_model,
            Some(context),
        )))
    }
}

impl ScriptInstance for LuaScriptInstance {
    fn set_context_view_model(
        &mut self,
        view_model: Option<ScriptViewModel>,
    ) -> std::result::Result<(), ScriptError> {
        *self.context_view_model.borrow_mut() = view_model;
        Ok(())
    }

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
        if method == ScriptMethod::Init
            && let Some(context) = self.context.as_ref()
        {
            call_args.push_back(Value::UserData(context.clone()));
        }
        let value: Value = function.call(call_args).map_err(script_error)?;
        Ok(script_value_from_lua(value).map_err(script_error)?)
    }

    fn call_method_with_factory(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
        host: &mut dyn ScriptHost,
        factory: &mut dyn RenderFactory,
    ) -> std::result::Result<ScriptValue, ScriptError> {
        let bindings = self.renderer_bindings.clone();
        bindings.with_factory_context(factory, || self.call_method(method, args, host))
    }

    fn call_path_effect_update(
        &mut self,
        source: rive_render_api::RawPath,
        node: rive_runtime::ScriptNode,
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<rive_render_api::RawPath, ScriptError> {
        renderer::call_path_effect_update(&self.table, source, node).map_err(script_error)
    }

    fn call_draw(
        &mut self,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<(), ScriptError> {
        self.renderer_bindings
            .call_draw(&self.table, factory, renderer)
            .map_err(script_error)
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

    fn set_artboard_input(
        &mut self,
        name: &str,
        artboard: Box<dyn ScriptArtboard>,
    ) -> std::result::Result<(), ScriptError> {
        let lua = self.table.lua();
        let artboard = self
            .renderer_bindings
            .create_scripted_artboard(&lua, artboard)
            .map_err(script_error)?;
        self.table.set(name, artboard).map_err(script_error)
    }

    fn set_view_model_input(
        &mut self,
        name: &str,
        view_model: ScriptViewModel,
    ) -> std::result::Result<(), ScriptError> {
        let lua = self.table.lua();
        let view_model = create_scripted_view_model(&lua, view_model).map_err(script_error)?;
        self.table.set(name, view_model).map_err(script_error)?;
        Ok(())
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
