//! The audio processor on the vendored VM (`luaur-vm`), driven through its
//! raw C-style API: the `vendored-vm` opt-out's processor. By default the
//! processor is `safe.rs` instead.
use super::*;
use std::collections::BTreeSet;
use std::ffi::{CString, c_int, c_void};
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

struct Context {
    shared: Arc<Shared>,
    deadline: Option<Instant>,
    expired: bool,
    modules: BTreeMap<String, Arc<[u8]>>,
    loaded: BTreeMap<String, c_int>,
    loading: BTreeSet<String>,
    channels: Vec<SharedValue>,
    pcm_readers: Vec<super::super::luau_audio_samples::PcmStreamReader>,
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
pub(super) struct RawProcessor {
    state: *mut lua_State,
    context: Box<Context>,
    object: c_int,
    process: c_int,
    output: c_int,
    sample_rate: u32,
    pub(super) channels: u32,
    pub(super) max_frames: u32,
    pub(super) hard_blocks: u32,
    pub(super) warm: bool,
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
    /// The lane's state, shared with its handle (the browser kernel's and
    /// the tests' view; native hosts reach it through the handle).
    #[cfg(any(all(test, feature = "compiler"), target_arch = "wasm32"))]
    pub(super) fn shared(&self) -> &Arc<Shared> {
        &self.context.shared
    }
    pub(super) fn new(
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
        super::super::luau_audio_web::require_host()?;
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
        let descriptor = match super::super::luau_audio_samples::PcmStreamDescriptor::from_shared(&value) {
            Ok(value) => value,
            Err(e) => return host_error(l, &e),
        };
        let reader = match super::super::luau_audio_samples::PcmStreamReader::new(descriptor) {
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
