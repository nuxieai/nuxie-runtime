//! The VM state and the call protocol (C++ `ldo.cpp`, `lstate.cpp`).
//!
//! Lane 1 has one thread: one value stack, one frame stack. The stack is a
//! `Vec<Value>` addressed by slot index; a frame's registers are the slots
//! `base .. base + maxstacksize`. Every slot access is a checked `get`; the
//! verifier (D7) and the frame setup below make the checks always succeed.
//!
//! **Calls.** Lua-to-Lua calls made by `CALL` push a frame and continue in
//! the same `execute` loop (no Rust recursion), bounded by
//! `Limits::max_calls`. Everything that re-enters the interpreter from Rust
//! (metamethods, `pcall`, host callbacks) goes through [`Vm::call_value`],
//! bounded by `Limits::max_c_calls`, which is what bounds the Rust stack.
//!
//! **Errors.** A Lua error is a `Throw` carried by `Result`. Frames above
//! the catcher are left in place while it propagates; the catcher (`pcall`,
//! `xpcall`, or the host entry points) truncates the frame stack and closes
//! upvalues back to its own level, as C++ `luaD_pcall` does.
//!
//! **Safepoints.** Collection runs only in [`Vm::safepoint`]. The rooting
//! rule (heap.rs) holds because every live value is in a stack slot below
//! the high-water mark, a frame, the globals, the registry or the pinned
//! set; see `gc_roots`.
//!
//! Seam for lane 4 (coroutines): the thread state (stack, frames, open
//! upvalues, `top`) moves into a `Thread` in its own arena; `Vm` keeps the
//! running thread's id. `interrupt` marks every point C++ checks
//! `VM_INTERRUPT`; the interrupt callback and yields hang off it.

use std::rc::Rc;

use crate::bytecode::Proto;
use crate::error::{ErrorKind, Throw, VmResult};
use crate::heap::{CollectStats, Function, Heap, Upval};
use crate::host::{HostCtx, HostError, HostErrorKind};
use crate::limits::Limits;
use crate::native::{Args, HostFn, NativeFn};
use crate::table::{Table, TableError};
use crate::value::{FnId, StrId, TableId, UpvalId, Value};

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
}

const TM_NAMES: [&str; 21] = [
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
];

/// A call frame (C++ `CallInfo`).
#[derive(Clone)]
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
    /// Returning from this frame returns from `execute` (C++
    /// `LUA_CALLINFO_RETURN`).
    pub(crate) boundary: bool,
}

/// The VM: heap, globals and the one thread of lane 1.
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
    /// Nested `call_value` depth (C++ `nCcalls`).
    pub(crate) c_calls: u32,
    /// The running Lua frame's `savedpc`.
    pub(crate) pc: usize,
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
        let globals = heap.new_table(Table::new(0, 32))?;
        let registry = heap.new_table(Table::new(0, 4))?;
        let mut tm_names = Vec::with_capacity(TM_NAMES.len());
        for name in TM_NAMES {
            tm_names.push(heap.intern(name.as_bytes())?);
        }
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
            pc: 0,
            namecall: None,
            pinned: Vec::new(),
            host_errors: Vec::new(),
        })
    }

    #[inline]
    pub(crate) fn tm(&self, tm: Tm) -> StrId {
        self.tm_names.get(tm as usize).copied().unwrap_or_default()
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
        if len <= self.stack.len() {
            return Ok(());
        }
        if len > self.limits.max_stack_slots {
            return Err(self.runtime_error("stack overflow"));
        }
        let grown = len
            .max(self.stack.len().saturating_mul(2))
            .min(self.limits.max_stack_slots);
        let bytes = grown.saturating_sub(self.stack.len()).saturating_mul(16);
        self.heap.charge(bytes)?;
        self.heap.external_bytes = self.heap.external_bytes.saturating_add(bytes);
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
            TableError::Overflow => self.runtime_error("table overflow"),
        }
    }

    /// The `LUA_ERRERR` error.
    pub(crate) fn error_in_handler(&mut self) -> Throw {
        self.throw_string(b"error in error handling", ErrorKind::ErrorHandler)
    }

    // ---- raw table access with memory accounting -------------------------

    /// A raw store that charges table growth to the memory budget.
    pub(crate) fn raw_set(&mut self, table: TableId, key: Value, value: Value) -> VmResult<()> {
        let before = self.heap.table(table).capacity_bytes();
        let result = self.heap.table_mut(table).set(key, value);
        let after = self.heap.table(table).capacity_bytes();
        result.map_err(|e| self.table_error(e))?;
        if after > before {
            self.heap.charge(after.saturating_sub(before))?;
        }
        Ok(())
    }

    pub(crate) fn raw_set_str(&mut self, table: TableId, key: &str, value: Value) -> VmResult<()> {
        let key = self.heap.intern(key.as_bytes())?;
        self.raw_set(table, Value::String(key), value)
    }

    pub(crate) fn new_table(&mut self, narray: usize, nhash: usize) -> VmResult<TableId> {
        let narray = narray.min(crate::table::MAXSIZE);
        let nhash = nhash.min(crate::table::MAXSIZE);
        self.heap.check_alloc(
            narray
                .saturating_mul(16)
                .saturating_add(nhash.saturating_mul(40)),
        )?;
        self.heap.new_table(Table::new(narray, nhash))
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
                let id = self.heap.new_upval(Upval::Open(slot32))?;
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

    #[inline]
    pub(crate) fn upval_get(&self, id: UpvalId) -> Value {
        match self.heap.upval(id) {
            Upval::Open(slot) => self.get(slot as usize),
            Upval::Closed(v) => v,
        }
    }

    #[inline]
    pub(crate) fn upval_set(&mut self, id: UpvalId, value: Value) {
        match self.heap.upval(id) {
            Upval::Open(slot) => self.set(slot as usize, value),
            Upval::Closed(_) => self.heap.set_upval(id, Upval::Closed(value)),
        }
    }

    // ---- calls -------------------------------------------------------

    /// Records the running Lua frame's pc before a frame is pushed above it.
    #[inline]
    fn save_caller_pc(&mut self) {
        if let Some(frame) = self.frames.last_mut()
            && frame.proto.is_some()
        {
            frame.savedpc = self.pc;
        }
    }

    fn check_call_depth(&mut self) -> VmResult<()> {
        let depth = self.frames.len();
        let max = self.limits.max_calls;
        if depth == max {
            return Err(self.runtime_error("stack overflow"));
        }
        if depth > max.saturating_add(200) {
            return Err(self.error_in_handler());
        }
        Ok(())
    }

    /// C++ `luaD_call`: calls the value at `func` with the `nargs` values
    /// above it. Results are left at `func ..`: exactly `nresults` of them,
    /// or all of them with `top` after the last when `nresults` is
    /// `MULTRET`.
    pub(crate) fn call_value(&mut self, func: usize, nargs: usize, nresults: i32) -> VmResult<()> {
        self.c_calls = self.c_calls.saturating_add(1);
        let max = self.limits.max_c_calls;
        if self.c_calls >= max {
            let error = if self.c_calls == max {
                Some(self.runtime_error("C stack overflow"))
            } else if self.c_calls >= max.saturating_add(max / 8) {
                Some(self.error_in_handler())
            } else {
                None
            };
            if let Some(error) = error {
                self.c_calls = self.c_calls.saturating_sub(1);
                return Err(error);
            }
        }
        let saved_pc = self.pc;
        let result = self.call_inner(func, nargs, nresults);
        self.pc = saved_pc;
        self.c_calls = self.c_calls.saturating_sub(1);
        result
    }

    fn call_inner(&mut self, func: usize, nargs: usize, nresults: i32) -> VmResult<()> {
        let (id, nargs) = self.resolve_callable(func, nargs)?;
        if matches!(self.heap.function(id), Function::Lua(_)) {
            self.enter_lua(func, id, nargs, nresults, true)?;
            self.execute()
        } else {
            self.call_native(func, id, nargs, nresults)
        }
    }

    /// The function to call at `func`: the value itself, or its `__call`
    /// metamethod with the value shifted in as the first argument (C++
    /// `luaV_tryfuncTM`).
    pub(crate) fn resolve_callable(
        &mut self,
        func: usize,
        nargs: usize,
    ) -> VmResult<(FnId, usize)> {
        let callee = self.get(func);
        if let Value::Function(id) = callee {
            return Ok((id, nargs));
        }
        let tm = self.tm_by_obj(callee, Tm::Call);
        let Value::Function(id) = tm else {
            return Err(self.type_error(callee, "call"));
        };
        let end = func.saturating_add(nargs).saturating_add(2);
        self.ensure_stack(end.saturating_add(MIN_STACK))?;
        self.copy_slots(func, func.saturating_add(1), nargs.saturating_add(1));
        self.set(func, tm);
        self.top = end;
        Ok((id, nargs.saturating_add(1)))
    }

    /// Pushes a Lua frame for the closure at `func` (C++ `luaD_precall`
    /// plus `PREPVARARGS`, done here so the frame layout is fixed from the
    /// first instruction).
    pub(crate) fn enter_lua(
        &mut self,
        func: usize,
        id: FnId,
        nargs: usize,
        nresults: i32,
        boundary: bool,
    ) -> VmResult<()> {
        self.check_call_depth()?;
        let Function::Lua(closure) = self.heap.function(id) else {
            return Err(self.type_error(Value::Function(id), "call"));
        };
        let proto = Rc::clone(&closure.proto);
        self.save_caller_pc();
        let numparams = usize::from(proto.numparams);
        let maxstack = usize::from(proto.maxstacksize);
        let args = func.saturating_add(1);
        let argtop = args.saturating_add(nargs.max(numparams));
        let need = argtop
            .saturating_add(numparams)
            .saturating_add(maxstack)
            .saturating_add(MIN_STACK);
        self.ensure_stack(need)?;
        self.fill_nil(args.saturating_add(nargs), argtop);
        let base = if proto.is_vararg {
            for i in 0..numparams {
                let v = self.get(args.saturating_add(i));
                self.set(argtop.saturating_add(i), v);
                self.set(args.saturating_add(i), Value::Nil);
            }
            argtop
        } else {
            args
        };
        let top = base.saturating_add(maxstack);
        self.fill_nil(base.saturating_add(numparams), top);
        self.frames.push(Frame {
            func,
            base,
            top,
            savedpc: 0,
            nresults,
            closure: id,
            proto: Some(proto),
            boundary,
        });
        self.top = top;
        Ok(())
    }

    /// Calls the native function `id` at `func` and moves its results.
    pub(crate) fn call_native(
        &mut self,
        func: usize,
        id: FnId,
        nargs: usize,
        nresults: i32,
    ) -> VmResult<()> {
        self.check_call_depth()?;
        let Function::Native(native) = self.heap.function(id) else {
            return Err(self.type_error(Value::Function(id), "call"));
        };
        let f = native.func.clone();
        self.save_caller_pc();
        let base = func.saturating_add(1);
        let argtop = base.saturating_add(nargs);
        self.ensure_stack(argtop.saturating_add(MIN_STACK))?;
        self.top = argtop;
        self.frames.push(Frame {
            func,
            base,
            top: argtop.saturating_add(MIN_STACK),
            savedpc: 0,
            nresults,
            closure: id,
            proto: None,
            boundary: false,
        });
        let args = Args { base, nargs };
        let n = match f {
            NativeFn::Builtin(b) | NativeFn::BuiltinWith(b, _) => b(self, args)?,
            NativeFn::Host(h) => self.call_host(&h, args)?,
        };
        let n = n.min(self.top.saturating_sub(base));
        let first = self.top.saturating_sub(n);
        self.frames.pop();
        self.move_results(func, first, n, nresults)?;
        self.safepoint();
        Ok(())
    }

    fn call_host(&mut self, h: &Rc<HostFn>, args: Args) -> VmResult<usize> {
        let pending = self.host_errors.len();
        let result = {
            let mut ctx = HostCtx { vm: self, args };
            h(&mut ctx)
        };
        self.host_errors.truncate(pending);
        match result {
            Ok(n) => Ok(n),
            Err(HostError(HostErrorKind::Message(message))) => Err(self.lib_error(message)),
            Err(HostError(HostErrorKind::Raw(message))) => {
                Err(self.throw_string(message.as_bytes(), ErrorKind::Runtime))
            }
            Err(HostError(HostErrorKind::Throw(throw))) => Err(throw),
        }
    }

    /// Moves `n` results at `first` to `func ..`, adjusting to `nresults`.
    pub(crate) fn move_results(
        &mut self,
        func: usize,
        first: usize,
        n: usize,
        nresults: i32,
    ) -> VmResult<()> {
        if let Ok(want) = usize::try_from(nresults) {
            self.ensure_stack(func.saturating_add(want))?;
            let copy = n.min(want);
            self.copy_slots(first, func, copy);
            self.fill_nil(func.saturating_add(copy), func.saturating_add(want));
            self.top = func.saturating_add(want);
        } else {
            self.copy_slots(first, func, n);
            self.top = func.saturating_add(n);
        }
        Ok(())
    }

    /// C++ `callTMres`: calls `f(a, b)` and returns its first result.
    pub(crate) fn call_tm_res(&mut self, f: Value, a: Value, b: Value) -> VmResult<Value> {
        let saved_top = self.top;
        let at = self.scratch();
        self.ensure_stack(at.saturating_add(3).saturating_add(MIN_STACK))?;
        self.set(at, f);
        self.set(at.saturating_add(1), a);
        self.set(at.saturating_add(2), b);
        self.top = at.saturating_add(3);
        self.call_value(at, 2, 1)?;
        let result = self.get(at);
        self.top = saved_top;
        Ok(result)
    }

    /// C++ `callTM`: calls `f(a, b, c)` and discards its results.
    pub(crate) fn call_tm(&mut self, f: Value, a: Value, b: Value, c: Value) -> VmResult<()> {
        let saved_top = self.top;
        let at = self.scratch();
        self.ensure_stack(at.saturating_add(4).saturating_add(MIN_STACK))?;
        self.set(at, f);
        self.set(at.saturating_add(1), a);
        self.set(at.saturating_add(2), b);
        self.set(at.saturating_add(3), c);
        self.top = at.saturating_add(4);
        self.call_value(at, 3, 0)?;
        self.top = saved_top;
        Ok(())
    }

    /// Restores the thread after a caught error (C++ `luaD_pcall`'s
    /// unwinding): closes upvalues at or above `func` and drops the frames
    /// above `depth`.
    pub(crate) fn unwind(&mut self, depth: usize, func: usize, c_calls: u32) {
        self.close_upvals(func);
        self.frames.truncate(depth);
        self.c_calls = c_calls;
    }

    /// Lane 4 seam: C++ `VM_INTERRUPT`. Lane 1 has no interrupt callback.
    #[inline]
    pub(crate) fn interrupt(&mut self) -> VmResult<()> {
        Ok(())
    }

    // ---- collection --------------------------------------------------

    /// Runs a collection if the allocation debt calls for one.
    #[inline]
    pub(crate) fn safepoint(&mut self) {
        if self.heap.needs_collect() {
            self.full_gc();
        }
    }

    /// A full stop-the-world collection from the roots.
    pub(crate) fn full_gc(&mut self) -> CollectStats {
        let high = self
            .frames
            .iter()
            .map(|f| f.top)
            .fold(self.top, usize::max)
            .min(self.stack.len());
        let stack_len = self.stack.len();
        self.fill_nil(high, stack_len);
        let mut roots: Vec<Value> = Vec::with_capacity(high.saturating_add(64));
        roots.extend(self.stack.iter().take(high).copied());
        roots.push(Value::Table(self.globals));
        roots.push(Value::Table(self.registry));
        if let Some(mt) = self.string_meta {
            roots.push(Value::Table(mt));
        }
        roots.push(Value::String(StrId(0)));
        roots.extend(self.tm_names.iter().map(|&s| Value::String(s)));
        roots.extend(self.frames.iter().map(|f| Value::Function(f.closure)));
        roots.extend(self.open_upvals.iter().map(|&(_, id)| Value::UpRef(id)));
        roots.extend(self.pinned.iter().copied());
        roots.extend(self.host_errors.iter().copied());
        if let Some(name) = self.namecall {
            roots.push(Value::String(name));
        }
        self.heap.collect(roots, &[])
    }
}
