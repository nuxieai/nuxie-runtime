//! Isolated Luau audio lanes. Only this private raw VM crosses threads: no
//! luaur-rt handle, host callback capture, registry TLS or scene object does.
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{c_int, c_void, CString};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

pub(super) use super::luau_audio_shared::{SharedRing, SharedSnapshot, SharedValue};
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
    lua_rawseti::lua_rawseti, lua_ref::lua_ref, lua_setfield::lua_setfield, lua_settop::lua_settop,
    lua_toboolean::lua_toboolean, lua_tobuffer::lua_tobuffer, lua_tolstring::lua_tolstring,
    lua_tonumberx::lua_tonumberx, lua_topointer::lua_topointer, lua_type::lua_type,
    luau_load::luau_load,
};
use luaur_vm::macros::{
    lua_globalsindex::LUA_GLOBALSINDEX, lua_registryindex::LUA_REGISTRYINDEX,
    lua_upvalueindex::lua_upvalueindex,
};
use luaur_vm::records::lua_exception::{lua_exception, LuaResult};
use luaur_vm::type_aliases::lua_state::lua_State;
use nuxie_runtime::{AudioEngine, AudioRenderer, AudioRendererNode};

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
}
impl Shared {
    fn fail(&self, mut error: String) {
        truncate(&mut error, 1024);
        let _ = self.error.set(error);
        let _ = self
            .state
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |state| {
                (state < STOPPED).then_some(FAILED)
            });
    }
}
struct Owner {
    shared: Arc<Shared>,
    engine: AudioEngine,
    node: Mutex<Option<AudioRendererNode>>,
}
impl Owner {
    fn stop(&self) {
        self.shared.cancelled.store(true, Ordering::Release);
        let _ = self
            .shared
            .state
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |s| {
                (s < STOPPED).then_some(STOPPED)
            });
        if let Some(node) = lock(&self.node).take() {
            self.engine.detach_renderer(node);
        }
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.stop();
    }
}
#[derive(Clone)]
pub(super) struct ProcessorHandle(Arc<Owner>);
impl ProcessorHandle {
    pub(super) fn start(
        engine: AudioEngine,
        name: String,
        modules: BTreeMap<String, Arc<[u8]>>,
        input: SharedValue,
        hard_blocks: u32,
        memory_limit: usize,
    ) -> Result<Self, String> {
        if !modules.contains_key(&name) {
            return Err(format!("unknown audio processor module {name}"));
        }
        let shared = Arc::new(Shared {
            state: AtomicU8::new(STARTING),
            cancelled: AtomicBool::new(false),
            error: OnceLock::new(),
            logs: LogRing::new(),
        });
        let owner = Arc::new(Owner {
            shared: shared.clone(),
            engine: engine.clone(),
            node: Mutex::new(None),
        });
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
                    engine.sample_rate(),
                    engine.channels(),
                    1024,
                    hard_blocks,
                    memory_limit,
                )
            }));
            match result {
                Ok(Ok(vm)) => {
                    if let Some(owner) = weak.upgrade() {
                        let mut node = lock(&owner.node);
                        if shared.state.load(Ordering::Acquire) == STARTING {
                            *node = Some(engine.attach_renderer(vm));
                            let _ = shared.state.compare_exchange(
                                STARTING,
                                RUNNING,
                                Ordering::AcqRel,
                                Ordering::Acquire,
                            );
                        }
                    }
                }
                Ok(Err(error)) => shared.fail(error),
                Err(_) => shared.fail("audio processor initialization panicked".into()),
            }
        }))?;
        Ok(Self(owner))
    }
    pub(super) fn state(&self) -> &'static str {
        match self.0.shared.state.load(Ordering::Acquire) {
            STARTING => "starting",
            RUNNING => "running",
            STOPPED => "stopped",
            _ => "failed",
        }
    }
    pub(super) fn error(&self) -> Option<String> {
        self.0.shared.error.get().cloned()
    }
    pub(super) fn take_logs(&self) -> Vec<String> {
        self.0.shared.logs.take()
    }
    pub(super) fn stop(&self) {
        self.0.stop();
    }
}

type InitJob = Box<dyn FnOnce() + Send>;
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

struct Context {
    shared: Arc<Shared>,
    deadline: Option<Instant>,
    expired: bool,
    modules: BTreeMap<String, Arc<[u8]>>,
    loaded: BTreeMap<String, c_int>,
    loading: BTreeSet<String>,
    channels: Vec<SharedValue>,
    bytes: usize,
    memory_limit: usize,
    oom: bool,
    initializing: bool,
    failure: Option<String>,
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
unsafe fn context(l: *mut lua_State) -> *mut Context {
    (*lua_callbacks(l)).userdata.cast()
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
    let ctx = context(l);
    if (*ctx).initializing && (*ctx).shared.cancelled.load(Ordering::Acquire) {
        return error(l, "audio processor stopped");
    }
    if (*ctx).deadline.is_some_and(|end| Instant::now() >= end) {
        (*ctx).expired = true;
    }
    if (*ctx).expired {
        return error(l, "audio processor execution budget exceeded");
    }
    Ok(())
}
unsafe fn diagnostic(l: *mut lua_State, fallback: &str) -> String {
    if lua_type(l, -1) == lua_Type::LUA_TSTRING as c_int {
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
unsafe fn call(l: *mut lua_State, args: c_int, results: c_int) -> Result<(), String> {
    match lua_pcall(l, args, results, 0) {
        Ok(0) => Ok(()),
        _ => Err(diagnostic(l, "audio processor Lua call failed")),
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
        let mut ctx = Box::new(Context {
            shared,
            deadline: Some(Instant::now() + Duration::from_millis(1000)),
            expired: false,
            modules,
            loaded: BTreeMap::new(),
            loading: BTreeSet::new(),
            channels: Vec::new(),
            bytes: 0,
            memory_limit,
            oom: false,
            initializing: true,
            failure: None,
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
            (*lua_callbacks(l)).interrupt = Some(interrupt);
        }
        let result: LuaResult<()> = unsafe {
            (|| {
                lua_l_openlibs(l)?;
                lua_pushcclosurek(l, Some(print), c"print".as_ptr(), 0, None)?;
                lua_setfield(l, LUA_GLOBALSINDEX, c"print".as_ptr())?;
                lua_pushcclosurek(l, Some(require), c"require".as_ptr(), 0, None)?;
                lua_setfield(l, LUA_GLOBALSINDEX, c"require".as_ptr())?;
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
                self.process_block(&mut out[first..end], count as u32, start + offset as u64)
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
        let text =
            String::from_utf8_lossy(std::slice::from_raw_parts(text.cast(), size.min(remaining)));
        line.push_str(&text);
        truncate(&mut line, 1024);
        lua_settop(l, -2)?;
    }
    (*context(l)).shared.logs.push(&line);
    Ok(0)
}
unsafe fn require(l: *mut lua_State) -> LuaResult<c_int> {
    if lua_type(l, 1) != lua_Type::LUA_TSTRING as c_int {
        return error(l, "require expects module name");
    }
    let mut size = 0;
    let p = lua_tolstring(l, 1, &mut size)?;
    let name = String::from_utf8_lossy(std::slice::from_raw_parts(p.cast(), size)).into_owned();
    load_module(l, &name)?;
    Ok(1)
}
unsafe fn load_module(l: *mut lua_State, name: &str) -> LuaResult<()> {
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

unsafe fn push(l: *mut lua_State, value: &SharedValue) -> LuaResult<()> {
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
        SharedValue::Ring(_) | SharedValue::Snapshot(_) => {
            let ctx = context(l);
            let index = (*ctx).channels.len();
            (*ctx).channels.push(value.clone());
            lua_createtable(l, 0, 4)?;
            let methods: &[(
                &std::ffi::CStr,
                unsafe fn(*mut lua_State) -> LuaResult<c_int>,
            )] = match value {
                SharedValue::Ring(_) => &[
                    (c"send", ring_send),
                    (c"receive", ring_receive),
                    (c"receiveInto", ring_receive_into),
                ],
                _ => &[(c"publish", snapshot_publish), (c"latest", snapshot_latest)],
            };
            for (name, method) in methods {
                lua_pushinteger(l, index as c_int)?;
                lua_pushcclosurek(l, Some(*method), name.as_ptr(), 1, None)?;
                lua_setfield(l, -2, name.as_ptr())?;
            }
        }
    }
    Ok(())
}
unsafe fn channel(l: *mut lua_State) -> SharedValue {
    let i = lua_tonumberx(l, lua_upvalueindex(1), std::ptr::null_mut()) as usize;
    (*context(l)).channels[i].clone()
}
unsafe fn read(
    l: *mut lua_State,
    index: c_int,
    seen: &mut BTreeSet<usize>,
) -> LuaResult<SharedValue> {
    if seen.len() >= 200 {
        return error(l, "shared payload exceeds Luau nesting limit");
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
                        return error(l, "shared arrays need consecutive positive integer keys");
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
unsafe fn ring_send(l: *mut lua_State) -> LuaResult<c_int> {
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
unsafe fn ring_receive(l: *mut lua_State) -> LuaResult<c_int> {
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
unsafe fn ring_receive_into(l: *mut lua_State) -> LuaResult<c_int> {
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
unsafe fn snapshot_publish(l: *mut lua_State) -> LuaResult<c_int> {
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
unsafe fn snapshot_latest(l: *mut lua_State) -> LuaResult<c_int> {
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
        Arc::new(Shared {
            state: AtomicU8::new(RUNNING),
            cancelled: AtomicBool::new(false),
            error: OnceLock::new(),
            logs: LogRing::new(),
        })
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
        assert!(messages
            .send(SharedValue::Object(BTreeMap::from([(
                "gain".into(),
                SharedValue::Number(0.25)
            )])))
            .unwrap());
        vm = std::thread::spawn(move || {
            let mut out = [0.0; 128];
            vm.render(&mut out, 64, 123);
            assert!(out.chunks_exact(2).all(|p| p == [0.25, -0.25]));
            vm
        })
        .join()
        .unwrap();
        assert!(messages
            .send(SharedValue::Object(BTreeMap::from([(
                "gain".into(),
                SharedValue::Number(0.5)
            )])))
            .unwrap());
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
        let mut good=processor("return {process=function(self,out,n) for i=0,n*2-1 do buffer.writef32(out,i*4,0.25) end end}",SharedValue::Nil);
        let mut out = [1.0; 128];
        broken.render(&mut out, 64, 0);
        assert!(out.iter().all(|v| *v == 0.0));
        assert!(broken.finished());
        assert!(broken
            .context
            .shared
            .error
            .get()
            .unwrap()
            .contains("budget"));
        good.render(&mut out, 64, 0);
        assert!(out.iter().all(|v| *v == 0.25));
        assert!(!good.finished());
    }
    #[test]
    fn failed_chunk_preserves_preceding_successful_chunk() {
        let mut vm=processor("return {process=function(self,out,n,start) if start>=1024 then error('bad block') end for i=0,n*2-1 do buffer.writef32(out,i*4,0.125) end end}",SharedValue::Nil);
        let mut out = vec![0.0; 4096];
        vm.render(&mut out, 2048, 0);
        assert!(out[..2048].iter().all(|v| *v == 0.125));
        assert!(out[2048..].iter().all(|v| *v == 0.0));
        assert!(vm.finished());
    }
    #[test]
    fn caught_memory_exhaustion_is_terminal() {
        let mut vm=processor("return {process=function(self,out,n) pcall(function() buffer.create(32*1024*1024) end) end}",SharedValue::Nil);
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
}
