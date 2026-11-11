//! The heap (plan decision D2): typed arenas per object kind, referenced by
//! plain `u32` indices, with collection only at safepoints.
//!
//! - Each object kind has an [`Arena`]: a `Vec` of objects, a color per
//!   slot and a free list. A reference is a typed index (`TableId`, `FnId`,
//!   ...). There is no generation counter and no liveness check on access.
//! - An arena never shrinks and a freed slot is reset to an empty object of
//!   its kind, so an index always reads *some* object of the right kind.
//!   Memory safety therefore never depends on liveness. Semantic
//!   correctness does, and the rooting rule guarantees it: collection runs
//!   only at safepoints (`gc.rs`, which needs `&mut Core`), and every
//!   reference live across a safepoint is reachable from a root (a stack
//!   slot, the globals, the registry, a frame, a closure, a constant).
//! - Allocation never collects. It charges the [`Budget`] (budget.rs, the
//!   one accounting point) before the object exists, and the sweep credits
//!   it back, so `Budget::used` is exactly the cost (`object_cost`) of the
//!   live objects plus the VM's external buffers.
//!
//! **The collector** is incremental, C++ Luau's tri-color mark with Lua
//! 5.1's weak tables, driven by `gc.rs`:
//!
//! - *Pause*: the first step marks the roots gray.
//! - *Propagate*: each step blackens gray objects until its work limit (in
//!   cost bytes, as C++ `propagatemark` counts them). The mutator runs
//!   between steps. A store into a black object goes through a write
//!   barrier (`table_mut`, `table_set`, `function_mut`, `set_upval`) that
//!   turns the object gray again and queues it on `gray_again` (C++
//!   `luaC_barrierback`). The stack is not barriered: the atomic step
//!   rescans the roots.
//! - *Atomic* (one step): the roots again, everything gray, `gray_again`,
//!   then the weak-table clearing pass and the sweep. The sweep is part of
//!   the atomic step, not incremental as in C++, so a white object is never
//!   reachable between steps: an object is only white-and-dead inside the
//!   atomic step, which the mutator cannot observe (the interner can never
//!   hand out a string that is about to be swept).
//!
//! New objects are allocated white. That is sound because anything they
//! are stored into is either white (traversed later), gray (traversed
//! later) or black (barriered), and anything only the stack holds is
//! marked by the atomic rescan.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::rc::Rc;
use std::sync::Arc;

use crate::budget::Budget;
use crate::bytecode::Proto;
use crate::error::{ErrorKind, Throw, VmResult};
use crate::native::{HostFnId, NativeFn};
use crate::table::{Table, TableError};
use crate::thread::Thread;
use crate::userdata::Userdata;
use crate::value::{BufId, FnId, ProtoId, StrId, TableId, ThreadId, UdId, UpvalId, Value};

/// A slot's state for the collector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Color {
    Free,
    /// Allocated, not (yet) reached in the current mark.
    White,
    /// Reached, its references not yet traversed (on a gray list).
    Gray,
    /// Reached and traversed.
    Black,
}

pub(crate) struct Arena<T> {
    items: Vec<T>,
    colors: Vec<Color>,
    free: Vec<u32>,
    live: usize,
    /// Returned for an id that is out of range, which the rooting rules make
    /// impossible; reading it yields an empty object instead of a fault.
    dummy: T,
}

impl<T: Default> Arena<T> {
    fn new() -> Arena<T> {
        Arena {
            items: Vec::new(),
            colors: Vec::new(),
            free: Vec::new(),
            live: 0,
            dummy: T::default(),
        }
    }

    fn alloc(&mut self, value: T) -> Option<u32> {
        if let Some(index) = self.free.pop() {
            let slot = usize::try_from(index).ok()?;
            if let (Some(item), Some(color)) = (self.items.get_mut(slot), self.colors.get_mut(slot))
            {
                *item = value;
                *color = Color::White;
                self.live = self.live.saturating_add(1);
                return Some(index);
            }
            return None;
        }
        let index = u32::try_from(self.items.len()).ok()?;
        self.items.push(value);
        self.colors.push(Color::White);
        self.live = self.live.saturating_add(1);
        Some(index)
    }

    #[inline]
    pub(crate) fn get(&self, id: u32) -> &T {
        self.items.get(id as usize).unwrap_or(&self.dummy)
    }

    /// Mutable access without the write barrier: only the heap's own
    /// barriered accessors and the collector call it.
    #[inline]
    fn get_mut(&mut self, id: u32) -> &mut T {
        match self.items.get_mut(id as usize) {
            Some(item) => item,
            None => {
                self.dummy = T::default();
                &mut self.dummy
            }
        }
    }

    pub(crate) fn live(&self) -> usize {
        self.live
    }

    #[inline]
    fn color(&self, id: u32) -> Color {
        self.colors.get(id as usize).copied().unwrap_or(Color::Free)
    }

    #[inline]
    fn set_color(&mut self, id: u32, to: Color) {
        if let Some(color) = self.colors.get_mut(id as usize) {
            *color = to;
        }
    }

    /// White to `to`; returns true if the slot was white.
    #[inline]
    fn mark(&mut self, id: u32, to: Color) -> bool {
        match self.colors.get_mut(id as usize) {
            Some(color) if *color == Color::White => {
                *color = to;
                true
            }
            _ => false,
        }
    }

    /// Black to gray (the write barrier); returns true if it was black.
    #[inline]
    fn regray(&mut self, id: u32) -> bool {
        match self.colors.get_mut(id as usize) {
            Some(color) if *color == Color::Black => {
                *color = Color::Gray;
                true
            }
            _ => false,
        }
    }

    /// Every allocated slot back to white (a cycle abandoned).
    fn whiten(&mut self) {
        for color in &mut self.colors {
            if *color != Color::Free {
                *color = Color::White;
            }
        }
    }

    /// Frees every white slot (`free` resets it and returns its cost) and
    /// whitens every reached one. Returns the slots and bytes freed.
    fn sweep(&mut self, mut free: impl FnMut(&mut T) -> usize) -> (usize, usize) {
        let mut freed = 0usize;
        let mut bytes = 0usize;
        for (index, (item, color)) in self
            .items
            .iter_mut()
            .zip(self.colors.iter_mut())
            .enumerate()
        {
            match *color {
                Color::White => {
                    bytes = bytes.saturating_add(free(item));
                    *color = Color::Free;
                    if let Ok(index) = u32::try_from(index) {
                        self.free.push(index);
                    }
                    freed = freed.saturating_add(1);
                }
                Color::Gray | Color::Black => *color = Color::White,
                Color::Free => {}
            }
        }
        self.live = self.live.saturating_sub(freed);
        (freed, bytes)
    }

    fn iter_live(&self) -> impl Iterator<Item = &T> {
        self.items
            .iter()
            .zip(self.colors.iter())
            .filter(|(_, c)| **c != Color::Free)
            .map(|(item, _)| item)
    }
}

/// A Lua closure: a prototype, its environment and its upvalue slots. A
/// slot holds the captured value itself (by-value capture, `LCT_VAL`) or
/// `Value::UpRef` for a by-reference capture (`LCT_REF`).
#[derive(Default)]
pub(crate) struct LuaClosure {
    pub(crate) proto: Rc<Proto>,
    pub(crate) proto_id: ProtoId,
    pub(crate) env: TableId,
    pub(crate) upvals: Vec<Value>,
    /// Set on closures created at load time for `DUPCLOSURE` constants until
    /// their upvalues are first filled (C++ `preload`).
    pub(crate) preload: bool,
}

/// A host or builtin function.
pub(crate) struct NativeClosure {
    pub(crate) func: NativeFn,
    /// The debug name used in argument errors (C++ `debugname`).
    pub(crate) name: Box<str>,
    /// A builtin's upvalues (C++ `lua_pushcclosure` with `nup > 0`).
    pub(crate) upvals: Box<[Value]>,
}

pub(crate) enum Function {
    Lua(LuaClosure),
    Native(NativeClosure),
}

impl Default for Function {
    fn default() -> Function {
        Function::Lua(LuaClosure::default())
    }
}

/// An upvalue captured by reference: open while its variable is still in a
/// live stack frame (of the named thread), closed once the frame returns.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Upval {
    Open(ThreadId, u32),
    Closed(Value),
}

impl Default for Upval {
    fn default() -> Upval {
        Upval::Closed(Value::Nil)
    }
}

#[derive(Default)]
pub(crate) struct StrObj {
    pub(crate) bytes: Arc<[u8]>,
}

/// FxHash, for the interner (no external dependency; hash flooding is not a
/// concern inside one VM).
#[derive(Default)]
pub(crate) struct FxHasher(u64);

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut word = [0u8; 8];
            for (dst, src) in word.iter_mut().zip(chunk) {
                *dst = *src;
            }
            self.0 = (self.0.rotate_left(5) ^ u64::from_le_bytes(word))
                .wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
        }
    }

    #[inline]
    fn write_usize(&mut self, n: usize) {
        self.0 = (self.0.rotate_left(5) ^ n as u64).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

type FxBuild = BuildHasherDefault<FxHasher>;

/// Object costs for the memory budget and the collector's work count.
/// They approximate C++'s `sizeof`s; what matters is that `object_cost`
/// gives one fixed answer per object state, so charges and credits match.
const TABLE_BYTES: usize = 64;
const FUNCTION_BYTES: usize = 48;
const UPVAL_BYTES: usize = 32;
const STRING_BYTES: usize = 24;
const PROTO_BYTES: usize = 128;
/// A thread's fixed part. Its stack is an external buffer (budget.rs, rule
/// 4), charged as it grows and credited when the thread dies.
const THREAD_BYTES: usize = 160;
const SLOT_BYTES: usize = 16;

fn table_cost(table: &Table) -> usize {
    TABLE_BYTES.saturating_add(table.capacity_bytes())
}

fn function_cost(function: &Function) -> usize {
    let slots = match function {
        Function::Lua(closure) => closure.upvals.len(),
        Function::Native(native) => native.upvals.len(),
    };
    FUNCTION_BYTES.saturating_add(slots.saturating_mul(SLOT_BYTES))
}

fn string_cost(len: usize) -> usize {
    STRING_BYTES.saturating_add(len)
}

const BUFFER_BYTES: usize = 24;
/// A userdata's fixed part; its payload adds the size the host declared.
const USERDATA_BYTES: usize = 40;

fn userdata_cost(ud: &Userdata) -> usize {
    USERDATA_BYTES.saturating_add(ud.size)
}
/// C++ `MAX_BUFFER_SIZE`.
pub(crate) const MAX_BUFFER_SIZE: usize = 1 << 30;

fn buffer_cost(len: usize) -> usize {
    BUFFER_BYTES.saturating_add(len)
}

/// What a prototype costs: the header, its code and its constants.
fn proto_bytes(proto: &Proto) -> usize {
    PROTO_BYTES
        .saturating_add(proto.code.len().saturating_mul(4))
        .saturating_add(proto.k.len().saturating_mul(SLOT_BYTES))
}

/// What a collection did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CollectStats {
    pub tables_freed: usize,
    pub functions_freed: usize,
    pub strings_freed: usize,
    pub upvalues_freed: usize,
    pub protos_freed: usize,
    pub threads_freed: usize,
    pub buffers_freed: usize,
    pub userdata_freed: usize,
}

/// An object on a gray list. Strings are leaves and never gray.
///
/// A thread's traversal (`Thread::trace`) marks its live stack, its
/// frames' closures and its open upvalues, and it then goes on
/// `gray_again` instead of black, because stack stores are not barriered
/// (C++ `traversestack` keeps threads gray for the atomic rescan). In the
/// atomic step it turns black.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GcRef {
    Table(u32),
    Function(u32),
    Upval(u32),
    Proto(u32),
    Thread(u32),
    Userdata(u32),
}

/// The collector's phase (C++ `gcstate`, with the sweep inside the atomic
/// step).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Pause,
    Propagate,
    /// The objects the barrier re-queued during `Propagate`, once. What it
    /// re-queues meanwhile waits for the atomic step, so a mutator that
    /// keeps storing into marked objects cannot keep the cycle from ending
    /// (C++ `GCSpropagateagain`).
    PropagateAgain,
    /// Marking is done except for the atomic step, which runs next.
    Atomic,
}

/// Collector state and pacing (C++ `global_State`'s GC fields).
pub(crate) struct Gc {
    pub(crate) phase: Phase,
    gray: Vec<GcRef>,
    gray_again: Vec<GcRef>,
    /// Weak tables traversed this cycle (C++ `g->weak`).
    weak: Vec<u32>,
    /// A step runs at the next safepoint once `Budget::used` reaches this
    /// (C++ `GCthreshold`; `usize::MAX` while stopped).
    pub(crate) threshold: usize,
    /// C++ `gcgoal`, `gcstepmul` (percent) and `gcstepsize` (bytes).
    pub(crate) goal: i32,
    pub(crate) stepmul: i32,
    pub(crate) stepsize: i32,
    /// `Budget::used` after the last finished cycle.
    pub(crate) live_after_cycle: usize,
    pub(crate) cycles: usize,
    pub(crate) last_stats: CollectStats,
}

pub(crate) struct Heap {
    pub(crate) tables: Arena<Table>,
    pub(crate) functions: Arena<Function>,
    pub(crate) upvals: Arena<Upval>,
    pub(crate) strings: Arena<StrObj>,
    pub(crate) protos: Arena<Rc<Proto>>,
    threads: Arena<Thread>,
    /// Buffer objects. They hold no references, so the collector treats
    /// them as leaves, like strings.
    pub(crate) buffers: Arena<Vec<u8>>,
    pub(crate) userdata: Arena<Userdata>,
    interned: HashMap<Arc<[u8]>, u32, FxBuild>,
    /// The one accounting point (budget.rs).
    pub(crate) budget: Budget,
    pub(crate) gc: Gc,
    /// `Limits::max_string_bytes`: `intern` refuses longer strings.
    pub(crate) string_limit: usize,
    /// The interned `"__mode"`, read from a table's metatable when it is
    /// traversed (C++ `gettablemode`).
    mode_key: Option<StrId>,
    /// Reused by traversals so marking does not allocate per object.
    scratch: Vec<Value>,
    /// Per host entry (`HostFnId`): whether its natives' data names host
    /// state the host releases (hosted.rs).
    host_owned: Vec<bool>,
    /// `(entry, data)` of every freed native whose entry owns its data, in
    /// the order the sweep freed them, until the host takes them
    /// (`take_freed_natives`).
    freed_natives: Vec<(HostFnId, u32)>,
}

impl Heap {
    pub(crate) fn new(memory_limit: Option<usize>, string_limit: usize) -> Heap {
        Heap {
            tables: Arena::new(),
            functions: Arena::new(),
            upvals: Arena::new(),
            strings: Arena::new(),
            protos: Arena::new(),
            threads: Arena::new(),
            buffers: Arena::new(),
            userdata: Arena::new(),
            interned: HashMap::default(),
            budget: Budget::new(memory_limit),
            gc: Gc {
                phase: Phase::Pause,
                gray: Vec::new(),
                gray_again: Vec::new(),
                weak: Vec::new(),
                threshold: 0,
                goal: 200,
                stepmul: 200,
                stepsize: 1 << 10,
                live_after_cycle: 0,
                cycles: 0,
                last_stats: CollectStats::default(),
            },
            string_limit,
            mode_key: None,
            scratch: Vec::new(),
            host_owned: Vec::new(),
            freed_natives: Vec::new(),
        }
    }

    /// Marks whether natives of host entry `id` own their data.
    pub(crate) fn set_host_owned(&mut self, id: HostFnId, owns_data: bool) {
        let index = id.index();
        if self.host_owned.len() <= index {
            self.host_owned.resize(index.saturating_add(1), false);
        }
        if let Some(flag) = self.host_owned.get_mut(index) {
            *flag = owns_data;
        }
    }

    /// The freed natives the host has not taken yet; taking empties the
    /// list, so each one is handed out exactly once.
    pub(crate) fn take_freed_natives(&mut self) -> Vec<(HostFnId, u32)> {
        std::mem::take(&mut self.freed_natives)
    }

    pub(crate) fn set_mode_key(&mut self, key: StrId) {
        self.mode_key = Some(key);
    }

    /// Bytes in use (C++ `totalbytes`).
    pub(crate) fn total_bytes(&self) -> usize {
        self.budget.used()
    }

    /// Fails with the memory error if `bytes` more would pass the budget,
    /// without charging them: for temporary buffers (budget.rs, rule 3).
    pub(crate) fn check_alloc(&self, bytes: usize) -> VmResult<()> {
        self.budget.check(bytes).map_err(|_| self.memory_error())
    }

    /// Charges a VM buffer outside the arenas (budget.rs, rule 4).
    pub(crate) fn charge_external(&mut self, bytes: usize) -> VmResult<()> {
        self.budget
            .charge_external(bytes)
            .map_err(|_| self.memory_error())
    }

    fn charge(&mut self, bytes: usize) -> VmResult<()> {
        self.budget.charge(bytes).map_err(|_| self.memory_error())
    }

    /// The `LUA_ERRMEM` error. Its message string is preallocated at VM
    /// creation (C++ keeps it in `memerrmsg`) so raising it never allocates.
    pub(crate) fn memory_error(&self) -> Throw {
        Throw {
            value: Value::String(StrId(0)),
            kind: ErrorKind::Memory,
        }
    }

    /// A new empty table with room for `narray` array and `nhash` hash
    /// entries (each clamped to `MAXSIZE`), charged before it is built.
    pub(crate) fn new_table(&mut self, narray: usize, nhash: usize) -> VmResult<TableId> {
        self.charge(TABLE_BYTES.saturating_add(Table::capacity_for(narray, nhash)))?;
        self.insert_table(Table::new(narray, nhash))
    }

    /// A copy of `source` (DUPTABLE), charged before it is built.
    pub(crate) fn clone_table(&mut self, source: TableId) -> VmResult<TableId> {
        self.check_alloc(table_cost(self.tables.get(source.0)))?;
        self.charge(TABLE_BYTES)?;
        let Heap { tables, budget, .. } = self;
        let copy = match tables.get(source.0).try_clone(budget) {
            Ok(copy) => copy,
            Err(_) => {
                budget.credit(TABLE_BYTES);
                return Err(self.memory_error());
            }
        };
        self.insert_table(copy)
    }

    /// Puts an already-charged table in the arena; refunds it if the arena
    /// is full.
    fn insert_table(&mut self, table: Table) -> VmResult<TableId> {
        let cost = table_cost(&table);
        match self.tables.alloc(table) {
            Some(id) => Ok(TableId(id)),
            None => {
                self.budget.credit(cost);
                Err(self.memory_error())
            }
        }
    }

    pub(crate) fn new_function(&mut self, function: Function) -> VmResult<FnId> {
        let cost = function_cost(&function);
        self.charge(cost)?;
        match self.functions.alloc(function) {
            Some(id) => Ok(FnId(id)),
            None => {
                self.budget.credit(cost);
                Err(self.memory_error())
            }
        }
    }

    pub(crate) fn new_upval(&mut self, upval: Upval) -> VmResult<UpvalId> {
        self.charge(UPVAL_BYTES)?;
        match self.upvals.alloc(upval) {
            Some(id) => Ok(UpvalId(id)),
            None => {
                self.budget.credit(UPVAL_BYTES);
                Err(self.memory_error())
            }
        }
    }

    pub(crate) fn new_proto(&mut self, proto: Rc<Proto>) -> VmResult<ProtoId> {
        let cost = proto_bytes(&proto);
        self.charge(cost)?;
        match self.protos.alloc(proto) {
            Some(id) => Ok(ProtoId(id)),
            None => {
                self.budget.credit(cost);
                Err(self.memory_error())
            }
        }
    }

    /// A thread, charged before it exists: its fixed part, and its initial
    /// stack as an external buffer.
    pub(crate) fn new_thread(&mut self, thread: Thread) -> VmResult<ThreadId> {
        let stack = thread.stack_bytes();
        self.charge(THREAD_BYTES)?;
        if let Err(throw) = self.charge_external(stack) {
            self.budget.credit(THREAD_BYTES);
            return Err(throw);
        }
        match self.threads.alloc(thread) {
            Some(id) => Ok(ThreadId(id)),
            None => {
                self.budget.credit(THREAD_BYTES);
                self.budget.credit_external(stack);
                Err(self.memory_error())
            }
        }
    }

    #[inline]
    pub(crate) fn thread(&self, id: ThreadId) -> &Thread {
        self.threads.get(id.0)
    }

    /// A thread's parked state, status and links. No write barrier: a
    /// thread is never black while the mutator runs (`GcRef`).
    #[inline]
    pub(crate) fn thread_mut(&mut self, id: ThreadId) -> &mut Thread {
        self.threads.get_mut(id.0)
    }

    /// The bytes of every parked thread's stack (`Core::accounting_drift`).
    pub(crate) fn thread_stack_bytes(&self) -> usize {
        self.threads
            .iter_live()
            .map(Thread::stack_bytes)
            .fold(0, usize::saturating_add)
    }

    /// Returns the bytes of a thread stack that was dropped or shrunk.
    pub(crate) fn credit_external(&mut self, bytes: usize) {
        self.budget.credit_external(bytes);
    }

    /// Interns a string. Every Luau string is interned, so equal strings
    /// have equal ids.
    pub(crate) fn intern(&mut self, bytes: &[u8]) -> VmResult<StrId> {
        if let Some(&id) = self.interned.get(bytes) {
            return Ok(StrId(id));
        }
        if bytes.len() > self.string_limit {
            return Err(self.memory_error());
        }
        let cost = string_cost(bytes.len());
        self.charge(cost)?;
        let shared: Arc<[u8]> = Arc::from(bytes);
        let Some(id) = self.strings.alloc(StrObj {
            bytes: Arc::clone(&shared),
        }) else {
            self.budget.credit(cost);
            return Err(self.memory_error());
        };
        self.interned.insert(shared, id);
        Ok(StrId(id))
    }

    /// A zero-filled buffer of `len` bytes. The caller checks
    /// [`MAX_BUFFER_SIZE`] first (it raises a different error); the whole
    /// cost is charged before the bytes are allocated.
    pub(crate) fn new_buffer(&mut self, len: usize) -> VmResult<BufId> {
        let cost = buffer_cost(len);
        self.charge(cost)?;
        match self.buffers.alloc(vec![0; len]) {
            Some(id) => Ok(BufId(id)),
            None => {
                self.budget.credit(cost);
                Err(self.memory_error())
            }
        }
    }

    /// A new userdata; charges its whole cost first (budget.rs, rule 1).
    pub(crate) fn new_userdata(&mut self, ud: Userdata) -> VmResult<UdId> {
        let cost = userdata_cost(&ud);
        self.charge(cost)?;
        match self.userdata.alloc(ud) {
            Some(id) => Ok(UdId(id)),
            None => {
                self.budget.credit(cost);
                Err(self.memory_error())
            }
        }
    }

    #[inline]
    pub(crate) fn userdata(&self, id: UdId) -> &Userdata {
        self.userdata.get(id.0)
    }

    /// Sets a userdata's metatable, behind the write barrier.
    pub(crate) fn set_userdata_meta(&mut self, id: UdId, meta: Option<TableId>) {
        self.barrier(GcRef::Userdata(id.0));
        self.userdata.get_mut(id.0).meta = meta;
    }

    #[inline]
    pub(crate) fn buffer(&self, id: BufId) -> &[u8] {
        self.buffers.get(id.0)
    }

    /// A buffer's bytes, to write. Buffers reference nothing, so stores
    /// into them need no write barrier.
    #[inline]
    pub(crate) fn buffer_mut(&mut self, id: BufId) -> &mut [u8] {
        self.buffers.get_mut(id.0)
    }

    /// Copies bytes `src` of buffer `source` to `dst_start` in `target`
    /// with no intermediate copy: a distinct target is moved out while the
    /// bytes are copied. Ranges that do not fit copy nothing; callers
    /// check bounds first.
    pub(crate) fn copy_buffer(
        &mut self,
        source: BufId,
        src: std::ops::Range<usize>,
        target: BufId,
        dst_start: usize,
    ) {
        let len = src.len();
        if source == target {
            let buffer = self.buffers.get_mut(target.0);
            if src.end <= buffer.len() && dst_start.saturating_add(len) <= buffer.len() {
                buffer.copy_within(src, dst_start);
            }
            return;
        }
        let mut out = std::mem::take(self.buffers.get_mut(target.0));
        if let (Some(from), Some(to)) = (
            self.buffers.get(source.0).get(src),
            out.get_mut(dst_start..dst_start.saturating_add(len)),
        ) {
            to.copy_from_slice(from);
        }
        *self.buffers.get_mut(target.0) = out;
    }

    /// Interns the bytes `range` of a buffer (all of it for `None`) without
    /// copying them first: the buffer is moved out while the string is made.
    pub(crate) fn intern_buffer_slice(
        &mut self,
        id: BufId,
        range: Option<std::ops::Range<usize>>,
    ) -> VmResult<StrId> {
        let buffer = std::mem::take(self.buffers.get_mut(id.0));
        let bytes = match range {
            Some(range) => buffer.get(range).unwrap_or_default(),
            None => &buffer,
        };
        let result = self.intern(bytes);
        *self.buffers.get_mut(id.0) = buffer;
        result
    }

    #[inline]
    pub(crate) fn str_bytes(&self, id: StrId) -> &[u8] {
        &self.strings.get(id.0).bytes
    }

    /// The bytes of a string, shared, for code that calls back into the VM
    /// while it reads them.
    pub(crate) fn str_rc(&self, id: StrId) -> Arc<[u8]> {
        Arc::clone(&self.strings.get(id.0).bytes)
    }

    #[inline]
    pub(crate) fn table(&self, id: TableId) -> &Table {
        self.tables.get(id.0)
    }

    /// A table for an in-place change (a value, the metatable, a flag).
    /// Passes the write barrier; the table cannot grow through it
    /// (`table_set` and `table_resize_array` do that).
    #[inline]
    pub(crate) fn table_mut(&mut self, id: TableId) -> &mut Table {
        self.barrier(GcRef::Table(id.0));
        self.tables.get_mut(id.0)
    }

    /// A raw store, with the write barrier and the growth charged.
    #[inline]
    pub(crate) fn table_set(
        &mut self,
        id: TableId,
        key: Value,
        value: Value,
    ) -> Result<(), TableError> {
        self.barrier(GcRef::Table(id.0));
        let Heap { tables, budget, .. } = self;
        tables.get_mut(id.0).set(key, value, budget)
    }

    /// C++ `luaH_resizearray`, with the growth charged.
    pub(crate) fn table_resize_array(
        &mut self,
        id: TableId,
        size: usize,
    ) -> Result<(), TableError> {
        self.barrier(GcRef::Table(id.0));
        let Heap { tables, budget, .. } = self;
        tables.get_mut(id.0).resize_array(size, budget)
    }

    #[inline]
    pub(crate) fn function(&self, id: FnId) -> &Function {
        self.functions.get(id.0)
    }

    /// A function for an in-place change (upvalue slots, environment).
    /// Passes the write barrier.
    #[inline]
    pub(crate) fn function_mut(&mut self, id: FnId) -> &mut Function {
        self.barrier(GcRef::Function(id.0));
        self.functions.get_mut(id.0)
    }

    #[inline]
    pub(crate) fn upval(&self, id: UpvalId) -> Upval {
        *self.upvals.get(id.0)
    }

    #[inline]
    pub(crate) fn set_upval(&mut self, id: UpvalId, upval: Upval) {
        self.barrier(GcRef::Upval(id.0));
        *self.upvals.get_mut(id.0) = upval;
    }

    /// While set, every allocation fails (the conformance harness's
    /// `setblockallocations`). Collection itself never allocates.
    pub(crate) fn set_blocked(&mut self, blocked: bool) {
        self.budget.set_blocked(blocked);
    }

    // ---- the collector -------------------------------------------------

    /// C++ `luaC_barrierback`: a store into a black object turns it gray
    /// again, to be traversed again before the cycle ends.
    #[inline]
    fn barrier(&mut self, object: GcRef) {
        if self.gc.phase == Phase::Pause {
            return;
        }
        let was_black = match object {
            GcRef::Table(id) => self.tables.regray(id),
            GcRef::Function(id) => self.functions.regray(id),
            GcRef::Upval(id) => self.upvals.regray(id),
            GcRef::Proto(id) => self.protos.regray(id),
            GcRef::Thread(id) => self.threads.regray(id),
            GcRef::Userdata(id) => self.userdata.regray(id),
        };
        if was_black {
            self.gc.gray_again.push(object);
        }
    }

    /// Whether the next step needs the root set (it starts or ends a cycle).
    pub(crate) fn step_needs_roots(&self) -> bool {
        matches!(self.gc.phase, Phase::Pause | Phase::Atomic)
    }

    /// C++ `gcstep`: one step of at most about `limit` work, from the
    /// phase the collector is in. `roots` is used (and needed) only when
    /// `step_needs_roots` says so. Returns the work done.
    pub(crate) fn gc_step(&mut self, limit: usize, roots: &[Value]) -> usize {
        match self.gc.phase {
            Phase::Pause => {
                for &root in roots {
                    self.mark_value(root);
                }
                self.gc.phase = Phase::Propagate;
                0
            }
            Phase::Propagate => {
                let work = self.propagate(limit);
                if self.gc.gray.is_empty() {
                    self.gc.gray = std::mem::take(&mut self.gc.gray_again);
                    self.gc.phase = Phase::PropagateAgain;
                }
                work
            }
            Phase::PropagateAgain => {
                let work = self.propagate(limit);
                if self.gc.gray.is_empty() {
                    self.gc.phase = Phase::Atomic;
                }
                work
            }
            Phase::Atomic => self.atomic(roots),
        }
    }

    /// Abandons a cycle in progress (C++ `luaC_fullgc` resetting the gray
    /// lists): every object white again, nothing queued.
    pub(crate) fn abandon_cycle(&mut self) {
        if self.gc.phase == Phase::Pause {
            return;
        }
        self.tables.whiten();
        self.functions.whiten();
        self.upvals.whiten();
        self.strings.whiten();
        self.protos.whiten();
        self.threads.whiten();
        self.buffers.whiten();
        self.userdata.whiten();
        self.gc.gray.clear();
        self.gc.gray_again.clear();
        self.gc.weak.clear();
        self.gc.phase = Phase::Pause;
    }

    /// Traverses gray objects until `limit` work is done or none is left.
    fn propagate(&mut self, limit: usize) -> usize {
        let mut work = 0usize;
        while work < limit {
            let Some(object) = self.gc.gray.pop() else {
                break;
            };
            work = work.saturating_add(self.traverse(object));
        }
        work
    }

    fn propagate_all(&mut self) -> usize {
        let mut work = 0usize;
        loop {
            work = work.saturating_add(self.propagate(usize::MAX));
            if self.gc.gray_again.is_empty() {
                return work;
            }
            self.gc.gray = std::mem::take(&mut self.gc.gray_again);
        }
    }

    /// C++ `atomic` plus the whole sweep: the roots again, everything
    /// gray, the weak tables cleared, every white object freed.
    fn atomic(&mut self, roots: &[Value]) -> usize {
        // The weak tables are traversed again (C++ moves `g->weak` to the
        // gray list): a table's mode may have changed since its traversal,
        // and one that is strong now must have its values marked.
        for id in std::mem::take(&mut self.gc.weak) {
            if self.tables.color(id) == Color::Black {
                self.tables.set_color(id, Color::Gray);
                self.gc.gray.push(GcRef::Table(id));
            }
        }
        for &root in roots {
            self.mark_value(root);
        }
        self.remark_open_upvals();
        let mut work = self.propagate_all();
        work = work.saturating_add(self.clear_weak_tables());
        self.close_dead_upvals();
        let mut stats = CollectStats::default();
        let mut freed_bytes = 0usize;
        let (n, bytes) = self.tables.sweep(|table| {
            let cost = table_cost(table);
            *table = Table::default();
            cost
        });
        stats.tables_freed = n;
        freed_bytes = freed_bytes.saturating_add(bytes);
        let host_owned = &self.host_owned;
        let freed_natives = &mut self.freed_natives;
        let (n, bytes) = self.functions.sweep(|function| {
            if let Function::Native(NativeClosure {
                func: NativeFn::Host(id, data),
                ..
            }) = function
                && host_owned.get(id.index()).copied().unwrap_or(false)
            {
                freed_natives.push((*id, *data));
            }
            let cost = function_cost(function);
            *function = Function::default();
            cost
        });
        stats.functions_freed = n;
        freed_bytes = freed_bytes.saturating_add(bytes);
        let (n, bytes) = self.upvals.sweep(|upval| {
            *upval = Upval::default();
            UPVAL_BYTES
        });
        stats.upvalues_freed = n;
        freed_bytes = freed_bytes.saturating_add(bytes);
        let (n, bytes) = self.protos.sweep(|proto| {
            let cost = proto_bytes(proto);
            *proto = Rc::default();
            cost
        });
        stats.protos_freed = n;
        freed_bytes = freed_bytes.saturating_add(bytes);
        let mut stacks = 0usize;
        let (n, bytes) = self.threads.sweep(|thread| {
            stacks = stacks.saturating_add(thread.stack_bytes());
            *thread = Thread::default();
            THREAD_BYTES
        });
        stats.threads_freed = n;
        freed_bytes = freed_bytes.saturating_add(bytes);
        self.budget.credit_external(stacks);
        let interned = &mut self.interned;
        let (n, bytes) = self.strings.sweep(|string| {
            let cost = string_cost(string.bytes.len());
            interned.remove(&string.bytes);
            *string = StrObj::default();
            cost
        });
        stats.strings_freed = n;
        freed_bytes = freed_bytes.saturating_add(bytes);
        let (n, bytes) = self.buffers.sweep(|buffer| {
            let cost = buffer_cost(buffer.len());
            *buffer = Vec::new();
            cost
        });
        stats.buffers_freed = n;
        freed_bytes = freed_bytes.saturating_add(bytes);
        // Freeing a slot drops its payload `Rc`: the host value's `Drop`
        // (its destructor) runs here unless the host still borrows it. It
        // cannot reach the VM (userdata.rs).
        let (n, bytes) = self.userdata.sweep(|ud| {
            let cost = userdata_cost(ud);
            *ud = Userdata::default();
            cost
        });
        stats.userdata_freed = n;
        freed_bytes = freed_bytes.saturating_add(bytes);
        self.budget.credit(freed_bytes);
        // The sweep's work, counted like the mark's (C++ counts swept
        // pages; here the bytes freed stand in for them).
        work = work.saturating_add(freed_bytes);
        self.gc.weak.clear();
        self.gc.phase = Phase::Pause;
        self.gc.cycles = self.gc.cycles.saturating_add(1);
        self.gc.live_after_cycle = self.budget.used();
        self.gc.last_stats = stats;
        work
    }

    /// The value of an open upvalue: slot `slot` of parked thread `thread`.
    /// `None` for the running thread, whose arena slot is empty while it
    /// runs and whose stack is a root.
    fn open_slot_value(&self, thread: ThreadId, slot: u32) -> Option<Value> {
        self.threads
            .get(thread.0)
            .exec
            .stack
            .get(slot as usize)
            .copied()
    }

    /// C++ `remarkupvals`: marks the current value of every marked open
    /// upvalue. A closure can store through an open upvalue into a parked
    /// thread's stack after the upvalue was traversed, and an unreachable
    /// thread is not traversed again, so the new value is marked here.
    fn remark_open_upvals(&mut self) {
        let mut values = std::mem::take(&mut self.scratch);
        for (upval, color) in self.upvals.items.iter().zip(&self.upvals.colors) {
            if let (Color::Gray | Color::Black, Upval::Open(thread, slot)) = (color, upval) {
                values.extend(self.open_slot_value(*thread, *slot));
            }
        }
        for &v in &values {
            self.mark_value(v);
        }
        values.clear();
        self.scratch = values;
    }

    /// C++ `luaE_freethread` closing a dead thread's open upvalues: each
    /// one that is still reachable takes its slot's (marked) value.
    fn close_dead_upvals(&mut self) {
        let Heap {
            threads, upvals, ..
        } = self;
        for (thread, color) in threads.items.iter().zip(&threads.colors) {
            if *color != Color::White {
                continue;
            }
            for &(slot, id) in &thread.exec.open_upvals {
                if upvals.color(id.0) != Color::White {
                    let v = thread.exec.stack.get(slot).copied().unwrap_or_default();
                    *upvals.get_mut(id.0) = Upval::Closed(v);
                }
            }
        }
    }

    /// C++ `cleartable` over every weak table this cycle traversed: an
    /// entry goes when its key or value is an unmarked object. Strings are
    /// values, never cleared (C++ `isobjcleared` marks them instead).
    fn clear_weak_tables(&mut self) -> usize {
        let weak = std::mem::take(&mut self.gc.weak);
        let mut work = 0usize;
        let Heap {
            tables,
            functions,
            upvals,
            strings,
            threads,
            buffers,
            userdata,
            ..
        } = self;
        for &id in &weak {
            // Colors of other tables are read while this one is changed, so
            // the table is taken out of its slot for the pass.
            let mut table = std::mem::take(tables.get_mut(id));
            work = work.saturating_add(table_cost(&table));
            table.clear_entries(|v| match v {
                Value::String(s) => {
                    strings.mark(s.0, Color::Black);
                    false
                }
                Value::Table(t) => tables.color(t.0) == Color::White,
                Value::Function(f) => functions.color(f.0) == Color::White,
                Value::UpRef(u) => upvals.color(u.0) == Color::White,
                Value::Thread(t) => threads.color(t.0) == Color::White,
                Value::Buffer(b) => buffers.color(b.0) == Color::White,
                Value::Userdata(u) => userdata.color(u.0) == Color::White,
                Value::Nil
                | Value::Boolean(_)
                | Value::Number(_)
                | Value::Integer(_)
                | Value::Vector(_)
                | Value::LightUserdata(..) => false,
            });
            *tables.get_mut(id) = table;
        }
        self.gc.weak = weak;
        work
    }

    /// The weak mode of a table with metatable `meta` (C++ `gettablemode`
    /// and the `strchr` scan in `traversetable`): (weak keys, weak values).
    fn weak_mode(&self, meta: Option<TableId>) -> (bool, bool) {
        let (Some(meta), Some(key)) = (meta, self.mode_key) else {
            return (false, false);
        };
        let Value::String(mode) = self.tables.get(meta.0).get_str(key) else {
            return (false, false);
        };
        let bytes = self.str_bytes(mode);
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let mode = bytes.get(..end).unwrap_or_default();
        (mode.contains(&b'k'), mode.contains(&b'v'))
    }

    /// Marks `value`: a string black, any other object gray (queued).
    fn mark_value(&mut self, value: Value) {
        match value {
            Value::String(id) => {
                self.strings.mark(id.0, Color::Black);
            }
            Value::Buffer(id) => {
                self.buffers.mark(id.0, Color::Black);
            }
            Value::Table(id) => {
                if self.tables.mark(id.0, Color::Gray) {
                    self.gc.gray.push(GcRef::Table(id.0));
                }
            }
            Value::Function(id) => {
                if self.functions.mark(id.0, Color::Gray) {
                    self.gc.gray.push(GcRef::Function(id.0));
                }
            }
            Value::UpRef(id) => {
                if self.upvals.mark(id.0, Color::Gray) {
                    self.gc.gray.push(GcRef::Upval(id.0));
                }
            }
            Value::Thread(id) => {
                if self.threads.mark(id.0, Color::Gray) {
                    self.gc.gray.push(GcRef::Thread(id.0));
                }
            }
            Value::Userdata(id) => {
                if self.userdata.mark(id.0, Color::Gray) {
                    self.gc.gray.push(GcRef::Userdata(id.0));
                }
            }
            Value::Nil
            | Value::Boolean(_)
            | Value::Number(_)
            | Value::Integer(_)
            | Value::Vector(_)
            | Value::LightUserdata(..) => {}
        }
    }

    fn mark_proto(&mut self, id: ProtoId) {
        if self.protos.mark(id.0, Color::Gray) {
            self.gc.gray.push(GcRef::Proto(id.0));
        }
    }

    /// Blackens one gray object, marking what it references (C++
    /// `propagatemark`). Returns the object's cost as the work done.
    fn traverse(&mut self, object: GcRef) -> usize {
        let mut children = std::mem::take(&mut self.scratch);
        let mut protos: Option<Rc<Proto>> = None;
        let mut proto_id = None;
        let cost = match object {
            GcRef::Table(id) => {
                let table = self.tables.get(id);
                let meta = table.meta;
                let (weak_keys, weak_values) = self.weak_mode(meta);
                table.trace(weak_keys, weak_values, |v| children.push(v));
                if let Some(meta) = meta {
                    children.push(Value::Table(meta));
                }
                let cost = table_cost(table);
                if weak_keys || weak_values {
                    self.gc.weak.push(id);
                }
                self.tables.set_color(id, Color::Black);
                cost
            }
            GcRef::Function(id) => {
                let function = self.functions.get(id);
                match function {
                    Function::Lua(closure) => {
                        children.extend(closure.upvals.iter().copied());
                        children.push(Value::Table(closure.env));
                        proto_id = Some(closure.proto_id);
                    }
                    Function::Native(native) => children.extend(native.upvals.iter().copied()),
                }
                let cost = function_cost(function);
                self.functions.set_color(id, Color::Black);
                cost
            }
            GcRef::Upval(id) => {
                match self.upval(UpvalId(id)) {
                    Upval::Closed(v) => children.push(v),
                    // The value is a slot of that thread's stack. It is
                    // marked, not the thread (C++ `traverseclosure`): an
                    // unreachable thread dies, and `close_dead_upvals`
                    // closes its upvalues that are still reachable. A slot
                    // of the running thread is a root already.
                    Upval::Open(thread, slot) => {
                        children.extend(self.open_slot_value(thread, slot));
                    }
                }
                self.upvals.set_color(id, Color::Black);
                UPVAL_BYTES
            }
            GcRef::Thread(id) => {
                // Slots above the live top are dead: cleared, so an id
                // there cannot root a reused slot once the thread resumes
                // (as `gc_roots` does for the running thread).
                self.threads.get_mut(id).clear_dead_slots();
                self.threads.get(id).trace(|v| children.push(v));
                if self.gc.phase == Phase::Atomic {
                    self.threads.set_color(id, Color::Black);
                } else {
                    // Stack stores are not barriered: traverse it again.
                    self.threads.set_color(id, Color::Gray);
                    self.gc.gray_again.push(object);
                }
                THREAD_BYTES
            }
            GcRef::Userdata(id) => {
                let ud = self.userdata.get(id);
                children.extend(ud.meta.map(Value::Table));
                let cost = userdata_cost(ud);
                self.userdata.set_color(id, Color::Black);
                cost
            }
            GcRef::Proto(id) => {
                let proto = Rc::clone(self.protos.get(id));
                children.extend(proto.k.iter().copied());
                children.extend(proto.source.map(Value::String));
                children.extend(proto.debugname.map(Value::String));
                let cost = proto_bytes(&proto);
                protos = Some(proto);
                self.protos.set_color(id, Color::Black);
                cost
            }
        };
        for &child in &children {
            self.mark_value(child);
        }
        children.clear();
        self.scratch = children;
        if let Some(id) = proto_id {
            self.mark_proto(id);
        }
        if let Some(proto) = protos {
            for &child in &proto.children {
                self.mark_proto(child);
            }
        }
        cost
    }

    /// The bytes the live objects and external buffers cost, recounted
    /// from the arenas. Equal to `Budget::used` unless something allocated
    /// around the budget (`Core::accounting_drift`).
    pub(crate) fn recount(&self) -> usize {
        let tables = self
            .tables
            .iter_live()
            .map(table_cost)
            .fold(0, usize::saturating_add);
        let strings = self
            .strings
            .iter_live()
            .map(|s| string_cost(s.bytes.len()))
            .fold(0, usize::saturating_add);
        let buffers = self
            .buffers
            .iter_live()
            .map(|b| buffer_cost(b.len()))
            .fold(0, usize::saturating_add);
        let userdata = self
            .userdata
            .iter_live()
            .map(userdata_cost)
            .fold(0, usize::saturating_add);
        let functions = self
            .functions
            .iter_live()
            .map(function_cost)
            .fold(0, usize::saturating_add);
        let protos = self
            .protos
            .iter_live()
            .map(|p| proto_bytes(p))
            .fold(0, usize::saturating_add);
        tables
            .saturating_add(strings)
            .saturating_add(buffers)
            .saturating_add(userdata)
            .saturating_add(functions)
            .saturating_add(self.upvals.live().saturating_mul(UPVAL_BYTES))
            .saturating_add(protos)
            .saturating_add(self.threads.live().saturating_mul(THREAD_BYTES))
            .saturating_add(self.budget.external())
    }
}
