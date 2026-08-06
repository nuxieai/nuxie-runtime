//! Luau VM boot + execution on `luaur` (pure-Rust Luau).
//!
//! Mirrors the shape of C++ `nuxie::ScriptingVM`
//! (`include/rive/lua/scripting_vm.hpp`, `src/lua/rive_lua_libs.cpp`):
//! the runtime never compiles Luau source — the editor ships precompiled
//! Luau bytecode inside `ScriptAsset`, and `ScriptingVM::loadModule` feeds
//! it straight to `luau_load`. [`ScriptVm::load_bytecode`] is the Rust
//! equivalent, built on `luaur_rt::Lua::exec_raw` + `luaur_vm::luau_load`.
//!
//! Source compilation is deliberately owned by editor tooling. This baseline
//! surface accepts the precompiled Luau bytecode carried by `.riv` files.

mod buffer_ext;
mod bytecode;
mod command_server;
mod listener_invocation;
mod logging_scripting_context;
pub(crate) mod lua_blob;
mod lua_color;
mod lua_data_value;
mod lua_font;
mod lua_image;
mod lua_image_decode;
mod lua_mat4;
mod lua_math;
mod lua_mesh;
mod lua_promise;
mod lua_rive_base;
mod lua_vec2d;
mod renderer;

mod lua_artboards;
mod lua_audio;
mod lua_mat2d;
mod lua_paint;
mod lua_path;
mod lua_renderer;
mod lua_renderer_library;
mod resource_limits;
mod view_model;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::CString;
use std::rc::Rc;

use bytecode::validate_luau_bytecode;
use logging_scripting_context::LoggingScriptingContext;
use lua_math::install_math_globals;
use lua_rive_base::install_host_print;
use luaur_rt::ffi::lua_error;
use luaur_rt::{
    AnyUserData, FromLuaMulti, Function, IntoLuaMulti, Lua, MultiValue, Table, Value,
    Vector as LuaVector, VmState,
};
use luaur_vm::functions::lua_callbacks::lua_callbacks;
use luaur_vm::functions::luau_load::luau_load;
use nuxie_render_api::{Factory as RenderFactory, Renderer};
use nuxie_runtime::{
    ScriptArtboard, ScriptCoreString, ScriptDataConverterMethod, ScriptDataConverterOptionalCall,
    ScriptError, ScriptHost, ScriptInstance, ScriptInterpolatorMethod, ScriptListenerActionMethod,
    ScriptListenerInvocation, ScriptMethod, ScriptOptionalMethodResult, ScriptOptionalNumberResult,
    ScriptValue, ScriptViewModel, ScriptingVm as RuntimeScriptingVm,
};
pub(crate) use renderer::RendererBindings;
use view_model::{ScriptViewModelFrameContext, ScriptedContext, create_scripted_view_model};

use crate::envelope::SignedContent;
use crate::gpu_canvas::{
    ImportedGpuCanvasInstance, ImportedGpuCanvasShaderAssetEntry, ImportedGpuCanvasShaderAssets,
    RegisteredGpuCanvasShaderAsset,
};

pub use logging_scripting_context::{ScriptingLogLevel, ScriptingLogSink};
pub use luaur_rt::{Error, Result};
pub use resource_limits::{ScriptResourceGuard, ScriptResourceLimit};

/// Registry key for the require cache (C++: `registeredCacheTableKey` in
/// `src/lua/rive_lua_libs.cpp`).
const MODULE_CACHE_KEY: &str = "rive_scripting_registered_modules";
const SCRIPT_VM_MEMORY_LIMIT_BYTES: usize = 16 * 1024 * 1024;
const SCRIPT_SAFEPOINTS_PER_CYCLE: usize = 100_000;

// Direct port of the compile-time atom table in
// `src/lua/rive_lua_libs.cpp`. The numeric values are the corresponding
// `LuaAtoms` discriminants from `rive_lua_libs.hpp`.
const RIVE_LUA_ATOMS: &[(&[u8], i16)] = &[
    (b"length", 0),
    (b"lengthSquared", 1),
    (b"normalized", 2),
    (b"distance", 3),
    (b"distanceSquared", 4),
    (b"dot", 5),
    (b"lerp", 6),
    (b"moveTo", 7),
    (b"lineTo", 8),
    (b"quadTo", 9),
    (b"cubicTo", 10),
    (b"close", 11),
    (b"type", 16),
    (b"reset", 12),
    (b"add", 13),
    (b"contours", 14),
    (b"measure", 15),
    (b"invert", 18),
    (b"isIdentity", 19),
    (b"width", 20),
    (b"height", 21),
    (b"clamp", 22),
    (b"repeat", 23),
    (b"mirror", 24),
    (b"bilinear", 25),
    (b"nearest", 26),
    (b"style", 27),
    (b"join", 28),
    (b"cap", 29),
    (b"thickness", 30),
    (b"blendMode", 31),
    (b"feather", 32),
    (b"gradient", 33),
    (b"color", 34),
    (b"stroke", 35),
    (b"fill", 36),
    (b"miter", 37),
    (b"round", 38),
    (b"bevel", 39),
    (b"butt", 40),
    (b"square", 41),
    (b"srcOver", 42),
    (b"screen", 43),
    (b"overlay", 44),
    (b"darken", 45),
    (b"lighten", 46),
    (b"colorDodge", 47),
    (b"colorBurn", 48),
    (b"hardLight", 49),
    (b"softLight", 50),
    (b"difference", 51),
    (b"exclusion", 52),
    (b"multiply", 53),
    (b"hue", 54),
    (b"saturation", 55),
    (b"luminosity", 56),
    (b"copy", 57),
    (b"drawPath", 58),
    (b"drawImage", 59),
    (b"drawImageMesh", 60),
    (b"clipPath", 61),
    (b"save", 62),
    (b"restore", 63),
    (b"transform", 64),
    (b"value", 65),
    (b"red", 66),
    (b"green", 67),
    (b"blue", 68),
    (b"alpha", 69),
    (b"getNumber", 70),
    (b"getTrigger", 71),
    (b"getString", 72),
    (b"getBoolean", 73),
    (b"getColor", 74),
    (b"getList", 75),
    (b"getViewModel", 76),
    (b"getEnum", 77),
    (b"getIndex", 78),
    (b"getImage", 79),
    (b"getFont", 80),
    (b"getBlob", 81),
    (b"values", 82),
    (b"addListener", 83),
    (b"removeListener", 84),
    (b"fire", 85),
    (b"push", 86),
    (b"insert", 87),
    (b"pop", 89),
    (b"swap", 90),
    (b"shift", 88),
    (b"clear", 91),
    (b"draw", 92),
    (b"advance", 93),
    (b"frameOrigin", 94),
    (b"data", 95),
    (b"instance", 96),
    (b"animation", 97),
    (b"new", 98),
    (b"bounds", 99),
    (b"pointerDown", 100),
    (b"pointerUp", 102),
    (b"pointerMove", 101),
    (b"pointerExit", 103),
    (b"isNumber", 106),
    (b"isString", 107),
    (b"isBoolean", 108),
    (b"isColor", 109),
    (b"hit", 110),
    (b"id", 111),
    (b"position", 112),
    (b"rotation", 113),
    (b"scale", 114),
    (b"worldTransform", 115),
    (b"scaleX", 116),
    (b"scaleY", 117),
    (b"decompose", 118),
    (b"children", 119),
    (b"parent", 120),
    (b"node", 121),
    (b"paint", 122),
    (b"asPath", 124),
    (b"asPaint", 123),
    (b"addToPath", 104),
    (b"positionAndTangent", 125),
    (b"warp", 126),
    (b"extract", 127),
    (b"next", 128),
    (b"isClosed", 129),
    (b"markNeedsUpdate", 130),
    (b"viewModel", 131),
    (b"rootViewModel", 132),
    (b"dataContext", 136),
    (b"image", 133),
    (b"blob", 134),
    (b"size", 135),
    (b"name", 105),
    (b"duration", 153),
    (b"setTime", 154),
    (b"setTimeFrames", 155),
    (b"setTimePercentage", 156),
    (b"isPointerEvent", 159),
    (b"isKeyboardEvent", 160),
    (b"isTextInput", 161),
    (b"previousPosition", 157),
    (b"timeStamp", 158),
    (b"isFocus", 162),
    (b"isReportedEvent", 163),
    (b"isViewModelChange", 164),
    (b"isNone", 165),
    (b"isGamepadConnected", 166),
    (b"isGamepadEvent", 167),
    (b"isGamepadDisconnected", 168),
    (b"asPointerEvent", 169),
    (b"asKeyboardEvent", 170),
    (b"asTextInput", 171),
    (b"asFocus", 172),
    (b"asReportedEvent", 173),
    (b"asViewModelChange", 174),
    (b"asGamepadConnected", 175),
    (b"asGamepadEvent", 176),
    (b"asGamepadDisconnected", 177),
    (b"gamepadEvent", 178),
    (b"gamepadConnected", 179),
    (b"gamepadDisconnected", 180),
    (b"asNone", 181),
    (b"key", 182),
    (b"shift", 88),
    (b"alt", 183),
    (b"control", 184),
    (b"meta", 185),
    (b"text", 186),
    (b"phase", 187),
    (b"delaySeconds", 188),
    (b"deviceId", 189),
    (b"buttonMask", 190),
    (b"remove", 191),
    (b"removeAt", 192),
    (b"removeAllOf", 193),
    (b"axes", 232),
    (b"gamepadMapping", 233),
    (b"mapping", 234),
    (b"isStandardMapping", 235),
    (b"buttons", 236),
    (b"buttonPressed", 237),
    (b"buttonValue", 238),
    (b"axis", 239),
    (b"west", 240),
    (b"south", 241),
    (b"north", 242),
    (b"east", 243),
    (b"leftShoulder", 244),
    (b"rightShoulder", 245),
    (b"back", 246),
    (b"forward", 247),
    (b"leftStickButton", 248),
    (b"rightStickButton", 249),
    (b"dpadUp", 250),
    (b"dpadDown", 251),
    (b"dpadLeft", 252),
    (b"dpadRight", 253),
    (b"start", 256),
    (b"leftStick", 254),
    (b"rightStick", 255),
    (b"leftTrigger", 257),
    (b"rightTrigger", 258),
    (b"leftTriggerPressed", 259),
    (b"rightTriggerPressed", 260),
    (b"changeKind", 261),
    (b"changeIndex", 262),
    (b"changeValue", 263),
    (b"hasStandardButtonIntent", 264),
    (b"hasStandardAxisIntent", 265),
    (b"intentButton", 266),
    (b"intentAxis", 267),
    (b"audio", 137),
    (b"play", 138),
    (b"playAtTime", 139),
    (b"playInTime", 140),
    (b"playAtFrame", 141),
    (b"playInFrame", 142),
    (b"stop", 143),
    (b"pause", 144),
    (b"resume", 145),
    (b"seek", 146),
    (b"seekFrame", 147),
    (b"volume", 148),
    (b"completed", 149),
    (b"time", 150),
    (b"timeFrame", 151),
    (b"sampleRate", 152),
    (b"write", 194),
    (b"upload", 195),
    (b"view", 196),
    (b"setPipeline", 197),
    (b"setVertexBuffer", 198),
    (b"setIndexBuffer", 199),
    (b"setBindGroup", 200),
    (b"setViewport", 201),
    (b"setScissorRect", 202),
    (b"setStencilReference", 203),
    (b"drawIndexed", 205),
    (b"finish", 206),
    (b"beginRenderPass", 207),
    (b"beginFrame", 208),
    (b"endFrame", 209),
    (b"colorView", 210),
    (b"depthView", 211),
    (b"setBlendColor", 204),
    (b"resize", 212),
    (b"canvas", 213),
    (b"gpuCanvas", 214),
    (b"features", 216),
    (b"drawCanvas", 215),
    (b"shader", 217),
    (b"format", 218),
    (b"andThen", 219),
    (b"catch", 220),
    (b"finally", 221),
    (b"cancel", 222),
    (b"onCancel", 223),
    (b"getStatus", 224),
    (b"decodeImage", 225),
    (b"transpose", 226),
    (b"transformPoint", 227),
    (b"transformVec4", 228),
    (b"writeToBuffer", 229),
    (b"invertAffine", 230),
    (b"writeVec4", 231),
];

const RIVE_LUA_ATOM_SLOT_COUNT: usize = 1024;

const fn hash_rive_lua_atom(name: &[u8]) -> u32 {
    let mut hash = 2_166_136_261_u32;
    let mut index = 0;
    while index < name.len() {
        hash = (hash ^ name[index] as u32).wrapping_mul(16_777_619);
        index += 1;
    }
    hash
}

const fn rive_lua_atom_names_equal(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

const fn longest_rive_lua_atom_name() -> usize {
    let mut longest = 0;
    let mut index = 0;
    while index < RIVE_LUA_ATOMS.len() {
        let length = RIVE_LUA_ATOMS[index].0.len();
        if length > longest {
            longest = length;
        }
        index += 1;
    }
    longest
}

const RIVE_LUA_MAX_ATOM_NAME_LENGTH: usize = longest_rive_lua_atom_name();

const fn build_rive_lua_atom_slots() -> [u16; RIVE_LUA_ATOM_SLOT_COUNT] {
    let mut slots = [0; RIVE_LUA_ATOM_SLOT_COUNT];
    let mut index = 0;
    while index < RIVE_LUA_ATOMS.len() {
        let name = RIVE_LUA_ATOMS[index].0;
        let mut slot = hash_rive_lua_atom(name) as usize & (RIVE_LUA_ATOM_SLOT_COUNT - 1);
        while slots[slot] != 0
            && !rive_lua_atom_names_equal(RIVE_LUA_ATOMS[slots[slot] as usize - 1].0, name)
        {
            slot = (slot + 1) & (RIVE_LUA_ATOM_SLOT_COUNT - 1);
        }
        if slots[slot] == 0 {
            slots[slot] = index as u16 + 1;
        }
        index += 1;
    }
    slots
}

const RIVE_LUA_ATOM_SLOTS: [u16; RIVE_LUA_ATOM_SLOT_COUNT] = build_rive_lua_atom_slots();

fn find_rive_lua_atom(name: &[u8]) -> i16 {
    if name.len() > RIVE_LUA_MAX_ATOM_NAME_LENGTH {
        return -1;
    }
    let mut slot = hash_rive_lua_atom(name) as usize & (RIVE_LUA_ATOM_SLOT_COUNT - 1);
    loop {
        let biased_index = RIVE_LUA_ATOM_SLOTS[slot];
        if biased_index == 0 {
            return -1;
        }
        let (candidate, atom) = RIVE_LUA_ATOMS[biased_index as usize - 1];
        if candidate == name {
            return atom;
        }
        slot = (slot + 1) & (RIVE_LUA_ATOM_SLOT_COUNT - 1);
    }
}

unsafe extern "C" fn resolve_rive_lua_atom(
    _state: *mut luaur_vm::records::lua_state::lua_State,
    chars: *const core::ffi::c_char,
    length: usize,
) -> i16 {
    if chars.is_null() {
        return -1;
    }
    find_rive_lua_atom(unsafe { std::slice::from_raw_parts(chars.cast::<u8>(), length) })
}

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
    initialization_error: Option<String>,
    execution_budget: Option<ScriptExecutionBudget>,
    rive_globals_installed: Cell<bool>,
    renderer_bindings: RendererBindings,
    view_model_frame_context: ScriptViewModelFrameContext,
    view_models: BTreeMap<String, ScriptViewModel>,
    default_context_view_model: Option<ScriptViewModel>,
    default_context_parent_view_models: Vec<Option<ScriptViewModel>>,
    script_safepoints: Rc<Cell<usize>>,
    script_cycle_active: Rc<Cell<bool>>,
    resource_limits: resource_limits::ResourceLimitTracker,
    blob_assets: lua_blob::ScriptedBlobAssets,
    audio_assets: lua_audio::ScriptedAudioAssets,
    gpu_canvas_shaders: ImportedGpuCanvasShaderAssets,
    logging: LoggingScriptingContext,
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
    /// Rust's initialized equivalent of C++ `ScriptedObject::m_self`.
    ///
    /// `None` is the literal `m_self == 0` state after generator/init failure
    /// or disposal. Keeping this optional is important: retaining a failed
    /// table would keep all of its captured Lua values alive after pinned C++
    /// has already `lua_unref`'d the occurrence.
    table: Option<Table>,
    execution_budget: Option<ScriptExecutionBudget>,
    script_safepoints: Option<Rc<Cell<usize>>>,
    script_cycle_active: Option<Rc<Cell<bool>>>,
    renderer_bindings: RendererBindings,
    context_view_model: Rc<RefCell<Option<ScriptViewModel>>>,
    context_present: Rc<Cell<bool>>,
    context: Option<AnyUserData>,
    context_alive: Option<Rc<Cell<bool>>>,
    context_missing_requested_data: Rc<Cell<bool>>,
    context_view_model_is_resolved: bool,
    context_parent_view_models: Vec<Option<ScriptViewModel>>,
    generator: Option<Function>,
    user_init_done: bool,
    init_retry_requires_recreation: bool,
    resource_limits: resource_limits::ResourceLimitTracker,
    gpu_canvas: Option<ImportedGpuCanvasInstance>,
    gpu_canvas_context: Option<crate::gpu_canvas::GpuCanvasContextBindings>,
    logging: LoggingScriptingContext,
}

impl LuaScriptInstance {
    pub fn new(table: Table) -> Self {
        let frame_context = ScriptViewModelFrameContext::for_lua(&table.lua());
        Self {
            table: Some(table),
            execution_budget: None,
            script_safepoints: None,
            script_cycle_active: None,
            renderer_bindings: RendererBindings::new(frame_context),
            context_view_model: Rc::new(RefCell::new(None)),
            context_present: Rc::new(Cell::new(false)),
            context: None,
            context_alive: None,
            context_missing_requested_data: Rc::new(Cell::new(false)),
            context_view_model_is_resolved: false,
            context_parent_view_models: Vec::new(),
            generator: None,
            user_init_done: false,
            init_retry_requires_recreation: false,
            resource_limits: resource_limits::ResourceLimitTracker::default(),
            gpu_canvas: None,
            gpu_canvas_context: None,
            logging: LoggingScriptingContext::default(),
        }
    }

    fn with_renderer_bindings(
        table: Table,
        renderer_bindings: RendererBindings,
        context_view_model: Rc<RefCell<Option<ScriptViewModel>>>,
        context_present: Rc<Cell<bool>>,
        context: Option<AnyUserData>,
        context_alive: Option<Rc<Cell<bool>>>,
        context_missing_requested_data: Rc<Cell<bool>>,
        context_parent_view_models: Vec<Option<ScriptViewModel>>,
        generator: Option<Function>,
        resource_limits: resource_limits::ResourceLimitTracker,
        gpu_canvas: Option<ImportedGpuCanvasInstance>,
        gpu_canvas_context: Option<crate::gpu_canvas::GpuCanvasContextBindings>,
        execution_budget: Option<ScriptExecutionBudget>,
        script_safepoints: Option<Rc<Cell<usize>>>,
        script_cycle_active: Option<Rc<Cell<bool>>>,
        logging: LoggingScriptingContext,
    ) -> Self {
        Self {
            table: Some(table),
            execution_budget,
            script_safepoints,
            script_cycle_active,
            renderer_bindings,
            context_view_model,
            context_present,
            context,
            context_alive,
            context_missing_requested_data,
            context_view_model_is_resolved: false,
            context_parent_view_models,
            generator,
            user_init_done: false,
            init_retry_requires_recreation: false,
            resource_limits,
            gpu_canvas,
            gpu_canvas_context,
            logging,
        }
    }

    pub fn table(&self) -> &Table {
        self.table
            .as_ref()
            .expect("LuaScriptInstance has no live C++ m_self table")
    }

    fn live_table(&self) -> std::result::Result<Table, ScriptError> {
        self.table
            .clone()
            .ok_or_else(|| ScriptError::new("scripted object has no live Lua table"))
    }

    fn script_error(&self, error: Error) -> ScriptError {
        self.logging.log_error(&error);
        tracked_script_error(error, &self.resource_limits)
    }

    fn reset_execution_budget(&self) {
        if let Some(lua) = self
            .table
            .as_ref()
            .map(Table::lua)
            .or_else(|| self.generator.as_ref().map(Function::lua))
        {
            if let Err(error) = lua_image_decode::poll_completed(&lua) {
                self.logging.log_error(&error);
            }
        }
        if self
            .script_cycle_active
            .as_ref()
            .is_some_and(|active| !active.get())
        {
            self.resource_limits.begin_cycle();
            if let Some(script_safepoints) = self.script_safepoints.as_ref() {
                script_safepoints.set(0);
            }
        }
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
        let Some(table) = self.table.clone() else {
            return Ok(Value::Nil);
        };
        let value: Value = table
            .get(method.as_str())
            .map_err(|error| self.script_error(error))?;
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

        let lua = table.lua();
        let mut call_args = MultiValue::with_capacity(args.len() + 1);
        call_args.push_back(Value::Table(table));
        for arg in args {
            call_args.push_back(script_value_to_lua(&lua, arg));
        }
        if method == ScriptMethod::Init
            && let Some(context) = self.context.as_ref()
        {
            call_args.push_back(Value::UserData(context.clone()));
        }
        function
            .call(call_args)
            .map_err(|error| self.script_error(error))
    }

    fn dispose_script_lifetime(&mut self) {
        // Pinned `tryLuaUserInit` drops `m_self` before disposing the Context
        // (`scripted_object.cpp:277-303`). Taking the registry-backed Table
        // here immediately releases the failed occurrence and its captures.
        self.table.take();
        if let Some(context_alive) = self.context_alive.take() {
            context_alive.set(false);
        }
        self.context = None;
    }

    fn call_init_with_optional_factory(
        &mut self,
        factory: Option<&mut dyn RenderFactory>,
    ) -> std::result::Result<bool, ScriptError> {
        let bindings = self.renderer_bindings.clone();
        let call_init = || {
            let missing_before = self.context_missing_requested_data.replace(false);
            self.reset_execution_budget();
            let table = self.live_table()?;
            let value: Value = table
                .get(ScriptMethod::Init.as_str())
                .map_err(|error| self.script_error(error))?;
            let Value::Function(function) = value else {
                // `tryLuaUserInit` treats a missing/non-function field as a
                // completed initialization. Resolve the field only once:
                // a metatable may return a different value on a second read
                // (`scripted_object.cpp:259-278`).
                self.context_missing_requested_data.set(false);
                return Ok(true);
            };
            let mut args = MultiValue::with_capacity(2);
            args.push_back(Value::Table(table));
            if let Some(context) = self.context.as_ref() {
                args.push_back(Value::UserData(context.clone()));
            }
            let value = function
                .call(args)
                .map_err(|error| self.script_error(error));
            let missing_during = self.context_missing_requested_data.replace(false);
            let value = value?;
            let missing_requested_data = missing_before || missing_during;
            Ok(!missing_requested_data && !matches!(value, Value::Nil | Value::Boolean(false)))
        };
        if let Some(factory) = factory {
            bindings
                .verify_render_context(factory)
                .map_err(|error| self.script_error(error))?;
        }
        let result = call_init();
        match result {
            Ok(initialized) => {
                self.user_init_done = initialized;
                self.init_retry_requires_recreation = !initialized && self.generator.is_some();
                if !initialized {
                    self.dispose_script_lifetime();
                }
                Ok(initialized)
            }
            Err(error) => {
                self.user_init_done = false;
                self.init_retry_requires_recreation = self.generator.is_some();
                self.dispose_script_lifetime();
                Err(error)
            }
        }
    }

    fn prepare_init_retry_with_optional_factory(
        &mut self,
        factory: Option<&mut dyn RenderFactory>,
    ) -> std::result::Result<(), ScriptError> {
        if !self.init_retry_requires_recreation {
            return Ok(());
        }
        let Some(generator) = self.generator.clone() else {
            self.context_missing_requested_data.set(false);
            self.init_retry_requires_recreation = false;
            return Ok(());
        };

        let lua = generator.lua();
        let context_view_model = Rc::clone(&self.context_view_model);
        let context_present = Rc::clone(&self.context_present);
        let context_parent_view_models = self.context_parent_view_models.clone();
        let bindings = self.renderer_bindings.clone();
        let context_alive = Rc::new(Cell::new(true));
        let recreate = || {
            let missing_requested_data = Rc::new(Cell::new(false));
            let context = lua
                .create_userdata(ScriptedContext::new_with_lifetime(
                    context_view_model,
                    context_present,
                    context_parent_view_models,
                    Rc::clone(&missing_requested_data),
                    self.gpu_canvas_context.clone(),
                    Rc::clone(&context_alive),
                ))
                .map_err(|error| self.script_error(error))?;
            self.reset_execution_budget();
            let table = generator
                .call(context.clone())
                .map_err(|error| self.script_error(error))?;
            Ok((table, context, missing_requested_data))
        };
        if let Some(factory) = factory {
            bindings
                .verify_render_context(factory)
                .map_err(|error| self.script_error(error))?;
        }
        let result = recreate();
        let (table, context, missing_requested_data) = match result {
            Ok(result) => result,
            Err(error) => {
                context_alive.set(false);
                return Err(error);
            }
        };

        self.table = Some(table);
        self.context = Some(context);
        self.context_alive = Some(context_alive);
        self.context_missing_requested_data = missing_requested_data;
        self.user_init_done = false;
        self.init_retry_requires_recreation = false;
        Ok(())
    }

    fn call_data_converter_once(
        &mut self,
        method: ScriptDataConverterMethod,
        value: ScriptValue,
    ) -> std::result::Result<Option<ScriptValue>, ScriptError> {
        Ok(
            match self.call_optional_data_converter_once(method, Some(value))? {
                ScriptDataConverterOptionalCall::Missing
                | ScriptDataConverterOptionalCall::UnsupportedInput => None,
                ScriptDataConverterOptionalCall::Returned(value) => Some(value),
            },
        )
    }

    fn call_optional_data_converter_once(
        &mut self,
        method: ScriptDataConverterMethod,
        value: Option<ScriptValue>,
    ) -> std::result::Result<ScriptDataConverterOptionalCall, ScriptError> {
        self.reset_execution_budget();
        let Some(table) = self.table.clone() else {
            return Ok(ScriptDataConverterOptionalCall::Missing);
        };
        let field: Value = table
            .get(method.as_str())
            .map_err(|error| self.script_error(error))?;
        let Value::Function(function) = field else {
            return Ok(ScriptDataConverterOptionalCall::Missing);
        };
        let Some(value) = value else {
            return Ok(ScriptDataConverterOptionalCall::UnsupportedInput);
        };
        let lua = table.lua();
        let input = lua_data_value::create_data_value(&lua, value)
            .map_err(|error| self.script_error(error))?;
        let output: AnyUserData = function
            .call((table, input))
            .map_err(|error| self.script_error(error))?;
        let output = output
            .borrow::<lua_data_value::ScriptedDataValue>()
            .map_err(|error| self.script_error(error))?;
        Ok(ScriptDataConverterOptionalCall::Returned(
            output.value().clone(),
        ))
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

impl ScriptVm {
    fn script_error(&self, error: Error) -> ScriptError {
        tracked_script_error(error, &self.resource_limits)
    }

    /// Attach the VM's one C++-lifetime render context before file import.
    ///
    /// The first factory establishes identity for the VM. Repeating the call
    /// with that same factory is harmless; attempting to switch factories is
    /// rejected.
    pub fn install_render_factory(
        &self,
        factory: &mut dyn RenderFactory,
    ) -> std::result::Result<(), ScriptError> {
        self.renderer_bindings
            .bootstrap_render_context(factory)
            .map_err(|error| self.script_error(error))
    }

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

    /// Execute one protocol ScriptAsset chunk and return the generator it
    /// produces. FileAsset-identity caching belongs to the caller because a
    /// name and library scope are not unique protocol-script identities.
    pub fn register_protocol_script_with_factory(
        &self,
        name: &str,
        payload: &[u8],
        factory: &mut dyn RenderFactory,
    ) -> std::result::Result<ScriptProgram, ScriptError> {
        self.install_render_factory(factory)?;
        self.install_rive_globals()
            .map_err(|error| self.script_error(error))?;
        let chunk = self
            .load_script_asset_payload(name, payload)
            .map_err(|error| self.script_error(error))?;
        self.reset_execution_budget();
        let generator = self
            .execute_loaded_module(name, chunk)
            .map_err(|error| self.script_error(error))?;
        Ok(ScriptProgram { generator })
    }

    /// Invoke a File-registered protocol generator for one concrete drawable
    /// occurrence, verifying the caller still supplies the VM's retained
    /// renderer-factory identity.
    pub fn instantiate_registered_script_with_factory(
        &self,
        program: &ScriptProgram,
        host: &mut dyn ScriptHost,
        factory: &mut dyn RenderFactory,
    ) -> std::result::Result<Box<dyn ScriptInstance>, ScriptError> {
        self.instantiate_registered_script_with_factory_and_context(
            program,
            host,
            factory,
            self.default_context_view_model.clone(),
            self.default_context_parent_view_models.clone(),
        )
    }

    /// Invoke a registered protocol generator with the live occurrence
    /// DataContext already installed.
    ///
    /// C++ assigns `ScriptedObject::m_dataContext` before
    /// `ScriptedDataConverter::reinit`, so the generator itself (not only
    /// later hydration/init) can resolve `context:viewModel()`
    /// (`scripted_data_converter.cpp:170-176`;
    /// `lua_scripted_context.cpp:125-185`).
    pub fn instantiate_registered_script_with_factory_and_context(
        &self,
        program: &ScriptProgram,
        _host: &mut dyn ScriptHost,
        factory: &mut dyn RenderFactory,
        context_view_model_value: Option<ScriptViewModel>,
        context_parent_view_models: Vec<Option<ScriptViewModel>>,
    ) -> std::result::Result<Box<dyn ScriptInstance>, ScriptError> {
        self.instantiate_registered_script_with_optional_factory_and_context(
            program,
            Some(factory),
            context_view_model_value,
            context_parent_view_models,
        )
    }

    /// Invoke a registered protocol generator without a callback-local factory
    /// argument. A render context installed on the VM before import remains
    /// available, matching pinned C++; a headless VM remains headless.
    pub fn instantiate_registered_script_with_context(
        &self,
        program: &ScriptProgram,
        context_view_model_value: Option<ScriptViewModel>,
        context_parent_view_models: Vec<Option<ScriptViewModel>>,
    ) -> std::result::Result<Box<dyn ScriptInstance>, ScriptError> {
        self.instantiate_registered_script_with_optional_factory_and_context(
            program,
            None,
            context_view_model_value,
            context_parent_view_models,
        )
    }

    /// Invoke a registered protocol generator with the VM's retained default
    /// DataContext and render bindings. Stateful keyframe interpolators use
    /// this after file bootstrap, when evaluation no longer owns the caller's
    /// mutable render-factory borrow.
    pub fn instantiate_registered_script(
        &self,
        program: &ScriptProgram,
    ) -> std::result::Result<Box<dyn ScriptInstance>, ScriptError> {
        self.instantiate_registered_script_with_optional_factory_and_context(
            program,
            None,
            self.default_context_view_model.clone(),
            self.default_context_parent_view_models.clone(),
        )
    }

    fn instantiate_registered_script_with_optional_factory_and_context(
        &self,
        program: &ScriptProgram,
        factory: Option<&mut dyn RenderFactory>,
        context_view_model_value: Option<ScriptViewModel>,
        context_parent_view_models: Vec<Option<ScriptViewModel>>,
    ) -> std::result::Result<Box<dyn ScriptInstance>, ScriptError> {
        let bindings = self.renderer_bindings.clone();
        let context_alive = Rc::new(Cell::new(true));
        let instantiate = || {
            // A retained parent slot proves that the DataContext exists even
            // when its own main ViewModel is null. Pinned C++ pushes the
            // DataContext userdata independently from mainViewModelInstance.
            let context_present = Rc::new(Cell::new(
                context_view_model_value.is_some() || !context_parent_view_models.is_empty(),
            ));
            let context_view_model = Rc::new(RefCell::new(context_view_model_value));
            let context_missing_requested_data = Rc::new(Cell::new(false));
            let (gpu_canvas, gpu_canvas_context) = ImportedGpuCanvasInstance::new(
                Rc::clone(&self.gpu_canvas_shaders),
                self.renderer_bindings.clone(),
            );
            let context = self
                .lua
                .create_userdata(ScriptedContext::new_with_lifetime(
                    Rc::clone(&context_view_model),
                    Rc::clone(&context_present),
                    context_parent_view_models.clone(),
                    Rc::clone(&context_missing_requested_data),
                    Some(gpu_canvas_context.clone()),
                    Rc::clone(&context_alive),
                ))
                .map_err(|error| self.script_error(error))?;
            self.reset_execution_budget();
            let instance: Table = self
                .track_resource_result(program.generator.call(context.clone()))
                .map_err(|error| self.script_error(error))?;
            Ok(Box::new(LuaScriptInstance::with_renderer_bindings(
                instance,
                self.renderer_bindings.clone(),
                context_view_model,
                context_present,
                Some(context),
                Some(Rc::clone(&context_alive)),
                context_missing_requested_data,
                context_parent_view_models,
                Some(program.generator.clone()),
                self.resource_limits.clone(),
                Some(gpu_canvas),
                Some(gpu_canvas_context),
                self.execution_budget.clone(),
                Some(Rc::clone(&self.script_safepoints)),
                Some(Rc::clone(&self.script_cycle_active)),
                self.logging.clone(),
            )) as Box<dyn ScriptInstance>)
        };
        if let Some(factory) = factory {
            bindings
                .verify_render_context(factory)
                .map_err(|error| self.script_error(error))?;
        }
        let result = instantiate();
        if result.is_err() {
            context_alive.set(false);
        }
        result
    }

    /// Boot a VM with the Luau standard libraries open.
    pub fn new() -> Self {
        let lua = Lua::new();
        // Install Rive's atom resolver before any Rive globals or imported
        // bytecode can intern their method/property strings.
        unsafe {
            (*lua_callbacks(lua.current_thread().state())).useratom = Some(resolve_rive_lua_atom);
        }
        let view_model_frame_context = ScriptViewModelFrameContext::default();
        lua.set_app_data(view_model_frame_context.clone());
        let blob_assets = lua_blob::ScriptedBlobAssets::install(&lua);
        let audio_assets = lua_audio::ScriptedAudioAssets::install(&lua);
        let initialization_error = lua
            .set_memory_limit(SCRIPT_VM_MEMORY_LIMIT_BYTES)
            .err()
            .map(|error| format!("failed to configure the script VM memory ceiling: {error}"));
        let resource_limits = resource_limits::ResourceLimitTracker::default();
        let script_safepoints = Rc::new(Cell::new(0));
        let script_cycle_active = Rc::new(Cell::new(false));
        let interrupt_safepoints = Rc::clone(&script_safepoints);
        let interrupt_resource_limits = resource_limits.clone();
        lua.set_interrupt(move |_| {
            let used = interrupt_safepoints.get();
            if used >= SCRIPT_SAFEPOINTS_PER_CYCLE {
                return Err(interrupt_resource_limits.fail(ScriptResourceLimit::Safepoints));
            }
            interrupt_safepoints.set(used + 1);
            Ok(luaur_rt::VmState::Continue)
        });
        Self {
            lua,
            initialization_error,
            execution_budget: None,
            rive_globals_installed: Cell::new(false),
            renderer_bindings: RendererBindings::new(view_model_frame_context.clone()),
            view_model_frame_context,
            view_models: BTreeMap::new(),
            default_context_view_model: None,
            default_context_parent_view_models: Vec::new(),
            script_safepoints,
            script_cycle_active,
            resource_limits,
            blob_assets,
            audio_assets,
            gpu_canvas_shaders: Rc::new(RefCell::new(Vec::new())),
            logging: LoggingScriptingContext::default(),
        }
    }

    /// Boot a VM whose script console and Lua errors are routed to `sink`.
    pub fn new_with_log_sink(sink: impl Fn(ScriptingLogLevel, &[u8]) + 'static) -> Self {
        let vm = Self::new();
        vm.set_log_sink(sink);
        vm
    }

    /// Route future complete script log lines to a host-provided sink.
    ///
    /// This may be called before or after [`Self::install_rive_globals`].
    pub fn set_log_sink(&self, sink: impl Fn(ScriptingLogLevel, &[u8]) + 'static) {
        self.logging.set_sink(Rc::new(sink));
    }

    /// Stop routing script log lines to the currently configured sink.
    pub fn clear_log_sink(&self) {
        self.logging.clear_sink();
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
        let interrupt_safepoints = Rc::clone(&vm.script_safepoints);
        let interrupt_resource_limits = vm.resource_limits.clone();
        vm.lua.set_interrupt(move |_| {
            let used = interrupt_safepoints.get();
            if used >= SCRIPT_SAFEPOINTS_PER_CYCLE {
                return Err(interrupt_resource_limits.fail(ScriptResourceLimit::Safepoints));
            }
            interrupt_safepoints.set(used + 1);
            let remaining = interrupt_budget.remaining.get();
            if remaining == 0 {
                return Err(interrupt_resource_limits.fail_with_message(
                    ScriptResourceLimit::Safepoints,
                    format!(
                        "trusted Luau callback exceeded {} interrupt safepoints",
                        interrupt_budget.maximum
                    ),
                ));
            }
            interrupt_budget.remaining.set(remaining - 1);
            Ok(VmState::Continue)
        });
        vm.execution_budget = Some(budget);
        Ok(vm)
    }

    fn reset_execution_budget(&self) {
        if let Err(error) = lua_image_decode::poll_completed(&self.lua) {
            self.logging.log_error(&error);
        }
        // File/Artboard facade callbacks do not all have an explicit flow
        // operation around them. Treat each such callback as its own bounded
        // cycle so the cumulative main-VM ceiling cannot poison an otherwise
        // healthy script after enough ordinary frames. Explicit flow cycles
        // retain their aggregate command/content/safepoint budget.
        if !self.script_cycle_active.get() {
            self.resource_limits.begin_cycle();
            self.script_safepoints.set(0);
        }
        if let Some(budget) = self.execution_budget.as_ref() {
            budget.reset();
        }
    }

    /// Retain one imported ShaderAsset before protocol generators execute.
    /// Neutral decoding/indexing is attempted once and an invalid state is
    /// retained, matching the C++ file importer that ignores the decoder
    /// boolean. Backend target selection is attempted only if this exact name
    /// is requested; any lookup failure is exposed to Luau as nil.
    pub fn register_gpu_canvas_shader_asset(
        &self,
        name: &str,
        payload: &[u8],
    ) -> std::result::Result<(), ScriptError> {
        if self
            .gpu_canvas_shaders
            .borrow()
            .iter()
            .any(|entry| entry.name == name)
        {
            return Err(ScriptError::new(format!(
                "ShaderAsset name '{name}' is duplicated"
            )));
        }
        self.register_gpu_canvas_shader_asset_with_short_name(name, name, payload)
    }

    /// Retain one imported ShaderAsset under its prelinked full name while
    /// preserving the authored short name used by caller-scoped lookup.
    pub fn register_gpu_canvas_shader_asset_with_short_name(
        &self,
        name: &str,
        short_name: &str,
        payload: &[u8],
    ) -> std::result::Result<(), ScriptError> {
        let owner = Rc::new(RefCell::new(RegisteredGpuCanvasShaderAsset::new(
            name, payload,
        )));
        self.gpu_canvas_shaders
            .borrow_mut()
            .push(ImportedGpuCanvasShaderAssetEntry {
                name: name.to_owned(),
                short_name: short_name.to_owned(),
                owner,
            });
        Ok(())
    }

    /// Retain one imported BlobAsset for scoped `Context:blob` lookup.
    pub fn register_blob_asset(
        &self,
        name: &str,
        payload: &[u8],
    ) -> std::result::Result<(), ScriptError> {
        self.register_blob_asset_with_short_name(name, name, payload)
    }

    pub fn register_blob_asset_with_short_name(
        &self,
        name: &str,
        short_name: &str,
        payload: &[u8],
    ) -> std::result::Result<(), ScriptError> {
        self.blob_assets
            .register(name, short_name, payload)
            .map_err(|error| ScriptError::new(error.to_string()))
    }

    /// Retain one decoded AudioAsset for exact-name `Context:audio` lookup.
    pub fn register_audio_asset(
        &self,
        name: &str,
        source: std::sync::Arc<nuxie_runtime::AudioSource>,
    ) {
        self.audio_assets.register(name, source);
    }

    /// Attach the current file-owned AudioAsset sources. Lookups consult the
    /// owner on every call so host-loader completion after VM boot is visible.
    pub fn set_audio_asset_owners(
        &self,
        owners: std::sync::Arc<nuxie_runtime::RuntimeAudioAssetOwners>,
    ) {
        self.audio_assets.set_file_owners(owners);
    }

    /// Register one serialized AudioAsset identity for exact-name lookup.
    pub fn register_audio_asset_identity(&self, name: &str, global_id: u32) {
        self.audio_assets.register_file_asset(name, global_id);
    }

    /// Attach the file-owned decoded ImageAsset catalog without performing or
    /// scheduling any decode work.
    pub fn set_image_asset_owners(
        &self,
        owners: std::sync::Arc<nuxie_runtime::RuntimeImageAssetOwners>,
    ) {
        lua_image::set_image_asset_owners(&self.lua, owners);
    }

    /// Attach the file-owned decoded FontAsset catalog. Scripted Font values
    /// retain the resolved owner so assignment survives registry replacement.
    pub fn set_font_asset_owners(
        &self,
        owners: std::sync::Arc<nuxie_runtime::RuntimeFontAssetOwners>,
    ) {
        lua_font::set_font_asset_owners(&self.lua, owners);
    }

    /// Retain one imported ShaderAsset owner under all of its file lookup
    /// aliases. Every alias is preflighted before the registry is changed.
    pub fn register_gpu_canvas_shader_asset_aliases(
        &self,
        aliases: &[&str],
        payload: &[u8],
    ) -> std::result::Result<(), ScriptError> {
        let Some(owner_name) = aliases.first().copied() else {
            return Err(ScriptError::new(
                "ShaderAsset registration requires at least one alias",
            ));
        };
        let shaders = self.gpu_canvas_shaders.borrow();
        let mut requested_aliases = BTreeSet::new();
        for alias in aliases {
            if !requested_aliases.insert(*alias) || shaders.iter().any(|entry| entry.name == *alias)
            {
                return Err(ScriptError::new(format!(
                    "ShaderAsset name '{alias}' is duplicated"
                )));
            }
        }
        drop(shaders);

        let owner = Rc::new(RefCell::new(RegisteredGpuCanvasShaderAsset::new(
            owner_name, payload,
        )));
        let mut shaders = self.gpu_canvas_shaders.borrow_mut();
        for alias in aliases {
            shaders.push(ImportedGpuCanvasShaderAssetEntry {
                name: (*alias).to_owned(),
                short_name: (*alias).to_owned(),
                owner: Rc::clone(&owner),
            });
        }
        Ok(())
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
        parents: Vec<Option<ScriptViewModel>>,
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
        self.ensure_initialized()?;
        if self.rive_globals_installed.get() {
            return Ok(());
        }

        install_host_print(&self.lua, self.logging.clone())?;
        install_math_globals(&self.lua)?;
        listener_invocation::install_pointer_event_global(&self.lua)?;
        lua_data_value::install_data_value_global(&self.lua)?;
        buffer_ext::install_buffer_extensions(&self.lua)?;
        lua_promise::install_promise_globals(&self.lua)?;
        lua_image_decode::install(&self.lua);
        lua_audio::install_audio_global(&self.lua)?;
        view_model::install_property_binding_support(&self.lua)?;

        let late = self
            .lua
            .create_function(|_, _: MultiValue| Ok(Value::Nil))?;
        self.lua.globals().set("late", late)?;

        let cache = self.ensure_module_cache()?;
        self.install_require_global(cache)?;
        self.renderer_bindings.install(&self.lua)?;
        crate::gpu_canvas::install_gpu_canvas_globals(self)?;
        view_model::install_data_global(&self.lua, &self.view_models)?;
        resource_limits::install_protected_call_guards(&self.lua, self.resource_limits.clone())?;

        self.lua.sandbox(true)?;
        self.rive_globals_installed.set(true);
        Ok(())
    }

    #[cfg(test)]
    fn eval<R: FromLuaMulti>(&self, source: &str) -> Result<R> {
        self.ensure_initialized()?;
        self.reserve_parent_stack_headroom()?;
        self.reset_execution_budget();
        let result = self.lua.load(source).eval();
        self.track_resource_result(result)
    }

    #[cfg(test)]
    fn load(&self, name: &str, source: &str) -> Result<Function> {
        self.ensure_initialized()?;
        self.reserve_parent_stack_headroom()?;
        let result = self.lua.load(source).set_name(name).into_function();
        self.track_resource_result(result)
    }

    /// Load precompiled Luau *bytecode* (the payload `.riv` files carry)
    /// into an unexecuted closure — the Rust twin of C++
    /// `ScriptingVM::loadModule`'s `luau_load` call.
    ///
    /// Structural bytecode errors are rejected before reaching `luau_load`.
    /// The pinned luaur loader mirrors C++ pointer-heavy deserialization, so
    /// hostile `.riv` payloads get a safe Rust preflight before the raw VM call.
    pub fn load_bytecode(&self, chunk_name: &str, bytecode: &[u8]) -> Result<Function> {
        self.ensure_initialized()?;
        if let Err(error) = validate_luau_bytecode(bytecode) {
            return self.track_resource_result(Err(Error::runtime(format!(
                "ScriptAsset '{chunk_name}': malformed Luau bytecode: {error}"
            ))));
        }
        let name = CString::new(format!("={chunk_name}"))
            .unwrap_or_else(|_| CString::new("=script").expect("static"));
        let result = unsafe {
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
        };
        self.track_resource_result(result)
    }

    /// Load and *execute* a script/module payload, returning what the chunk
    /// returns (protocol scripts return their generator function; utility
    /// modules return a table) — the twin of C++ `ScriptingVM::loadModule` /
    /// `executeModule`. Every chunk executes with its own writable global proxy,
    /// so one ScriptAsset cannot overwrite another ScriptAsset's globals.
    pub fn run_bytecode<R: FromLuaMulti>(&self, chunk_name: &str, bytecode: &[u8]) -> Result<R> {
        let chunk = self.load_bytecode(chunk_name, bytecode)?;
        self.reset_execution_budget();
        self.execute_loaded_module(chunk_name, chunk)
    }

    /// Evaluate precompiled Luau bytecode in the VM's shared global
    /// environment while preserving the baseline execution accounting used by
    /// other VM entry points.
    ///
    /// Runtime modules should use [`Self::run_bytecode`] for isolated globals.
    /// Editor tooling uses this bytecode-only seam when interactive source
    /// evaluation must define globals for later calls.
    pub fn eval_bytecode<R: FromLuaMulti>(&self, chunk_name: &str, bytecode: &[u8]) -> Result<R> {
        self.ensure_initialized()?;
        self.reserve_parent_stack_headroom()?;
        let chunk = self.load_bytecode(chunk_name, bytecode)?;
        self.reset_execution_budget();
        let result = chunk.call(());
        self.track_resource_result(result)
    }

    /// Execute a loaded script/module with the same environment isolation as
    /// C++ `loadModule`: the chunk gets a fresh writable globals proxy whose
    /// reads fall through to the VM's sandboxed Rive globals. C++ installs this
    /// table on a temporary coroutine; setting the loaded closure environment
    /// directly preserves the same retained-closure behavior without relying
    /// on a coroutine after registration returns.
    fn execute_loaded_module<R: FromLuaMulti>(
        &self,
        display_name: &str,
        chunk: Function,
    ) -> Result<R> {
        let environment = self.lua.create_table();
        let metatable = self.lua.create_table();
        // This metatable is fresh and private, so no __newindex behavior can
        // apply. Avoid the protected-set trampoline, which needs extra stack
        // headroom that large candidate bytecode chunks do not always leave.
        metatable.raw_set("__index", self.lua.globals())?;
        metatable.set_readonly(true);
        environment.set_metatable(Some(metatable))?;
        if !chunk.set_environment(environment)? {
            return Err(Error::runtime(format!(
                "module '{display_name}' could not install its sandbox environment"
            )));
        }
        self.track_resource_result(chunk.call(()))
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

    /// Install Rive's custom `require`. Library dependencies are prelinked by
    /// the exporter, so runtime lookup uses the requested module name verbatim.
    fn install_require_global(&self, cache: Table) -> Result<()> {
        let lookup = cache.clone();
        let require = self.lua.create_function(move |_, name: String| {
            match lookup.get::<Value>(name.as_str())? {
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
        self.reserve_parent_stack_headroom()?;
        self.ensure_module_cache()
    }

    /// Re-open checked headroom on luaur's parent C frame before pushing a
    /// registry/table handle. A failed large bytecode chunk can leave that
    /// frame exactly at `ci->top` even though `pcall` restored its logical
    /// stack top. `exec_raw` calls `lua_checkstack` before its first push, so a
    /// no-op protected call safely restores the invariant for the next host
    /// operation.
    fn reserve_parent_stack_headroom(&self) -> Result<()> {
        // `exec_raw` reserves `nargs + 2` slots on the *parent* before it
        // pushes its trampoline. Dummy nil arguments are consumed by the
        // protected call and give subsequent raw table conversion enough room
        // for table, key, result, and the reference-value duplicate.
        let padding = MultiValue::from_vec(vec![Value::Nil; 8]);
        let result = unsafe { self.lua.exec_raw::<(), _>(padding, |_| {}) };
        self.track_resource_result(result)
    }

    /// The module previously registered under `name`, if any.
    pub fn registered_module(&self, name: &str) -> Result<Value> {
        self.module_cache()?.raw_get(name)
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
    ) -> std::result::Result<Value, ScriptError> {
        self.install_render_factory(factory)?;
        self.register_module(name, payload)
            .map_err(|error| self.script_error(error))
    }

    fn register_module_after_init(&self, name: &str, payload: &[u8]) -> Result<Value> {
        if let value @ (Value::Table(_) | Value::Function(_)) =
            self.module_cache()?.raw_get::<Value>(name)?
        {
            return Ok(value);
        }
        let chunk = self.load_script_asset_payload(name, payload)?;
        self.reset_execution_budget();
        let result = self.execute_loaded_module(name, chunk)?;
        self.cache_registered_module(name, result)
    }

    fn cache_registered_module(&self, name: &str, result: Value) -> Result<Value> {
        let cache = self.module_cache()?;
        match &result {
            Value::Table(_) | Value::Function(_) => {}
            other => {
                return self.track_resource_result(Err(Error::runtime(format!(
                    "module '{name}' must return a table or function, got {other:?}"
                ))));
            }
        }
        // The module cache is a private plain table with no metamethods. Raw
        // insertion is therefore the exact operation we want, and it avoids
        // luaur-rt's protected Table::set trampoline needing a fourth stack
        // slot after a large candidate module graph has filled the base frame.
        cache.raw_set(name, result.clone())?;
        Ok(result)
    }

    #[cfg(test)]
    fn register_source_module(&self, name: &str, source: &str) -> Result<Value> {
        self.install_rive_globals()?;
        if let value @ (Value::Table(_) | Value::Function(_)) =
            self.module_cache()?.raw_get::<Value>(name)?
        {
            return Ok(value);
        }
        let function = self.load(name, source)?;
        self.reset_execution_budget();
        let result: Value = self.execute_loaded_module(name, function)?;
        self.cache_registered_module(name, result)
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
        let result = function.call(args);
        self.track_resource_result(result)
    }

    /// Read a global value.
    pub fn global(&self, name: &str) -> Result<Value> {
        self.lua.globals().get(name)
    }

    /// Start a bounded unit of script work controlled by an embedding host.
    pub fn begin_script_cycle(&self) {
        self.script_cycle_active.set(true);
        self.resource_limits.begin_cycle();
        self.script_safepoints.set(0);
    }

    /// End the embedding host's bounded unit of script work.
    pub fn end_script_cycle(&self) {
        self.script_cycle_active.set(false);
    }

    /// Machine-readable resource identity retained after terminal script
    /// exhaustion and cleared only by [`Self::begin_script_cycle`].
    pub fn terminal_resource_limit(&self) -> Option<ScriptResourceLimit> {
        self.resource_limits.terminal_limit()
    }

    /// A cloneable terminal-resource side channel for an injected host module.
    pub fn resource_guard(&self) -> ScriptResourceGuard {
        ScriptResourceGuard::new(self.resource_limits.clone())
    }

    /// Register an embedding-host module for lookup through Rive `require`.
    pub fn register_host_module(&self, name: &str, module: Table) -> Result<()> {
        self.ensure_initialized()?;
        self.ensure_module_cache()?.set(name, module)
    }

    pub fn script_instance_from_table(&self, table: Table) -> LuaScriptInstance {
        LuaScriptInstance::with_renderer_bindings(
            table,
            self.renderer_bindings.clone(),
            Rc::new(RefCell::new(None)),
            Rc::new(Cell::new(false)),
            None,
            None,
            Rc::new(Cell::new(false)),
            Vec::new(),
            None,
            self.resource_limits.clone(),
            None,
            None,
            self.execution_budget.clone(),
            Some(Rc::clone(&self.script_safepoints)),
            Some(Rc::clone(&self.script_cycle_active)),
            self.logging.clone(),
        )
    }

    fn track_resource_result<T>(&self, result: Result<T>) -> Result<T> {
        if let Err(error) = &result {
            self.resource_limits.observe_vm_error(error);
            self.logging.log_error(error);
        }
        result
    }

    fn ensure_initialized(&self) -> Result<()> {
        match self.initialization_error.as_deref() {
            Some(message) => Err(Error::runtime(message)),
            None => Ok(()),
        }
    }
}

impl RuntimeScriptingVm for ScriptVm {
    fn install_rive_globals(&mut self) -> std::result::Result<(), ScriptError> {
        ScriptVm::install_rive_globals(self).map_err(|error| self.script_error(error))
    }

    fn register_module(
        &mut self,
        name: &str,
        payload: &[u8],
    ) -> std::result::Result<(), ScriptError> {
        ScriptVm::install_rive_globals(self).map_err(|error| self.script_error(error))?;
        ScriptVm::register_module(self, name, payload)
            .map(|_| ())
            .map_err(|error| self.script_error(error))
    }

    fn instantiate_script(
        &mut self,
        name: &str,
        payload: &[u8],
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<Box<dyn ScriptInstance>, ScriptError> {
        ScriptVm::install_rive_globals(self).map_err(|error| self.script_error(error))?;
        let chunk = self
            .load_script_asset_payload(name, payload)
            .map_err(|error| self.script_error(error))?;
        self.reset_execution_budget();
        let generator: Function = self
            .execute_loaded_module(name, chunk)
            .map_err(|error| self.script_error(error))?;
        let context_view_model = Rc::new(RefCell::new(self.default_context_view_model.clone()));
        let context_present = Rc::new(Cell::new(
            self.default_context_view_model.is_some()
                || !self.default_context_parent_view_models.is_empty(),
        ));
        let context_missing_requested_data = Rc::new(Cell::new(false));
        let context_alive = Rc::new(Cell::new(true));
        let context_parent_view_models = self.default_context_parent_view_models.clone();
        let (gpu_canvas, gpu_canvas_context) = ImportedGpuCanvasInstance::new(
            Rc::clone(&self.gpu_canvas_shaders),
            self.renderer_bindings.clone(),
        );
        let context = self
            .lua
            .create_userdata(ScriptedContext::new_with_lifetime(
                Rc::clone(&context_view_model),
                Rc::clone(&context_present),
                context_parent_view_models.clone(),
                Rc::clone(&context_missing_requested_data),
                Some(gpu_canvas_context.clone()),
                Rc::clone(&context_alive),
            ))
            .map_err(|error| self.script_error(error))?;
        self.reset_execution_budget();
        let instance: Table = match self
            .track_resource_result(generator.call(context.clone()))
            .map_err(|error| self.script_error(error))
        {
            Ok(instance) => instance,
            Err(error) => {
                // `ensureScriptInitialized` clears the freshly-created
                // ScriptedContext on a generator error or non-table result,
                // even when the generator captured that userdata globally
                // (`scripted_object.cpp:361-388`).
                context_alive.set(false);
                return Err(error);
            }
        };
        Ok(Box::new(LuaScriptInstance::with_renderer_bindings(
            instance,
            self.renderer_bindings.clone(),
            context_view_model,
            context_present,
            Some(context),
            Some(context_alive),
            context_missing_requested_data,
            context_parent_view_models,
            Some(generator),
            self.resource_limits.clone(),
            Some(gpu_canvas),
            Some(gpu_canvas_context),
            self.execution_budget.clone(),
            Some(Rc::clone(&self.script_safepoints)),
            Some(Rc::clone(&self.script_cycle_active)),
            self.logging.clone(),
        )))
    }

    fn advance_detached_view_models(&mut self) -> bool {
        ScriptVm::advance_detached_view_models(self)
    }
}

impl Drop for LuaScriptInstance {
    fn drop(&mut self) {
        self.dispose_script_lifetime();
    }
}

impl ScriptInstance for LuaScriptInstance {
    fn poll_async_work(&mut self) -> std::result::Result<bool, ScriptError> {
        let Some(lua) = self
            .table
            .as_ref()
            .map(Table::lua)
            .or_else(|| self.generator.as_ref().map(Function::lua))
        else {
            return Ok(false);
        };
        lua_image_decode::poll_completed(&lua).map_err(|error| self.script_error(error))
    }

    fn set_context_view_model(
        &mut self,
        view_model: Option<ScriptViewModel>,
    ) -> std::result::Result<(), ScriptError> {
        *self.context_view_model.borrow_mut() = view_model;
        // Attaching a DataContext and resolving its main ViewModel are
        // separate states. If this value is None and init asks for it, the
        // Context marks missingRequestedData and C++ rejects that lifetime
        // (`lua_scripted_context.cpp:126-145`;
        // `scripted_object.cpp:289-303`).
        self.context_view_model_is_resolved = true;
        self.context_present.set(true);
        Ok(())
    }

    fn set_context_view_model_chain(
        &mut self,
        view_model: Option<ScriptViewModel>,
        parents: Vec<Option<ScriptViewModel>>,
    ) -> std::result::Result<(), ScriptError> {
        *self.context_view_model.borrow_mut() = view_model;
        self.context_parent_view_models = parents.clone();
        if let Some(context) = self.context.as_ref() {
            let mut context = context
                .borrow_mut::<ScriptedContext>()
                .map_err(|error| self.script_error(error))?;
            context.set_parents(parents);
        }
        self.context_view_model_is_resolved = true;
        self.context_present.set(true);
        Ok(())
    }

    fn clear_unresolved_context_view_model(&mut self) -> std::result::Result<(), ScriptError> {
        *self.context_view_model.borrow_mut() = None;
        self.context_view_model_is_resolved = false;
        self.context_present.set(false);
        Ok(())
    }

    fn has_method(&self, method: ScriptMethod) -> std::result::Result<bool, ScriptError> {
        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(false);
        }
        let table = self.live_table()?;
        let value: Value = table
            .get(method.as_str())
            .map_err(|error| self.script_error(error))?;
        Ok(matches!(value, Value::Function(_)))
    }

    fn call_method(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<ScriptValue, ScriptError> {
        let value = self.call_method_value(method, args)?;
        script_value_from_lua(value).map_err(|error| self.script_error(error))
    }

    fn call_optional_method(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<ScriptOptionalMethodResult, ScriptError> {
        self.reset_execution_budget();
        let Some(table) = self.table.clone() else {
            return Ok(ScriptOptionalMethodResult::Missing);
        };
        let value: Value = table
            .get(method.as_str())
            .map_err(|error| self.script_error(error))?;
        let Value::Function(function) = value else {
            return Ok(ScriptOptionalMethodResult::Missing);
        };
        let lua = table.lua();
        let mut call_args = MultiValue::with_capacity(args.len() + 1);
        call_args.push_back(Value::Table(table));
        for arg in args {
            call_args.push_back(script_value_to_lua(&lua, arg));
        }
        let returned: Value = function
            .call(call_args)
            .map_err(|error| self.script_error(error))?;
        script_value_from_lua(returned)
            .map(ScriptOptionalMethodResult::Returned)
            .map_err(|error| self.script_error(error))
    }

    fn call_interpolator(
        &mut self,
        method: ScriptInterpolatorMethod,
        args: &[f32],
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<ScriptOptionalNumberResult, ScriptError> {
        self.reset_execution_budget();
        let Some(table) = self.table.clone() else {
            return Ok(ScriptOptionalNumberResult::Missing);
        };
        let value: Value = table
            .get(method.as_str())
            .map_err(|error| self.script_error(error))?;
        let Value::Function(function) = value else {
            return Ok(ScriptOptionalNumberResult::Missing);
        };
        let lua = table.lua();
        let mut call_args = MultiValue::with_capacity(args.len() + 1);
        call_args.push_back(Value::Table(table));
        for value in args {
            call_args.push_back(Value::Number(f64::from(*value)));
        }
        let returned: Value = function
            .call(call_args)
            .map_err(|error| self.script_error(error))?;
        let number = lua
            .coerce_number(returned)
            .map_err(|error| self.script_error(error))?
            .unwrap_or(0.0) as f32;
        Ok(ScriptOptionalNumberResult::Returned(number))
    }

    fn call_advance_truthy(
        &mut self,
        elapsed_seconds: f32,
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<bool, ScriptError> {
        self.reset_execution_budget();
        let Some(table) = self.table.clone() else {
            return Ok(false);
        };
        let value: Value = table
            .get(ScriptMethod::Advance.as_str())
            .map_err(|error| self.script_error(error))?;
        let Value::Function(function) = value else {
            return Ok(false);
        };
        let value: Value = function
            .call((table, f64::from(elapsed_seconds)))
            .map_err(|error| self.script_error(error))?;
        Ok(!matches!(value, Value::Nil | Value::Boolean(false)))
    }

    fn call_method_with_factory(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
        host: &mut dyn ScriptHost,
        factory: &mut dyn RenderFactory,
    ) -> std::result::Result<ScriptValue, ScriptError> {
        self.renderer_bindings
            .verify_render_context(factory)
            .map_err(|error| self.script_error(error))?;
        self.call_method(method, args, host)
    }

    fn call_listener_action(
        &mut self,
        method: ScriptListenerActionMethod,
        invocation: &ScriptListenerInvocation,
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<(), ScriptError> {
        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(());
        }
        let table = self.live_table()?;
        let function: Function = table
            .get(method.as_script_method().as_str())
            .map_err(|error| self.script_error(error))?;
        let lua = table.lua();
        let invocation = listener_invocation::listener_action_argument(&lua, method, invocation)
            .map_err(|error| self.script_error(error))?;
        function
            .call((table, invocation))
            .map_err(|error| self.script_error(error))
    }

    fn call_preferred_listener_action(
        &mut self,
        invocation: &ScriptListenerInvocation,
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<bool, ScriptError> {
        self.reset_execution_budget();
        let Some(table) = self.table.clone() else {
            return Ok(false);
        };
        let perform_action: Value = table
            .get(
                ScriptListenerActionMethod::PerformAction
                    .as_script_method()
                    .as_str(),
            )
            .map_err(|error| self.script_error(error))?;
        let (method, function) = match perform_action {
            Value::Function(function) => (ScriptListenerActionMethod::PerformAction, function),
            _ => {
                let perform: Value = table
                    .get(
                        ScriptListenerActionMethod::Perform
                            .as_script_method()
                            .as_str(),
                    )
                    .map_err(|error| self.script_error(error))?;
                let Value::Function(function) = perform else {
                    return Ok(false);
                };
                (ScriptListenerActionMethod::Perform, function)
            }
        };
        let lua = table.lua();
        let invocation = listener_invocation::listener_action_argument(&lua, method, invocation)
            .map_err(|error| self.script_error(error))?;
        function
            .call::<()>((table, invocation))
            .map_err(|error| self.script_error(error))?;
        Ok(true)
    }

    fn call_scripted_drawable_input(
        &mut self,
        invocation: &ScriptListenerInvocation,
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<nuxie_runtime::ScriptedDrawableInputResult, ScriptError> {
        let (method, gamepad) = match invocation {
            ScriptListenerInvocation::Keyboard { .. } => ("keyboardEvent", false),
            ScriptListenerInvocation::TextInput { .. } => ("textEvent", false),
            ScriptListenerInvocation::GamepadConnected { .. } => ("gamepadConnected", true),
            ScriptListenerInvocation::GamepadEvent { .. } => ("gamepadEvent", true),
            ScriptListenerInvocation::GamepadDisconnected { .. } => ("gamepadDisconnected", true),
            ScriptListenerInvocation::Pointer { .. }
            | ScriptListenerInvocation::Focus { .. }
            | ScriptListenerInvocation::ReportedEvent { .. }
            | ScriptListenerInvocation::ViewModelChange { .. }
            | ScriptListenerInvocation::None
            | ScriptListenerInvocation::Semantic { .. } => {
                return Ok(nuxie_runtime::ScriptedDrawableInputResult::default());
            }
        };

        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(nuxie_runtime::ScriptedDrawableInputResult::default());
        }
        let table = self.live_table()?;
        let value: Value = table
            .get(method)
            .map_err(|error| self.script_error(error))?;
        let Value::Function(function) = value else {
            // C++ treats missing and non-function direct input methods as an
            // unhandled no-op.
            return Ok(nuxie_runtime::ScriptedDrawableInputResult::default());
        };
        let lua = table.lua();
        let Some(argument) =
            listener_invocation::scripted_drawable_input_argument(&lua, invocation)
                .map_err(|error| self.script_error(error))?
        else {
            return Ok(nuxie_runtime::ScriptedDrawableInputResult::default());
        };

        if gamepad {
            // `ScriptedDrawable::gamepadDispatch` consumes protected-call
            // failures and reports the drawable as dispatched whenever the
            // selected method exists.
            return match function.call::<()>((table, argument)) {
                Ok(()) => Ok(nuxie_runtime::ScriptedDrawableInputResult {
                    invoked: true,
                    handled: true,
                }),
                Err(error) => {
                    let error = self.script_error(error);
                    if error.resource_code().is_some() {
                        Err(error)
                    } else {
                        Ok(nuxie_runtime::ScriptedDrawableInputResult {
                            invoked: true,
                            handled: true,
                        })
                    }
                }
            };
        }

        match function.call::<Value>((table, argument)) {
            Ok(Value::Boolean(handled)) => Ok(nuxie_runtime::ScriptedDrawableInputResult {
                invoked: true,
                handled,
            }),
            Ok(_) => Ok(nuxie_runtime::ScriptedDrawableInputResult {
                invoked: true,
                handled: false,
            }),
            // Keyboard/text protected-call failures are consumed and return
            // the default unhandled result. Rust's terminal resource fence
            // remains fail-closed.
            Err(error) => {
                let error = self.script_error(error);
                if error.resource_code().is_some() {
                    Err(error)
                } else {
                    Ok(nuxie_runtime::ScriptedDrawableInputResult {
                        invoked: true,
                        handled: false,
                    })
                }
            }
        }
    }

    fn call_scripted_drawable_pointer(
        &mut self,
        method: nuxie_runtime::ScriptMethod,
        pointer_id: i32,
        local_x: f32,
        local_y: f32,
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<nuxie_runtime::ScriptedDrawablePointerResult, ScriptError> {
        self.reset_execution_budget();
        let Some(table) = self.table.clone() else {
            return Ok(nuxie_runtime::ScriptedDrawablePointerResult::default());
        };
        let value: Value = table
            .get(method.as_str())
            .map_err(|error| self.script_error(error))?;
        let Value::Function(function) = value else {
            return Ok(nuxie_runtime::ScriptedDrawablePointerResult::default());
        };
        let lua = table.lua();
        let (argument, hit_result) = listener_invocation::scripted_drawable_pointer_argument(
            &lua, pointer_id, local_x, local_y,
        )
        .map_err(|error| self.script_error(error))?;

        let call_result = function.call::<()>((table, argument));
        let hit = match hit_result.get() {
            listener_invocation::ScriptedPointerHitResult::None => {
                nuxie_runtime::ScriptedDrawablePointerHit::None
            }
            listener_invocation::ScriptedPointerHitResult::Hit => {
                nuxie_runtime::ScriptedDrawablePointerHit::Hit
            }
            listener_invocation::ScriptedPointerHitResult::HitOpaque => {
                nuxie_runtime::ScriptedDrawablePointerHit::HitOpaque
            }
        };
        if let Err(error) = call_result {
            let error = self.script_error(error);
            if error.resource_code().is_some() {
                return Err(error);
            }
        }
        Ok(nuxie_runtime::ScriptedDrawablePointerResult { invoked: true, hit })
    }

    fn call_input_trigger(
        &mut self,
        name: &str,
        host: &mut dyn ScriptHost,
    ) -> std::result::Result<(), ScriptError> {
        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(());
        }
        let table = self.live_table()?;
        let value: Value = table.get(name).map_err(|error| self.script_error(error))?;
        let Value::Function(function) = value else {
            return Ok(());
        };
        let result = function
            .call::<()>(table)
            .map_err(|error| self.script_error(error));
        // Pinned `ScriptedObject::trigger` dirties after the protected call,
        // regardless of its ordinary success/failure result
        // (`scripted_object.cpp:158-176`). Missing/non-function fields still
        // return above without dirt.
        host.mark_script_update();
        result
    }

    fn call_input_trigger_core(
        &mut self,
        name: &ScriptCoreString,
        host: &mut dyn ScriptHost,
    ) -> std::result::Result<(), ScriptError> {
        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(());
        }
        let table = self.live_table()?;
        let lua = table.lua();
        let key = lua.create_string(name.as_c_str_bytes());
        let value: Value = table.get(key).map_err(|error| self.script_error(error))?;
        let Value::Function(function) = value else {
            return Ok(());
        };
        let result = function
            .call::<()>(table)
            .map_err(|error| self.script_error(error));
        host.mark_script_update();
        result
    }

    fn call_init(&mut self, _host: &mut dyn ScriptHost) -> std::result::Result<bool, ScriptError> {
        self.call_init_with_optional_factory(None)
    }

    fn call_init_with_factory(
        &mut self,
        _host: &mut dyn ScriptHost,
        factory: &mut dyn RenderFactory,
    ) -> std::result::Result<bool, ScriptError> {
        self.call_init_with_optional_factory(Some(factory))
    }

    fn user_init_pending(&mut self) -> std::result::Result<bool, ScriptError> {
        if self.user_init_done {
            return Ok(false);
        }
        if self.table.is_none() {
            // C++ has already set `m_self = 0`; the retained ScriptAsset
            // generator makes this occurrence pending for recreation at the
            // next explicit initialization boundary.
            return Ok(self.init_retry_requires_recreation);
        }
        // This is only the occurrence's lifecycle bit. Looking up `init`
        // here and again in `call_init` would observe a metatable twice,
        // unlike pinned `tryLuaUserInit` (`scripted_object.cpp:259-278`).
        Ok(true)
    }

    fn script_lifetime_valid(&self) -> bool {
        self.table.is_some()
    }

    fn invalidate_for_init_retry(&mut self) {
        self.user_init_done = false;
        self.init_retry_requires_recreation = self.generator.is_some();
        self.dispose_script_lifetime();
    }

    fn prepare_init_retry(&mut self) -> std::result::Result<(), ScriptError> {
        self.prepare_init_retry_with_optional_factory(None)
    }

    fn prepare_init_retry_with_factory(
        &mut self,
        factory: &mut dyn RenderFactory,
    ) -> std::result::Result<(), ScriptError> {
        self.prepare_init_retry_with_optional_factory(Some(factory))
    }

    fn call_path_effect_update(
        &mut self,
        source: nuxie_render_api::RawPath,
        node: nuxie_runtime::ScriptNode,
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<nuxie_render_api::RawPath, ScriptError> {
        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(source);
        }
        let table = self.live_table()?;
        renderer::call_path_effect_update(&table, source, node)
            .map_err(|error| self.script_error(error))
    }

    fn call_draw(
        &mut self,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        _host: &mut dyn ScriptHost,
    ) -> std::result::Result<(), ScriptError> {
        if self.table.is_none() {
            return Ok(());
        }
        let table = self.live_table()?;
        if let Some(gpu_canvas) = self.gpu_canvas.as_ref() {
            self.reset_execution_budget();
            gpu_canvas
                .execute_draw_canvas(&table, factory)
                .map_err(|error| self.script_error(error))?;
        }
        self.reset_execution_budget();
        self.renderer_bindings
            .call_draw(&table, factory, renderer)
            .map_err(|error| self.script_error(error))
    }

    fn call_data_converter(
        &mut self,
        method: ScriptDataConverterMethod,
        value: ScriptValue,
    ) -> std::result::Result<ScriptValue, ScriptError> {
        let fallback = value.clone();
        Ok(self
            .call_data_converter_once(method, value)?
            .unwrap_or(fallback))
    }

    fn call_data_converter_if_present(
        &mut self,
        method: ScriptDataConverterMethod,
        value: ScriptValue,
    ) -> std::result::Result<Option<ScriptValue>, ScriptError> {
        self.call_data_converter_once(method, value)
    }

    fn call_optional_data_converter(
        &mut self,
        method: ScriptDataConverterMethod,
        value: Option<ScriptValue>,
    ) -> std::result::Result<ScriptDataConverterOptionalCall, ScriptError> {
        self.call_optional_data_converter_once(method, value)
    }

    fn has_data_converter_method(
        &self,
        method: ScriptDataConverterMethod,
    ) -> std::result::Result<bool, ScriptError> {
        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(false);
        }
        let table = self.live_table()?;
        let value: Value = table
            .get(method.as_str())
            .map_err(|error| self.script_error(error))?;
        Ok(matches!(value, Value::Function(_)))
    }

    fn get_input(&self, name: &str) -> std::result::Result<ScriptValue, ScriptError> {
        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(ScriptValue::Nil);
        }
        let table = self.live_table()?;
        let value: Value = table.get(name).map_err(|error| self.script_error(error))?;
        script_value_from_lua(value).map_err(|error| self.script_error(error))
    }

    fn set_input(
        &mut self,
        name: &str,
        value: ScriptValue,
    ) -> std::result::Result<(), ScriptError> {
        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(());
        }
        let table = self.live_table()?;
        let lua = table.lua();
        table
            .set(name, script_value_to_lua(&lua, &value))
            .map_err(|error| self.script_error(error))
    }

    fn set_input_core(
        &mut self,
        name: &ScriptCoreString,
        value: ScriptValue,
    ) -> std::result::Result<(), ScriptError> {
        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(());
        }
        let table = self.live_table()?;
        let lua = table.lua();
        let key = lua.create_string(name.as_c_str_bytes());
        table
            .set(key, script_value_to_lua(&lua, &value))
            .map_err(|error| self.script_error(error))
    }

    fn set_artboard_input(
        &mut self,
        name: &str,
        artboard: Box<dyn ScriptArtboard>,
    ) -> std::result::Result<(), ScriptError> {
        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(());
        }
        let table = self.live_table()?;
        let lua = table.lua();
        let artboard = self
            .renderer_bindings
            .create_scripted_artboard(&lua, artboard)
            .map_err(|error| self.script_error(error))?;
        table
            .set(name, artboard)
            .map_err(|error| self.script_error(error))
    }

    fn set_artboard_input_core(
        &mut self,
        name: &ScriptCoreString,
        artboard: Box<dyn ScriptArtboard>,
    ) -> std::result::Result<(), ScriptError> {
        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(());
        }
        let table = self.live_table()?;
        let lua = table.lua();
        let key = lua.create_string(name.as_c_str_bytes());
        let artboard = self
            .renderer_bindings
            .create_scripted_artboard(&lua, artboard)
            .map_err(|error| self.script_error(error))?;
        table
            .set(key, artboard)
            .map_err(|error| self.script_error(error))
    }

    fn set_view_model_input(
        &mut self,
        name: &str,
        view_model: ScriptViewModel,
    ) -> std::result::Result<(), ScriptError> {
        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(());
        }
        let table = self.live_table()?;
        let lua = table.lua();
        let view_model = create_scripted_view_model(&lua, view_model)
            .map_err(|error| self.script_error(error))?;
        table
            .set(name, view_model)
            .map_err(|error| self.script_error(error))?;
        Ok(())
    }

    fn set_view_model_input_core(
        &mut self,
        name: &ScriptCoreString,
        view_model: ScriptViewModel,
    ) -> std::result::Result<(), ScriptError> {
        self.reset_execution_budget();
        if self.table.is_none() {
            return Ok(());
        }
        let table = self.live_table()?;
        let lua = table.lua();
        let key = lua.create_string(name.as_c_str_bytes());
        let view_model = create_scripted_view_model(&lua, view_model)
            .map_err(|error| self.script_error(error))?;
        table
            .set(key, view_model)
            .map_err(|error| self.script_error(error))?;
        Ok(())
    }
}

fn tracked_script_error(
    error: Error,
    resource_limits: &resource_limits::ResourceLimitTracker,
) -> ScriptError {
    resource_limits.observe_vm_error(&error);
    match resource_limits.terminal_limit() {
        Some(limit) => ScriptError::with_resource_code(error.to_string(), limit.code()),
        None => ScriptError::new(error.to_string()),
    }
}

fn script_value_to_lua(lua: &Lua, value: &ScriptValue) -> Value {
    match value {
        ScriptValue::Nil => Value::Nil,
        ScriptValue::Bool(value) => Value::Boolean(*value),
        ScriptValue::Number(value) => Value::Number(*value),
        ScriptValue::String(value) => Value::String(lua.create_string(value)),
        ScriptValue::CoreString(value) => Value::String(lua.create_string(value.as_c_str_bytes())),
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
        Value::String(value) => match value.to_str() {
            Ok(value) => ScriptValue::String(value),
            Err(_) => {
                ScriptValue::CoreString(ScriptCoreString::from_bytes(value.as_bytes().to_vec()))
            }
        },
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
    use luaur_rt::UserData;
    use luaur_vm::functions::lua_pushlstring::lua_pushlstring;
    use luaur_vm::functions::lua_tostringatom::lua_tostringatom;
    use luaur_vm::macros::lua_pop::lua_pop;
    use nuxie_render_api::{NullFactory, PersistentFactory};
    use nuxie_runtime::NoopScriptHost;

    #[derive(Debug)]
    struct TruthyUserData;

    impl UserData for TruthyUserData {}

    #[test]
    fn compile_time_atom_table_resolves_every_upstream_name_and_exact_id() {
        assert!(RIVE_LUA_ATOMS.len() < RIVE_LUA_ATOM_SLOT_COUNT);
        for &(name, atom) in RIVE_LUA_ATOMS {
            assert_eq!(
                find_rive_lua_atom(name),
                atom,
                "{}",
                String::from_utf8_lossy(name)
            );
        }
        assert_eq!(find_rive_lua_atom(b"unknownRiveAtom"), -1);
        assert_eq!(find_rive_lua_atom(b"length\0suffix"), -1);
        assert_eq!(
            find_rive_lua_atom(&vec![b'x'; RIVE_LUA_MAX_ATOM_NAME_LENGTH + 1]),
            -1
        );
    }

    #[test]
    fn script_vm_installs_the_compile_time_atom_resolver() {
        let vm = ScriptVm::new();
        let state = vm.lua.current_thread().state();
        let name = b"invertAffine";
        let mut atom = -1;
        unsafe {
            lua_pushlstring(state, name.as_ptr().cast(), name.len());
            assert!(!lua_tostringatom(state, -1, &mut atom).is_null());
            lua_pop(state, 1);
        }
        assert_eq!(atom, 230);
    }

    #[test]
    fn converter_advance_uses_native_lua_truthiness_for_every_value_kind() {
        let lua = Lua::new();
        let table = lua.create_table();
        let mut instance = LuaScriptInstance::new(table.clone());
        let mut host = NoopScriptHost;

        for (label, source) in [
            ("table", "return function() return {} end"),
            ("function", "return function() return function() end end"),
            (
                "thread",
                "return function() return coroutine.create(function() end) end",
            ),
        ] {
            let advance: Function = lua.load(source).eval().expect(label);
            table.set("advance", advance).expect(label);
            assert!(
                instance.call_advance_truthy(0.25, &mut host).expect(label),
                "{label} is truthy in Luau"
            );
        }

        let userdata = lua
            .create_userdata(TruthyUserData)
            .expect("truthy userdata");
        let advance = lua
            .create_function(move |_, (_self, _seconds): (Table, f64)| Ok(userdata.clone()))
            .expect("userdata-returning advance");
        table.set("advance", advance).expect("userdata advance");
        assert!(
            instance
                .call_advance_truthy(0.25, &mut host)
                .expect("userdata"),
            "userdata is truthy in Luau"
        );

        for (label, source) in [
            ("nil", "return function() return nil end"),
            ("false", "return function() return false end"),
        ] {
            let advance: Function = lua.load(source).eval().expect(label);
            table.set("advance", advance).expect(label);
            assert!(
                !instance.call_advance_truthy(0.25, &mut host).expect(label),
                "{label} is the false side of Lua truthiness"
            );
        }
    }

    #[test]
    fn optional_callbacks_lookup_each_lua_field_once_before_invocation() {
        let lua = Lua::new();
        let (table, lookups): (Table, Table) = lua
            .load(
                r#"
                    local lookups = {}
                    local instance = {}
                    setmetatable(instance, {
                        __index = function(_self, key)
                            lookups[key] = (lookups[key] or 0) + 1
                            if key == "init" then
                                return function(_self, _context) return true end
                            elseif key == "convert" or key == "reverseConvert" then
                                return function(_self, value) return value end
                            elseif key == "advance" then
                                return function(_self, _seconds) return true end
                            elseif key == "performAction" then
                                return function(_self, _invocation) end
                            elseif key == "gamepadConnected" then
                                return function(_self, _event) end
                            end
                            return nil
                        end,
                    })
                    return instance, lookups
                "#,
            )
            .eval()
            .expect("build one-lookup scripted object");
        let mut instance = LuaScriptInstance::new(table);
        let mut host = NoopScriptHost;

        assert!(instance.user_init_pending().expect("init lifecycle bit"));
        assert_eq!(lookups.get::<i64>("init").unwrap_or(0), 0);
        assert!(instance.call_init(&mut host).expect("single init lookup"));
        assert_eq!(
            instance
                .call_data_converter_if_present(
                    ScriptDataConverterMethod::Convert,
                    ScriptValue::Number(3.0),
                )
                .expect("single convert lookup"),
            Some(ScriptValue::Number(3.0))
        );
        assert_eq!(
            instance
                .call_data_converter_if_present(
                    ScriptDataConverterMethod::ReverseConvert,
                    ScriptValue::Number(4.0),
                )
                .expect("single reverse lookup"),
            Some(ScriptValue::Number(4.0))
        );
        assert!(
            instance
                .call_advance_truthy(0.25, &mut host)
                .expect("single advance lookup")
        );
        assert!(
            instance
                .call_preferred_listener_action(&ScriptListenerInvocation::None, &mut host)
                .expect("single performAction lookup")
        );
        assert_eq!(
            instance
                .call_scripted_drawable_input(
                    &ScriptListenerInvocation::GamepadConnected {
                        snapshot: nuxie_runtime::ScriptGamepadSnapshot {
                            device_id: 7,
                            button_mask: 0,
                            button_values: Vec::new(),
                            axes: Vec::new(),
                            mapping: nuxie_runtime::ScriptGamepadMappingKind::Standard,
                        },
                    },
                    &mut host,
                )
                .expect("single gamepad lookup"),
            nuxie_runtime::ScriptedDrawableInputResult {
                invoked: true,
                handled: true,
            }
        );

        for key in [
            "init",
            "convert",
            "reverseConvert",
            "advance",
            "performAction",
            "gamepadConnected",
        ] {
            assert_eq!(
                lookups.get::<i64>(key).unwrap_or(0),
                1,
                "pinned C++ resolves {key} once and invokes that exact value"
            );
        }

        let (fallback, fallback_lookups): (Table, Table) = lua
            .load(
                r#"
                    local lookups = {}
                    local instance = {}
                    setmetatable(instance, {
                        __index = function(_self, key)
                            lookups[key] = (lookups[key] or 0) + 1
                            if key == "performAction" then
                                return 17
                            elseif key == "perform" then
                                return function(_self, _pointer) end
                            end
                            return nil
                        end,
                    })
                    return instance, lookups
                "#,
            )
            .eval()
            .expect("build one-lookup legacy listener");
        let mut fallback = LuaScriptInstance::new(fallback);
        assert!(
            fallback
                .call_preferred_listener_action(&ScriptListenerInvocation::None, &mut host)
                .expect("single legacy fallback lookup")
        );
        assert_eq!(fallback_lookups.get::<i64>("performAction").unwrap_or(0), 1);
        assert_eq!(fallback_lookups.get::<i64>("perform").unwrap_or(0), 1);
    }

    #[test]
    fn optional_converter_lookup_and_call_are_one_atomic_operation() {
        let lua = Lua::new();
        let (table, lookups, calls): (Table, Table, Table) = lua
            .load(
                r#"
                    local lookups = {}
                    local calls = {}
                    local instance = {}
                    setmetatable(instance, {
                        __index = function(_self, key)
                            lookups[key] = (lookups[key] or 0) + 1
                            if lookups[key] ~= 1 then
                                return nil
                            end
                            if key == "convert" or key == "reverseConvert" then
                                return function(_self, value)
                                    calls[key] = (calls[key] or 0) + 1
                                    return value
                                end
                            end
                            return nil
                        end,
                    })
                    return instance, lookups, calls
                "#,
            )
            .eval()
            .expect("build alternating converter lookup");
        let mut instance = LuaScriptInstance::new(table);

        for method in [
            ScriptDataConverterMethod::Convert,
            ScriptDataConverterMethod::ReverseConvert,
        ] {
            assert_eq!(
                instance
                    .call_optional_data_converter(method, Some(ScriptValue::Number(3.0)))
                    .expect("atomic optional converter call"),
                ScriptDataConverterOptionalCall::Returned(ScriptValue::Number(3.0))
            );
            assert_eq!(lookups.get::<i64>(method.as_str()).unwrap_or(0), 1);
            assert_eq!(calls.get::<i64>(method.as_str()).unwrap_or(0), 1);
        }

        lookups.set("convert", 0).expect("reset lookup count");
        calls.set("convert", 0).expect("reset call count");
        assert_eq!(
            instance
                .call_optional_data_converter(ScriptDataConverterMethod::Convert, None)
                .expect("unsupported input still resolves once"),
            ScriptDataConverterOptionalCall::UnsupportedInput
        );
        assert_eq!(lookups.get::<i64>("convert").unwrap_or(0), 1);
        assert_eq!(
            calls.get::<i64>("convert").unwrap_or(0),
            0,
            "C++ resolves the function but does not call it when pushDataValue fails"
        );
    }

    #[test]
    fn missing_or_non_function_converter_advance_is_not_callable() {
        for (label, non_function) in [("missing", None), ("number", Some(17_i64))] {
            let lua = Lua::new();
            let table = lua.create_table();
            if let Some(non_function) = non_function {
                table.set("advance", non_function).expect(label);
            }
            let instance = LuaScriptInstance::new(table);
            assert!(
                !instance.has_method(ScriptMethod::Advance).expect(label),
                "{label} advance must stay inert when the authored method bit is enabled"
            );
        }
    }

    #[test]
    fn missing_or_non_function_init_latches_complete_without_blocking_dispatch() {
        for (label, initial_init) in [("missing", None), ("non-function", Some(17_i64))] {
            let lua = Lua::new();
            lua.globals().set("lateInitCalls", 0).expect(label);
            lua.globals().set("actionCalls", 0).expect(label);
            lua.globals().set("converterCalls", 0).expect(label);
            let table = lua.create_table();
            if let Some(initial_init) = initial_init {
                table.set("init", initial_init).expect(label);
            }
            let perform: Function = lua
                .load(
                    "return function(_self, _event)
                        actionCalls += 1
                    end",
                )
                .eval()
                .expect(label);
            table.set("performAction", perform).expect(label);
            let convert: Function = lua
                .load(
                    "return function(_self, value)
                        converterCalls += 1
                        return value
                    end",
                )
                .eval()
                .expect(label);
            table.set("convert", convert).expect(label);
            let mut instance = LuaScriptInstance::new(table.clone());
            let mut host = NoopScriptHost;

            assert!(
                instance.user_init_pending().expect(label),
                "{label} init remains a lifecycle operation until its one field lookup"
            );
            assert!(
                instance.call_init(&mut host).expect(label),
                "{label} init completes successfully without invoking a callback"
            );
            assert!(
                !instance.user_init_pending().expect(label),
                "{label} init completion is latched exactly once"
            );

            let late_init: Function = lua
                .load(
                    "return function(_self)
                        lateInitCalls += 1
                        return true
                    end",
                )
                .eval()
                .expect(label);
            table.set("init", late_init).expect(label);
            assert!(
                !instance.user_init_pending().expect(label),
                "{label} init stays latched after the table is mutated"
            );

            instance
                .call_listener_action(
                    ScriptListenerActionMethod::PerformAction,
                    &ScriptListenerInvocation::None,
                    &mut host,
                )
                .expect(label);
            assert_eq!(
                instance
                    .call_data_converter(
                        ScriptDataConverterMethod::Convert,
                        ScriptValue::Number(3.5),
                    )
                    .expect(label),
                ScriptValue::Number(3.5)
            );
            assert_eq!(lua.globals().get::<i64>("lateInitCalls").unwrap(), 0);
            assert_eq!(lua.globals().get::<i64>("actionCalls").unwrap(), 1);
            assert_eq!(lua.globals().get::<i64>("converterCalls").unwrap(), 1);
        }
    }

    #[test]
    fn attached_empty_context_rejects_only_when_init_requests_missing_data() {
        for (label, init_source, expected) in [
            (
                "does-not-request",
                "return function(_self, _context) return true end",
                true,
            ),
            (
                "requests-view-model",
                "return function(_self, context)
                    context:viewModel()
                    return true
                end",
                false,
            ),
            (
                "requests-only-data-context",
                "return function(_self, context)
                    local dataContext = context:dataContext()
                    return dataContext ~= nil and dataContext:viewModel() == nil
                end",
                true,
            ),
        ] {
            let lua = Lua::new();
            let frame_context = ScriptViewModelFrameContext::default();
            lua.set_app_data(frame_context.clone());
            let table = lua.create_table();
            table
                .set(
                    "init",
                    lua.load(init_source).eval::<Function>().expect(label),
                )
                .expect(label);
            let context_view_model = Rc::new(RefCell::new(None));
            let context_present = Rc::new(Cell::new(false));
            let missing_requested_data = Rc::new(Cell::new(false));
            let context_alive = Rc::new(Cell::new(true));
            let context = lua
                .create_userdata(ScriptedContext::new_with_lifetime(
                    Rc::clone(&context_view_model),
                    Rc::clone(&context_present),
                    Vec::new(),
                    Rc::clone(&missing_requested_data),
                    None,
                    Rc::clone(&context_alive),
                ))
                .expect(label);
            let bindings = RendererBindings::new(frame_context);
            let mut factory = PersistentFactory::new(NullFactory::new());
            bindings
                .bootstrap_render_context(&mut factory)
                .expect("pre-import render context");
            let mut instance = LuaScriptInstance::with_renderer_bindings(
                table,
                bindings,
                context_view_model,
                context_present,
                Some(context),
                Some(context_alive),
                missing_requested_data,
                Vec::new(),
                None,
                resource_limits::ResourceLimitTracker::default(),
                None,
                None,
                None,
                None,
                None,
                LoggingScriptingContext::default(),
            );
            instance
                .set_context_view_model(None)
                .expect("attach explicit empty context");

            assert_eq!(
                instance
                    .call_init_with_factory(&mut NoopScriptHost, &mut factory)
                    .expect(label),
                expected,
                "{label}"
            );
            assert_eq!(instance.script_lifetime_valid(), expected, "{label}");
        }
    }

    #[test]
    fn generator_preserves_a_nil_main_context_with_a_parent_slot() {
        let vm = ScriptVm::new();
        let generator: Function = vm
            .lua
            .load(
                r#"
                return function(context)
                    generatorSawDataContext = context:dataContext() ~= nil
                    return {}
                end
                "#,
            )
            .eval()
            .expect("script generator");
        let program = ScriptProgram { generator };

        let instance = vm
            .instantiate_registered_script_with_context(&program, None, vec![None])
            .expect("nil-main DataContext remains present during generation");

        assert!(
            vm.lua
                .globals()
                .get::<bool>("generatorSawDataContext")
                .expect("generator observation")
        );
        drop(instance);
    }

    #[test]
    fn failed_init_replaces_and_poison_disposes_each_captured_context() {
        let lua = Lua::new();
        let frame_context = ScriptViewModelFrameContext::default();
        lua.set_app_data(frame_context.clone());
        lua.globals()
            .set("savedContexts", lua.create_table())
            .expect("saved contexts");
        lua.globals().set("generation", 0).expect("generation");
        let generator: Function = lua
            .load(
                r#"
                return function(context)
                    generation += 1
                    savedContexts[generation] = context
                    local thisGeneration = generation
                    return {
                        init = function()
                            return thisGeneration > 1
                        end,
                    }
                end
                "#,
            )
            .eval()
            .expect("script generator");
        let context_view_model = Rc::new(RefCell::new(None));
        let context_present = Rc::new(Cell::new(false));
        let missing_requested_data = Rc::new(Cell::new(false));
        let first_context_alive = Rc::new(Cell::new(true));
        let first_context = lua
            .create_userdata(ScriptedContext::new_with_lifetime(
                Rc::clone(&context_view_model),
                Rc::clone(&context_present),
                Vec::new(),
                Rc::clone(&missing_requested_data),
                None,
                Rc::clone(&first_context_alive),
            ))
            .expect("first context");
        let first_table: Table = generator
            .call(first_context.clone())
            .expect("first script table");
        let bindings = RendererBindings::new(frame_context);
        let mut factory = PersistentFactory::new(NullFactory::new());
        bindings
            .bootstrap_render_context(&mut factory)
            .expect("pre-import render context");
        let mut instance = LuaScriptInstance::with_renderer_bindings(
            first_table,
            bindings,
            context_view_model,
            context_present,
            Some(first_context),
            Some(first_context_alive),
            missing_requested_data,
            Vec::new(),
            Some(generator),
            resource_limits::ResourceLimitTracker::default(),
            None,
            None,
            None,
            None,
            None,
            LoggingScriptingContext::default(),
        );
        assert!(
            !instance
                .call_init_with_factory(&mut NoopScriptHost, &mut factory)
                .expect("first init returns false")
        );
        let first_context_live: bool = lua
            .load(
                "local ok = pcall(function() savedContexts[1]:viewModel() end)
                 return ok",
            )
            .eval()
            .expect("probe first context");
        assert!(!first_context_live, "failed init poisons its Context");

        instance
            .prepare_init_retry_with_factory(&mut factory)
            .expect("recreate script lifetime");
        assert!(
            instance
                .call_init_with_factory(&mut NoopScriptHost, &mut factory)
                .expect("second init succeeds")
        );
        let second_context_live: bool = lua
            .load(
                "local ok = pcall(function() savedContexts[2]:viewModel() end)
                 return ok",
            )
            .eval()
            .expect("probe second context");
        assert!(second_context_live, "replacement Context is live");

        drop(instance);
        let second_context_live_after_drop: bool = lua
            .load(
                "local ok = pcall(function() savedContexts[2]:viewModel() end)
                 return ok",
            )
            .eval()
            .expect("probe dropped context");
        assert!(
            !second_context_live_after_drop,
            "dropping the occurrence poisons its captured Context"
        );
    }

    #[test]
    fn failed_init_immediately_releases_the_lua_table_before_retry() {
        let lua = Lua::new();
        let frame_context = ScriptViewModelFrameContext::default();
        lua.set_app_data(frame_context.clone());
        lua.globals().set("generation", 0).expect("generation");
        lua.globals()
            .set(
                "weakTables",
                lua.load("return setmetatable({}, { __mode = 'v' })")
                    .eval::<Table>()
                    .expect("weak-value table"),
            )
            .expect("weak table global");
        let generator: Function = lua
            .load(
                r#"
                return function(_context)
                    generation += 1
                    local thisGeneration = generation
                    local occurrence = {
                        generation = thisGeneration,
                        init = function()
                            return thisGeneration > 1
                        end,
                    }
                    weakTables[thisGeneration] = occurrence
                    return occurrence
                end
                "#,
            )
            .eval()
            .expect("script generator");
        let context_view_model = Rc::new(RefCell::new(None));
        let context_present = Rc::new(Cell::new(false));
        let missing_requested_data = Rc::new(Cell::new(false));
        let first_context_alive = Rc::new(Cell::new(true));
        let first_context = lua
            .create_userdata(ScriptedContext::new_with_lifetime(
                Rc::clone(&context_view_model),
                Rc::clone(&context_present),
                Vec::new(),
                Rc::clone(&missing_requested_data),
                None,
                Rc::clone(&first_context_alive),
            ))
            .expect("first context");
        let first_table: Table = generator
            .call(first_context.clone())
            .expect("first script table");
        let bindings = RendererBindings::new(frame_context);
        let mut factory = PersistentFactory::new(NullFactory::new());
        bindings
            .bootstrap_render_context(&mut factory)
            .expect("pre-import render context");
        let mut instance = LuaScriptInstance::with_renderer_bindings(
            first_table,
            bindings,
            context_view_model,
            context_present,
            Some(first_context),
            Some(first_context_alive),
            missing_requested_data,
            Vec::new(),
            Some(generator),
            resource_limits::ResourceLimitTracker::default(),
            None,
            None,
            None,
            None,
            None,
            LoggingScriptingContext::default(),
        );
        assert!(
            !instance
                .call_init_with_factory(&mut NoopScriptHost, &mut factory)
                .expect("first init returns false")
        );
        assert!(
            instance.table.is_none(),
            "failed init is the Rust representation of C++ m_self == 0"
        );
        lua.gc_collect().expect("collect failed lifetime");
        assert!(
            lua.load("return weakTables[1] == nil")
                .eval::<bool>()
                .expect("probe failed table"),
            "the failed table must be collectible before any retry boundary"
        );
        let mut host = NoopScriptHost;
        assert!(
            !instance
                .has_method(ScriptMethod::Advance)
                .expect("dead method lookup")
        );
        assert_eq!(
            instance
                .call_method(ScriptMethod::Update, &[], &mut host)
                .expect("dead generic callback"),
            ScriptValue::Nil
        );
        assert!(
            !instance
                .call_advance_truthy(0.25, &mut host)
                .expect("dead advance")
        );
        instance
            .call_listener_action(
                ScriptListenerActionMethod::PerformAction,
                &ScriptListenerInvocation::None,
                &mut host,
            )
            .expect("dead listener callback");
        assert_eq!(
            instance
                .call_scripted_drawable_input(
                    &ScriptListenerInvocation::Keyboard {
                        key: 65,
                        modifiers: 0,
                        is_pressed: true,
                        is_repeat: false,
                    },
                    &mut host,
                )
                .expect("dead direct input"),
            nuxie_runtime::ScriptedDrawableInputResult::default(),
            "a dead C++ state() is not an invoked keyboard/text target"
        );
        assert_eq!(
            instance
                .call_data_converter(ScriptDataConverterMethod::Convert, ScriptValue::Number(7.5),)
                .expect("dead converter"),
            ScriptValue::Number(7.5)
        );
        instance
            .set_input("ignored", ScriptValue::Number(1.0))
            .expect("dead scalar setter");
        instance
            .call_input_trigger("ignored", &mut host)
            .expect("dead trigger");

        instance
            .prepare_init_retry_with_factory(&mut factory)
            .expect("recreate script lifetime");
        assert_eq!(instance.table().get::<i64>("generation").unwrap(), 2);
        lua.gc_collect().expect("collect with live replacement");
        assert!(
            lua.load("return weakTables[2] ~= nil")
                .eval::<bool>()
                .expect("probe live replacement"),
            "the recreated occurrence owns its replacement table"
        );
        assert!(
            instance
                .call_init_with_factory(&mut NoopScriptHost, &mut factory)
                .expect("second init succeeds")
        );

        drop(instance);
        lua.gc_collect().expect("collect dropped replacement");
        assert!(
            lua.load("return weakTables[2] == nil")
                .eval::<bool>()
                .expect("probe dropped replacement"),
            "final occurrence drop releases the replacement table exactly once"
        );
    }

    #[test]
    fn generator_failure_or_non_table_result_poisons_captured_context() {
        for (label, generator_body) in [
            ("error", "error('expected generator failure')"),
            ("non-table", "return 17"),
        ] {
            let vm = ScriptVm::new();
            vm.lua
                .globals()
                .set("savedContext", Value::Nil)
                .expect(label);
            let source = format!(
                "return function(context)
                    savedContext = context
                    {generator_body}
                end"
            );
            let generator: Function = vm.lua.load(&source).eval().expect(label);
            let program = ScriptProgram { generator };
            let error = match vm.instantiate_registered_script_with_factory_and_context(
                &program,
                &mut NoopScriptHost,
                &mut PersistentFactory::new(NullFactory::new()),
                None,
                Vec::new(),
            ) {
                Ok(_) => panic!("{label} unexpectedly produced a script instance"),
                Err(error) => error,
            };
            assert!(!error.message().is_empty());
            let context_live: bool = vm
                .lua
                .load(
                    "local ok = pcall(function() savedContext:viewModel() end)
                     return ok",
                )
                .eval()
                .expect(label);
            assert!(!context_live, "{label} must poison its captured Context");
        }
    }

    #[test]
    fn authored_core_strings_keep_bytes_until_the_lua_c_string_boundary() {
        let lua = Lua::new();
        let table = lua.create_table();
        let probe = table.clone();
        let mut instance = LuaScriptInstance::new(table);
        let name = ScriptCoreString::from_bytes(vec![0xff, b'k', 0, b't']);
        let value = ScriptCoreString::from_bytes(vec![0xfe, b'v', 0, b'x']);

        instance
            .set_input_core(&name, ScriptValue::CoreString(value.clone()))
            .expect("set raw authored input");
        let stored: Value = probe
            .get(lua.create_string(&[0xff, b'k']))
            .expect("read raw Lua key");
        let Value::String(stored) = stored else {
            panic!("raw authored input was not a Lua string");
        };
        assert_eq!(stored.as_bytes(), &[0xfe, b'v']);
        assert_eq!(
            value.as_bytes(),
            &[0xfe, b'v', 0, b'x'],
            "the owned CoreString keeps its suffix after Lua projection"
        );

        let fired = Rc::new(Cell::new(false));
        let fired_from_lua = Rc::clone(&fired);
        probe
            .set(
                lua.create_string(&[0xff, b'k']),
                lua.create_function(move |_, _self: Table| {
                    fired_from_lua.set(true);
                    Ok(())
                })
                .expect("trigger callback"),
            )
            .expect("install raw-name trigger");
        instance
            .call_input_trigger_core(&name, &mut NoopScriptHost)
            .expect("call raw-name trigger");
        assert!(fired.get());
    }

    #[test]
    fn throwing_trigger_callbacks_still_mark_the_scripted_owner_dirty() {
        #[derive(Default)]
        struct RecordingHost {
            updates: usize,
        }

        impl ScriptHost for RecordingHost {
            fn mark_script_update(&mut self) {
                self.updates += 1;
            }
        }

        let lua = Lua::new();
        let table = lua.create_table();
        table
            .set(
                "pulse",
                lua.load("return function() error('expected trigger failure') end")
                    .eval::<Function>()
                    .expect("throwing trigger"),
            )
            .expect("install throwing trigger");
        let mut instance = LuaScriptInstance::new(table);
        let mut host = RecordingHost::default();

        let error = instance
            .call_input_trigger("pulse", &mut host)
            .expect_err("throwing trigger must report its protected-call failure");
        assert!(error.resource_code().is_none());
        assert_eq!(
            host.updates, 1,
            "C++ addScriptedDirt runs after invoking a present trigger function even when it throws"
        );

        instance
            .call_input_trigger("missing", &mut host)
            .expect("missing trigger is inert");
        assert_eq!(
            host.updates, 1,
            "missing/non-function trigger fields do not dirty"
        );
    }

    #[test]
    fn scripted_data_value_preserves_unchanged_core_string_but_assignment_truncates() {
        let lua = Lua::new();
        let table = lua.create_table();
        let unchanged: Function = lua
            .load("return function(_self, value) return value end")
            .eval()
            .expect("unchanged converter");
        table.set("convert", unchanged).expect("converter method");
        let mut instance = LuaScriptInstance::new(table.clone());
        let full = ScriptValue::CoreString(ScriptCoreString::from_bytes(vec![0xfe, b'A', 0, b'B']));

        assert_eq!(
            instance
                .call_data_converter(ScriptDataConverterMethod::Convert, full.clone())
                .expect("unchanged conversion"),
            full,
            "returning the wrapper unchanged retains its full DataValueString"
        );

        let assigned: Function = lua
            .load(
                "return function(_self, value)
                    value.value = value.value
                    return value
                end",
            )
            .eval()
            .expect("assigning converter");
        table.set("convert", assigned).expect("replace converter");
        assert_eq!(
            instance
                .call_data_converter(
                    ScriptDataConverterMethod::Convert,
                    ScriptValue::CoreString(ScriptCoreString::from_bytes(vec![
                        0xfe, b'A', 0, b'B',
                    ])),
                )
                .expect("assigning conversion"),
            ScriptValue::CoreString(ScriptCoreString::from_bytes(vec![0xfe, b'A'])),
            "luaL_checkstring followed by std::string(const char*) truncates at the first NUL"
        );
    }

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
    fn registered_protocol_chunks_keep_their_writable_globals_isolated() {
        let vm = ScriptVm::new();
        vm.install_rive_globals().expect("Rive globals");
        let first_chunk = vm
            .load(
                "first",
                r#"
                CollisionValue = "first"
                return function()
                    return { value = CollisionValue }
                end
                "#,
            )
            .expect("first chunk");
        let second_chunk = vm
            .load(
                "second",
                r#"
                CollisionValue = "second"
                return function()
                    return { value = CollisionValue }
                end
                "#,
            )
            .expect("second chunk");

        let first: Function = vm
            .execute_loaded_module("first", first_chunk)
            .expect("register first protocol");
        let second: Function = vm
            .execute_loaded_module("second", second_chunk)
            .expect("register second protocol");
        let first_instance: Table = first.call(()).expect("instantiate first protocol");
        let second_instance: Table = second.call(()).expect("instantiate second protocol");

        assert_eq!(first_instance.get::<String>("value").unwrap(), "first");
        assert_eq!(second_instance.get::<String>("value").unwrap(), "second");
        assert!(matches!(vm.global("CollisionValue").unwrap(), Value::Nil));
    }

    #[test]
    fn registered_utility_modules_keep_their_writable_globals_isolated() {
        let vm = ScriptVm::new();
        let first = vm
            .register_source_module(
                "first",
                r#"
                CollisionValue = "first"
                return { read = function() return CollisionValue end }
                "#,
            )
            .expect("register first module");
        let second = vm
            .register_source_module(
                "second",
                r#"
                CollisionValue = "second"
                return { read = function() return CollisionValue end }
                "#,
            )
            .expect("register second module");
        let Value::Table(first) = first else {
            panic!("first module did not return a table");
        };
        let Value::Table(second) = second else {
            panic!("second module did not return a table");
        };
        let first_read: Function = first.get("read").unwrap();
        let second_read: Function = second.get("read").unwrap();

        assert_eq!(first_read.call::<String>(()).unwrap(), "first");
        assert_eq!(second_read.call::<String>(()).unwrap(), "second");
    }

    #[test]
    fn trusted_callbacks_do_not_accumulate_safepoints_without_an_explicit_host_cycle() {
        let vm = ScriptVm::new_with_execution_limits(
            ScriptExecutionLimits::new().with_max_interrupts_per_callback(80_000),
        )
        .expect("trusted limits");
        vm.eval::<()>(
            r#"
            function boundedWork()
                local total = 0
                for index = 1, 70000 do
                    total += index
                end
                return total
            end
            "#,
        )
        .expect("bounded callback installs");

        for _ in 0..3 {
            vm.call_global::<f64>("boundedWork", ())
                .expect("each ordinary callback gets a fresh cycle budget");
        }
        assert_eq!(vm.terminal_resource_limit(), None);
    }

    #[test]
    fn trusted_callbacks_share_the_main_safepoint_ceiling_inside_an_explicit_host_cycle() {
        let vm = ScriptVm::new_with_execution_limits(
            ScriptExecutionLimits::new().with_max_interrupts_per_callback(80_000),
        )
        .expect("trusted limits");
        vm.eval::<()>(
            r#"
            function boundedCycleWork()
                local total = 0
                for index = 1, 70000 do
                    total += index
                end
                return total
            end
            "#,
        )
        .expect("bounded callback installs");

        vm.begin_script_cycle();
        vm.call_global::<f64>("boundedCycleWork", ())
            .expect("first callback stays within both ceilings");
        let error = vm
            .call_global::<f64>("boundedCycleWork", ())
            .expect_err("the explicit cycle keeps the cumulative main ceiling");

        assert!(error.to_string().contains("100000 script safepoints"));
        assert_eq!(
            vm.terminal_resource_limit(),
            Some(ScriptResourceLimit::Safepoints)
        );
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
        let context_alive = Rc::new(Cell::new(true));
        let context_view_model = Rc::new(RefCell::new(None));
        let context_present = Rc::new(Cell::new(false));
        let context = lua
            .create_userdata(ScriptedContext::new_with_lifetime(
                Rc::clone(&context_view_model),
                Rc::clone(&context_present),
                Vec::new(),
                Rc::clone(&missing_requested_data),
                None,
                Rc::clone(&context_alive),
            ))
            .expect("scripted context");
        let table: Table = generator.call(context.clone()).expect("script table");
        let bindings = RendererBindings::new(frame_context);
        let mut factory = PersistentFactory::new(NullFactory::new());
        bindings
            .bootstrap_render_context(&mut factory)
            .expect("pre-import render context");
        let mut instance = LuaScriptInstance::with_renderer_bindings(
            table,
            bindings,
            context_view_model,
            context_present,
            Some(context),
            Some(context_alive),
            missing_requested_data,
            Vec::new(),
            Some(generator),
            resource_limits::ResourceLimitTracker::default(),
            None,
            None,
            None,
            None,
            None,
            LoggingScriptingContext::default(),
        );
        let mut host = NoopScriptHost;

        instance.context_missing_requested_data.set(true);
        instance
            .clear_unresolved_context_view_model()
            .expect("clear unresolved context");
        assert!(!instance.context_view_model_is_resolved);
        assert!(
            instance.context_missing_requested_data.get(),
            "constructor-time context misses remain sticky until cold init"
        );
        assert!(
            !instance
                .call_init_with_factory(&mut host, &mut factory)
                .expect("cold init")
        );
        assert!(instance.user_init_pending().expect("pending cold init"));
        instance
            .prepare_init_retry_with_factory(&mut factory)
            .expect("recreate script lifetime");
        assert_eq!(instance.table().get::<i64>("generation").unwrap(), 2);
        assert!(
            instance
                .call_init_with_factory(&mut host, &mut factory)
                .expect("bound retry")
        );
        assert!(!instance.user_init_pending().expect("completed retry"));
    }
}

#[cfg(test)]
mod gpu_canvas_tests {
    use super::*;

    fn put_u16(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn malformed_shader_container_registration_retains_invalid_state() {
        let vm = ScriptVm::new();
        let mut payload = vec![0];
        put_u32(&mut payload, 0x5253_5442);
        put_u16(&mut payload, 4);
        payload.extend_from_slice(&[1, 0, 0]);
        payload.extend_from_slice(&[0, 0, 0]);

        vm.register_gpu_canvas_shader_asset("broken", &payload)
            .expect("the C++ file importer ignores neutral decode failure");

        assert!(
            vm.gpu_canvas_shaders
                .borrow()
                .iter()
                .any(|entry| entry.name == "broken")
        );
    }

    #[test]
    fn multi_alias_shader_registration_shares_one_owner_and_rejects_atomically() {
        let vm = ScriptVm::new();
        vm.register_gpu_canvas_shader_asset_aliases(&["scene", "effects/scene"], &[0, 1, 2, 3])
            .expect("aliases register");

        let shaders = vm.gpu_canvas_shaders.borrow();
        let scene = shaders.iter().find(|entry| entry.name == "scene").unwrap();
        let nested = shaders
            .iter()
            .find(|entry| entry.name == "effects/scene")
            .unwrap();
        assert!(Rc::ptr_eq(&scene.owner, &nested.owner,));
        drop(shaders);

        vm.register_gpu_canvas_shader_asset_aliases(&["unused/new-alias", "scene"], &[4, 5, 6, 7])
            .expect_err("a duplicate alias rejects the whole registration");
        assert!(
            !vm.gpu_canvas_shaders
                .borrow()
                .iter()
                .any(|entry| entry.name == "unused/new-alias")
        );
    }
}
