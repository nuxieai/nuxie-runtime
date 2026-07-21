//! Luau VM boot + execution on `luaur` (pure-Rust Luau).
//!
//! Mirrors the shape of C++ `nuxie::ScriptingVM`
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
    AnyUserData, FromLuaMulti, Function, IntoLuaMulti, Lua, MultiValue, Table, UserData,
    UserDataFields, UserDataMethods, Value, Vector as LuaVector, VmState,
};
use luaur_vm::functions::luau_load::luau_load;
use nuxie_render_api::{Factory as RenderFactory, Renderer};
use nuxie_runtime::{
    ScriptArtboard, ScriptDataConverterMethod, ScriptError, ScriptHost, ScriptInstance,
    ScriptMethod, ScriptValue, ScriptViewModel, ScriptingVm as RuntimeScriptingVm,
};
use renderer::RendererBindings;
use view_model::{ScriptedContext, create_scripted_view_model};

use crate::envelope::SignedContent;
use crate::gpu_canvas::ImportedGpuCanvasInstance;
use crate::shader_asset::decode_shader_asset;

pub use luaur_rt::{Error, Result};

/// Registry key for the require cache (C++: `registeredCacheTableKey` in
/// `src/lua/rive_lua_libs.cpp`).
const MODULE_CACHE_KEY: &str = "rive_scripting_registered_modules";

/// Default ceiling for one trusted imported File's Luau VM.
pub const DEFAULT_SCRIPT_VM_MEMORY_BYTES: usize = 64 * 1024 * 1024;
/// Default interrupt safepoints available to each trusted callback.
pub const DEFAULT_SCRIPT_INTERRUPTS_PER_CALLBACK: u32 = 50_000;

/// Resource limits applied to one explicitly trusted imported script VM.
///
/// Every host-to-Luau callback receives a fresh interrupt budget. The memory
/// ceiling covers the entire File-owned VM, including module/protocol setup
/// and every retained occurrence table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptExecutionLimits {
    max_memory_bytes: usize,
    max_interrupts_per_callback: u32,
}

impl ScriptExecutionLimits {
    pub const fn new() -> Self {
        Self {
            max_memory_bytes: DEFAULT_SCRIPT_VM_MEMORY_BYTES,
            max_interrupts_per_callback: DEFAULT_SCRIPT_INTERRUPTS_PER_CALLBACK,
        }
    }

    pub const fn with_max_memory_bytes(mut self, maximum: usize) -> Self {
        self.max_memory_bytes = maximum;
        self
    }

    pub const fn with_max_interrupts_per_callback(mut self, maximum: u32) -> Self {
        self.max_interrupts_per_callback = maximum;
        self
    }

    pub const fn max_memory_bytes(self) -> usize {
        self.max_memory_bytes
    }

    pub const fn max_interrupts_per_callback(self) -> u32 {
        self.max_interrupts_per_callback
    }

    pub fn validate(self) -> Result<()> {
        if self.max_memory_bytes == 0 {
            return Err(Error::runtime(
                "trusted script VM memory limit must be greater than zero",
            ));
        }
        if self.max_interrupts_per_callback == 0 {
            return Err(Error::runtime(
                "trusted script callback interrupt limit must be greater than zero",
            ));
        }
        Ok(())
    }
}

impl Default for ScriptExecutionLimits {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct ScriptExecutionBudget {
    remaining: Rc<Cell<u32>>,
    maximum: u32,
}

impl ScriptExecutionBudget {
    fn reset(&self) {
        self.remaining.set(self.maximum);
    }
}

/// A booted Luau VM.
///
/// Thin wrapper over [`luaur_rt::Lua`] with the Rive-specific entry points;
/// [`ScriptVm::lua`] exposes the full mlua-style API for binding work.
pub struct ScriptVm {
    lua: Lua,
    execution_budget: Option<ScriptExecutionBudget>,
    rive_globals_installed: Cell<bool>,
    renderer_bindings: RendererBindings,
    view_models: BTreeMap<String, ScriptViewModel>,
    default_context_view_model: Option<ScriptViewModel>,
    default_context_parent_view_models: Vec<ScriptViewModel>,
    gpu_canvas_shaders: Rc<RefCell<BTreeMap<String, nuxie_render_api::GpuCanvasShader>>>,
}

/// File-registered protocol generator. The script chunk has already executed;
/// each drawable occurrence calls this generator to create a fresh table.
#[derive(Clone)]
pub struct ScriptProgram {
    generator: Function,
}

impl std::fmt::Debug for ScriptProgram {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScriptProgram")
            .finish_non_exhaustive()
    }
}

/// A luaur-backed scripted object instance table.
pub struct LuaScriptInstance {
    table: Table,
    execution_budget: Option<ScriptExecutionBudget>,
    renderer_bindings: RendererBindings,
    context_view_model: Rc<RefCell<Option<ScriptViewModel>>>,
    context: Option<AnyUserData>,
    gpu_canvas: Option<ImportedGpuCanvasInstance>,
}

#[derive(Debug, Clone)]
struct ScriptedDataValue {
    value: ScriptValue,
}

impl ScriptedDataValue {
    fn new(value: ScriptValue) -> Self {
        Self { value }
    }

    fn color_channel(&self, shift: u32) -> Option<u32> {
        let ScriptValue::Color(value) = self.value else {
            return None;
        };
        Some((value >> shift) & 0xff)
    }

    fn set_color_channel(&mut self, shift: u32, channel: u32) {
        if let ScriptValue::Color(value) = &mut self.value {
            let mask = !(0xff << shift);
            *value = (*value & mask) | ((channel & 0xff) << shift);
        }
    }
}

impl UserData for ScriptedDataValue {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |lua, this| {
            Ok(script_value_to_lua(lua, &this.value))
        });
        fields.add_field_method_set("value", |_, this, value: Value| {
            this.value = match (&this.value, value) {
                (ScriptValue::Number(_), Value::Integer(value)) => {
                    ScriptValue::Number(value as f64)
                }
                (ScriptValue::Number(_), Value::Number(value)) => ScriptValue::Number(value),
                (ScriptValue::String(_), Value::String(value)) => {
                    ScriptValue::String(value.to_str()?)
                }
                (ScriptValue::Bool(_), Value::Boolean(value)) => ScriptValue::Bool(value),
                (ScriptValue::Color(_), Value::Integer(value)) => ScriptValue::Color(value as u32),
                (ScriptValue::Color(_), Value::Number(value)) => ScriptValue::Color(value as u32),
                (expected, value) => {
                    return Err(Error::runtime(format!(
                        "cannot assign Lua {} to scripted data value {expected:?}",
                        value.type_name()
                    )));
                }
            };
            Ok(())
        });
        for (name, shift) in [("red", 16), ("green", 8), ("blue", 0), ("alpha", 24)] {
            fields.add_field_method_get(name, move |_, this| Ok(this.color_channel(shift)));
            fields.add_field_method_set(name, move |_, this, value: u32| {
                this.set_color_channel(shift, value);
                Ok(())
            });
        }
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("isNumber", |_, this, ()| {
            Ok(matches!(this.value, ScriptValue::Number(_)))
        });
        methods.add_method("isString", |_, this, ()| {
            Ok(matches!(this.value, ScriptValue::String(_)))
        });
        methods.add_method("isBoolean", |_, this, ()| {
            Ok(matches!(this.value, ScriptValue::Bool(_)))
        });
        methods.add_method("isColor", |_, this, ()| {
            Ok(matches!(this.value, ScriptValue::Color(_)))
        });
    }
}

impl LuaScriptInstance {
    pub fn new(table: Table) -> Self {
        Self {
            table,
            execution_budget: None,
            renderer_bindings: RendererBindings::default(),
            context_view_model: Rc::new(RefCell::new(None)),
            context: None,
            gpu_canvas: None,
        }
    }

    fn with_renderer_bindings(
        table: Table,
        renderer_bindings: RendererBindings,
        context_view_model: Rc<RefCell<Option<ScriptViewModel>>>,
        context: Option<AnyUserData>,
        gpu_canvas: Option<ImportedGpuCanvasInstance>,
        execution_budget: Option<ScriptExecutionBudget>,
    ) -> Self {
        Self {
            table,
            execution_budget,
            renderer_bindings,
            context_view_model,
            context,
            gpu_canvas,
        }
    }

    pub fn table(&self) -> &Table {
        &self.table
    }

    fn reset_execution_budget(&self) {
        if let Some(budget) = self.execution_budget.as_ref() {
            budget.reset();
        }
    }

    fn call_method_value(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
    ) -> std::result::Result<Value, ScriptError> {
        self.reset_execution_budget();
        self.call_method_value_with_current_budget(method, args)
    }

    fn call_method_value_with_current_budget(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
    ) -> std::result::Result<Value, ScriptError> {
        let value: Value = self.table.get(method.as_str()).map_err(script_error)?;
        let Value::Function(function) = value else {
            return match value {
                Value::Nil => Ok(Value::Nil),
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
        function.call(call_args).map_err(script_error)
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
        let program = self.register_protocol_script_with_factory(name, payload, factory)?;
        self.instantiate_registered_script_with_factory(&program, host, factory)
    }

    /// Execute one protocol ScriptAsset chunk exactly once for a File and
    /// retain the generator it returns.
    pub fn register_protocol_script_with_factory(
        &self,
        name: &str,
        payload: &[u8],
        factory: &mut dyn RenderFactory,
    ) -> std::result::Result<ScriptProgram, ScriptError> {
        let bindings = self.renderer_bindings.clone();
        bindings.with_factory_context(factory, || {
            self.install_rive_globals().map_err(script_error)?;
            self.reset_execution_budget();
            let generator = self
                .load_script_asset_payload(name, payload)
                .map_err(script_error)?
                .call(())
                .map_err(script_error)?;
            Ok(ScriptProgram { generator })
        })
    }

    /// Invoke a File-registered protocol generator for one concrete drawable
    /// occurrence while the caller's renderer factory is in scope.
    pub fn instantiate_registered_script_with_factory(
        &self,
        program: &ScriptProgram,
        _host: &mut dyn ScriptHost,
        factory: &mut dyn RenderFactory,
    ) -> std::result::Result<Box<dyn ScriptInstance>, ScriptError> {
        let bindings = self.renderer_bindings.clone();
        bindings.with_factory_context(factory, || {
            let context_view_model = Rc::new(RefCell::new(self.default_context_view_model.clone()));
            let (gpu_canvas, gpu_canvas_context) =
                ImportedGpuCanvasInstance::new(Rc::clone(&self.gpu_canvas_shaders));
            let context = self
                .lua
                .create_userdata(ScriptedContext::new(
                    Rc::clone(&context_view_model),
                    self.default_context_parent_view_models.clone(),
                    Some(gpu_canvas_context),
                ))
                .map_err(script_error)?;
            self.reset_execution_budget();
            let instance: Table = program
                .generator
                .call(context.clone())
                .map_err(script_error)?;
            Ok(Box::new(LuaScriptInstance::with_renderer_bindings(
                instance,
                self.renderer_bindings.clone(),
                context_view_model,
                Some(context),
                Some(gpu_canvas),
                self.execution_budget.clone(),
            )) as Box<dyn ScriptInstance>)
        })
    }

    /// Boot a VM with the Luau standard libraries open.
    pub fn new() -> Self {
        Self {
            lua: Lua::new(),
            execution_budget: None,
            rive_globals_installed: Cell::new(false),
            renderer_bindings: RendererBindings::default(),
            view_models: BTreeMap::new(),
            default_context_view_model: None,
            default_context_parent_view_models: Vec::new(),
            gpu_canvas_shaders: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    /// Boot a VM whose total memory and each host-to-Luau callback are
    /// explicitly bounded.
    pub fn new_with_execution_limits(limits: ScriptExecutionLimits) -> Result<Self> {
        limits.validate()?;
        let mut vm = Self::new();
        vm.lua.set_memory_limit(limits.max_memory_bytes())?;
        let budget = ScriptExecutionBudget {
            remaining: Rc::new(Cell::new(limits.max_interrupts_per_callback())),
            maximum: limits.max_interrupts_per_callback(),
        };
        let interrupt_budget = budget.clone();
        vm.lua.set_interrupt(move |_| {
            let remaining = interrupt_budget.remaining.get();
            if remaining == 0 {
                return Err(Error::runtime(format!(
                    "trusted Luau callback exceeded {} interrupt safepoints",
                    interrupt_budget.maximum
                )));
            }
            interrupt_budget.remaining.set(remaining - 1);
            Ok(VmState::Continue)
        });
        vm.execution_budget = Some(budget);
        Ok(vm)
    }

    fn reset_execution_budget(&self) {
        if let Some(budget) = self.execution_budget.as_ref() {
            budget.reset();
        }
    }

    /// Decode and register one imported ShaderAsset before protocol generators
    /// execute. Names are the exact `context:shader(name)` lookup keys.
    pub fn register_gpu_canvas_shader_asset(
        &self,
        name: &str,
        payload: &[u8],
    ) -> std::result::Result<(), ScriptError> {
        let shader = decode_shader_asset(name, payload).map_err(script_error)?;
        let mut shaders = self.gpu_canvas_shaders.borrow_mut();
        if shaders.contains_key(name) {
            return Err(ScriptError::new(format!(
                "ShaderAsset name '{name}' is duplicated"
            )));
        }
        shaders.insert(name.to_owned(), shader);
        Ok(())
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
        install_data_value_global(&self.lua)?;

        let late = self
            .lua
            .create_function(|_, _: MultiValue| Ok(Value::Nil))?;
        self.lua.globals().set("late", late)?;

        let cache = self.ensure_module_cache()?;
        self.install_require_global(cache)?;
        self.renderer_bindings.install(&self.lua)?;
        crate::gpu_canvas::install_gpu_canvas_globals(self)?;
        view_model::install_data_global(&self.lua, &self.view_models)?;

        self.lua.sandbox(true)?;
        self.rive_globals_installed.set(true);
        Ok(())
    }

    /// Compile and evaluate Luau *source*, returning the chunk's results.
    pub fn eval<R: FromLuaMulti>(&self, source: &str) -> Result<R> {
        self.reset_execution_budget();
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
        self.reset_execution_budget();
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

    /// Register a module while exposing the draw call's renderer factory to
    /// module top-level code (for example a module that constructs a `Paint`).
    pub fn register_module_with_factory(
        &self,
        name: &str,
        payload: &[u8],
        factory: &mut dyn RenderFactory,
    ) -> Result<Value> {
        let bindings = self.renderer_bindings.clone();
        bindings.with_factory_context(factory, || self.register_module(name, payload))
    }

    fn register_module_after_init(&self, name: &str, payload: &[u8]) -> Result<Value> {
        let cache = self.module_cache()?;
        if let value @ (Value::Table(_) | Value::Function(_)) = cache.get::<Value>(name)? {
            return Ok(value);
        }
        self.reset_execution_budget();
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
        self.reset_execution_budget();
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
            None,
            self.execution_budget.clone(),
        )
    }
}

// Coarsely translated from nuxie-runtime/src/lua/math/lua_vec2d.cpp. The C++
// API is a static-method table; __call retains the constructor shape exposed
// by the initial Rust scripting seam.
fn install_data_value_global(lua: &Lua) -> Result<()> {
    let data_value = lua.create_table();
    data_value.set(
        "number",
        lua.create_function(|lua, ()| {
            lua.create_userdata(ScriptedDataValue::new(ScriptValue::Number(0.0)))
        })?,
    )?;
    data_value.set(
        "string",
        lua.create_function(|lua, ()| {
            lua.create_userdata(ScriptedDataValue::new(ScriptValue::String(String::new())))
        })?,
    )?;
    data_value.set(
        "boolean",
        lua.create_function(|lua, ()| {
            lua.create_userdata(ScriptedDataValue::new(ScriptValue::Bool(false)))
        })?,
    )?;
    data_value.set(
        "color",
        lua.create_function(|lua, ()| {
            lua.create_userdata(ScriptedDataValue::new(ScriptValue::Color(0)))
        })?,
    )?;
    data_value.set_readonly(true);
    lua.globals().set("DataValue", data_value)
}

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
        self.reset_execution_budget();
        let generator: Function = self
            .load_script_asset_payload(name, payload)
            .map_err(script_error)?
            .call(())
            .map_err(script_error)?;
        let context_view_model = Rc::new(RefCell::new(self.default_context_view_model.clone()));
        let (gpu_canvas, gpu_canvas_context) =
            ImportedGpuCanvasInstance::new(Rc::clone(&self.gpu_canvas_shaders));
        let context = self
            .lua
            .create_userdata(ScriptedContext::new(
                Rc::clone(&context_view_model),
                self.default_context_parent_view_models.clone(),
                Some(gpu_canvas_context),
            ))
            .map_err(script_error)?;
        self.reset_execution_budget();
        let instance: Table = generator.call(context.clone()).map_err(script_error)?;
        Ok(Box::new(LuaScriptInstance::with_renderer_bindings(
            instance,
            self.renderer_bindings.clone(),
            context_view_model,
            Some(context),
            Some(gpu_canvas),
            self.execution_budget.clone(),
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
        self.reset_execution_budget();
        let value: Value = self.table.get(method.as_str()).map_err(script_error)?;
        Ok(matches!(value, Value::Function(_)))
    }

    fn call_method(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<ScriptValue, ScriptError> {
        let value = self.call_method_value(method, args)?;
        script_value_from_lua(value).map_err(script_error)
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

    fn call_init_with_factory(
        &mut self,
        _host: &mut dyn ScriptHost,
        factory: &mut dyn RenderFactory,
    ) -> std::result::Result<bool, ScriptError> {
        let bindings = self.renderer_bindings.clone();
        bindings.with_factory_context(factory, || {
            let value = self.call_method_value(ScriptMethod::Init, &[])?;
            Ok(!matches!(value, Value::Nil | Value::Boolean(false)))
        })
    }

    fn call_path_effect_update(
        &mut self,
        source: nuxie_render_api::RawPath,
        node: nuxie_runtime::ScriptNode,
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<nuxie_render_api::RawPath, ScriptError> {
        self.reset_execution_budget();
        renderer::call_path_effect_update(&self.table, source, node).map_err(script_error)
    }

    fn call_draw(
        &mut self,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<(), ScriptError> {
        if let Some(gpu_canvas) = self.gpu_canvas.as_ref() {
            self.reset_execution_budget();
            gpu_canvas
                .execute_draw_canvas(&self.table, factory)
                .map_err(script_error)?;
        }
        self.reset_execution_budget();
        self.renderer_bindings
            .call_draw(&self.table, factory, renderer)
            .map_err(script_error)
    }

    fn call_data_converter(
        &mut self,
        method: ScriptDataConverterMethod,
        value: ScriptValue,
    ) -> std::result::Result<ScriptValue, ScriptError> {
        self.reset_execution_budget();
        let function: Function = self.table.get(method.as_str()).map_err(script_error)?;
        let lua = self.table.lua();
        let input = lua
            .create_userdata(ScriptedDataValue::new(value))
            .map_err(script_error)?;
        let output: AnyUserData = function
            .call((self.table.clone(), input))
            .map_err(script_error)?;
        let output = output.borrow::<ScriptedDataValue>().map_err(script_error)?;
        Ok(output.value.clone())
    }

    fn get_input(&self, name: &str) -> std::result::Result<ScriptValue, ScriptError> {
        self.reset_execution_budget();
        let value: Value = self.table.get(name).map_err(script_error)?;
        script_value_from_lua(value).map_err(script_error)
    }

    fn set_input(
        &mut self,
        name: &str,
        value: ScriptValue,
    ) -> std::result::Result<(), ScriptError> {
        self.reset_execution_budget();
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
        self.reset_execution_budget();
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
        self.reset_execution_budget();
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
        ScriptValue::Color(value) => Value::Integer(i64::from(*value)),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn put_u16(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn put_string(bytes: &mut Vec<u8>, value: &str) {
        put_u16(bytes, value.len() as u16);
        bytes.extend_from_slice(value.as_bytes());
    }

    fn shader_payload(fragment_color: &str) -> Vec<u8> {
        let vertex = "#version 300 es\nvoid main() { gl_Position = vec4(0.0); }";
        let fragment = format!(
            "#version 300 es\nout vec4 color;\nvoid main() {{ color = {fragment_color}; }}"
        );
        let mut entries = vec![2];
        for (stage, logical, source) in [(0, "vs_main", vertex), (1, "fs_main", fragment.as_str())]
        {
            entries.push(stage);
            put_string(&mut entries, logical);
            put_string(&mut entries, "main");
            put_u32(&mut entries, source.len() as u32);
            entries.extend_from_slice(source.as_bytes());
        }

        let mut payload = vec![0];
        put_u32(&mut payload, 0x5253_5442);
        put_u16(&mut payload, 4);
        payload.extend_from_slice(&[1, 0, 1]);
        put_u32(&mut payload, 0);
        put_u32(&mut payload, entries.len() as u32);
        payload.extend(entries);
        payload
    }

    #[test]
    fn duplicate_shader_registration_preserves_the_first_shader() {
        let vm = ScriptVm::new();
        vm.register_gpu_canvas_shader_asset("scene", &shader_payload("vec4(1.0)"))
            .expect("first shader registers");
        let first = vm.gpu_canvas_shaders.borrow()["scene"].clone();

        let error = vm
            .register_gpu_canvas_shader_asset("scene", &shader_payload("vec4(0.0)"))
            .expect_err("duplicate shader name is rejected");

        assert!(error.to_string().contains("duplicated"));
        assert_eq!(vm.gpu_canvas_shaders.borrow()["scene"], first);
    }
}
