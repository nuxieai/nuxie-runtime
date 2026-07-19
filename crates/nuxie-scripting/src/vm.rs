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
mod mat4;
mod renderer;
mod view_model;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::ffi::CString;
use std::rc::Rc;

use bytecode::validate_luau_bytecode;
use luaur_rt::ffi::lua_error;
use luaur_rt::{
    AnyUserData, Buffer as LuaBuffer, FromLuaMulti, Function, IntoLuaMulti, Lua, MultiValue, Table,
    UserData, UserDataFields, UserDataMethods, Value, Vector as LuaVector,
};
use luaur_vm::functions::luau_load::luau_load;
use mat4::install_mat4_global;
use nuxie_render_api::{Factory as RenderFactory, Renderer};
use nuxie_runtime::{
    ScriptArtboard, ScriptDataConverterMethod, ScriptError, ScriptHost, ScriptInstance,
    ScriptListenerInvocation, ScriptMethod, ScriptValue, ScriptViewModel,
    ScriptingVm as RuntimeScriptingVm,
};
use renderer::RendererBindings;
use view_model::{ScriptViewModelFrameContext, ScriptedContext, create_scripted_view_model};

use crate::envelope::SignedContent;

pub use luaur_rt::{Error, Result};

/// Registry key for the require cache (C++: `registeredCacheTableKey` in
/// `src/lua/rive_lua_libs.cpp`).
const MODULE_CACHE_KEY: &str = "rive_scripting_registered_modules";

/// Library version a script or module belongs to. `(0, 0)` is the host file.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeKey {
    pub library_id: u64,
    pub library_version_id: u64,
}

impl ScopeKey {
    pub const ROOT: Self = Self::new(0, 0);

    const UNPINNED: Self = Self::new(u64::MAX, u64::MAX);

    pub const fn new(library_id: u64, library_version_id: u64) -> Self {
        Self {
            library_id,
            library_version_id,
        }
    }

    pub const fn is_root(self) -> bool {
        self.library_id == 0 && self.library_version_id == 0
    }
}

#[derive(Debug, Default)]
struct ScriptScopes {
    /// One table per caller scope: import label -> pinned library version.
    pins: BTreeMap<ScopeKey, BTreeMap<String, ScopeKey>>,
    /// Readable chunkname -> scope. This also covers editor-style callers
    /// without a retained module descriptor.
    chunk_scopes: BTreeMap<String, ScopeKey>,
}

/// A booted Luau VM.
///
/// Thin wrapper over [`luaur_rt::Lua`] with the Rive-specific entry points;
/// [`ScriptVm::lua`] exposes the full mlua-style API for binding work.
pub struct ScriptVm {
    lua: Lua,
    rive_globals_installed: Cell<bool>,
    renderer_bindings: RendererBindings,
    view_model_frame_context: ScriptViewModelFrameContext,
    scopes: Rc<RefCell<ScriptScopes>>,
    view_models: BTreeMap<String, ScriptViewModel>,
    default_context_view_model: Option<ScriptViewModel>,
    default_context_parent_view_models: Vec<ScriptViewModel>,
}

/// Cloneable handle for the detached view-model roots owned by one scripting
/// VM. Hosts that retain Lua-backed script instances without retaining the
/// [`ScriptVm`] wrapper use this to perform the one root-frame tail advance.
#[derive(Debug, Clone)]
pub struct DetachedViewModelFrame {
    context: ScriptViewModelFrameContext,
}

impl DetachedViewModelFrame {
    pub fn advance(&self) -> bool {
        self.context.advance_detached()
    }
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
    renderer_bindings: RendererBindings,
    context_view_model: Rc<RefCell<Option<ScriptViewModel>>>,
    context: Option<AnyUserData>,
    context_missing_requested_data: Rc<Cell<bool>>,
    context_parent_view_models: Vec<ScriptViewModel>,
    generator: Option<Function>,
    user_init_done: bool,
    init_retry_requires_recreation: bool,
}

#[derive(Debug, Clone)]
struct ScriptedDataValue {
    value: ScriptValue,
}

#[derive(Debug, Clone, Copy)]
struct ScriptedPointerEvent {
    invocation: ScriptListenerInvocation,
}

impl ScriptedPointerEvent {
    fn new(invocation: ScriptListenerInvocation) -> Self {
        Self { invocation }
    }

    fn pointer(self) -> Option<(i32, (f32, f32), (f32, f32), &'static str, f32)> {
        match self.invocation {
            ScriptListenerInvocation::None => None,
            ScriptListenerInvocation::Pointer {
                pointer_id,
                position,
                previous_position,
                kind,
                time_stamp,
            } => Some((
                pointer_id,
                position,
                previous_position,
                kind.as_str(),
                time_stamp,
            )),
        }
    }
}

impl UserData for ScriptedPointerEvent {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id", |_, this| {
            Ok(this.pointer().map(|value| value.0).unwrap_or(0))
        });
        fields.add_field_method_get("position", |_, this| {
            let (x, y) = this.pointer().map(|value| value.1).unwrap_or((0.0, 0.0));
            Ok(LuaVector::new(x, y, 0.0))
        });
        fields.add_field_method_get("previousPosition", |_, this| {
            let (x, y) = this.pointer().map(|value| value.2).unwrap_or((0.0, 0.0));
            Ok(LuaVector::new(x, y, 0.0))
        });
        fields.add_field_method_get("type", |_, this| {
            Ok(this.pointer().map(|value| value.3).unwrap_or("unknown"))
        });
        fields.add_field_method_get("timeStamp", |_, this| {
            Ok(this.pointer().map(|value| value.4).unwrap_or(0.0))
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Hit opacity feeds the C++ hit-test result. State-machine listener
        // actions run after the hit has already been resolved, so retaining
        // the callable surface is sufficient here.
        methods.add_method_mut("hit", |_, _, _: Option<bool>| Ok(()));
    }
}

#[derive(Debug, Clone, Copy)]
struct ScriptedInvocation {
    invocation: ScriptListenerInvocation,
}

impl ScriptedInvocation {
    fn new(invocation: ScriptListenerInvocation) -> Self {
        Self { invocation }
    }
}

impl UserData for ScriptedInvocation {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("isPointerEvent", |_, this, ()| {
            Ok(matches!(
                this.invocation,
                ScriptListenerInvocation::Pointer { .. }
            ))
        });
        methods.add_method("isNone", |_, this, ()| {
            Ok(matches!(this.invocation, ScriptListenerInvocation::None))
        });
        methods.add_method("asPointerEvent", |lua, this, ()| match this.invocation {
            ScriptListenerInvocation::Pointer { .. } => Ok(Some(
                lua.create_userdata(ScriptedPointerEvent::new(this.invocation))?,
            )),
            ScriptListenerInvocation::None => Ok(None),
        });
    }
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
        let frame_context = ScriptViewModelFrameContext::for_lua(&table.lua());
        Self {
            table,
            renderer_bindings: RendererBindings::new(frame_context),
            context_view_model: Rc::new(RefCell::new(None)),
            context: None,
            context_missing_requested_data: Rc::new(Cell::new(false)),
            context_parent_view_models: Vec::new(),
            generator: None,
            user_init_done: false,
            init_retry_requires_recreation: false,
        }
    }

    fn with_renderer_bindings(
        table: Table,
        renderer_bindings: RendererBindings,
        context_view_model: Rc<RefCell<Option<ScriptViewModel>>>,
        context: Option<AnyUserData>,
        context_missing_requested_data: Rc<Cell<bool>>,
        context_parent_view_models: Vec<ScriptViewModel>,
        generator: Option<Function>,
    ) -> Self {
        Self {
            table,
            renderer_bindings,
            context_view_model,
            context,
            context_missing_requested_data,
            context_parent_view_models,
            generator,
            user_init_done: false,
            init_retry_requires_recreation: false,
        }
    }

    pub fn table(&self) -> &Table {
        &self.table
    }

    fn call_method_value(
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

fn normalize_chunk_source(source: &str) -> &str {
    source
        .strip_prefix('=')
        .or_else(|| source.strip_prefix('@'))
        .unwrap_or(source)
}

fn caller_chunk_source(lua: &Lua) -> Option<String> {
    // Level zero is this Rust callback. The immediate Lua caller is normally
    // level one; keep walking through native helper frames (for example pcall)
    // until the actual defining Lua chunk is found.
    for level in 1..=32 {
        let frame = lua.inspect_stack(level)?;
        let Some(source) = frame.source() else {
            continue;
        };
        let source = normalize_chunk_source(source);
        if source != "[C]" {
            return Some(source.to_owned());
        }
    }
    None
}

fn resolve_require_key(scopes: &ScriptScopes, caller_chunkname: &str, request: &str) -> String {
    let caller = scopes
        .chunk_scopes
        .get(normalize_chunk_source(caller_chunkname))
        .copied()
        .unwrap_or(ScopeKey::ROOT);

    let (path, target) = if let Some(rest) = request.strip_prefix("lib:") {
        let (label, path) = rest.split_once('/').unwrap_or((rest, ""));
        let target = scopes
            .pins
            .get(&caller)
            .and_then(|pins| pins.get(label))
            .copied()
            .unwrap_or(ScopeKey::UNPINNED);
        (path, target)
    } else {
        (request, caller)
    };

    ScriptVm::scoped_module_key(path, target)
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
        self.register_protocol_script_with_factory_scoped(name, ScopeKey::ROOT, payload, factory)
    }

    /// Scope-aware protocol registration. The readable scoped chunkname stays
    /// attached to the returned generator and every closure it creates.
    pub fn register_protocol_script_with_factory_scoped(
        &self,
        name: &str,
        scope: ScopeKey,
        payload: &[u8],
        factory: &mut dyn RenderFactory,
    ) -> std::result::Result<ScriptProgram, ScriptError> {
        let bindings = self.renderer_bindings.clone();
        bindings.with_factory_context(factory, || {
            self.install_rive_globals().map_err(script_error)?;
            let cache_key = Self::scoped_module_key(name, scope);
            match self
                .module_cache()
                .map_err(script_error)?
                .get::<Value>(cache_key.as_str())
                .map_err(script_error)?
            {
                // C++ registerScript consults the shared utility cache first.
                Value::Function(generator) => return Ok(ScriptProgram { generator }),
                Value::Table(_) => {
                    return Err(ScriptError::new(format!(
                        "protocol ScriptAsset '{}' resolved to a registered table",
                        Self::readable_chunkname(name, scope)
                    )));
                }
                _ => {}
            }
            let chunkname = Self::readable_chunkname(name, scope);
            self.set_chunkname_scope(&chunkname, scope);
            let generator = self
                .load_script_asset_payload(&chunkname, payload)
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
            let context_missing_requested_data = Rc::new(Cell::new(false));
            let context_parent_view_models = self.default_context_parent_view_models.clone();
            let context = self
                .lua
                .create_userdata(ScriptedContext::new(
                    Rc::clone(&context_view_model),
                    context_parent_view_models.clone(),
                    Rc::clone(&context_missing_requested_data),
                ))
                .map_err(script_error)?;
            let instance: Table = program
                .generator
                .call(context.clone())
                .map_err(script_error)?;
            Ok(Box::new(LuaScriptInstance::with_renderer_bindings(
                instance,
                self.renderer_bindings.clone(),
                context_view_model,
                Some(context),
                context_missing_requested_data,
                context_parent_view_models,
                Some(program.generator.clone()),
            )) as Box<dyn ScriptInstance>)
        })
    }

    /// Boot a VM with the Luau standard libraries open.
    pub fn new() -> Self {
        let lua = Lua::new();
        let view_model_frame_context = ScriptViewModelFrameContext::default();
        lua.set_app_data(view_model_frame_context.clone());
        Self {
            lua,
            rive_globals_installed: Cell::new(false),
            renderer_bindings: RendererBindings::new(view_model_frame_context.clone()),
            view_model_frame_context,
            scopes: Rc::new(RefCell::new(ScriptScopes::default())),
            view_models: BTreeMap::new(),
            default_context_view_model: None,
            default_context_parent_view_models: Vec::new(),
        }
    }

    /// Register the file's view-model definitions and keep `Data` current even
    /// when this VM's globals were initialized before the file was attached.
    pub fn set_view_models(&mut self, view_models: BTreeMap<String, ScriptViewModel>) {
        self.view_models = view_models;
        if self.rive_globals_installed.get() {
            view_model::install_data_global(&self.lua, &self.view_models)
                .expect("refreshing the initialized Data global should succeed");
        }
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

    /// Consume every owner-tracked, parentless script view-model root at the
    /// end of the host frame. The host calls this once after its root state
    /// machine advance; script-driven child artboards deliberately do not.
    pub fn advance_detached_view_models(&self) -> bool {
        self.view_model_frame_context.advance_detached()
    }

    /// Retain the detached-view-model frame state independently of the VM
    /// wrapper. Lua values remain owned by the script instances themselves.
    pub fn detached_view_model_frame(&self) -> DetachedViewModelFrame {
        DetachedViewModelFrame {
            context: self.view_model_frame_context.clone(),
        }
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
        install_mat4_global(&self.lua)?;
        install_math_fround(&self.lua)?;
        install_data_value_global(&self.lua)?;

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

    /// Require-cache key for a module in `scope`; root keeps its bare name.
    pub fn scoped_module_key(name: &str, scope: ScopeKey) -> String {
        if scope.is_root() {
            name.to_owned()
        } else {
            format!(
                "{name}\u{1f}{}:{}",
                scope.library_id, scope.library_version_id
            )
        }
    }

    /// Human-readable chunkname baked into bytecode and source functions.
    /// Scoped names deliberately contain no colon because trace tooling treats
    /// a colon as the separator before a line number.
    pub fn readable_chunkname(name: &str, scope: ScopeKey) -> String {
        if scope.is_root() {
            name.to_owned()
        } else {
            format!("{}-{}/{name}", scope.library_id, scope.library_version_id)
        }
    }

    /// Pin one `lib:` label for a caller scope to a concrete library version.
    pub fn add_import(&self, caller: ScopeKey, library_name: &str, target: ScopeKey) {
        self.scopes
            .borrow_mut()
            .pins
            .entry(caller)
            .or_default()
            .insert(library_name.to_owned(), target);
    }

    /// Seed scope for a chunk without retained module metadata.
    pub fn set_chunkname_scope(&self, chunkname: &str, scope: ScopeKey) {
        self.scopes
            .borrow_mut()
            .chunk_scopes
            .insert(normalize_chunk_source(chunkname).to_owned(), scope);
    }

    /// Resolve a request relative to a named caller chunk. Public for tooling
    /// and conformance probes; `require` itself derives this chunk from the
    /// active Luau stack.
    pub fn resolve_require(&self, caller_chunkname: &str, request: &str) -> String {
        resolve_require_key(
            &self.scopes.borrow(),
            normalize_chunk_source(caller_chunkname),
            request,
        )
    }

    /// Install Rive's custom `require`, resolving bare names in the caller's
    /// scope and `lib:` names through that caller scope's serialized pins.
    fn install_require_global(&self, cache: Table) -> Result<()> {
        let lookup = cache.clone();
        let scopes = Rc::clone(&self.scopes);
        let require = self.lua.create_function(move |lua, name: String| {
            // The defining chunk remains on the Luau stack when a returned
            // closure lazily calls require. Reading that frame is therefore
            // scope-correct even long after module registration finished.
            let caller = caller_chunk_source(lua).unwrap_or_default();
            let cache_key = resolve_require_key(&scopes.borrow(), &caller, &name);
            match lookup.get::<Value>(cache_key.as_str())? {
                Value::Nil => Err(Error::runtime(format!(
                    "require could not find a script named {name}"
                ))),
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
        self.registered_module_scoped(name, ScopeKey::ROOT)
    }

    /// The module previously registered under `name` in `scope`, if any.
    pub fn registered_module_scoped(&self, name: &str, scope: ScopeKey) -> Result<Value> {
        self.module_cache()?
            .get(Self::scoped_module_key(name, scope))
    }

    /// Execute a `ScriptAsset` payload (envelope + Luau bytecode) and cache
    /// its result under `name` so scripts can `require` it — the twin of
    /// C++ `ScriptingVM::registerModule`. Idempotent per name.
    pub fn register_module(&self, name: &str, payload: &[u8]) -> Result<Value> {
        self.register_module_scoped(name, ScopeKey::ROOT, payload)
    }

    /// Scope-aware twin of [`Self::register_module`].
    pub fn register_module_scoped(
        &self,
        name: &str,
        scope: ScopeKey,
        payload: &[u8],
    ) -> Result<Value> {
        self.install_rive_globals()?;
        self.register_module_after_init(name, scope, payload)
    }

    /// Register a module while exposing the draw call's renderer factory to
    /// module top-level code (for example a module that constructs a `Paint`).
    pub fn register_module_with_factory(
        &self,
        name: &str,
        payload: &[u8],
        factory: &mut dyn RenderFactory,
    ) -> Result<Value> {
        self.register_module_with_factory_scoped(name, ScopeKey::ROOT, payload, factory)
    }

    /// Scope-aware module registration with a renderer factory in context.
    pub fn register_module_with_factory_scoped(
        &self,
        name: &str,
        scope: ScopeKey,
        payload: &[u8],
        factory: &mut dyn RenderFactory,
    ) -> Result<Value> {
        let bindings = self.renderer_bindings.clone();
        bindings.with_factory_context(factory, || {
            self.register_module_scoped(name, scope, payload)
        })
    }

    fn register_module_after_init(
        &self,
        name: &str,
        scope: ScopeKey,
        payload: &[u8],
    ) -> Result<Value> {
        let cache_key = Self::scoped_module_key(name, scope);
        if let value @ (Value::Table(_) | Value::Function(_)) =
            self.module_cache()?.get::<Value>(cache_key.as_str())?
        {
            return Ok(value);
        }
        let chunkname = Self::readable_chunkname(name, scope);
        self.set_chunkname_scope(&chunkname, scope);
        let function = self.load_script_asset_payload(&chunkname, payload)?;
        self.register_loaded_module(name, scope, function)
    }

    fn register_loaded_module(
        &self,
        name: &str,
        scope: ScopeKey,
        function: Function,
    ) -> Result<Value> {
        let cache = self.module_cache()?;
        let cache_key = Self::scoped_module_key(name, scope);
        if let value @ (Value::Table(_) | Value::Function(_)) =
            cache.get::<Value>(cache_key.as_str())?
        {
            return Ok(value);
        }
        let result: Value = function.call(())?;
        match &result {
            Value::Table(_) | Value::Function(_) => {}
            other => {
                let display = Self::readable_chunkname(name, scope);
                return Err(Error::runtime(format!(
                    "module '{display}' must return a table or function, got {other:?}"
                )));
            }
        }
        cache.set(cache_key, result.clone())?;
        Ok(result)
    }

    /// Compile and register a source module. Runtime `.riv` execution uses the
    /// bytecode methods; this source entry point supports editor/tooling flows
    /// and conformance tests with the identical scope machinery.
    pub fn register_source_module_scoped(
        &self,
        name: &str,
        scope: ScopeKey,
        source: &str,
    ) -> Result<Value> {
        self.install_rive_globals()?;
        let cache_key = Self::scoped_module_key(name, scope);
        if let value @ (Value::Table(_) | Value::Function(_)) =
            self.module_cache()?.get::<Value>(cache_key.as_str())?
        {
            return Ok(value);
        }
        let chunkname = Self::readable_chunkname(name, scope);
        self.set_chunkname_scope(&chunkname, scope);
        let function = self.lua.load(source).set_name(&chunkname).into_function()?;
        self.register_loaded_module(name, scope, function)
    }

    /// Register a batch of modules, retrying until a pass makes no progress
    /// — the spike-sized twin of C++ `ScriptingContext::performRegistration`,
    /// which retries modules whose dependencies had not registered yet.
    /// Returns the names that still failed, with their last error.
    pub fn perform_registration<'a>(
        &self,
        modules: impl IntoIterator<Item = (&'a str, &'a [u8])>,
    ) -> Vec<(&'a str, Error)> {
        self.perform_scoped_registration(
            modules
                .into_iter()
                .map(|(name, payload)| (name, ScopeKey::ROOT, payload)),
        )
        .into_iter()
        .map(|(name, _, error)| (name, error))
        .collect()
    }

    /// Scope-aware registration retry loop. A missing dependency may become
    /// available in a later pass, including under a different library pin.
    pub fn perform_scoped_registration<'a>(
        &self,
        modules: impl IntoIterator<Item = (&'a str, ScopeKey, &'a [u8])>,
    ) -> Vec<(&'a str, ScopeKey, Error)> {
        let mut pending: Vec<(&str, ScopeKey, &[u8])> = modules.into_iter().collect();
        loop {
            let mut failures = Vec::new();
            let before = pending.len();
            for (name, scope, payload) in pending {
                if let Err(error) = self.register_module_scoped(name, scope, payload) {
                    failures.push((name, scope, payload, error));
                }
            }
            if failures.is_empty() {
                return Vec::new();
            }
            if failures.len() == before {
                // No progress this pass: report what is left.
                return failures
                    .into_iter()
                    .map(|(name, scope, _, error)| (name, scope, error))
                    .collect();
            }
            pending = failures
                .into_iter()
                .map(|(name, scope, payload, _)| (name, scope, payload))
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
            Rc::new(Cell::new(false)),
            Vec::new(),
            None,
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
            let z = lhs.z() - rhs.z();
            Ok((x * x + y * y + z * z).sqrt())
        })?,
    )?;
    vector.set(
        "distanceSquared",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| {
            let x = lhs.x() - rhs.x();
            let y = lhs.y() - rhs.y();
            let z = lhs.z() - rhs.z();
            Ok(x * x + y * y + z * z)
        })?,
    )?;
    vector.set(
        "dot",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| Ok(vector_dot3(lhs, rhs)))?,
    )?;
    vector.set(
        "cross",
        lua.create_function(|_, (lhs, rhs): (LuaVector, LuaVector)| {
            Ok(lhs.x() * rhs.y() - lhs.y() * rhs.x())
        })?,
    )?;
    vector.set(
        "cross3",
        lua.create_function(|_, (a, b): (LuaVector, LuaVector)| {
            Ok(LuaVector::new(
                a.y() * b.z() - a.z() * b.y(),
                a.z() * b.x() - a.x() * b.z(),
                a.x() * b.y() - a.y() * b.x(),
            ))
        })?,
    )?;
    vector.set(
        "scaleAndAdd",
        lua.create_function(|_, (a, b, scale): (LuaVector, LuaVector, f32)| {
            Ok(LuaVector::new(
                a.x() + b.x() * scale,
                a.y() + b.y() * scale,
                a.z() + b.z() * scale,
            ))
        })?,
    )?;
    vector.set(
        "scaleAndSub",
        lua.create_function(|_, (a, b, scale): (LuaVector, LuaVector, f32)| {
            Ok(LuaVector::new(
                a.x() - b.x() * scale,
                a.y() - b.y() * scale,
                a.z() - b.z() * scale,
            ))
        })?,
    )?;
    vector.set(
        "lerp",
        lua.create_function(|_, (lhs, rhs, factor): (LuaVector, LuaVector, f32)| {
            Ok(LuaVector::new(
                vector_lerp_component(lhs.x(), rhs.x(), factor),
                vector_lerp_component(lhs.y(), rhs.y(), factor),
                vector_lerp_component(lhs.z(), rhs.z(), factor),
            ))
        })?,
    )?;
    vector.set(
        "xy",
        lua.create_function(|_, (x, y): (Option<f32>, Option<f32>)| {
            Ok(LuaVector::new(x.unwrap_or(0.0), y.unwrap_or(0.0), 0.0))
        })?,
    )?;
    vector.set(
        "xyz",
        lua.create_function(|_, (x, y, z): (Option<f32>, Option<f32>, Option<f32>)| {
            Ok(LuaVector::new(
                x.unwrap_or(0.0),
                y.unwrap_or(0.0),
                z.unwrap_or(0.0),
            ))
        })?,
    )?;
    vector.set(
        "origin",
        lua.create_function(|_, ()| Ok(LuaVector::zero()))?,
    )?;
    vector.set(
        "length",
        lua.create_function(|_, value: LuaVector| Ok(vector_dot3(value, value).sqrt()))?,
    )?;
    vector.set(
        "lengthSquared",
        lua.create_function(|_, value: LuaVector| Ok(vector_dot3(value, value)))?,
    )?;
    vector.set(
        "normalized",
        lua.create_function(|_, value: LuaVector| {
            let length_squared = vector_dot3(value, value);
            let scale = if length_squared > 0.0 {
                1.0 / length_squared.sqrt()
            } else {
                1.0
            };
            Ok(LuaVector::new(
                value.x() * scale,
                value.y() * scale,
                value.z() * scale,
            ))
        })?,
    )?;
    vector.set(
        "writeToBuffer",
        lua.create_function(|_, (value, buffer, offset): (LuaVector, LuaBuffer, i64)| {
            write_vector_buffer(value, &buffer, offset, None, "writeToBuffer")
        })?,
    )?;
    vector.set(
        "writeVec4",
        lua.create_function(
            |_, (value, buffer, offset, w): (LuaVector, LuaBuffer, i64, f32)| {
                write_vector_buffer(value, &buffer, offset, Some(w), "writeVec4")
            },
        )?,
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

    // Rive replaces Luau's built-in vector metatable so instance syntax
    // (`value:length()`) reaches the same bindings as `Vector.length(value)`.
    // An __index callback also preserves axis and numeric component access.
    let methods = vector.clone();
    let value_metatable = lua.create_table();
    value_metatable.set(
        "__index",
        lua.create_function(move |_, (value, key): (LuaVector, Value)| {
            if let Some(component) = vector_component(value, &key)? {
                return Ok(Value::Number(component as f64));
            }

            if let Value::String(name) = &key {
                let name = name.to_str()?;
                if matches!(
                    name.as_str(),
                    "length"
                        | "lengthSquared"
                        | "normalized"
                        | "distance"
                        | "distanceSquared"
                        | "dot"
                        | "lerp"
                        | "writeToBuffer"
                        | "writeVec4"
                ) {
                    let method: Value = methods.get(name.as_str())?;
                    if !matches!(method, Value::Nil) {
                        return Ok(method);
                    }
                }
            }

            Err(Error::runtime(format!(
                "'{}' is not a valid index of Vector",
                vector_index_name(&key)
            )))
        })?,
    )?;
    value_metatable.set_readonly(true);
    lua.set_type_metatable::<LuaVector>(Some(value_metatable));

    vector.set_readonly(true);
    lua.globals().set("Vector", vector)
}

#[inline]
fn vector_dot3(lhs: LuaVector, rhs: LuaVector) -> f32 {
    lhs.x() * rhs.x() + lhs.y() * rhs.y() + lhs.z() * rhs.z()
}

#[inline]
fn vector_lerp_component(a: f32, b: f32, factor: f32) -> f32 {
    if factor == 1.0 {
        b
    } else {
        a + (b - a) * factor
    }
}

fn write_vector_buffer(
    value: LuaVector,
    buffer: &LuaBuffer,
    offset: i64,
    w: Option<f32>,
    method: &'static str,
) -> Result<()> {
    let byte_len = if w.is_some() { 16 } else { 12 };
    let error = || Error::runtime(format!("Vector:{method} offset out of range"));
    let offset = usize::try_from(offset).map_err(|_| error())?;
    if offset > buffer.len().saturating_sub(byte_len) || buffer.len() < byte_len {
        return Err(error());
    }

    let mut bytes = [0_u8; 16];
    for (index, component) in [value.x(), value.y(), value.z()].into_iter().enumerate() {
        let start = index * 4;
        bytes[start..start + 4].copy_from_slice(&component.to_ne_bytes());
    }
    if let Some(w) = w {
        bytes[12..16].copy_from_slice(&w.to_ne_bytes());
    }
    buffer.write_bytes(offset, &bytes[..byte_len]);
    Ok(())
}

fn vector_component(value: LuaVector, key: &Value) -> Result<Option<f32>> {
    let index = match key {
        Value::String(name) => match name.to_str()?.as_str() {
            "x" | "1" => Some(0),
            "y" | "2" => Some(1),
            "z" | "3" => Some(2),
            _ => None,
        },
        Value::Integer(1) => Some(0),
        Value::Integer(2) => Some(1),
        Value::Integer(3) => Some(2),
        Value::Number(number) if *number == 1.0 => Some(0),
        Value::Number(number) if *number == 2.0 => Some(1),
        Value::Number(number) if *number == 3.0 => Some(2),
        _ => None,
    };
    Ok(index.map(|index| [value.x(), value.y(), value.z()][index]))
}

fn vector_index_name(key: &Value) -> String {
    match key {
        Value::String(value) => value
            .to_str()
            .unwrap_or_else(|_| "<non-utf8 string>".to_owned()),
        other => format!("{other:?}"),
    }
}

fn install_math_fround(lua: &Lua) -> Result<()> {
    let math: Table = lua.globals().get("math")?;
    math.set(
        "fround",
        lua.create_function(|_, value: f64| Ok(f64::from(value as f32)))?,
    )
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
        let context_missing_requested_data = Rc::new(Cell::new(false));
        let context_parent_view_models = self.default_context_parent_view_models.clone();
        let context = self
            .lua
            .create_userdata(ScriptedContext::new(
                Rc::clone(&context_view_model),
                context_parent_view_models.clone(),
                Rc::clone(&context_missing_requested_data),
            ))
            .map_err(script_error)?;
        let instance: Table = generator.call(context.clone()).map_err(script_error)?;
        Ok(Box::new(LuaScriptInstance::with_renderer_bindings(
            instance,
            self.renderer_bindings.clone(),
            context_view_model,
            Some(context),
            context_missing_requested_data,
            context_parent_view_models,
            Some(generator),
        )))
    }

    fn advance_detached_view_models(&mut self) -> bool {
        ScriptVm::advance_detached_view_models(self)
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
        let value = self.call_method_value(method, args)?;
        script_value_from_lua(value).map_err(script_error)
    }

    fn call_listener_action(
        &mut self,
        invocation: ScriptListenerInvocation,
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<(), ScriptError> {
        let perform_action: Value = self.table.get("performAction").map_err(script_error)?;
        if let Value::Function(function) = perform_action {
            let lua = self.table.lua();
            let invocation = lua
                .create_userdata(ScriptedInvocation::new(invocation))
                .map_err(script_error)?;
            let _: () = function
                .call((self.table.clone(), invocation))
                .map_err(script_error)?;
            return Ok(());
        }

        let perform: Value = self.table.get("perform").map_err(script_error)?;
        if let Value::Function(function) = perform {
            let lua = self.table.lua();
            let pointer = lua
                .create_userdata(ScriptedPointerEvent::new(invocation))
                .map_err(script_error)?;
            let _: () = function
                .call((self.table.clone(), pointer))
                .map_err(script_error)?;
        }
        Ok(())
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
        let result = bindings.with_factory_context(factory, || {
            let missing_before = self.context_missing_requested_data.replace(false);
            let value = self.call_method_value(ScriptMethod::Init, &[]);
            let missing_during = self.context_missing_requested_data.replace(false);
            let value = value?;
            Ok(!missing_before
                && !missing_during
                && !matches!(value, Value::Nil | Value::Boolean(false)))
        });
        match result {
            Ok(initialized) => {
                self.user_init_done = initialized;
                self.init_retry_requires_recreation = !initialized && self.generator.is_some();
                Ok(initialized)
            }
            Err(error) => {
                self.user_init_done = false;
                self.init_retry_requires_recreation = self.generator.is_some();
                Err(error)
            }
        }
    }

    fn user_init_pending(&self) -> std::result::Result<bool, ScriptError> {
        if self.user_init_done {
            return Ok(false);
        }
        let value: Value = self
            .table
            .get(ScriptMethod::Init.as_str())
            .map_err(script_error)?;
        Ok(matches!(value, Value::Function(_)))
    }

    fn invalidate_for_init_retry(&mut self) {
        self.user_init_done = false;
        self.init_retry_requires_recreation = self.generator.is_some();
    }

    fn prepare_init_retry_with_factory(
        &mut self,
        factory: &mut dyn RenderFactory,
    ) -> std::result::Result<(), ScriptError> {
        if !self.init_retry_requires_recreation {
            return Ok(());
        }
        let Some(generator) = self.generator.clone() else {
            self.context_missing_requested_data.set(false);
            self.init_retry_requires_recreation = false;
            return Ok(());
        };

        let lua = self.table.lua();
        let context_view_model = Rc::clone(&self.context_view_model);
        let context_parent_view_models = self.context_parent_view_models.clone();
        let bindings = self.renderer_bindings.clone();
        let (table, context, missing_requested_data) =
            bindings.with_factory_context(factory, || {
                let missing_requested_data = Rc::new(Cell::new(false));
                let context = lua
                    .create_userdata(ScriptedContext::new(
                        context_view_model,
                        context_parent_view_models,
                        Rc::clone(&missing_requested_data),
                    ))
                    .map_err(script_error)?;
                let table = generator.call(context.clone()).map_err(script_error)?;
                Ok((table, context, missing_requested_data))
            })?;

        self.table = table;
        self.context = Some(context);
        self.context_missing_requested_data = missing_requested_data;
        self.user_init_done = false;
        self.init_retry_requires_recreation = false;
        Ok(())
    }

    fn call_path_effect_update(
        &mut self,
        source: nuxie_render_api::RawPath,
        node: nuxie_runtime::ScriptNode,
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<nuxie_render_api::RawPath, ScriptError> {
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

    fn call_data_converter(
        &mut self,
        method: ScriptDataConverterMethod,
        value: ScriptValue,
    ) -> std::result::Result<ScriptValue, ScriptError> {
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
        let value: Value = self.table.get(name).map_err(script_error)?;
        script_value_from_lua(value).map_err(script_error)
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
mod context_init_tests {
    use super::*;
    use nuxie_render_api::NullFactory;
    use nuxie_runtime::NoopScriptHost;

    #[test]
    fn protected_host_callback_errors_remain_catchable() {
        let lua = Lua::new();
        let host_error = lua
            .create_function(|_, (): ()| -> Result<()> {
                Err(Error::runtime("expected host error"))
            })
            .expect("host callback");
        lua.globals()
            .set("hostError", host_error)
            .expect("host callback global");

        let (ok, message): (bool, String) = lua
            .load("return pcall(hostError)")
            .eval()
            .expect("pcall catches the host error");

        assert!(!ok);
        assert_eq!(message, "runtime error: expected host error");
    }

    #[test]
    fn failed_init_recreates_the_script_lifetime_before_retry() {
        let lua = Lua::new();
        let frame_context = ScriptViewModelFrameContext::default();
        lua.set_app_data(frame_context.clone());
        lua.globals()
            .set("generations", 0)
            .expect("generation global");
        let generator: Function = lua
            .load(
                r#"
                return function()
                    generations += 1
                    return {
                        generation = generations,
                        init = function() return true end,
                    }
                end
                "#,
            )
            .eval()
            .expect("script generator");
        let missing_requested_data = Rc::new(Cell::new(false));
        let context_view_model = Rc::new(RefCell::new(None));
        let context = lua
            .create_userdata(ScriptedContext::new(
                Rc::clone(&context_view_model),
                Vec::new(),
                Rc::clone(&missing_requested_data),
            ))
            .expect("scripted context");
        let table: Table = generator.call(context.clone()).expect("script table");
        let mut instance = LuaScriptInstance::with_renderer_bindings(
            table,
            RendererBindings::new(frame_context),
            context_view_model,
            Some(context),
            missing_requested_data,
            Vec::new(),
            Some(generator),
        );
        let mut factory = NullFactory::new();
        let mut host = NoopScriptHost;

        instance.context_missing_requested_data.set(true);
        assert!(
            !instance
                .call_init_with_factory(&mut host, &mut factory)
                .expect("cold init")
        );
        assert!(instance.user_init_pending().expect("pending cold init"));
        instance
            .prepare_init_retry_with_factory(&mut factory)
            .expect("recreate script lifetime");
        assert_eq!(instance.table.get::<i64>("generation").unwrap(), 2);
        assert!(
            instance
                .call_init_with_factory(&mut host, &mut factory)
                .expect("bound retry")
        );
        assert!(!instance.user_init_pending().expect("completed retry"));
    }
}
