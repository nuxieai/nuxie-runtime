//! The VM state and the call protocol (C++ `ldo.cpp`, `lstate.cpp`).
//!
//! Lane 1 has one thread: one value stack, one frame stack. The stack is a
//! `Vec<Value>` addressed by slot index; a frame's registers are the slots
//! `base .. base + maxstacksize`. Every slot access is a checked `get`; the
//! verifier (D7) and the frame setup below make the checks always succeed.
//!
//! **Calls** are in `call.rs`: every call is a frame in one driver loop,
//! metamethods and library callbacks included; only host callbacks recurse
//! on the Rust stack (`Vm::call_value`, bounded by `Limits::max_reentry`).
//! Lua call depth is bounded by `Limits::max_calls`, the C++ C-call count
//! by `Limits::max_c_calls`.
//!
//! **Errors.** A Lua error is a `Throw` carried by `Result`. Frames above
//! the catcher are left in place while it propagates; the catcher (`pcall`,
//! `xpcall` in `Vm::catch`, or the host entry points) truncates the frame
//! stack and closes upvalues back to its own level, as C++ `luaD_pcall`
//! does.
//!
//! **Threads** (`thread.rs`): the fields `stack`, `top`, `frames`,
//! `open_upvals`, `pc`, `c_calls`, `base_c_calls`, `epoch`, `resume_calls`
//! and `ci_overflow` are the running thread's; a switch swaps them with the
//! thread's arena slot. The collector roots the running thread through
//! these fields (gc.rs) and every other thread through `Thread::trace`.
//!
//! **Safepoints.** Collection runs only in `Vm::safepoint` (gc.rs). The
//! rooting rule (heap.rs) holds because every live value is in a stack
//! slot below the high-water mark, a frame, the globals, the registry or
//! the pinned set; see `gc_roots` in gc.rs.
//!
//! `interrupt` marks every point C++ checks `VM_INTERRUPT`; the interrupt
//! callback hangs off it.

use std::rc::Rc;

use crate::bytecode::Proto;
use crate::call::{Request, Saved, Wait};
use crate::error::{ErrorKind, Throw, VmResult};
use crate::heap::{Heap, Upval};
use crate::limits::Limits;
use crate::table::TableError;
use crate::thread::{Exec, Thread, ThreadStatus};
use crate::value::{FnId, StrId, TableId, ThreadId, UpvalId, Value};

/// C++ `LUA_MINSTACK`: slots a native function may push without checking.
pub(crate) const MIN_STACK: usize = 20;
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

/// A call frame (C++ `CallInfo`).
#[derive(Default)]
pub(crate) struct Frame {
    /// The slot holding the called function; results are moved here.
    pub(crate) func: usize,
    /// The first register (Lua) or argument (native).
    pub(crate) base: usize,
    /// One past the last slot the frame may use.
    pub(crate) top: usize,
    /// For a Lua frame that is not running: the index of the instruction
    /// after the one that called out (C++ `savedpc`).
    pub(crate) savedpc: usize,
    /// Results the caller wants, or `MULTRET`.
    pub(crate) nresults: i32,
    pub(crate) closure: FnId,
    /// Set for Lua frames.
    pub(crate) proto: Option<Rc<Proto>>,
    /// A native frame's argument count.
    pub(crate) nargs: usize,
    /// Returning from this frame returns from `execute` (C++
    /// `LUA_CALLINFO_RETURN`).
    pub(crate) boundary: bool,
    /// The bottom frame of a thread: returning from it finishes the thread.
    pub(crate) thread_base: bool,
    /// For a counted call: the C-call counters to restore when it returns.
    pub(crate) restore: Option<Saved>,
    /// What the frame waits for (`call.rs`).
    pub(crate) wait: Wait,
}

/// The VM: heap, globals, and the running thread's state.
pub struct Vm {
    pub(crate) heap: Heap,
    pub(crate) limits: Limits,
    pub(crate) stack: Vec<Value>,
    pub(crate) top: usize,
    pub(crate) frames: Vec<Frame>,
    /// Open upvalues, sorted by stack slot.
    pub(crate) open_upvals: Vec<(usize, UpvalId)>,
    pub(crate) globals: TableId,
    pub(crate) registry: TableId,
    /// The metatable shared by all strings (C++ `mt[LUA_TSTRING]`).
    pub(crate) string_meta: Option<TableId>,
    tm_names: Vec<StrId>,
    /// C++ `nCcalls` of the running thread.
    pub(crate) c_calls: u32,
    /// C++ `baseCcalls` of the running thread.
    pub(crate) base_c_calls: u32,
    /// Resumes of the running thread so far: saved counters from an older
    /// epoch are stale (`call::Saved`).
    pub(crate) epoch: u32,
    /// The C-call count the running thread was last resumed with.
    pub(crate) resume_calls: u32,
    /// The running thread overflowed its call limit and has the extra
    /// room C++ grants until it unwinds (`call::check_call_depth`).
    pub(crate) ci_overflow: bool,
    /// The running Lua frame's `savedpc`.
    pub(crate) pc: usize,
    /// The running thread and the main thread.
    pub(crate) current: ThreadId,
    pub(crate) main_thread: ThreadId,
    /// Nested `call_value` (Rust-stack) depth.
    pub(crate) reentry: u32,
    /// What the native that just returned asked for (`call.rs`).
    pub(crate) request: Option<Request>,
    /// The method name a `__namecall` metamethod is dispatching (C++
    /// `L->namecall`).
    pub(crate) namecall: Option<StrId>,
    /// Values held by library state outside any table (e.g. the `next`
    /// function `pairs` returns): always roots.
    pub(crate) pinned: Vec<Value>,
    /// Lua errors a host function received from `HostCtx` and has not
    /// returned yet; rooted until the host function returns.
    pub(crate) host_errors: Vec<Value>,
}

/// Bytes, cut at the first NUL and rendered lossily, as the vendored VM's
/// `CStr::to_string_lossy` formatting does.
pub(crate) fn c_lossy(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(bytes.get(..end).unwrap_or_default()).into_owned()
}

impl Vm {
    /// A VM with the given limits and no libraries.
    pub(crate) fn with_limits_bare(limits: Limits) -> VmResult<Vm> {
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
        // The main thread's state is checked out into the fields below.
        let main_thread = heap.new_thread(Thread {
            exec: Exec::default(),
            status: ThreadStatus::Running,
            resumer: None,
            error: Value::Nil,
        })?;
        Ok(Vm {
            heap,
            limits,
            stack: vec![Value::Nil; 64],
            top: 0,
            frames: Vec::new(),
            open_upvals: Vec::new(),
            globals,
            registry,
            string_meta: None,
            tm_names,
            c_calls: 0,
            base_c_calls: 0,
            epoch: 0,
            resume_calls: 0,
            ci_overflow: false,
            pc: 0,
            current: main_thread,
            main_thread,
            reentry: 0,
            request: None,
            namecall: None,
            pinned: Vec::new(),
            host_errors: Vec::new(),
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

    // ---- stack -------------------------------------------------------

    #[inline]
    pub(crate) fn get(&self, slot: usize) -> Value {
        self.stack.get(slot).copied().unwrap_or_default()
    }

    #[inline]
    pub(crate) fn set(&mut self, slot: usize, value: Value) {
        if let Some(s) = self.stack.get_mut(slot) {
            *s = value;
        }
    }

    /// Grows the stack to at least `len` slots.
    pub(crate) fn ensure_stack(&mut self, len: usize) -> VmResult<()> {
        // The limit comes first: the initial stack may be larger than a
        // small configured limit.
        if len > self.limits.max_stack_slots {
            return Err(self.runtime_error("stack overflow"));
        }
        if len <= self.stack.len() {
            return Ok(());
        }
        let grown = len
            .max(self.stack.len().saturating_mul(2))
            .min(self.limits.max_stack_slots);
        let bytes = grown.saturating_sub(self.stack.len()).saturating_mul(16);
        self.heap.charge_external(bytes)?;
        self.stack.resize(grown, Value::Nil);
        Ok(())
    }

    pub(crate) fn push(&mut self, value: Value) -> VmResult<()> {
        self.ensure_stack(self.top.saturating_add(1))?;
        self.set(self.top, value);
        self.top = self.top.saturating_add(1);
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
        for slot in self.stack.iter_mut().take(to).skip(from) {
            *slot = Value::Nil;
        }
    }

    /// A free slot above everything the running frame uses, for pushing a
    /// call (C++ uses `L->top`, which equals the frame top outside of
    /// multiple-result sequences).
    pub(crate) fn scratch(&self) -> usize {
        let frame_top = self.frames.last().map_or(0, |f| f.top);
        self.top.max(frame_top)
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

    /// Source and line of frame `index`, if it is a Lua frame.
    fn frame_position(&self, index: usize) -> Option<(StrId, i32)> {
        let frame = self.frames.get(index)?;
        let proto = frame.proto.as_ref()?;
        let savedpc = if index.saturating_add(1) == self.frames.len() {
            self.pc
        } else {
            frame.savedpc
        };
        let source = proto.source?;
        Some((source, proto.line(savedpc.saturating_sub(1))))
    }

    fn position_prefix(&self, source: StrId, line: i32) -> Vec<u8> {
        let mut prefix = self.heap.str_bytes(source).to_vec();
        prefix.extend_from_slice(format!(":{line}: ").as_bytes());
        prefix
    }

    /// C++ `luaG_runerror`: the running frame's position (or `":: "` when
    /// it is not a Lua frame), then the message cut to the buffer size.
    pub(crate) fn runtime_error(&mut self, message: impl AsRef<[u8]>) -> Throw {
        let message = message.as_ref();
        let message = message
            .get(..message.len().min(BUFFER_SIZE.saturating_sub(1)))
            .unwrap_or(message);
        let mut text = match self
            .frames
            .len()
            .checked_sub(1)
            .and_then(|i| self.frame_position(i))
        {
            Some((source, line)) => self.position_prefix(source, line),
            None => b":: ".to_vec(),
        };
        text.extend_from_slice(message);
        self.throw_string(&text, ErrorKind::Runtime)
    }

    /// C++ `luaL_where(level)`: level 0 is the running function, 1 its
    /// caller. Falls back to level 0, then to `":: "`.
    pub(crate) fn where_prefix(&self, level: usize) -> Vec<u8> {
        for level in [level, 0] {
            let index = self
                .frames
                .len()
                .checked_sub(1)
                .and_then(|top| top.checked_sub(level));
            if let Some((source, line)) = index.and_then(|i| self.frame_position(i))
                && line > 0
            {
                return self.position_prefix(source, line);
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
        if !matches!(value, Value::Table(_))
            && let Some(mt) = self.metatable(value)
            && let Value::String(name) = self.heap.table(mt).get_str(self.tm(Tm::Type))
        {
            return c_lossy(self.heap.str_bytes(name));
        }
        value.type_name().to_owned()
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
        match self.open_upvals.binary_search_by_key(&slot, |&(s, _)| s) {
            Ok(i) => match self.open_upvals.get(i) {
                Some(&(_, id)) => Ok(id),
                None => Err(self.heap.memory_error()),
            },
            Err(i) => {
                let Ok(slot32) = u32::try_from(slot) else {
                    return Err(self.heap.memory_error());
                };
                let id = self.heap.new_upval(Upval::Open(self.current, slot32))?;
                if i <= self.open_upvals.len() {
                    self.open_upvals.insert(i, (slot, id));
                }
                Ok(id)
            }
        }
    }

    /// C++ `luaF_close`: closes every open upvalue at or above `level`.
    pub(crate) fn close_upvals(&mut self, level: usize) {
        while let Some(&(slot, id)) = self.open_upvals.last() {
            if slot < level {
                break;
            }
            self.open_upvals.pop();
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

    // ---- calls -------------------------------------------------------
    //
    // The call protocol is in `call.rs`. These are the metamethod helpers
    // library code calls from Rust (bounded by `Limits::max_reentry`).

    /// Calls `f(args..)` from Rust with `nresults` results (0 or 1) and
    /// returns the first, restoring `top`.
    pub(crate) fn call_function(
        &mut self,
        f: Value,
        args: &[Value],
        nresults: i32,
    ) -> VmResult<Value> {
        let saved_top = self.top;
        let at = self.scratch();
        self.ensure_stack(
            at.saturating_add(1)
                .saturating_add(args.len())
                .saturating_add(MIN_STACK),
        )?;
        self.set(at, f);
        for (i, &v) in args.iter().enumerate() {
            self.set(at.saturating_add(1).saturating_add(i), v);
        }
        self.top = at.saturating_add(1).saturating_add(args.len());
        self.call_value(at, args.len(), nresults)?;
        let result = if nresults > 0 {
            self.get(at)
        } else {
            Value::Nil
        };
        self.top = saved_top;
        Ok(result)
    }

    /// Restores the thread after an error caught from Rust (C++
    /// `luaD_pcall`'s unwinding): closes upvalues at or above `func`, drops
    /// the frames above `depth` and restores the C-call counters.
    pub(crate) fn unwind(&mut self, depth: usize, func: usize, saved: crate::call::Saved) {
        self.close_upvals(func);
        self.frames.truncate(depth);
        if self.frames.len().saturating_add(1) < self.limits.max_calls {
            self.ci_overflow = false;
        }
        self.restore_calls(saved);
    }

    /// Lane 4 seam: C++ `VM_INTERRUPT`. There is no interrupt callback yet.
    #[inline]
    pub(crate) fn interrupt(&mut self) -> VmResult<()> {
        Ok(())
    }
}
