//! Isolated Luau audio lanes. Only this private raw VM crosses threads: no
//! luaur-rt handle, host callback capture, registry TLS or scene object does.
#[cfg(target_arch = "wasm32")]
use super::luau_audio_web::Instant;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{CString, c_int, c_void};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
mod browser_kernel;
#[cfg(target_arch = "wasm32")]
pub(super) use browser_kernel::BrowserKernel;

pub(super) use super::luau_audio_shared::{
    SharedBlob, SharedRegion, SharedRing, SharedSnapshot, SharedValue,
};
use luaur_vm::enums::lua_type::lua_Type;
use luaur_vm::functions::{
    l_alloc::l_alloc, lua_callbacks::lua_callbacks, lua_close::lua_close,
    lua_createtable::lua_createtable, lua_error::lua_error, lua_gc::lua_gc,
    lua_getfield::lua_getfield, lua_gettop::lua_gettop, lua_l_openlibs::lua_l_openlibs,
    lua_l_sandbox::lua_l_sandbox, lua_newbuffer::lua_newbuffer, lua_newstate::lua_newstate,
    lua_next::lua_next, lua_pcall::lua_pcall, lua_pushboolean::lua_pushboolean,
    lua_pushcclosurek::lua_pushcclosurek, lua_pushinteger::lua_pushinteger,
    lua_pushlstring::lua_pushlstring, lua_pushnil::lua_pushnil, lua_pushnumber::lua_pushnumber,
    lua_pushvalue::lua_pushvalue, lua_rawgeti::lua_rawgeti, lua_rawset::lua_rawset,
    lua_rawseti::lua_rawseti, lua_ref::lua_ref, lua_setfield::lua_setfield,
    lua_setmetatable::lua_setmetatable, lua_settop::lua_settop, lua_toboolean::lua_toboolean,
    lua_tobuffer::lua_tobuffer, lua_tolstring::lua_tolstring, lua_tonumberx::lua_tonumberx,
    lua_topointer::lua_topointer, lua_type::lua_type, luau_load::luau_load,
};
use luaur_vm::macros::{
    lua_globalsindex::LUA_GLOBALSINDEX, lua_registryindex::LUA_REGISTRYINDEX,
    lua_upvalueindex::lua_upvalueindex,
};
use luaur_vm::records::lua_exception::{LuaResult, lua_exception};
use luaur_vm::type_aliases::lua_state::lua_State;
use nuxie_runtime::source::r#async::work_pool::{vm_completions_queued, vm_completions_taken};
use nuxie_runtime::{AudioEngine, AudioRenderer, AudioRendererNode};

/// Interleaved processor output format. Offline hosts choose all three fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessorFormat {
    pub sample_rate: u32,
    pub channels: u32,
    pub max_frames: u32,
}
impl Default for ProcessorFormat {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            channels: 2,
            max_frames: 1024,
        }
    }
}
#[derive(Clone, Copy)]
struct OfflineRender {
    on: bool,
    format: ProcessorFormat,
    frame: u64,
}
static OFFLINE_RENDER: Mutex<OfflineRender> = Mutex::new(OfflineRender {
    on: false,
    format: ProcessorFormat {
        sample_rate: 48_000,
        channels: 2,
        max_frames: 1024,
    },
    frame: 0,
});
fn offline_state() -> OfflineRender {
    *lock(&OFFLINE_RENDER)
}
/// Like upstream, configuring either enabled or disabled mode resets its clock.
pub(super) fn set_render_offline(on: bool, format: ProcessorFormat) {
    *lock(&OFFLINE_RENDER) = OfflineRender {
        on,
        format: ProcessorFormat {
            sample_rate: format.sample_rate.max(1),
            channels: format.channels.max(1),
            max_frames: format.max_frames.max(1),
        },
        frame: 0,
    };
}
pub(super) fn render_offline() -> bool {
    offline_state().on
}
pub(super) fn set_offline_frame(frame: u64) {
    lock(&OFFLINE_RENDER).frame = frame;
}
pub(super) fn offline_clock() -> Option<(u64, u32)> {
    let state = offline_state();
    state.on.then_some((state.frame, state.format.sample_rate))
}

// Resolve the same target for format queries and processor startup. Queries
// return zero when startup would refuse a real-time lane.
struct ProcessorTarget {
    offline: bool,
    format: ProcessorFormat,
    plays: bool,
}
fn processor_target(engine: Option<&AudioEngine>) -> ProcessorTarget {
    let offline = offline_state();
    if offline.on {
        return ProcessorTarget {
            offline: true,
            format: offline.format,
            plays: true,
        };
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = engine;
        let format = super::luau_audio_web::request(serde_json::json!({"op":"processor.format"}))
            .ok()
            .and_then(|value| browser_processor_format(&value).ok());
        ProcessorTarget {
            offline: false,
            plays: format.is_some(),
            format: format.unwrap_or_default(),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    ProcessorTarget {
        offline: false,
        plays: engine.is_some(),
        format: engine
            .map(|engine| ProcessorFormat {
                sample_rate: engine.sample_rate(),
                channels: engine.channels(),
                ..ProcessorFormat::default()
            })
            .unwrap_or_default(),
    }
}
#[cfg(target_arch = "wasm32")]
fn browser_processor_format(value: &serde_json::Value) -> Result<ProcessorFormat, String> {
    let field = |name: &str| {
        value[name]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .filter(|n| *n > 0)
            .ok_or_else(|| format!("invalid browser processor {name}"))
    };
    Ok(ProcessorFormat {
        sample_rate: field("sampleRate")?,
        channels: field("channels")?,
        max_frames: field("maxFrames")?,
    })
}
pub(super) fn processor_format(engine: Option<&AudioEngine>) -> ProcessorFormat {
    let target = processor_target(engine);
    if target.plays {
        target.format
    } else {
        ProcessorFormat {
            sample_rate: 0,
            channels: 0,
            max_frames: 0,
        }
    }
}

const STARTING: u8 = 0;
const RUNNING: u8 = 1;
const STOPPED: u8 = 2;
const FAILED: u8 = 3;
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn truncate(value: &mut String, maximum: usize) {
    if value.len() > maximum {
        let mut end = maximum;
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        value.truncate(end);
    }
}
// Native upstream lane console: 8 KiB, 2-byte record length, 1024-byte lines.
// One serialized VM writer never waits for the control-thread reader.
struct LogRing {
    bytes: Box<[AtomicU8]>,
    write: AtomicUsize,
    read: AtomicUsize,
    dropped: AtomicUsize,
    reader: Mutex<()>,
}
impl LogRing {
    fn new() -> Self {
        Self {
            bytes: (0..8192).map(|_| AtomicU8::new(0)).collect(),
            write: AtomicUsize::new(0),
            read: AtomicUsize::new(0),
            dropped: AtomicUsize::new(0),
            reader: Mutex::new(()),
        }
    }
    fn push(&self, line: &str) {
        let write = self.write.load(Ordering::Relaxed);
        let read = self.read.load(Ordering::Acquire);
        let mut length = line.len().min(1024);
        while !line.is_char_boundary(length) {
            length -= 1;
        }
        if length + 2 > 8192 - write.wrapping_sub(read) {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        }
        for (offset, byte) in (length as u16)
            .to_le_bytes()
            .iter()
            .chain(line.as_bytes()[..length].iter())
            .enumerate()
        {
            self.bytes[(write.wrapping_add(offset)) & 8191].store(*byte, Ordering::Relaxed);
        }
        self.write
            .store(write.wrapping_add(length + 2), Ordering::Release);
    }
    fn has_lines(&self) -> bool {
        self.write.load(Ordering::Acquire) != self.read.load(Ordering::Acquire)
            || self.dropped.load(Ordering::Acquire) != 0
    }
    fn take(&self) -> Vec<String> {
        let _reader = lock(&self.reader);
        let mut read = self.read.load(Ordering::Relaxed);
        let write = self.write.load(Ordering::Acquire);
        let mut result = Vec::new();
        while read != write {
            let length = self.bytes[read & 8191].load(Ordering::Relaxed) as usize
                | ((self.bytes[(read.wrapping_add(1)) & 8191].load(Ordering::Relaxed) as usize)
                    << 8);
            let bytes: Vec<u8> = (0..length)
                .map(|i| self.bytes[read.wrapping_add(i + 2) & 8191].load(Ordering::Relaxed))
                .collect();
            result.push(String::from_utf8_lossy(&bytes).into_owned());
            read = read.wrapping_add(length + 2);
        }
        self.read.store(read, Ordering::Release);
        let dropped = self.dropped.swap(0, Ordering::AcqRel);
        if dropped != 0 {
            result.push(format!("processor dropped {dropped} console lines: it wrote more than 8 KB between two handovers"));
        }
        result
    }
}

struct Shared {
    state: AtomicU8,
    cancelled: AtomicBool,
    error: OnceLock<String>,
    logs: LogRing,
    pending: AtomicBool,
    failure_reported: AtomicBool,
    retired: AtomicBool,
}
impl Shared {
    fn new(state: u8) -> Self {
        let result = Self {
            state: AtomicU8::new(state),
            cancelled: AtomicBool::new(false),
            error: OnceLock::new(),
            logs: LogRing::new(),
            pending: AtomicBool::new(false),
            failure_reported: AtomicBool::new(false),
            retired: AtomicBool::new(false),
        };
        if state == STARTING {
            result.notify();
        }
        result
    }
    fn notify(&self) {
        if self.retired.load(Ordering::Acquire) {
            return;
        }
        // Count first: a simultaneous poll must never decrement before the
        // corresponding increment. Undo the speculative count if coalesced.
        vm_completions_queued(1);
        if self.pending.swap(true, Ordering::AcqRel) {
            vm_completions_taken(1);
        }
        // Retirement may have raced the early check and publication. Whichever
        // side clears the bit owns the matching decrement.
        if self.retired.load(Ordering::Acquire) && self.pending.swap(false, Ordering::AcqRel) {
            vm_completions_taken(1);
        }
    }
    fn retire(&self) {
        self.retired.store(true, Ordering::Release);
        if self.pending.swap(false, Ordering::AcqRel) {
            vm_completions_taken(1);
        }
    }
    fn poll_notification(&self) -> bool {
        if self.state.load(Ordering::Acquire) == STARTING {
            // Keep the root awake without reporting an undelivered transition.
            return false;
        }
        if self.pending.swap(false, Ordering::AcqRel) {
            vm_completions_taken(1);
            true
        } else {
            false
        }
    }

    fn fail(&self, mut error: String) {
        truncate(&mut error, 1024);
        let _ = self.error.set(error);
        let _ = self
            .state
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |state| {
                (state < STOPPED).then_some(FAILED)
            });
        self.notify();
    }
}
impl Drop for Shared {
    fn drop(&mut self) {
        if self.pending.swap(false, Ordering::AcqRel) {
            vm_completions_taken(1);
        }
    }
}
struct Owner {
    shared: Arc<Shared>,
    engine: Option<AudioEngine>,
    node: Mutex<Option<AudioRendererNode>>,
    offline_vm: Mutex<Option<RawProcessor>>,
    offline: bool,
    #[cfg(target_arch = "wasm32")]
    web_id: Option<u32>,
    format: ProcessorFormat,
}
impl Owner {
    fn stop(&self) {
        #[cfg(target_arch = "wasm32")]
        if let Some(id) = self.web_id {
            if let Err(error) =
                super::luau_audio_web::request(serde_json::json!({"op":"processor.stop","id":id}))
            {
                self.shared.fail(error);
            }
        }
        self.shared.cancelled.store(true, Ordering::Release);
        let transitioned =
            self.shared
                .state
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |s| {
                    (s < STOPPED).then_some(STOPPED)
                });
        if transitioned.is_ok() {
            self.shared.notify();
        }
        if let Some(node) = lock(&self.node).take() {
            if let Some(engine) = self.engine.as_ref() {
                engine.detach_renderer(node);
            }
        }
        // Wait for an in-flight host pull before dropping its VM, as native
        // detachRenderer does for a device callback.
        lock(&self.offline_vm).take();
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.stop();
        #[cfg(target_arch = "wasm32")]
        if let Some(id) = self.web_id {
            WEB_OWNERS.with(|owners| {
                owners.borrow_mut().remove(&id);
            });
            let _ = super::luau_audio_web::request(
                serde_json::json!({"op":"processor.release","id":id}),
            );
        }
    }
}
#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_OWNERS: std::cell::RefCell<BTreeMap<u32, std::sync::Weak<Owner>>> = const { std::cell::RefCell::new(BTreeMap::new()) };
}
#[cfg(target_arch = "wasm32")]
pub(super) fn browser_notify(id: u32) {
    let owner =
        WEB_OWNERS.with(|owners| owners.borrow().get(&id).and_then(std::sync::Weak::upgrade));
    if let Some(owner) = owner {
        owner.shared.notify();
    }
}
#[derive(Clone)]
pub struct ProcessorHandle(Arc<Owner>);
impl ProcessorHandle {
    pub(super) fn start(
        engine: Option<AudioEngine>,
        name: String,
        modules: BTreeMap<String, Arc<[u8]>>,
        input: SharedValue,
        hard_blocks: u32,
        memory_limit: usize,
    ) -> Result<Self, String> {
        if !modules.contains_key(&name) {
            return Err(format!("unknown audio processor module {name}"));
        }
        let target = processor_target(engine.as_ref());
        // Offline lanes have no engine attachment or block deadline.
        let engine = if target.offline { None } else { engine };
        let format = target.format;
        let hard_blocks = if target.offline { 0 } else { hard_blocks };
        #[cfg(target_arch = "wasm32")]
        if !target.offline {
            let encoded = super::luau_audio_shared::encode_wire(&input)?;
            let module_bytes: BTreeMap<_, _> = modules
                .iter()
                .map(|(name, bytes)| (name, bytes.as_ref()))
                .collect();
            let response = super::luau_audio_web::request(serde_json::json!({
                "op":"processor.start", "name":name, "modules":module_bytes, "input":encoded,
                "format":{"sampleRate":format.sample_rate,"channels":format.channels,"maxFrames":format.max_frames},
                "hardBlocks":hard_blocks,"memoryLimit":memory_limit
            }))?;
            let id = response["id"]
                .as_u64()
                .and_then(|id| u32::try_from(id).ok())
                .ok_or("invalid browser processor id")?;
            let format = browser_processor_format(&response["format"])?;
            let owner = Arc::new(Owner {
                shared: Arc::new(Shared::new(STARTING)),
                engine: None,
                node: Mutex::new(None),
                offline_vm: Mutex::new(None),
                offline: false,
                format,
                web_id: Some(id),
            });
            WEB_OWNERS.with(|owners| {
                owners.borrow_mut().insert(id, Arc::downgrade(&owner));
            });
            return Ok(Self(owner));
        }
        let shared = Arc::new(Shared::new(STARTING));
        let owner = Arc::new(Owner {
            shared: shared.clone(),
            engine: engine.clone(),
            node: Mutex::new(None),
            offline_vm: Mutex::new(None),
            offline: target.offline,
            #[cfg(target_arch = "wasm32")]
            web_id: None,
            format,
        });
        if engine.is_none() && !target.offline {
            shared.fail("real time audio has no audio engine".into());
            return Ok(Self(owner));
        }
        let weak = Arc::downgrade(&owner);
        submit_initialization(Box::new(move || {
            if shared.cancelled.load(Ordering::Acquire) {
                return;
            }
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                RawProcessor::new(
                    shared.clone(),
                    name,
                    modules,
                    input,
                    format.sample_rate,
                    format.channels,
                    format.max_frames,
                    hard_blocks,
                    memory_limit,
                )
            }));
            match result {
                Ok(Ok(vm)) => {
                    if let Some(owner) = weak.upgrade() {
                        let mut node = lock(&owner.node);
                        if shared.state.load(Ordering::Acquire) == STARTING {
                            if let Some(engine) = engine.as_ref() {
                                *node = Some(engine.attach_renderer(vm));
                            } else {
                                *lock(&owner.offline_vm) = Some(vm);
                            }
                            let _ = shared.state.compare_exchange(
                                STARTING,
                                RUNNING,
                                Ordering::AcqRel,
                                Ordering::Acquire,
                            );
                            shared.notify();
                        }
                    }
                }
                Ok(Err(error)) => shared.fail(error),
                Err(_) => shared.fail("audio processor initialization panicked".into()),
            }
        }))?;
        Ok(Self(owner))
    }
    /// Format captured when this lane started; later host configuration changes
    /// do not change an already initialized processor.
    pub fn format(&self) -> ProcessorFormat {
        self.0.format
    }
    pub fn is_offline(&self) -> bool {
        self.0.offline
    }
    /// Pull this offline lane directly, without a device or mixer attachment.
    /// A starting/stopped/failed lane produces silence. Hosts wait for running
    /// when they need initialization to finish, and advance the shared offline
    /// clock explicitly after pulling all lanes for a block.
    pub fn render_offline(
        &self,
        output: &mut [f32],
        frames: u32,
        start_frame: u64,
    ) -> Result<(), String> {
        if !self.0.offline {
            return Err("processor belongs to the realtime audio engine".into());
        }
        let expected = (frames as usize)
            .checked_mul(self.0.format.channels as usize)
            .ok_or("offline audio output size overflow")?;
        if output.len() != expected {
            return Err("offline audio output does not match processor format".into());
        }
        output.fill(0.0);
        if let Some(vm) = lock(&self.0.offline_vm).as_mut() {
            vm.render(output, frames, start_frame);
        }
        Ok(())
    }
    #[cfg(target_arch = "wasm32")]
    fn sync_web(&self) {
        let Some(id) = self.0.web_id else {
            return;
        };
        let result =
            super::luau_audio_web::request(serde_json::json!({"op":"processor.poll","id":id}));
        match result {
            Ok(value) => {
                if let Some(lines) = value["logs"].as_array() {
                    for line in lines {
                        if let Some(line) = line.as_str() {
                            self.0.shared.logs.push(line);
                        }
                    }
                }
                match value["state"].as_str() {
                    Some("running") => {
                        let _ = self.0.shared.state.compare_exchange(
                            STARTING,
                            RUNNING,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        );
                    }
                    Some("failed") if self.0.shared.state.load(Ordering::Acquire) < STOPPED => {
                        self.0.shared.fail(
                            value["error"]
                                .as_str()
                                .unwrap_or("browser audio processor failed")
                                .into(),
                        )
                    }
                    Some("stopped") => {
                        let _ = self.0.shared.state.fetch_update(
                            Ordering::AcqRel,
                            Ordering::Acquire,
                            |s| (s < STOPPED).then_some(STOPPED),
                        );
                    }
                    _ => {}
                }
            }
            Err(error) if self.0.shared.state.load(Ordering::Acquire) < STOPPED => {
                self.0.shared.fail(error)
            }
            Err(_) => {}
        }
    }
    pub fn state(&self) -> &'static str {
        #[cfg(target_arch = "wasm32")]
        self.sync_web();
        match self.0.shared.state.load(Ordering::Acquire) {
            STARTING => "starting",
            RUNNING => "running",
            STOPPED => "stopped",
            _ => "failed",
        }
    }
    pub fn error(&self) -> Option<String> {
        self.0.shared.error.get().cloned()
    }
    pub(super) fn strong_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }
    pub(super) fn has_logs(&self) -> bool {
        self.0.shared.logs.has_lines()
    }
    pub(super) fn has_pending_work(&self) -> bool {
        !self.0.shared.retired.load(Ordering::Acquire)
            && (self.state() == "starting" || self.0.shared.pending.load(Ordering::Acquire))
    }
    pub(super) fn abandon_notifications(&self) {
        self.0.shared.retire();
    }
    pub(super) fn poll_notification(&self) -> bool {
        self.0.shared.poll_notification()
    }
    pub(super) fn take_failure(&self) -> Option<String> {
        let error = self.0.shared.error.get()?;
        if self.0.shared.failure_reported.swap(true, Ordering::AcqRel) {
            None
        } else {
            Some(error.clone())
        }
    }
    pub(super) fn is_unreferenced_terminal(&self) -> bool {
        self.strong_count() == 1
            && matches!(self.state(), "stopped" | "failed")
            && !self.has_pending_work()
            && !self.has_logs()
            && (self.0.shared.error.get().is_none()
                || self.0.shared.failure_reported.load(Ordering::Acquire))
    }
    pub(super) fn take_logs(&self) -> Vec<String> {
        self.0.shared.logs.take()
    }
    pub fn stop(&self) {
        self.0.stop();
    }
}

type InitJob = Box<dyn FnOnce() + Send>;
#[cfg(not(target_arch = "wasm32"))]
fn submit_initialization(job: InitJob) -> Result<(), String> {
    static QUEUE: OnceLock<Result<std::sync::mpsc::Sender<InitJob>, String>> = OnceLock::new();
    let sender = QUEUE
        .get_or_init(|| {
            let (sender, receiver) = std::sync::mpsc::channel::<InitJob>();
            std::thread::Builder::new()
                .name("luau-audio-init".into())
                .spawn(move || {
                    while let Ok(job) = receiver.recv() {
                        job();
                    }
                })
                .map_err(|error| format!("cannot initialize audio processor worker: {error}"))?;
            Ok(sender)
        })
        .as_ref()
        .map_err(Clone::clone)?;
    sender
        .send(job)
        .map_err(|_| "audio processor initialization worker stopped".into())
}

#[cfg(target_arch = "wasm32")]
fn submit_initialization(job: InitJob) -> Result<(), String> {
    // Browser realtime init is a worklet message, never process(). Offline
    // hosts explicitly pull on their own thread and may initialize synchronously.
    job();
    Ok(())
}

struct Context {
    shared: Arc<Shared>,
    deadline: Option<Instant>,
    expired: bool,
    modules: BTreeMap<String, Arc<[u8]>>,
    loaded: BTreeMap<String, c_int>,
    loading: BTreeSet<String>,
    channels: Vec<SharedValue>,
    pcm_readers: Vec<super::luau_audio_samples::PcmStreamReader>,
    format: ProcessorFormat,
    channel_objects: BTreeMap<usize, usize>,
    channel_refs: Vec<c_int>,
    bytes: usize,
    memory_limit: usize,
    oom: bool,
    initializing: bool,
    failure: Option<String>,
    stop_error: c_int,
    budget_error: c_int,
}
// Context is owned by one RawProcessor. It is accessed only while that processor
// is exclusively borrowed; allocator/callback pointers remain stable in its Box.
struct RawProcessor {
    state: *mut lua_State,
    context: Box<Context>,
    object: c_int,
    process: c_int,
    output: c_int,
    sample_rate: u32,
    channels: u32,
    max_frames: u32,
    hard_blocks: u32,
    warm: bool,
}
// SAFETY: every pointer and registry reference belongs exclusively to this raw VM.
// No luaur-rt/TLS-backed APIs are used. All installed callbacks are stateless raw
// functions; their context contains only owned/Send data. The entire VM moves
// after initialization and is subsequently serialized by AudioRenderer's owner.
// No VM pointer or Lua object is exposed through ProcessorHandle.
unsafe impl Send for RawProcessor {}
impl Drop for RawProcessor {
    fn drop(&mut self) {
        if !self.state.is_null() {
            unsafe { lua_close(self.state) };
        }
    }
}
unsafe extern "C" fn allocate(
    ud: *mut c_void,
    ptr: *mut c_void,
    old: usize,
    new: usize,
) -> *mut c_void {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        let ctx = ud.cast::<Context>();
        if new > old && (*ctx).bytes.saturating_add(new - old) > (*ctx).memory_limit {
            (*ctx).oom = true;
            return std::ptr::null_mut();
        }
        let out = l_alloc(std::ptr::null_mut(), ptr, old, new);
        if new == 0 || !out.is_null() {
            (*ctx).bytes = (*ctx).bytes.saturating_sub(old).saturating_add(new);
        } else {
            (*ctx).oom = true;
        }
        out
    }
}
unsafe fn context(l: *mut lua_State) -> *mut Context {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe { (*lua_callbacks(l)).userdata.cast() }
}
unsafe fn error<T>(l: *mut lua_State, message: &str) -> LuaResult<T> {
    lua_pushlstring(l, message.as_ptr().cast(), message.len())?;
    lua_error(l)
}
unsafe fn host_error<T>(l: *mut lua_State, message: &str) -> LuaResult<T> {
    (*context(l)).failure = Some(message.into());
    error(l, message)
}
unsafe fn interrupt(l: *mut lua_State, _: c_int) -> LuaResult<()> {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        let ctx = context(l);
        if (*ctx).initializing && (&(*ctx).shared).cancelled.load(Ordering::Acquire) {
            lua_rawgeti(l, LUA_REGISTRYINDEX, (*ctx).stop_error)?;
            return lua_error(l);
        }
        if (*ctx).deadline.is_some_and(|end| Instant::now() >= end) {
            (*ctx).expired = true;
        }
        if (*ctx).expired {
            lua_rawgeti(l, LUA_REGISTRYINDEX, (*ctx).budget_error)?;
            return lua_error(l);
        }
        Ok(())
    }
}
unsafe fn diagnostic(l: *mut lua_State, fallback: &str) -> String {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        if lua_gettop(l) > 0 && lua_type(l, -1) == lua_Type::LUA_TSTRING as c_int {
            let mut size = 0;
            if let Ok(p) = lua_tolstring(l, -1, &mut size) {
                if !p.is_null() {
                    return String::from_utf8_lossy(std::slice::from_raw_parts(p.cast(), size))
                        .into_owned();
                }
            }
        }
        fallback.into()
    }
}
unsafe fn call(l: *mut lua_State, args: c_int, results: c_int) -> Result<(), String> {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        match lua_pcall(l, args, results, 0) {
            Ok(0) => Ok(()),
            _ => Err(diagnostic(l, "audio processor Lua call failed")),
        }
    }
}
impl RawProcessor {
    fn new(
        shared: Arc<Shared>,
        name: String,
        modules: BTreeMap<String, Arc<[u8]>>,
        input: SharedValue,
        sample_rate: u32,
        channels: u32,
        max_frames: u32,
        hard_blocks: u32,
        memory_limit: usize,
    ) -> Result<Self, String> {
        // The execution deadlines below read the host clock; without the browser
        // audio host they could never expire, so processor code never runs.
        #[cfg(target_arch = "wasm32")]
        super::luau_audio_web::require_host()?;
        let mut ctx = Box::new(Context {
            shared,
            deadline: Some(Instant::now() + Duration::from_millis(1000)),
            expired: false,
            modules,
            loaded: BTreeMap::new(),
            loading: BTreeSet::new(),
            channels: Vec::new(),
            pcm_readers: Vec::new(),
            format: ProcessorFormat {
                sample_rate,
                channels,
                max_frames,
            },
            channel_objects: BTreeMap::new(),
            channel_refs: Vec::new(),
            bytes: 0,
            memory_limit,
            oom: false,
            initializing: true,
            failure: None,
            stop_error: 0,
            budget_error: 0,
        });
        let l = unsafe { lua_newstate(Some(allocate), (&mut *ctx as *mut Context).cast()) };
        if l.is_null() {
            return Err("cannot allocate audio processor VM".into());
        }
        let mut vm = Self {
            state: l,
            context: ctx,
            object: 0,
            process: 0,
            output: 0,
            sample_rate,
            channels,
            max_frames,
            hard_blocks,
            warm: false,
        };
        unsafe {
            (*lua_callbacks(l)).userdata = (&mut *vm.context as *mut Context).cast();
        }
        let result: LuaResult<()> = unsafe {
            (|| {
                for (text, slot) in [
                    ("audio processor stopped", &mut vm.context.stop_error),
                    (
                        "audio processor execution budget exceeded",
                        &mut vm.context.budget_error,
                    ),
                ] {
                    lua_pushlstring(l, text.as_ptr().cast(), text.len())?;
                    *slot = lua_ref(l, -1)?;
                    lua_settop(l, -2)?;
                }
                (*lua_callbacks(l)).interrupt = Some(interrupt);
                lua_l_openlibs(l)?;
                lua_pushcclosurek(l, Some(print), c"print".as_ptr(), 0, None)?;
                lua_setfield(l, LUA_GLOBALSINDEX, c"print".as_ptr())?;
                lua_pushcclosurek(l, Some(require), c"require".as_ptr(), 0, None)?;
                lua_setfield(l, LUA_GLOBALSINDEX, c"require".as_ptr())?;
                lua_createtable(l, 0, 3)?;
                for (name, callback) in [
                    (
                        c"streamReader",
                        pcm_reader as unsafe fn(*mut lua_State) -> LuaResult<c_int>,
                    ),
                    (c"processorSampleRate", processor_sample_rate),
                    (c"processorChannels", processor_channels),
                ] {
                    lua_pushcclosurek(l, Some(callback), name.as_ptr(), 0, None)?;
                    lua_setfield(l, -2, name.as_ptr())?;
                }
                lua_setfield(l, LUA_GLOBALSINDEX, c"Audio".as_ptr())?;
                lua_l_sandbox(l)?;
                load_module(l, &name)?;
                if lua_type(l, -1) != lua_Type::LUA_TTABLE as c_int {
                    return error(l, "audio processor module must return a table");
                }
                vm.object = lua_ref(l, -1)?;
                lua_getfield(l, -1, c"process".as_ptr())?;
                if lua_type(l, -1) != lua_Type::LUA_TFUNCTION as c_int {
                    return error(
                        l,
                        "audio processor needs process(self,out,frames,startFrame)",
                    );
                }
                vm.process = lua_ref(l, -1)?;
                lua_settop(l, 0)?;
                let bytes = (max_frames as usize)
                    .checked_mul(channels as usize)
                    .and_then(|n| n.checked_mul(4))
                    .ok_or_else(|| lua_exception::lua_exception(l, 4))?;
                lua_newbuffer(l, bytes)?;
                vm.output = lua_ref(l, -1)?;
                lua_settop(l, 0)?;
                lua_rawgeti(l, LUA_REGISTRYINDEX, vm.object)?;
                lua_getfield(l, -1, c"init".as_ptr())?;
                if lua_type(l, -1) == lua_Type::LUA_TFUNCTION as c_int {
                    lua_rawgeti(l, LUA_REGISTRYINDEX, vm.object)?;
                    lua_pushnumber(l, sample_rate as f64)?;
                    lua_pushnumber(l, max_frames as f64)?;
                    lua_pushnumber(l, channels as f64)?;
                    push(l, &input)?;
                    interrupt(l, 0)?;
                    // Upstream gives each native initialization step its own
                    // 1000 ms budget; the module entry and init are distinct.
                    vm.context.deadline = Some(Instant::now() + Duration::from_millis(1000));
                    if let Err(e) = call(l, 5, 0) {
                        return error(l, &e);
                    }
                } else if lua_type(l, -1) != lua_Type::LUA_TNIL as c_int {
                    return error(l, "audio processor init must be a function");
                }
                lua_settop(l, 0)?;
                interrupt(l, 0)?;
                Ok(())
            })()
        };
        if result.is_err() {
            return Err(unsafe { diagnostic(l, "audio processor initialization failed") });
        }
        if let Some(error) = vm.context.failure.as_ref() {
            return Err(error.clone());
        }
        if vm.context.oom {
            return Err("audio processor memory limit exceeded".into());
        }
        vm.context.deadline = None;
        vm.context.initializing = false;
        Ok(vm)
    }
    unsafe fn process_block(
        &mut self,
        out: &mut [f32],
        frames: u32,
        start: u64,
    ) -> Result<(), String> {
        // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
        // remain valid for the protected call and never escape the isolated lane.
        unsafe {
            let l = self.state;
            if frames > self.max_frames || out.len() != frames as usize * self.channels as usize {
                return Err("audio format exceeds initialized processor format".into());
            }
            let ms = if self.hard_blocks == 0 {
                0
            } else {
                let span = ((frames as u64) * 1000 * (self.hard_blocks as u64)
                    + (self.sample_rate as u64)
                    - 1)
                    / (self.sample_rate as u64);
                span.max(if self.warm { 1 } else { 100 })
            };
            self.context.expired = false;
            self.context.deadline = (ms != 0).then(|| Instant::now() + Duration::from_millis(ms));
            self.warm = true;
            let result: LuaResult<()> = (|| {
                lua_settop(l, 0)?;
                lua_rawgeti(l, LUA_REGISTRYINDEX, self.output)?;
                let mut size = 0;
                let buffer = lua_tobuffer(l, -1, &mut size).cast::<u8>();
                std::ptr::write_bytes(buffer, 0, size);
                lua_settop(l, 0)?;
                lua_rawgeti(l, LUA_REGISTRYINDEX, self.process)?;
                lua_rawgeti(l, LUA_REGISTRYINDEX, self.object)?;
                lua_rawgeti(l, LUA_REGISTRYINDEX, self.output)?;
                lua_pushnumber(l, frames as f64)?;
                lua_pushnumber(l, start as f64)?;
                if let Err(e) = call(l, 4, 0) {
                    return error(l, &e);
                }
                interrupt(l, 0)?;
                // Buffer is rooted and Luau's collector does not move objects. Copy
                // only the active frames; an author may retain the buffer safely.
                for (i, sample) in out.iter_mut().enumerate() {
                    let bytes = std::slice::from_raw_parts(buffer.add(i * 4), 4);
                    *sample = f32::from_le_bytes(bytes.try_into().unwrap());
                }
                lua_gc(l, 6, 0)?;
                interrupt(l, 0)?;
                Ok(())
            })();
            self.context.deadline = None;
            if let Some(error) = self.context.failure.as_ref() {
                return Err(error.clone());
            }
            if self.context.oom {
                return Err("audio processor memory limit exceeded".into());
            }
            if self.context.expired {
                return Err("audio processor execution budget exceeded".into());
            }
            result.map_err(|_| diagnostic(l, "audio processor processing failed"))
        }
    }
}
impl AudioRenderer for RawProcessor {
    fn render(&mut self, out: &mut [f32], frames: u32, start: u64) {
        out.fill(0.0);
        if self.context.shared.state.load(Ordering::Acquire) != RUNNING {
            return;
        }
        let mut offset = 0usize;
        while offset < frames as usize
            && self.context.shared.state.load(Ordering::Acquire) == RUNNING
        {
            let count = (frames as usize - offset).min(self.max_frames as usize);
            let first = offset * self.channels as usize;
            let end = first + count * self.channels as usize;
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
                self.process_block(
                    &mut out[first..end],
                    count as u32,
                    start.wrapping_add(offset as u64),
                )
            }));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    out[first..].fill(0.0);
                    self.context.shared.fail(error);
                    break;
                }
                Err(_) => {
                    out[first..].fill(0.0);
                    self.context.shared.fail("audio processor panicked".into());
                    break;
                }
            }
            offset += count;
        }
    }
    fn finished(&self) -> bool {
        self.context.shared.state.load(Ordering::Acquire) >= STOPPED
    }
}
unsafe fn print(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        let count = lua_gettop(l);
        let mut line = String::with_capacity(1024);
        for index in 1..=count {
            if index != 1 && line.len() < 1024 {
                line.push('\t');
            }
            lua_getfield(l, LUA_GLOBALSINDEX, c"tostring".as_ptr())?;
            lua_pushvalue(l, index)?;
            if let Err(message) = call(l, 1, 1) {
                return error(l, &message);
            }
            let mut size = 0;
            let text = lua_tolstring(l, -1, &mut size)?;
            if text.is_null() {
                return error(l, "tostring must return text for print");
            }
            let remaining = 1024usize.saturating_sub(line.len());
            let text = String::from_utf8_lossy(std::slice::from_raw_parts(
                text.cast(),
                size.min(remaining),
            ));
            line.push_str(&text);
            truncate(&mut line, 1024);
            lua_settop(l, -2)?;
        }
        (&(*context(l)).shared).logs.push(&line);
        (&(*context(l)).shared).notify();
        Ok(0)
    }
}
unsafe fn require(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        if lua_type(l, 1) != lua_Type::LUA_TSTRING as c_int {
            return error(l, "require expects module name");
        }
        let mut size = 0;
        let p = lua_tolstring(l, 1, &mut size)?;
        let name = String::from_utf8_lossy(std::slice::from_raw_parts(p.cast(), size)).into_owned();
        load_module(l, &name)?;
        Ok(1)
    }
}
unsafe fn load_module(l: *mut lua_State, name: &str) -> LuaResult<()> {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        let ctx = context(l);
        if let Some(reference) = (*ctx).loaded.get(name).copied() {
            lua_rawgeti(l, LUA_REGISTRYINDEX, reference)?;
            return Ok(());
        }
        let Some(bytes) = (*ctx).modules.get(name).cloned() else {
            return error(l, &format!("unknown processor dependency {name}"));
        };
        if !(*ctx).loading.insert(name.into()) {
            return error(l, "cyclic audio processor module dependency");
        }
        let chunk = CString::new(name).map_err(|_| lua_exception::lua_exception(l, 2))?;
        if luau_load(l, chunk.as_ptr(), bytes.as_ptr().cast(), bytes.len(), 0)? != 0 {
            return lua_error(l);
        }
        if let Err(e) = call(l, 0, 1) {
            return error(l, &e);
        }
        let reference = lua_ref(l, -1)?;
        (*ctx).loaded.insert(name.into(), reference);
        (*ctx).loading.remove(name);
        Ok(())
    }
}

unsafe fn push(l: *mut lua_State, value: &SharedValue) -> LuaResult<()> {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        match value {
            SharedValue::Nil => lua_pushnil(l)?,
            SharedValue::Boolean(v) => lua_pushboolean(l, *v as c_int)?,
            SharedValue::Number(v) => lua_pushnumber(l, *v)?,
            SharedValue::Bytes(v) => lua_pushlstring(l, v.as_ptr().cast(), v.len())?,
            SharedValue::Array(values) => {
                lua_createtable(l, values.len() as c_int, 0)?;
                for (i, v) in values.iter().enumerate() {
                    push(l, v)?;
                    lua_rawseti(l, -2, (i + 1) as c_int)?;
                }
            }
            SharedValue::Object(values) => {
                lua_createtable(l, 0, values.len() as c_int)?;
                for (k, v) in values {
                    lua_pushlstring(l, k.as_ptr().cast(), k.len())?;
                    push(l, v)?;
                    lua_rawset(l, -3)?;
                }
            }
            SharedValue::Ring(_)
            | SharedValue::Snapshot(_)
            | SharedValue::Blob(_)
            | SharedValue::Region(_) => {
                let ctx = context(l);
                let index = (*ctx).channels.len();
                (*ctx).channels.push(value.clone());
                lua_createtable(l, 0, 6)?;
                (*ctx).channel_refs.push(lua_ref(l, -1)?);
                (*ctx)
                    .channel_objects
                    .insert(lua_topointer(l, -1) as usize, index);
                let methods: &[(
                    &std::ffi::CStr,
                    unsafe fn(*mut lua_State) -> LuaResult<c_int>,
                )] = match value {
                    SharedValue::Ring(_) => &[
                        (c"send", ring_send),
                        (c"receive", ring_receive),
                        (c"receiveInto", ring_receive_into),
                    ],
                    SharedValue::Snapshot(_) => {
                        &[(c"publish", snapshot_publish), (c"latest", snapshot_latest)]
                    }
                    SharedValue::Blob(_) => &[(c"read", blob_read), (c"toBuffer", blob_to_buffer)],
                    SharedValue::Region(_) => &[
                        (c"atomicLoad", region_atomic_load),
                        (c"atomicStore", region_atomic_store),
                        (c"atomicAdd", region_atomic_add),
                        (c"atomicCas", region_atomic_cas),
                        (c"exclusive", region_exclusive),
                        (c"tryExclusive", region_exclusive),
                    ],
                    _ => unreachable!(),
                };
                for (name, method) in methods {
                    lua_pushinteger(l, index as c_int)?;
                    lua_pushcclosurek(l, Some(*method), name.as_ptr(), 1, None)?;
                    lua_setfield(l, -2, name.as_ptr())?;
                }
                if matches!(value, SharedValue::Blob(_) | SharedValue::Region(_)) {
                    lua_createtable(l, 0, 1)?;
                    lua_pushinteger(l, index as c_int)?;
                    lua_pushcclosurek(l, Some(shared_field), c"shared field".as_ptr(), 1, None)?;
                    lua_setfield(l, -2, c"__index".as_ptr())?;
                    lua_setmetatable(l, -2)?;
                }
            }
        }
        Ok(())
    }
}
unsafe fn processor_sample_rate(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: the context belongs to this exclusively borrowed VM.
    unsafe {
        lua_pushnumber(l, (*context(l)).format.sample_rate as f64)?;
        Ok(1)
    }
}
unsafe fn processor_channels(l: *mut lua_State) -> LuaResult<c_int> {
    unsafe {
        lua_pushnumber(l, (*context(l)).format.channels as f64)?;
        Ok(1)
    }
}
unsafe fn pcm_reader(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: construction runs in the owning VM; only Rust shared descriptors
    // enter the reader, never references to the main VM or its audio decoder.
    unsafe {
        let value = read(l, 1, &mut BTreeSet::new())?;
        let descriptor = match super::luau_audio_samples::PcmStreamDescriptor::from_shared(&value) {
            Ok(value) => value,
            Err(e) => return host_error(l, &e),
        };
        let reader = match super::luau_audio_samples::PcmStreamReader::new(descriptor) {
            Ok(value) => value,
            Err(e) => return host_error(l, &e),
        };
        let ctx = context(l);
        let index = (*ctx).pcm_readers.len();
        (*ctx).pcm_readers.push(reader);
        lua_createtable(l, 0, 1)?;
        lua_pushinteger(l, index as c_int)?;
        lua_pushcclosurek(l, Some(pcm_read_into), c"readInto".as_ptr(), 1, None)?;
        lua_setfield(l, -2, c"readInto".as_ptr())?;
        lua_createtable(l, 0, 1)?;
        lua_pushinteger(l, index as c_int)?;
        lua_pushcclosurek(l, Some(pcm_field), c"PCM field".as_ptr(), 1, None)?;
        lua_setfield(l, -2, c"__index".as_ptr())?;
        lua_setmetatable(l, -2)?;
        Ok(1)
    }
}
unsafe fn pcm_read_into(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: the output buffer is rooted on this callback's stack. No Lua VM
    // operation occurs during the direct write to its bytes.
    unsafe {
        let index = lua_tonumberx(l, lua_upvalueindex(1), std::ptr::null_mut()) as usize;
        let mut length = 0;
        let output = lua_tobuffer(l, 2, &mut length);
        if output.is_null() {
            return error(l, "readInto expects an output buffer");
        }
        let reader = &mut (&mut (*context(l)).pcm_readers)[index];
        let result = reader.read_into_bytes(std::slice::from_raw_parts_mut(output.cast(), length));
        match result {
            Ok(count) => lua_pushnumber(l, count as f64)?,
            Err(e) => return host_error(l, &e),
        }
        Ok(1)
    }
}
unsafe fn pcm_field(l: *mut lua_State) -> LuaResult<c_int> {
    unsafe {
        let index = lua_tonumberx(l, lua_upvalueindex(1), std::ptr::null_mut()) as usize;
        let mut length = 0;
        let key = lua_tolstring(l, 2, &mut length)?;
        if key.is_null() {
            lua_pushnil(l)?;
            return Ok(1);
        }
        let key = std::slice::from_raw_parts(key.cast::<u8>(), length);
        let reader = &(&(*context(l)).pcm_readers)[index];
        match key {
            b"frame" => lua_pushnumber(l, reader.frame())?,
            b"ended" => lua_pushboolean(l, reader.ended() as c_int)?,
            b"sampleRate" => lua_pushnumber(l, reader.descriptor().sample_rate as f64)?,
            b"channels" => lua_pushnumber(l, reader.descriptor().channels as f64)?,
            _ => lua_pushnil(l)?,
        }
        Ok(1)
    }
}

unsafe fn shared_field(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: callback and registry context belong to this exclusively owned VM.
    unsafe {
        if lua_type(l, 2) != lua_Type::LUA_TSTRING as c_int {
            lua_pushnil(l)?;
            return Ok(1);
        }
        let mut length = 0;
        let key = lua_tolstring(l, 2, &mut length)?;
        let key = std::slice::from_raw_parts(key.cast::<u8>(), length);
        match (channel(l), key) {
            (SharedValue::Blob(blob), b"size") => lua_pushnumber(l, blob.size() as f64)?,
            (SharedValue::Region(region), b"size") => lua_pushnumber(l, region.size() as f64)?,
            (SharedValue::Region(region), b"poisoned") => {
                lua_pushboolean(l, region.poisoned() as c_int)?
            }
            _ => lua_pushnil(l)?,
        }
        Ok(1)
    }
}
unsafe fn index_argument(l: *mut lua_State, index: c_int, maximum: f64) -> LuaResult<usize> {
    // SAFETY: stack values belong to the current raw callback.
    unsafe {
        let value = lua_tonumberx(l, index, std::ptr::null_mut());
        if lua_type(l, index) != lua_Type::LUA_TNUMBER as c_int
            || !value.is_finite()
            || value < 0.0
            || value.fract() != 0.0
            || value > maximum
        {
            return error(
                l,
                "shared offset/value must be a nonnegative integer in range",
            );
        }
        Ok(value as usize)
    }
}
unsafe fn blob_read(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: the managed output buffer stays on the callback stack; no Lua call
    // occurs while its mutable byte slice is borrowed by the shared Blob reader.
    unsafe {
        let SharedValue::Blob(blob) = channel(l) else {
            unreachable!()
        };
        let offset = index_argument(l, 2, usize::MAX as f64)?;
        let mut length = 0;
        let output = lua_tobuffer(l, 3, &mut length);
        if output.is_null() {
            return error(l, "Blob.read expects an output buffer");
        }
        let count = blob.read(
            offset,
            std::slice::from_raw_parts_mut(output.cast(), length),
        );
        lua_pushnumber(l, count as f64)?;
        Ok(1)
    }
}
unsafe fn blob_to_buffer(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: the new managed buffer is rooted on the stack before its bytes
    // are initialized, with no intervening VM call during the copy.
    unsafe {
        let SharedValue::Blob(blob) = channel(l) else {
            unreachable!()
        };
        let size = blob.size();
        let output = lua_newbuffer(l, size)?;
        blob.read(0, std::slice::from_raw_parts_mut(output.cast(), size));
        Ok(1)
    }
}
unsafe fn region_atomic(l: *mut lua_State, operation: u8) -> LuaResult<c_int> {
    // SAFETY: callbacks retain an Arc to the shared region and only expose its
    // checked SeqCst word operations, never an unchecked memory pointer.
    unsafe {
        let SharedValue::Region(region) = channel(l) else {
            unreachable!()
        };
        let offset = index_argument(l, 2, usize::MAX as f64)?;
        let value = if operation != 0 {
            index_argument(l, 3, u32::MAX as f64)? as u32
        } else {
            0
        };
        let result = match operation {
            0 => region.atomic_load(offset),
            1 => region.atomic_store(offset, value),
            2 => region.atomic_add(offset, value),
            _ => region.atomic_cas(offset, value, index_argument(l, 4, u32::MAX as f64)? as u32),
        };
        match result {
            Ok(value) => lua_pushnumber(l, value as f64)?,
            Err(message) => return host_error(l, &message),
        }
        Ok(1)
    }
}
unsafe fn region_atomic_load(l: *mut lua_State) -> LuaResult<c_int> {
    unsafe { region_atomic(l, 0) }
}
unsafe fn region_atomic_store(l: *mut lua_State) -> LuaResult<c_int> {
    unsafe { region_atomic(l, 1) }
}
unsafe fn region_atomic_add(l: *mut lua_State) -> LuaResult<c_int> {
    unsafe { region_atomic(l, 2) }
}
unsafe fn region_atomic_cas(l: *mut lua_State) -> LuaResult<c_int> {
    unsafe { region_atomic(l, 3) }
}
unsafe fn region_exclusive(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: this marks the lane failed without touching a region lock.
    unsafe {
        host_error(
            l,
            "a processor cannot lock a shared region; use atomic word operations",
        )
    }
}

unsafe fn channel(l: *mut lua_State) -> SharedValue {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        let i = lua_tonumberx(l, lua_upvalueindex(1), std::ptr::null_mut()) as usize;
        (&(*context(l)).channels)[i].clone()
    }
}
unsafe fn read(
    l: *mut lua_State,
    index: c_int,
    seen: &mut BTreeSet<usize>,
) -> LuaResult<SharedValue> {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        if seen.len() >= 64 {
            return error(l, "audio message nesting exceeds 64 tables");
        }
        let index = if index < 0 {
            lua_gettop(l) + index + 1
        } else {
            index
        };
        match lua_type(l, index) {
            t if t == lua_Type::LUA_TNIL as c_int => Ok(SharedValue::Nil),
            t if t == lua_Type::LUA_TBOOLEAN as c_int => {
                Ok(SharedValue::Boolean(lua_toboolean(l, index) != 0))
            }
            t if t == lua_Type::LUA_TNUMBER as c_int => Ok(SharedValue::Number(lua_tonumberx(
                l,
                index,
                std::ptr::null_mut(),
            ))),
            t if t == lua_Type::LUA_TSTRING as c_int => {
                let mut n = 0;
                let p = lua_tolstring(l, index, &mut n)?;
                Ok(SharedValue::Bytes(
                    std::slice::from_raw_parts(p.cast(), n).to_vec(),
                ))
            }
            t if t == lua_Type::LUA_TTABLE as c_int => {
                let identity = lua_topointer(l, index) as usize;
                if let Some(channel) = (*context(l)).channel_objects.get(&identity).copied() {
                    return Ok((&(*context(l)).channels)[channel].clone());
                }
                if !seen.insert(identity) {
                    return error(l, "cyclic shared data is not supported");
                }
                let mut numbers = BTreeMap::new();
                let mut strings = BTreeMap::new();
                lua_pushnil(l)?;
                while lua_next(l, index)? != 0 {
                    let v = read(l, -1, seen)?;
                    if lua_type(l, -2) == lua_Type::LUA_TSTRING as c_int {
                        let mut n = 0;
                        let p = lua_tolstring(l, -2, &mut n)?;
                        let k = std::str::from_utf8(std::slice::from_raw_parts(p.cast(), n))
                            .map_err(|_| lua_exception::lua_exception(l, 2))?;
                        strings.insert(k.to_owned(), v);
                    } else if lua_type(l, -2) == lua_Type::LUA_TNUMBER as c_int {
                        let key = lua_tonumberx(l, -2, std::ptr::null_mut());
                        if key < 1.0 || key.fract() != 0.0 || key > i32::MAX as f64 {
                            return error(
                                l,
                                "shared arrays need consecutive positive integer keys",
                            );
                        }
                        numbers.insert(key as usize, v);
                    } else {
                        return error(l, "shared data keys must be strings or array indices");
                    }
                    lua_settop(l, -2)?;
                }
                seen.remove(&identity);
                if !numbers.is_empty() {
                    if !strings.is_empty() || numbers.keys().copied().ne(1..=numbers.len()) {
                        return error(l, "shared table must be an object or dense array");
                    }
                    Ok(SharedValue::Array(numbers.into_values().collect()))
                } else {
                    Ok(SharedValue::Object(strings))
                }
            }
            _ => error(
                l,
                "shared payload must be plain data; handles are allowed only in processor input",
            ),
        }
    }
}
unsafe fn ring_send(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        let SharedValue::Ring(ring) = channel(l) else {
            unreachable!()
        };
        let value = read(l, 2, &mut BTreeSet::new())?;
        match ring.send_from_processor(value) {
            Ok(ok) => lua_pushboolean(l, ok as c_int)?,
            Err(e) => return host_error(l, &e),
        };
        Ok(1)
    }
}
unsafe fn ring_receive(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        let SharedValue::Ring(ring) = channel(l) else {
            unreachable!()
        };
        match ring.receive_for_processor().map_err(|e| {
            (*context(l)).failure = Some(e.clone());
            let _ = lua_pushlstring(l, e.as_ptr().cast(), e.len());
            lua_exception::lua_exception(l, 2)
        })? {
            Some(value) => push(l, &value)?,
            None => lua_pushnil(l)?,
        };
        Ok(1)
    }
}
unsafe fn ring_receive_into(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        if lua_type(l, 2) != lua_Type::LUA_TTABLE as c_int {
            return error(l, "receiveInto expects a destination table");
        }
        let SharedValue::Ring(ring) = channel(l) else {
            unreachable!()
        };
        if let Some(value) = ring.receive_for_processor().map_err(|e| {
            (*context(l)).failure = Some(e.clone());
            let _ = lua_pushlstring(l, e.as_ptr().cast(), e.len());
            lua_exception::lua_exception(l, 2)
        })? {
            let mut keys = Vec::new();
            lua_pushnil(l)?;
            while lua_next(l, 2)? != 0 {
                lua_settop(l, -2)?;
                keys.push(lua_ref(l, -1)?);
            }
            for key in keys {
                lua_rawgeti(l, LUA_REGISTRYINDEX, key)?;
                lua_pushnil(l)?;
                lua_rawset(l, 2)?;
                luaur_vm::functions::lua_unref::lua_unref(l, key);
            }
            match value {
                SharedValue::Object(fields) => {
                    for (k, v) in fields {
                        lua_pushlstring(l, k.as_ptr().cast(), k.len())?;
                        push(l, &v)?;
                        lua_rawset(l, 2)?;
                    }
                }
                SharedValue::Array(values) => {
                    for (i, v) in values.iter().enumerate() {
                        push(l, v)?;
                        lua_rawseti(l, 2, (i + 1) as c_int)?;
                    }
                }
                _ => return error(l, "receiveInto requires an object or array message"),
            };
            lua_pushboolean(l, 1)?;
        } else {
            lua_pushboolean(l, 0)?;
        }
        Ok(1)
    }
}
unsafe fn snapshot_publish(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        let SharedValue::Snapshot(snapshot) = channel(l) else {
            unreachable!()
        };
        let value = read(l, 2, &mut BTreeSet::new())?;
        match snapshot.publish_from_processor(value) {
            Ok(ok) => lua_pushboolean(l, ok as c_int)?,
            Err(e) => return host_error(l, &e),
        };
        Ok(1)
    }
}
unsafe fn snapshot_latest(l: *mut lua_State) -> LuaResult<c_int> {
    // SAFETY: callers exclusively own this VM; raw pointers and callback userdata
    // remain valid for the protected call and never escape the isolated lane.
    unsafe {
        let SharedValue::Snapshot(snapshot) = channel(l) else {
            unreachable!()
        };
        if let Some((sequence, value)) = snapshot.latest_for_processor().map_err(|e| {
            (*context(l)).failure = Some(e.clone());
            let _ = lua_pushlstring(l, e.as_ptr().cast(), e.len());
            lua_exception::lua_exception(l, 2)
        })? {
            lua_createtable(l, 0, 2)?;
            lua_pushnumber(l, sequence as f64)?;
            lua_setfield(l, -2, c"sequence".as_ptr())?;
            push(l, &value)?;
            lua_setfield(l, -2, c"value".as_ptr())?;
        } else {
            lua_pushnil(l)?;
        }
        Ok(1)
    }
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::*;
    fn compile(source: &str) -> Arc<[u8]> {
        luaur_common::set_all_flags(true);
        let mut size = 0;
        let bytes = luaur_compiler::functions::luau_compile::luau_compile(
            source.as_ptr().cast(),
            source.len(),
            std::ptr::null_mut(),
            &mut size,
        );
        assert!(!bytes.is_null());
        let result: Arc<[u8]> = unsafe { std::slice::from_raw_parts(bytes.cast(), size) }.into();
        unsafe extern "C" {
            fn free(pointer: *mut c_void);
        }
        unsafe {
            free(bytes.cast());
        }
        result
    }
    fn shared() -> Arc<Shared> {
        Arc::new(Shared::new(RUNNING))
    }
    fn processor(source: &str, input: SharedValue) -> RawProcessor {
        let mut vm = RawProcessor::new(
            shared(),
            "processor".into(),
            BTreeMap::from([("processor".into(), compile(source))]),
            input,
            48_000,
            2,
            1024,
            2,
            16 * 1024 * 1024,
        )
        .unwrap();
        // Native upstream gives the first block an extended warm-up budget.
        vm.warm = true;
        vm
    }
    #[test]
    fn pcm_stream_reader_runs_in_isolated_luau_with_actual_format() {
        let descriptor = super::super::luau_audio_samples::PcmStreamDescriptor::new(
            128,
            48_000,
            2,
            2,
            Arc::new(super::super::luau_audio_shared::SharedBudget::default()),
        )
        .unwrap();
        let mut record = Vec::new();
        record.extend(0u32.to_le_bytes());
        record.extend(0u32.to_le_bytes());
        record.extend(5f64.to_le_bytes());
        for value in [0.25f32, 0.5, 0.75, 1.0] {
            record.extend(value.to_le_bytes());
        }
        descriptor.ring.write_record(&record).unwrap();
        let mut vm = processor(
            r#"return {
            init=function(self,rate,maxFrames,channels,input)
                assert(Audio.processorSampleRate()==rate and Audio.processorChannels()==channels)
                self.reader=Audio.streamReader(input)
            end,
            process=function(self,out,frames,startFrame)
                assert(self.reader:readInto(out)==2)
                assert(self.reader.frame==7 and not self.reader.ended)
            end
        }"#,
            descriptor.shared_value(),
        );
        let mut output = [0.0f32; 4];
        vm.render(&mut output, 2, 0);
        assert_eq!(output, [0.25, 0.5, 0.75, 1.0]);
        assert_eq!(descriptor.reader_count().unwrap(), 1);
        drop(vm);
        assert_eq!(descriptor.reader_count().unwrap(), 0);
    }
    #[test]
    fn authored_pcm_and_shared_controls_survive_callback_thread_migration() {
        let messages = Arc::new(SharedRing::new(4096).unwrap());
        let meter = Arc::new(SharedSnapshot::new(4096).unwrap());
        let mut vm = processor(
            r#"
            return {
                init=function(self,rate,maxFrames,channels,input)
                    assert(rate==48000 and maxFrames==1024 and channels==2)
                    self.link=input; self.gain=0; self.next={gain=0}
                end,
                process=function(self,out,frames,start)
                    while self.link.messages:receiveInto(self.next) do self.gain=self.next.gain end
                    for i=0,frames-1 do
                        buffer.writef32(out,i*8,self.gain)
                        buffer.writef32(out,i*8+4,-self.gain)
                    end
                    self.link.meter:publish({frames=frames,start=start,gain=self.gain})
                end,
            }
        "#,
            SharedValue::Object(BTreeMap::from([
                ("messages".into(), SharedValue::Ring(messages.clone())),
                ("meter".into(), SharedValue::Snapshot(meter.clone())),
            ])),
        );
        assert!(
            messages
                .send(SharedValue::Object(BTreeMap::from([(
                    "gain".into(),
                    SharedValue::Number(0.25)
                )])))
                .unwrap()
        );
        vm = std::thread::spawn(move || {
            let mut out = [0.0; 128];
            vm.render(&mut out, 64, 123);
            assert!(out.chunks_exact(2).all(|p| p == [0.25, -0.25]));
            vm
        })
        .join()
        .unwrap();
        assert!(
            messages
                .send(SharedValue::Object(BTreeMap::from([(
                    "gain".into(),
                    SharedValue::Number(0.5)
                )])))
                .unwrap()
        );
        let mut out = [0.0; 128];
        vm.render(&mut out, 64, 187);
        assert!(out.chunks_exact(2).all(|p| p == [0.5, -0.5]));
        let (_, SharedValue::Object(values)) = meter.latest().unwrap() else {
            panic!("snapshot missing")
        };
        assert!(matches!(
            values.get("start"),
            Some(SharedValue::Number(187.0))
        ));
    }
    #[test]
    fn caught_execution_deadline_still_fails_only_its_processor() {
        let mut broken = processor(
            "return {process=function() pcall(function() while true do end end) end}",
            SharedValue::Nil,
        );
        let mut good = processor(
            "return {process=function(self,out,n) for i=0,n*2-1 do buffer.writef32(out,i*4,0.25) end end}",
            SharedValue::Nil,
        );
        let mut out = [1.0; 128];
        broken.render(&mut out, 64, 0);
        assert!(out.iter().all(|v| *v == 0.0));
        assert!(broken.finished());
        assert!(
            broken
                .context
                .shared
                .error
                .get()
                .unwrap()
                .contains("budget")
        );
        good.render(&mut out, 64, 0);
        assert!(out.iter().all(|v| *v == 0.25));
        assert!(!good.finished());
    }
    #[test]
    fn failed_chunk_preserves_preceding_successful_chunk() {
        let mut vm = processor(
            "return {process=function(self,out,n,start) if start>=1024 then error('bad block') end for i=0,n*2-1 do buffer.writef32(out,i*4,0.125) end end}",
            SharedValue::Nil,
        );
        let mut out = vec![0.0; 4096];
        vm.render(&mut out, 2048, 0);
        assert!(out[..2048].iter().all(|v| *v == 0.125));
        assert!(out[2048..].iter().all(|v| *v == 0.0));
        assert!(vm.finished());
    }
    #[test]
    fn caught_memory_exhaustion_is_terminal() {
        let mut vm = processor(
            "return {process=function(self,out,n) pcall(function() buffer.create(32*1024*1024) end) end}",
            SharedValue::Nil,
        );
        let mut out = [1.0; 128];
        vm.render(&mut out, 64, 0);
        assert!(out.iter().all(|v| *v == 0.0));
        assert!(vm.finished());
        assert!(vm.context.shared.error.get().unwrap().contains("memory"));
    }
    #[test]
    fn precompiled_dependency_is_loaded_and_cached_in_isolated_vm() {
        let vm=RawProcessor::new(shared(),"processor".into(),BTreeMap::from([
            ("processor".into(),compile("local a=require('helper');local b=require('helper');assert(a==b);return {process=function(self,out,n)for i=0,n*2-1 do buffer.writef32(out,i*4,a.value)end end}")),
            ("helper".into(),compile("return {value=0.25}")),
        ]),SharedValue::Nil,48000,2,1024,2,16*1024*1024).unwrap();
        let mut vm = vm;
        let mut out = [0.0; 128];
        vm.render(&mut out, 64, 0);
        assert!(out.iter().all(|v| *v == 0.25));
    }
    #[test]
    fn receive_into_clears_previous_fields_and_array_tail() {
        let ring = Arc::new(SharedRing::new(4096).unwrap());
        let mut vm = processor(
            r#"
            return {
                init=function(self,rate,maxFrames,channels,input)
                    self.ring=input;self.next={obsolete=99,[1]=10,[2]=20};self.calls=0
                end,
                process=function(self,out,n,start)
                    assert(self.ring:receiveInto(self.next))
                    assert(self.next.obsolete==nil and self.next[1]==nil and self.next[2]==nil)
                    if self.calls==0 then assert(self.next.fresh==7) else assert(self.next.fresh==nil) end
                    self.calls+=1
                end,
            }
        "#,
            SharedValue::Ring(ring.clone()),
        );
        assert!(
            ring.send(SharedValue::Object(BTreeMap::from([(
                "fresh".into(),
                SharedValue::Number(7.0)
            )])))
            .unwrap()
        );
        let mut out = [0.0; 128];
        vm.render(&mut out, 64, 0);
        assert!(!vm.finished(), "{:?}", vm.context.shared.error.get());
        assert!(ring.send(SharedValue::Object(BTreeMap::new())).unwrap());
        vm.render(&mut out, 64, 64);
        assert!(!vm.finished(), "{:?}", vm.context.shared.error.get());
    }
    #[test]
    fn bounded_logs_drop_whole_lines_and_failure_keeps_first() {
        let shared = Shared::new(RUNNING);
        let line = "💛".repeat(300);
        for _ in 0..20 {
            shared.logs.push(&line);
        }
        let logs = shared.logs.take();
        assert_eq!(logs.len(), 8);
        for text in &logs[..7] {
            assert_eq!(text.len(), 1024);
            assert_eq!(text, "💛".repeat(256).as_str());
        }
        assert!(logs[7].contains("dropped 13 console lines"));
        assert!(!shared.logs.has_lines());
        shared.fail(line.clone());
        shared.fail("second error".into());
        assert_eq!(shared.error.get().unwrap(), &"💛".repeat(256));
        assert!(shared.pending.load(Ordering::Acquire));
        assert!(shared.poll_notification());
        assert!(!shared.poll_notification());
    }
    #[test]
    fn pending_initialization_is_not_acknowledged_until_state_changes() {
        let shared = Shared::new(STARTING);
        assert!(!shared.poll_notification());
        assert!(shared.pending.load(Ordering::Acquire));
        shared.state.store(RUNNING, Ordering::Release);
        shared.notify();
        assert!(shared.poll_notification());
        assert!(!shared.pending.load(Ordering::Acquire));
    }
    #[test]
    fn retired_owner_cannot_leave_a_notification_for_a_dropped_vm() {
        let shared = Arc::new(Shared::new(STARTING));
        let worker = shared.clone();
        let background = std::thread::spawn(move || {
            for _ in 0..10000 {
                worker.notify();
            }
        });
        shared.retire();
        background.join().unwrap();
        assert!(!shared.pending.load(Ordering::Acquire));
        shared.state.store(RUNNING, Ordering::Release);
        shared.notify();
        shared.fail("late init failure".into());
        assert!(!shared.pending.load(Ordering::Acquire));
    }
    struct ResetOffline;
    impl Drop for ResetOffline {
        fn drop(&mut self) {
            set_render_offline(false, ProcessorFormat::default());
        }
    }
    #[test]
    fn queried_processor_format_uses_startup_target() {
        let _guard = super::super::lua_audio::AUDIO_TEST_LOCK.lock().unwrap();
        let _reset = ResetOffline;
        set_render_offline(false, ProcessorFormat::default());
        assert_eq!(
            processor_format(None),
            ProcessorFormat {
                sample_rate: 0,
                channels: 0,
                max_frames: 0
            }
        );
        let engine = AudioEngine::new(1, 32000).unwrap();
        assert_eq!(
            processor_format(Some(&engine)),
            ProcessorFormat {
                sample_rate: 32000,
                channels: 1,
                max_frames: 1024
            }
        );
        let format = ProcessorFormat {
            sample_rate: 22050,
            channels: 2,
            max_frames: 17,
        };
        set_render_offline(true, format);
        assert_eq!(processor_format(Some(&engine)), format);
        assert_eq!(processor_format(None), format);
    }
    #[test]
    fn offline_host_format_clock_and_rendering_follow_upstream() {
        let _guard = super::super::lua_audio::AUDIO_TEST_LOCK.lock().unwrap();
        let _reset = ResetOffline;
        set_render_offline(
            true,
            ProcessorFormat {
                sample_rate: 22050,
                channels: 1,
                max_frames: 17,
            },
        );
        assert_eq!(offline_clock(), Some((0, 22050)));
        let module = compile(
            r#"return {
            init=function(self,rate,maxFrames,channels,input)
                assert(rate==22050 and maxFrames==17 and channels==1)
            end,
            process=function(self,out,n,start)
                for i=0,n-1 do buffer.writef32(out,i*4,(start+i)/1000) end
            end,
        }"#,
        );
        let handle = ProcessorHandle::start(
            None,
            "offline".into(),
            BTreeMap::from([("offline".into(), module)]),
            SharedValue::Nil,
            999,
            16 * 1024 * 1024,
        )
        .unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        while handle.state() == "starting" && Instant::now() < end {
            std::thread::yield_now();
        }
        assert_eq!(handle.state(), "running", "{:?}", handle.error());
        assert!(handle.is_offline());
        assert!(handle.0.engine.is_none());
        assert_eq!(lock(&handle.0.offline_vm).as_ref().unwrap().hard_blocks, 0);
        let mut out = vec![0.0; 40];
        handle.render_offline(&mut out, 40, 50).unwrap();
        for (i, value) in out.iter().enumerate() {
            assert!((*value - (50 + i) as f32 / 1000.0).abs() < 1e-7);
        }
        // Pulls do not advance the main lane until the host has rendered every lane.
        assert_eq!(offline_clock(), Some((0, 22050)));
        set_offline_frame(22050);
        let vm = super::super::ScriptVm::new();
        vm.install_rive_globals().unwrap();
        let (time, frame, rate): (f64, f64, u32) = vm
            .lua()
            .load("return Audio.time(),Audio.timeFrame(),Audio.sampleRate()")
            .eval()
            .unwrap();
        assert_eq!((time, frame, rate), (1.0, 22050.0, 22050));
        set_render_offline(
            true,
            ProcessorFormat {
                sample_rate: 0,
                channels: 0,
                max_frames: 0,
            },
        );
        assert_eq!(offline_clock(), Some((0, 1)));
        assert_eq!(
            offline_state().format,
            ProcessorFormat {
                sample_rate: 1,
                channels: 1,
                max_frames: 1
            }
        );
        assert_eq!(
            handle.format(),
            ProcessorFormat {
                sample_rate: 22050,
                channels: 1,
                max_frames: 17
            }
        );
        handle.stop();
        out.fill(1.0);
        handle.render_offline(&mut out, 40, 90).unwrap();
        assert!(out.iter().all(|v| *v == 0.0));
        assert!(handle.render_offline(&mut out, 39, 90).is_err());
        set_render_offline(false, ProcessorFormat::default());
        assert_eq!(offline_clock(), None);
        assert_eq!(offline_state().frame, 0);
    }
    #[test]
    fn offline_block_has_no_deadline_even_after_warmup() {
        let _guard = super::super::lua_audio::AUDIO_TEST_LOCK.lock().unwrap();
        let _reset = ResetOffline;
        set_render_offline(
            true,
            ProcessorFormat {
                sample_rate: 48000,
                channels: 1,
                max_frames: 1,
            },
        );
        let module = compile(
            "return {process=function(self,out,n,start) local finish=os.clock()+0.01;while os.clock()<finish do end;buffer.writef32(out,0,0.25) end}",
        );
        let handle = ProcessorHandle::start(
            None,
            "offline".into(),
            BTreeMap::from([("offline".into(), module)]),
            SharedValue::Nil,
            1,
            16 * 1024 * 1024,
        )
        .unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        while handle.state() == "starting" && Instant::now() < end {
            std::thread::yield_now();
        }
        assert_eq!(handle.state(), "running", "{:?}", handle.error());
        let mut out = [0.0];
        handle.render_offline(&mut out, 1, 0).unwrap();
        handle.render_offline(&mut out, 1, 1).unwrap();
        assert_eq!(out, [0.25]);
        assert_eq!(handle.state(), "running", "{:?}", handle.error());
        handle.stop();
    }
    #[test]
    fn processor_reads_blob_and_region_atomic_words() {
        let budget = Arc::new(super::super::luau_audio_shared::SharedBudget::default());
        let blob = Arc::new(SharedBlob::with_budget(&[7, 8, 9, 10], budget.clone()).unwrap());
        let region = Arc::new(SharedRegion::with_budget(8, budget).unwrap());
        let mut vm = processor(
            r#"return {
            init=function(self,rate,maxFrames,channels,input)
                self.blob=input.blob;self.region=input.region
                assert(self.blob.size==4 and self.region.size==8 and not self.region.poisoned)
                self.copy=self.blob:toBuffer();buffer.writeu8(self.copy,0,99)
                assert(self.blob:read(1,self.copy)==3)
                assert(buffer.readu8(self.copy,0)==8)
                assert(buffer.readu8(self.copy,1)==9)
                assert(buffer.readu8(self.copy,2)==10)
                self.region:atomicStore(0,12)
            end,
            process=function(self,out,n,start)
                assert(self.region:atomicAdd(0,3)==12)
                assert(self.region:atomicCas(0,15,21)==15)
                assert(self.region:atomicLoad(0)==21)
                for i=0,n*2-1 do buffer.writef32(out,i*4,0.25) end
            end,
        }"#,
            SharedValue::Object(BTreeMap::from([
                ("blob".into(), SharedValue::Blob(blob.clone())),
                ("region".into(), SharedValue::Region(region.clone())),
            ])),
        );
        let mut out = [0.0; 128];
        vm.render(&mut out, 64, 0);
        assert_eq!(
            vm.context.shared.state.load(Ordering::Acquire),
            RUNNING,
            "{:?}",
            vm.context.shared.error.get()
        );
        assert_eq!(region.atomic_load(0).unwrap(), 21);
        assert!(out.iter().all(|v| *v == 0.25));
        let mut original = [0; 4];
        assert_eq!(blob.read(0, &mut original), 4);
        assert_eq!(original, [7, 8, 9, 10]);
    }
    #[test]
    fn processor_cannot_catch_and_continue_after_region_lock_or_handle_transfer() {
        let budget = Arc::new(super::super::luau_audio_shared::SharedBudget::default());
        let region = Arc::new(SharedRegion::with_budget(8, budget).unwrap());
        for source in [
            "return {init=function(self,r,m,c,input)self.input=input end,process=function(self,out,n,start)pcall(function()self.input.region:tryExclusive(function()end)end)end}",
            "return {init=function(self,r,m,c,input)self.input=input end,process=function(self,out,n,start)pcall(function()self.input.ring:send({region=self.input.region})end)end}",
        ] {
            let ring = Arc::new(SharedRing::new(4096).unwrap());
            let mut vm = processor(
                source,
                SharedValue::Object(BTreeMap::from([
                    ("region".into(), SharedValue::Region(region.clone())),
                    ("ring".into(), SharedValue::Ring(ring.clone())),
                ])),
            );
            let mut out = [1.0; 128];
            vm.render(&mut out, 64, 0);
            assert!(vm.finished());
            assert!(out.iter().all(|v| *v == 0.0));
            assert!(vm.context.shared.error.get().unwrap().contains("processor"));
            assert!(ring.receive().is_none());
        }
    }
}
