//! The VM state and the call protocol (C++ `ldo.cpp`, `lstate.cpp`).
//!
//! Lane 1 has one thread: one value stack, one frame stack. The stack is a
//! `Vec<Value>` addressed by slot index; a frame's registers are the slots
//! `base .. base + maxstacksize`. Every slot access is a checked `get`; the
//! verifier (D7) and the frame setup below make the checks always succeed.
//!
//! **Calls** are in `call.rs`: every call is a frame in one driver loop,
//! metamethods and library callbacks included; only host callbacks recurse
//! on the Rust stack (`HostedVm::call_value`, bounded by `Limits::max_reentry`).
//! Lua call depth is bounded by `Limits::max_calls`, the C++ C-call count
//! by `Limits::max_c_calls`.
//!
//! **Errors.** A Lua error is a `Throw` carried by `Result`. Frames above
//! the catcher are left in place while it propagates; the catcher (`pcall`,
//! `xpcall` through `Core::find_catcher`, or the host entry points) truncates the frame
//! stack and closes upvalues back to its own level, as C++ `luaD_pcall`
//! does.
//!
//! **Threads** (`thread.rs`): `exec` is the running thread's stack, frames,
//! open upvalues, pc, C-call counters and globals; a switch swaps it with
//! the thread's arena slot. The collector roots the running thread through
//! `exec` (gc.rs) and every other thread through `Thread::trace`.
//!
//! **Safepoints.** Collection runs only in `Core::safepoint` (gc.rs). The
//! rooting rule (heap.rs) holds because every live value is in a stack
//! slot below the high-water mark, a frame, the globals, the registry or
//! the pinned set; see `gc_roots` in gc.rs.
//!
//! `interrupt` marks every point C++ checks `VM_INTERRUPT`; the callback
//! is the host's (`Host::interrupt`), asked through the frame loop's
//! `Dispatch` when `interrupt_on` is set.

use crate::call::RequestSlot;
use crate::error::{ErrorKind, Throw, VmResult};
use crate::heap::{Heap, Upval};
use crate::hosted::Dispatch;
use crate::limits::Limits;
use crate::table::TableError;
use crate::thread::{Exec, Thread, ThreadStatus};
use crate::value::{FnId, ProtoId, StrId, TableId, ThreadId, UpvalId, Value};

/// `Core` is plain data: a VM moves between threads with it
/// (docs/luau-safe-vm-embedding-core.md, section 2).
const _: () = {
    const fn send<T: Send>() {}
    send::<Core>();
};

/// Slots of the main thread's stack when the VM is created.
const INITIAL_STACK: usize = 64;

/// C++ `LUA_MINSTACK`: slots a native function may push without checking.
pub(crate) const MIN_STACK: usize = 20;
/// Physical slots every running Lua frame has from its base: the register
/// window the dispatch loop borrows as `&mut [Value; WINDOW]`, indexed by
/// a `u8` operand with no check (docs/luau-safe-vm-register-window.md,
/// 4.1). Only the physical size: the stack limit binds as without it.
pub(crate) const WINDOW: usize = 256;
/// C++ `LUA_BUFFERSIZE`: runtime error messages are cut to this minus one.
const BUFFER_SIZE: usize = 512;
/// `nresults` for "all results".
pub(crate) const MULTRET: i32 = -1;

/// Metamethod names, in the order of `TM_NAMES`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tm {
    Index,
    NewIndex,
    NameCall,
    Call,
    Iter,
    Len,
    Eq,
    Add,
    Sub,
    Mul,
    Div,
    Idiv,
    Mod,
    Pow,
    Unm,
    Lt,
    Le,
    Concat,
    Type,
    Metatable,
    ToString,
    /// `__mode`, read only by the collector (heap.rs `weak_mode`).
    Mode,
}

const TM_NAMES: [&str; 22] = [
    "__index",
    "__newindex",
    "__namecall",
    "__call",
    "__iter",
    "__len",
    "__eq",
    "__add",
    "__sub",
    "__mul",
    "__div",
    "__idiv",
    "__mod",
    "__pow",
    "__unm",
    "__lt",
    "__le",
    "__concat",
    "__type",
    "__metatable",
    "__tostring",
    "__mode",
];

/// A call frame (C++ `CallInfo`): plain data, 32 bytes
/// (docs/luau-safe-vm-call-path.md, 4.3). What only some frames need (a
/// metamethod's completion, a native's continuation, counters to restore)
/// is on the frame list's pending side stack (`frames.rs`), flagged
/// [`Frame::PENDING`]. Slots fit a `u32`: `Limits::max_stack_slots` is
/// capped below it.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Frame {
    /// The slot holding the called function; results are moved here.
    func: u32,
    /// The first register (Lua) or argument (native).
    base: u32,
    /// One past the last slot the frame may use.
    top: u32,
    /// A Lua frame's `savedpc` (when it is not running: the index of the
    /// instruction after the one that called out); a native frame's
    /// argument count.
    aux: u32,
    pub(crate) closure: FnId,
    /// Meaningful when [`Frame::LUA`] is set.
    proto: ProtoId,
    /// Results the caller wants, or `MULTRET`.
    nresults: i32,
    flags: u8,
}

const _: () = {
    assert!(std::mem::size_of::<Frame>() <= 32);
    assert!(!std::mem::needs_drop::<Frame>());
};

/// A slot or count as a frame stores it.
#[inline]
fn narrow(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

impl Frame {
    /// A Lua frame.
    pub(crate) const LUA: u8 = 1;
    /// Returning from this frame returns from `execute` (C++
    /// `LUA_CALLINFO_RETURN`).
    pub(crate) const BOUNDARY: u8 = 2;
    /// The bottom frame of a thread: returning from it finishes the thread.
    pub(crate) const THREAD_BASE: u8 = 4;
    /// The frame has an entry on the pending side stack.
    pub(crate) const PENDING: u8 = 8;

    #[inline]
    pub(crate) fn lua(
        func: usize,
        base: usize,
        top: usize,
        nresults: i32,
        closure: FnId,
        proto: ProtoId,
    ) -> Frame {
        Frame {
            func: narrow(func),
            base: narrow(base),
            top: narrow(top),
            aux: 0,
            closure,
            proto,
            nresults,
            flags: Frame::LUA,
        }
    }

    #[inline]
    pub(crate) fn native(
        func: usize,
        base: usize,
        top: usize,
        nresults: i32,
        closure: FnId,
        nargs: usize,
    ) -> Frame {
        Frame {
            func: narrow(func),
            base: narrow(base),
            top: narrow(top),
            aux: narrow(nargs),
            closure,
            proto: ProtoId::default(),
            nresults,
            flags: 0,
        }
    }

    #[inline]
    pub(crate) fn func(&self) -> usize {
        self.func as usize
    }

    #[inline]
    pub(crate) fn base(&self) -> usize {
        self.base as usize
    }

    #[inline]
    pub(crate) fn top(&self) -> usize {
        self.top as usize
    }

    /// A Lua frame's `savedpc`.
    #[inline]
    pub(crate) fn savedpc(&self) -> usize {
        self.aux as usize
    }

    #[inline]
    pub(crate) fn set_savedpc(&mut self, pc: usize) {
        self.aux = narrow(pc);
    }

    /// A native frame's argument count.
    #[inline]
    pub(crate) fn nargs(&self) -> usize {
        self.aux as usize
    }

    #[inline]
    pub(crate) fn nresults(&self) -> i32 {
        self.nresults
    }

    /// The prototype of a Lua frame.
    #[inline]
    pub(crate) fn proto(&self) -> Option<ProtoId> {
        (self.flags & Frame::LUA != 0).then_some(self.proto)
    }

    /// The prototype field as stored: meaningful for a Lua frame only
    /// (the caller [`Frames::pop_plain`] returns is one).
    ///
    /// [`Frames::pop_plain`]: crate::frames::Frames::pop_plain
    #[inline]
    pub(crate) fn lua_proto(&self) -> ProtoId {
        self.proto
    }

    /// True when the frame is a Lua frame and nothing else: no pending
    /// entry, not a host boundary, not a thread's base.
    #[inline]
    pub(crate) fn is_plain_lua(&self) -> bool {
        self.flags == Frame::LUA
    }

    #[inline]
    pub(crate) fn has(&self, flag: u8) -> bool {
        self.flags & flag != 0
    }

    #[inline]
    pub(crate) fn set(&mut self, flag: u8) {
        self.flags |= flag;
    }
}

/// The VM: heap, globals, and the running thread's state.
pub struct Core {
    pub(crate) heap: Heap,
    pub(crate) limits: Limits,
    /// The running thread's execution state, checked out of its arena slot
    /// (thread.rs): stack, frames, open upvalues, pc, C-call counters and
    /// globals. A switch is one swap of this struct per side
    /// (docs/luau-safe-vm-call-path.md, 4.4; docs/luau-safe-vm-register-window.md,
    /// 4.8, as built: a boxed `Exec` cost more per call than it saved per
    /// switch).
    pub(crate) exec: Exec,
    pub(crate) registry: TableId,
    /// The metatable shared by all strings (C++ `mt[LUA_TSTRING]`).
    pub(crate) string_meta: Option<TableId>,
    /// The metatable shared by all vectors (C++ `mt[LUA_TVECTOR]`), set by
    /// the vector library.
    pub(crate) vector_meta: Option<TableId>,
    tm_names: Vec<StrId>,
    /// The interned type names, in `Value::type_index` order (GC roots).
    type_names: Vec<StrId>,
    /// The running thread and the main thread.
    pub(crate) current: ThreadId,
    pub(crate) main_thread: ThreadId,
    /// The callee slot plus one of the plain native running frameless
    /// from `CALL`, zero otherwise (docs/luau-safe-vm-call-path.md, 4.1).
    /// An observer of the frames turns it into a real frame first
    /// (`Core::frames_observed`).
    pub(crate) frameless: u32,
    /// Nested `call_value` (Rust-stack) depth.
    pub(crate) reentry: u32,
    /// What the native that just returned asked for (`call.rs`).
    pub(crate) request: RequestSlot,
    /// The method name a `__namecall` metamethod is dispatching (C++
    /// `L->namecall`).
    pub(crate) namecall: Option<StrId>,
    /// Values held by library state outside any table (e.g. the `next`
    /// function `pairs` returns): always roots.
    pub(crate) pinned: Vec<Value>,
    /// Lua errors a host function received from `HostCtx` and has not
    /// returned yet; rooted until the host function returns.
    pub(crate) host_errors: Vec<Value>,
    /// The collector's root set, kept between steps so a steady cycle
    /// allocates nothing (the audio thread steps the collector every
    /// block). Empty outside a step.
    pub(crate) gc_root_scratch: Vec<Value>,
    /// `math.random`'s PCG32 state (C++ `global_State::rngstate`).
    pub(crate) rng_state: u64,
    // ---- embedding ----
    /// Values the host keeps alive (`roots.rs`).
    pub(crate) roots: crate::roots::Roots,
    /// Host callbacks running (the driver's host call depth): 0 at the host's top
    /// level.
    pub(crate) host_depth: u32,
    /// The host has an interrupt (C++ `lua_callbacks()->interrupt` set):
    /// `VM_INTERRUPT` points and the string matcher ask it
    /// (`Host::interrupt`).
    pub(crate) interrupt_on: bool,
    /// C++ `global_State::udatamt`: the metatable per userdata tag.
    pub(crate) tag_metas: Vec<Option<TableId>>,
    /// One bit per userdata tag with direct-field getters (C++
    /// `lua_registeruserdatadirectfieldget`): `GETTABLEKS` asks the host
    /// (`Host::direct_field`) only for those tags.
    pub(crate) direct_field_tags: u128,
    /// `coroutine.resume`, for `Thread::resume` from the host.
    pub(crate) resume_fn: Option<FnId>,
}

/// Bytes, cut at the first NUL and rendered lossily, as the vendored VM's
/// `CStr::to_string_lossy` formatting does.
pub(crate) fn c_lossy(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(bytes.get(..end).unwrap_or_default()).into_owned()
}

impl Core {
    /// A VM with the given limits and no libraries.
    pub(crate) fn with_limits_bare(mut limits: Limits) -> VmResult<Core> {
        // Slots are stored as `u32` (`Core::frameless`); one below the
        // maximum so a slot plus one still fits.
        limits.max_stack_slots = limits.max_stack_slots.min(u32::MAX.saturating_sub(1) as usize);
        let mut heap = Heap::new(limits.max_memory_bytes, limits.max_string_bytes);
        // The memory error message is `StrId(0)` (heap.rs `memory_error`).
        let memerr = heap.intern(b"not enough memory")?;
        if memerr != StrId(0) {
            return Err(heap.memory_error());
        }
        let globals = heap.new_table(0, 32)?;
        let registry = heap.new_table(0, 4)?;
        let mut tm_names = Vec::with_capacity(TM_NAMES.len());
        for name in TM_NAMES {
            tm_names.push(heap.intern(name.as_bytes())?);
        }
        if let Some(&mode) = tm_names.get(Tm::Mode as usize) {
            heap.set_mode_key(mode);
        }
        // Interned once, so `type` and its fast body look a name up
        // (docs/luau-safe-vm-call-path.md, 4.2).
        let mut type_names = Vec::with_capacity(crate::value::TYPE_NAMES.len());
        for name in crate::value::TYPE_NAMES {
            type_names.push(heap.intern(name.as_bytes())?);
        }
        // The main thread's initial stack, an external buffer like every
        // thread stack (budget.rs, rule 4).
        heap.charge_external(INITIAL_STACK.saturating_mul(16))?;
        // The main thread's state is checked out into the fields below.
        let main_thread = heap.new_thread(Thread {
            exec: Exec::default(),
            status: ThreadStatus::Running,
            resumer: None,
            error: Value::Nil,
        })?;
        Ok(Core {
            heap,
            limits,
            exec: Exec {
                stack: vec![Value::Nil; INITIAL_STACK],
                globals,
                ..Exec::default()
            },
            registry,
            string_meta: None,
            vector_meta: None,
            tm_names,
            type_names,
            current: main_thread,
            main_thread,
            frameless: 0,
            reentry: 0,
            request: RequestSlot::default(),
            namecall: None,
            pinned: Vec::new(),
            host_errors: Vec::new(),
            gc_root_scratch: Vec::new(),
            rng_state: {
                // Upstream seeds from the state's address; a fixed seed
                // keeps unseeded runs reproducible.
                let mut state = 0;
                crate::libs::math::pcg32_seed(&mut state, 0);
                state
            },
            roots: crate::roots::Roots::new(),
            host_depth: 0,
            interrupt_on: false,
            tag_metas: Vec::new(),
            direct_field_tags: 0,
            resume_fn: None,
        })
    }

    #[inline]
    pub(crate) fn tm(&self, tm: Tm) -> StrId {
        self.tm_names.get(tm as usize).copied().unwrap_or_default()
    }

    /// The interned metamethod names (GC roots).
    pub(crate) fn tm_names(&self) -> &[StrId] {
        &self.tm_names
    }

    /// The interned type names (GC roots).
    pub(crate) fn type_names(&self) -> &[StrId] {
        &self.type_names
    }

    /// `value`'s type name, interned (C++ `ttname`).
    #[inline]
    pub(crate) fn type_name_str(&self, value: Value) -> StrId {
        self.type_names.get(value.type_index()).copied().unwrap_or_default()
    }

    // ---- stack -------------------------------------------------------

    #[inline]
    pub(crate) fn get(&self, slot: usize) -> Value {
        self.exec.stack.get(slot).copied().unwrap_or_default()
    }

    #[inline]
    pub(crate) fn set(&mut self, slot: usize, value: Value) {
        if let Some(s) = self.exec.stack.get_mut(slot) {
            *s = value;
        }
    }

    /// Grows the stack to at least `len` slots. The check is inline (two
    /// compares); growth is out of line (docs/luau-safe-vm-call-path.md,
    /// 4.3: "stack compare").
    #[inline]
    pub(crate) fn ensure_stack(&mut self, len: usize) -> VmResult<()> {
        if len <= self.exec.stack.len() && len <= self.limits.max_stack_slots {
            return Ok(());
        }
        self.grow_stack(len)
    }

    /// The stack check of a Lua frame entry at `base`: the limit binds on
    /// `need`, as [`Core::ensure_stack`]; physically the stack reaches the
    /// frame's register window as well (`base + WINDOW`), which may lie up
    /// to `WINDOW` slots past the limit (docs/luau-safe-vm-register-
    /// window.md, 4.1). Two compares inline, growth out of line.
    #[inline]
    pub(crate) fn ensure_frame(&mut self, need: usize, base: usize) -> VmResult<()> {
        let physical = need.max(base.saturating_add(WINDOW));
        if physical <= self.exec.stack.len() && need <= self.limits.max_stack_slots {
            return Ok(());
        }
        self.grow_frame(need, physical)
    }

    #[cold]
    #[inline(never)]
    fn grow_frame(&mut self, need: usize, physical: usize) -> VmResult<()> {
        if need > self.limits.max_stack_slots {
            return Err(self.runtime_error("stack overflow"));
        }
        if physical <= self.exec.stack.len() {
            return Ok(());
        }
        // As `grow_stack`, with the window's slots past the limit allowed.
        let grown = physical
            .max(self.exec.stack.len().saturating_mul(2))
            .min(self.limits.max_stack_slots.saturating_add(WINDOW))
            .max(physical);
        let bytes = grown.saturating_sub(self.exec.stack.len()).saturating_mul(16);
        self.heap.charge_external(bytes)?;
        self.exec.stack.resize(grown, Value::Nil);
        Ok(())
    }

    /// The register window of the Lua frame at `base` is missing: the
    /// frame-entry check ([`Core::ensure_frame`]) makes this unreachable.
    #[cold]
    #[inline(never)]
    pub(crate) fn window_error(&mut self) -> Throw {
        self.runtime_error("internal error: register window")
    }

    #[cold]
    #[inline(never)]
    fn grow_stack(&mut self, len: usize) -> VmResult<()> {
        // The limit comes first: the initial stack may be larger than a
        // small configured limit.
        if len > self.limits.max_stack_slots {
            return Err(self.runtime_error("stack overflow"));
        }
        if len <= self.exec.stack.len() {
            return Ok(());
        }
        let grown = len
            .max(self.exec.stack.len().saturating_mul(2))
            .min(self.limits.max_stack_slots);
        let bytes = grown.saturating_sub(self.exec.stack.len()).saturating_mul(16);
        self.heap.charge_external(bytes)?;
        self.exec.stack.resize(grown, Value::Nil);
        Ok(())
    }

    pub(crate) fn push(&mut self, value: Value) -> VmResult<()> {
        self.ensure_stack(self.exec.top.saturating_add(1))?;
        self.set(self.exec.top, value);
        self.exec.top = self.exec.top.saturating_add(1);
        Ok(())
    }

    /// Copies `n` slots from `from` to `to` (ranges may overlap).
    pub(crate) fn copy_slots(&mut self, from: usize, to: usize, n: usize) {
        if to <= from {
            for i in 0..n {
                let v = self.get(from.saturating_add(i));
                self.set(to.saturating_add(i), v);
            }
        } else {
            for i in (0..n).rev() {
                let v = self.get(from.saturating_add(i));
                self.set(to.saturating_add(i), v);
            }
        }
    }

    pub(crate) fn fill_nil(&mut self, from: usize, to: usize) {
        let to = to.min(self.exec.stack.len());
        if let Some(slots) = self.exec.stack.get_mut(from..to) {
            slots.fill(Value::Nil);
        }
    }

    // ---- errors ------------------------------------------------------

    pub(crate) fn throw_string(&mut self, bytes: &[u8], kind: ErrorKind) -> Throw {
        match self.heap.intern(bytes) {
            Ok(id) => Throw {
                value: Value::String(id),
                kind,
            },
            Err(throw) => throw,
        }
    }

    /// Source and line of frame `index` of `frames`, if it is a Lua frame.
    fn frame_position(frames: &crate::frames::Observed<'_>, index: usize) -> Option<(StrId, i32)> {
        let frame = frames.get(index)?;
        let proto = frames.heap.proto(frame.proto()?);
        let source = proto.source?;
        Some((source, proto.line(frames.pc_of(index).saturating_sub(1))))
    }

    fn position_prefix(&self, source: StrId, line: i32) -> Vec<u8> {
        let mut prefix = self.heap.str_bytes(source).to_vec();
        prefix.extend_from_slice(format!(":{line}: ").as_bytes());
        prefix
    }

    /// The host-facing form of a Lua error: strings and numbers as text,
    /// anything else by its type, as C++ `lua_tostring` would fail on it.
    pub(crate) fn error_of(&self, throw: Throw) -> crate::error::Error {
        let message = match throw.value {
            Value::String(id) => self.heap.str_bytes(id).to_vec(),
            Value::Number(n) => crate::num::num2str(n).into_bytes(),
            other => format!("(error object is a {} value)", other.type_name()).into_bytes(),
        };
        crate::error::Error::Lua {
            kind: throw.kind,
            message,
        }
    }

    /// Releases `root` (`roots.rs`): its value is no longer kept alive
    /// for the host. False, and nothing changes, for a stale root or
    /// another VM's.
    pub fn release_root(&mut self, root: crate::roots::Root) -> bool {
        self.roots.release(root)
    }

    /// `lua_topointer`: an identity for objects (stable while the object
    /// lives, unique per object kind), the word of a light userdata, 0
    /// for other values.
    pub(crate) fn pointer_of(&self, value: Value) -> u64 {
        match value {
            Value::String(_)
            | Value::Table(_)
            | Value::Function(_)
            | Value::Thread(_)
            | Value::Buffer(_)
            | Value::Userdata(_) => self.pseudo_address(value),
            Value::LightUserdata(p, _) => p,
            _ => 0,
        }
    }

    // ---- the host's controls -------------------------------------------

    /// Sets the memory limit in bytes (`None`: unlimited) and returns the
    /// previous one. Allocation past it fails with the out-of-memory error
    /// (`ErrorKind::Memory`); what is already allocated stays.
    pub fn set_memory_limit(&mut self, limit: Option<usize>) -> Option<usize> {
        self.heap.budget.set_limit(limit)
    }

    pub fn memory_limit(&self) -> Option<usize> {
        self.heap.budget.limit()
    }

    /// Bytes charged to the budget (objects and stacks).
    pub fn used_memory(&self) -> usize {
        self.heap.total_bytes()
    }

    /// Whether an allocation was refused since the last
    /// [`Core::clear_memory_exhausted`] (seam item 10; the audio engine's
    /// `oom` flag).
    pub fn memory_exhausted(&self) -> bool {
        self.heap.budget.exhausted()
    }

    pub fn clear_memory_exhausted(&self) {
        self.heap.budget.clear_exhausted();
    }

    /// Whether the frame loop asks the host's interrupt
    /// ([`Host::interrupt`](crate::hosted::Host::interrupt)).
    pub fn set_interrupt_enabled(&mut self, on: bool) {
        self.interrupt_on = on;
    }

    /// `lua_gc(LUA_GCSTEP, kb)`: about `kb` KiB worth of collector work;
    /// true when a cycle finished.
    pub fn collect_step(&mut self, kb: i32) -> bool {
        self.gc_step(kb)
    }

    /// C++ `luaL_sandbox`: the library tables, the string metatable and
    /// the global table become read-only, and the globals safe (imports
    /// may be cached). The scripts then run with these globals; C++
    /// `luaL_sandboxthread`, a writable global table per thread over them,
    /// is the lane-6 API's `Vm::sandbox`.
    pub fn sandbox_globals(&mut self) {
        let globals = self.exec.globals;
        let mut cursor = 0usize;
        let mut libs: Vec<TableId> = Vec::new();
        while let Some((next, _, v)) = self.heap.table(globals).next_from(cursor) {
            if let Value::Table(t) = v {
                libs.push(t);
            }
            cursor = next;
        }
        for t in libs {
            self.heap.table_mut(t).readonly = true;
        }
        if let Some(mt) = self.string_meta {
            self.heap.table_mut(mt).readonly = true;
        }
        self.heap.table_mut(globals).readonly = true;
        self.heap.table_mut(globals).safeenv = true;
    }

    /// C++ `luaG_runerror`: the running frame's position (or `":: "` when
    /// it is not a Lua frame), then the message cut to the buffer size.
    pub(crate) fn runtime_error(&mut self, message: impl AsRef<[u8]>) -> Throw {
        let message = message.as_ref();
        let message = message
            .get(..message.len().min(BUFFER_SIZE.saturating_sub(1)))
            .unwrap_or(message);
        let frames = self.frames_observed();
        let mut text = match frames
            .len()
            .checked_sub(1)
            .and_then(|i| Core::frame_position(&frames, i))
        {
            Some((source, line)) => frames.position_prefix(source, line),
            None => b":: ".to_vec(),
        };
        text.extend_from_slice(message);
        self.throw_string(&text, ErrorKind::Runtime)
    }

    /// C++ `luaL_where(level)`: level 0 is the running function, 1 its
    /// caller. Falls back to level 0, then to `":: "`.
    pub(crate) fn where_prefix(&mut self, level: usize) -> Vec<u8> {
        let frames = self.frames_observed();
        for level in [level, 0] {
            let index = frames
                .len()
                .checked_sub(1)
                .and_then(|top| top.checked_sub(level));
            if let Some((source, line)) = index.and_then(|i| Core::frame_position(&frames, i))
                && line > 0
            {
                return frames.position_prefix(source, line);
            }
        }
        b":: ".to_vec()
    }

    /// C++ `luaL_error`: the caller's position, then the message.
    pub(crate) fn lib_error(&mut self, message: impl AsRef<[u8]>) -> Throw {
        let mut text = self.where_prefix(1);
        text.extend_from_slice(message.as_ref());
        self.throw_string(&text, ErrorKind::Runtime)
    }

    /// C++ `luaT_objtypename`: `__type` from a userdata's or basic type's
    /// metatable, else the type name.
    pub(crate) fn obj_type_name(&self, value: Value) -> String {
        c_lossy(self.heap.str_bytes(self.obj_type_name_str(value)))
    }

    /// C++ `luaT_objtypenamestr`: [`Self::obj_type_name`] as the interned
    /// string, which it already is.
    pub(crate) fn obj_type_name_str(&self, value: Value) -> StrId {
        if !matches!(value, Value::Table(_))
            && let Some(mt) = self.metatable(value)
            && let Value::String(name) = self.heap.table(mt).get_str(self.tm(Tm::Type))
        {
            return name;
        }
        self.type_name_str(value)
    }

    /// C++ `luaG_typeerror`.
    pub(crate) fn type_error(&mut self, value: Value, op: &str) -> Throw {
        let name = self.obj_type_name(value);
        self.runtime_error(format!("attempt to {op} a {name} value"))
    }

    /// C++ `luaG_indexerror`.
    pub(crate) fn index_error(&mut self, object: Value, key: Value) -> Throw {
        let t1 = self.obj_type_name(object);
        let message = match key {
            Value::String(id) if self.heap.str_bytes(id).len() <= 64 => {
                format!(
                    "attempt to index {t1} with '{}'",
                    c_lossy(self.heap.str_bytes(id))
                )
            }
            _ => format!("attempt to index {t1} with {}", self.obj_type_name(key)),
        };
        self.runtime_error(message)
    }

    pub(crate) fn table_error(&mut self, error: TableError) -> Throw {
        match error {
            TableError::NilIndex => self.runtime_error("table index is nil"),
            TableError::NanIndex => self.runtime_error("table index is NaN"),
            TableError::VectorNanIndex => self.runtime_error("table index contains NaN"),
            TableError::Overflow => self.runtime_error("table overflow"),
            TableError::Memory => self.heap.memory_error(),
        }
    }

    /// The `LUA_ERRERR` error.
    pub(crate) fn error_in_handler(&mut self) -> Throw {
        self.throw_string(b"error in error handling", ErrorKind::ErrorHandler)
    }

    // ---- raw table access with memory accounting -------------------------

    /// A raw store (`Heap::table_set`): growth is checked against the
    /// memory budget before the table changes, so a failed store leaves
    /// the table as it was.
    pub(crate) fn raw_set(&mut self, table: TableId, key: Value, value: Value) -> VmResult<()> {
        self.heap
            .table_set(table, key, value)
            .map_err(|e| self.table_error(e))
    }

    pub(crate) fn raw_set_str(&mut self, table: TableId, key: &str, value: Value) -> VmResult<()> {
        let key = self.heap.intern(key.as_bytes())?;
        self.raw_set(table, Value::String(key), value)
    }

    pub(crate) fn new_table(&mut self, narray: usize, nhash: usize) -> VmResult<TableId> {
        self.heap.new_table(narray, nhash)
    }

    /// A raw get following `__index` tables (never functions); `None` when
    /// the chain meets a function or a non-table. Used by the loader to
    /// resolve imports without running code.
    pub(crate) fn raw_index_chain(&self, object: Value, key: Value) -> Option<Value> {
        let mut current = object;
        for _ in 0..self.limits.max_meta_chain {
            let Value::Table(id) = current else {
                return None;
            };
            let table = self.heap.table(id);
            let v = table.get(key);
            if !v.is_nil() {
                return Some(v);
            }
            match self.fast_tm(table.meta, Tm::Index) {
                None => return Some(Value::Nil),
                Some(next @ Value::Table(_)) => current = next,
                Some(_) => return None,
            }
        }
        None
    }

    // ---- metatables --------------------------------------------------

    pub(crate) fn metatable(&self, value: Value) -> Option<TableId> {
        match value {
            Value::Table(id) => self.heap.table(id).meta,
            Value::String(_) => self.string_meta,
            Value::Vector(_) => self.vector_meta,
            Value::Userdata(id) => self.heap.userdata(id).meta,
            _ => None,
        }
    }

    /// C++ `fasttm`: the metamethod in `mt`, if present and not nil.
    #[inline]
    pub(crate) fn fast_tm(&self, mt: Option<TableId>, tm: Tm) -> Option<Value> {
        let v = self.heap.table(mt?).get_str(self.tm(tm));
        if v.is_nil() { None } else { Some(v) }
    }

    /// C++ `luaT_gettmbyobj`.
    pub(crate) fn tm_by_obj(&self, value: Value, tm: Tm) -> Value {
        self.fast_tm(self.metatable(value), tm).unwrap_or_default()
    }

    // ---- upvalues ----------------------------------------------------

    /// C++ `luaF_findupval`.
    pub(crate) fn find_upval(&mut self, slot: usize) -> VmResult<UpvalId> {
        match self.exec.open_upvals.binary_search_by_key(&slot, |&(s, _)| s) {
            Ok(i) => match self.exec.open_upvals.get(i) {
                Some(&(_, id)) => Ok(id),
                None => Err(self.heap.memory_error()),
            },
            Err(i) => {
                let Ok(slot32) = u32::try_from(slot) else {
                    return Err(self.heap.memory_error());
                };
                let id = self.heap.new_upval(Upval::Open(self.current, slot32))?;
                if i <= self.exec.open_upvals.len() {
                    self.exec.open_upvals.insert(i, (slot, id));
                }
                Ok(id)
            }
        }
    }

    /// C++ `luaF_close`: closes every open upvalue at or above `level`.
    /// The common case, none, is one compare inline.
    #[inline]
    pub(crate) fn close_upvals(&mut self, level: usize) {
        if self.exec.open_upvals.last().is_some_and(|&(slot, _)| slot >= level) {
            self.close_upvals_from(level);
        }
    }

    #[inline(never)]
    fn close_upvals_from(&mut self, level: usize) {
        while let Some(&(slot, id)) = self.exec.open_upvals.last() {
            if slot < level {
                break;
            }
            self.exec.open_upvals.pop();
            let v = self.get(slot);
            self.heap.set_upval(id, Upval::Closed(v));
        }
    }

    /// An upvalue's value; an open one may name a parked thread's slot.
    #[inline]
    pub(crate) fn upval_get(&self, id: UpvalId) -> Value {
        match self.heap.upval(id) {
            Upval::Open(thread, slot) if thread == self.current => self.get(slot as usize),
            Upval::Open(thread, slot) => self
                .heap
                .thread(thread)
                .exec
                .stack
                .get(slot as usize)
                .copied()
                .unwrap_or_default(),
            Upval::Closed(v) => v,
        }
    }

    #[inline]
    pub(crate) fn upval_set(&mut self, id: UpvalId, value: Value) {
        match self.heap.upval(id) {
            Upval::Open(thread, slot) if thread == self.current => self.set(slot as usize, value),
            Upval::Open(thread, slot) => {
                if let Some(s) = self
                    .heap
                    .thread_mut(thread)
                    .exec
                    .stack
                    .get_mut(slot as usize)
                {
                    *s = value;
                }
            }
            Upval::Closed(_) => self.heap.set_upval(id, Upval::Closed(value)),
        }
    }

    /// C++ `VM_INTERRUPT`: asks the host's interrupt, if it has one, and
    /// raises its error.
    #[inline]
    pub(crate) fn interrupt(&mut self, host: &mut dyn Dispatch) -> VmResult<()> {
        if !self.interrupt_on {
            return Ok(());
        }
        match host.interrupt() {
            Ok(()) => Ok(()),
            Err(e) => Err(self.raise_host_error(e)),
        }
    }
}
