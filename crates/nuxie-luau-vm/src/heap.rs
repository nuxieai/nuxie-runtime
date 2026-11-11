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
//!   only at safepoints (`Vm::safepoint`, which needs `&mut Vm`), and every
//!   reference live across a safepoint is reachable from a root (a stack
//!   slot, the globals, the registry, a frame, a closure, a constant).
//! - Allocation never collects; it adds debt and checks the memory budget.
//!
//! Seam for lane 3 (collector): `gc.rs` holds the stop-the-world mark and
//! sweep. The incremental collector replaces it, using `Color` (which then
//! gains `Gray`) and a write barrier at the table-store sites that already
//! go through `Vm` methods. Weak tables (`__mode`) and userdata destructors
//! are not implemented here.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::rc::Rc;

use crate::bytecode::Proto;
use crate::error::{ErrorKind, Throw, VmResult};
use crate::native::NativeFn;
use crate::table::Table;
use crate::value::{FnId, ProtoId, StrId, TableId, UpvalId, Value};

/// A slot's state for the collector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Color {
    Free,
    /// Allocated, not (yet) reached in the current mark.
    White,
    /// Reached in the current mark.
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

    #[inline]
    pub(crate) fn get_mut(&mut self, id: u32) -> &mut T {
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

    /// Marks a slot; returns true the first time it is marked in a cycle.
    fn mark(&mut self, id: u32) -> bool {
        match self.colors.get_mut(id as usize) {
            Some(color) if *color == Color::White => {
                *color = Color::Black;
                true
            }
            _ => false,
        }
    }

    /// Frees every white slot (resetting it with `reset`) and whitens every
    /// black one. Returns the number of slots freed.
    fn sweep(&mut self, mut reset: impl FnMut(&mut T)) -> usize {
        let mut freed = 0usize;
        for (index, (item, color)) in self
            .items
            .iter_mut()
            .zip(self.colors.iter_mut())
            .enumerate()
        {
            match *color {
                Color::White => {
                    reset(item);
                    *color = Color::Free;
                    if let Ok(index) = u32::try_from(index) {
                        self.free.push(index);
                    }
                    freed = freed.saturating_add(1);
                }
                Color::Black => *color = Color::White,
                Color::Free => {}
            }
        }
        self.live = self.live.saturating_sub(freed);
        freed
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
    pub(crate) name: Rc<str>,
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
/// live stack frame, closed once the frame returns.
///
/// Seam for lane 4 (coroutines): an open upvalue names a stack slot of the
/// one thread lane 1 has; with coroutines it names (thread, slot).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Upval {
    Open(u32),
    Closed(Value),
}

impl Default for Upval {
    fn default() -> Upval {
        Upval::Closed(Value::Nil)
    }
}

#[derive(Default)]
pub(crate) struct StrObj {
    pub(crate) bytes: Rc<[u8]>,
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

/// Approximate object costs for the GC debt and the memory budget.
const TABLE_BYTES: usize = 64;
const FUNCTION_BYTES: usize = 48;
const UPVAL_BYTES: usize = 32;
const STRING_BYTES: usize = 24;
const PROTO_BYTES: usize = 128;

/// What a collection did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CollectStats {
    pub tables_freed: usize,
    pub functions_freed: usize,
    pub strings_freed: usize,
    pub upvalues_freed: usize,
    pub protos_freed: usize,
}

pub(crate) struct Heap {
    pub(crate) tables: Arena<Table>,
    pub(crate) functions: Arena<Function>,
    pub(crate) upvals: Arena<Upval>,
    pub(crate) strings: Arena<StrObj>,
    pub(crate) protos: Arena<Rc<Proto>>,
    interned: HashMap<Rc<[u8]>, u32, FxBuild>,
    /// Approximate bytes allocated since the last collection.
    pub(crate) debt: usize,
    /// Approximate bytes live after the last collection.
    pub(crate) live_bytes: usize,
    /// A collection runs at the next safepoint once `debt` passes this.
    pub(crate) threshold: usize,
    /// `collectgarbage("stop")` sets this.
    pub(crate) stopped: bool,
    pub(crate) collections: usize,
    pub(crate) memory_limit: Option<usize>,
    /// `Limits::max_string_bytes`: `intern` refuses longer strings.
    pub(crate) string_limit: usize,
    /// Bytes held outside the arenas (the value stack), charged when they
    /// grow and kept in the live estimate across collections.
    pub(crate) external_bytes: usize,
}

/// What a prototype costs: the header, its code and its constants.
fn proto_bytes(proto: &Proto) -> usize {
    PROTO_BYTES
        .saturating_add(proto.code.len().saturating_mul(4))
        .saturating_add(proto.k.len().saturating_mul(16))
}

impl Heap {
    pub(crate) fn new(memory_limit: Option<usize>, string_limit: usize) -> Heap {
        Heap {
            tables: Arena::new(),
            functions: Arena::new(),
            upvals: Arena::new(),
            strings: Arena::new(),
            protos: Arena::new(),
            interned: HashMap::default(),
            debt: 0,
            live_bytes: 0,
            threshold: 1 << 20,
            stopped: false,
            collections: 0,
            memory_limit,
            string_limit,
            external_bytes: 0,
        }
    }

    /// Approximate bytes in use: live after the last collection plus
    /// everything allocated since.
    pub(crate) fn total_bytes(&self) -> usize {
        self.live_bytes.saturating_add(self.debt)
    }

    #[inline]
    pub(crate) fn needs_collect(&self) -> bool {
        !self.stopped && self.debt > self.collect_threshold()
    }

    /// The debt that triggers a collection at the next safepoint: the
    /// growth threshold, but never more than half the budget left after the
    /// live set, so a small budget is collected before it runs out.
    fn collect_threshold(&self) -> usize {
        match self.memory_limit {
            Some(limit) => self
                .threshold
                .min(limit.saturating_sub(self.live_bytes) / 2),
            None => self.threshold,
        }
    }

    /// Adds `bytes` of debt, failing with a memory error (and charging
    /// nothing) when they would pass the budget.
    pub(crate) fn charge(&mut self, bytes: usize) -> VmResult<()> {
        self.check_alloc(bytes)?;
        self.debt = self.debt.saturating_add(bytes);
        Ok(())
    }

    /// Fails with a memory error if `bytes` more would pass the budget,
    /// without charging them (for sizing checks before a large allocation).
    pub(crate) fn check_alloc(&self, bytes: usize) -> VmResult<()> {
        match self.memory_limit {
            Some(limit) if self.total_bytes().saturating_add(bytes) > limit => {
                Err(self.memory_error())
            }
            _ => Ok(()),
        }
    }

    /// The `LUA_ERRMEM` error. Its message string is preallocated at VM
    /// creation (C++ keeps it in `memerrmsg`) so raising it never allocates.
    pub(crate) fn memory_error(&self) -> Throw {
        Throw {
            value: Value::String(StrId(0)),
            kind: ErrorKind::Memory,
        }
    }

    pub(crate) fn new_table(&mut self, table: Table) -> VmResult<TableId> {
        self.charge(TABLE_BYTES.saturating_add(table.capacity_bytes()))?;
        match self.tables.alloc(table) {
            Some(id) => Ok(TableId(id)),
            None => Err(self.memory_error()),
        }
    }

    pub(crate) fn new_function(&mut self, function: Function) -> VmResult<FnId> {
        let ups = match &function {
            Function::Lua(closure) => closure.upvals.len(),
            Function::Native(_) => 0,
        };
        self.charge(FUNCTION_BYTES.saturating_add(ups.saturating_mul(16)))?;
        match self.functions.alloc(function) {
            Some(id) => Ok(FnId(id)),
            None => Err(self.memory_error()),
        }
    }

    pub(crate) fn new_upval(&mut self, upval: Upval) -> VmResult<UpvalId> {
        self.charge(UPVAL_BYTES)?;
        match self.upvals.alloc(upval) {
            Some(id) => Ok(UpvalId(id)),
            None => Err(self.memory_error()),
        }
    }

    pub(crate) fn new_proto(&mut self, proto: Rc<Proto>) -> VmResult<ProtoId> {
        self.charge(proto_bytes(&proto))?;
        match self.protos.alloc(proto) {
            Some(id) => Ok(ProtoId(id)),
            None => Err(self.memory_error()),
        }
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
        self.charge(STRING_BYTES.saturating_add(bytes.len()))?;
        let shared: Rc<[u8]> = Rc::from(bytes);
        let Some(id) = self.strings.alloc(StrObj {
            bytes: Rc::clone(&shared),
        }) else {
            return Err(self.memory_error());
        };
        self.interned.insert(shared, id);
        Ok(StrId(id))
    }

    #[inline]
    pub(crate) fn str_bytes(&self, id: StrId) -> &[u8] {
        &self.strings.get(id.0).bytes
    }

    /// The bytes of a string, shared, for code that calls back into the VM
    /// while it reads them.
    pub(crate) fn str_rc(&self, id: StrId) -> Rc<[u8]> {
        Rc::clone(&self.strings.get(id.0).bytes)
    }

    #[inline]
    pub(crate) fn table(&self, id: TableId) -> &Table {
        self.tables.get(id.0)
    }

    #[inline]
    pub(crate) fn table_mut(&mut self, id: TableId) -> &mut Table {
        self.tables.get_mut(id.0)
    }

    #[inline]
    pub(crate) fn function(&self, id: FnId) -> &Function {
        self.functions.get(id.0)
    }

    #[inline]
    pub(crate) fn upval(&self, id: UpvalId) -> Upval {
        *self.upvals.get(id.0)
    }

    #[inline]
    pub(crate) fn set_upval(&mut self, id: UpvalId, upval: Upval) {
        *self.upvals.get_mut(id.0) = upval;
    }

    /// Stop-the-world mark and sweep from `roots`. The caller (`gc.rs`)
    /// gathers every root; see the module comment for the rule.
    pub(crate) fn collect(&mut self, roots: Vec<Value>, root_protos: &[ProtoId]) -> CollectStats {
        let mut gray: Vec<Value> = roots;
        let mut gray_protos: Vec<ProtoId> = root_protos.to_vec();
        loop {
            if let Some(value) = gray.pop() {
                self.mark_value(value, &mut gray, &mut gray_protos);
            } else if let Some(proto) = gray_protos.pop() {
                if self.protos.mark(proto.0) {
                    let proto = Rc::clone(self.protos.get(proto.0));
                    gray.extend(proto.k.iter().copied());
                    gray_protos.extend(proto.children.iter().copied());
                    if let Some(source) = proto.source {
                        gray.push(Value::String(source));
                    }
                    if let Some(name) = proto.debugname {
                        gray.push(Value::String(name));
                    }
                }
            } else {
                break;
            }
        }
        let mut stats = CollectStats {
            tables_freed: self.tables.sweep(|table| *table = Table::default()),
            functions_freed: self
                .functions
                .sweep(|function| *function = Function::default()),
            upvalues_freed: self.upvals.sweep(|upval| *upval = Upval::default()),
            protos_freed: self.protos.sweep(|proto| *proto = Rc::default()),
            strings_freed: 0,
        };
        let interned = &mut self.interned;
        stats.strings_freed = self.strings.sweep(|string| {
            interned.remove(&string.bytes);
            *string = StrObj::default();
        });
        self.live_bytes = self.estimate_live_bytes();
        self.debt = 0;
        self.threshold = self.live_bytes.max(1 << 20);
        self.collections = self.collections.saturating_add(1);
        stats
    }

    fn estimate_live_bytes(&self) -> usize {
        let tables: usize = self
            .tables
            .iter_live()
            .map(|t| TABLE_BYTES.saturating_add(t.capacity_bytes()))
            .fold(0, usize::saturating_add);
        let strings: usize = self
            .strings
            .iter_live()
            .map(|s| STRING_BYTES.saturating_add(s.bytes.len()))
            .fold(0, usize::saturating_add);
        // The same per-object costs `new_function` and `new_proto` charge.
        let functions: usize = self
            .functions
            .iter_live()
            .map(|f| {
                let ups = match f {
                    Function::Lua(closure) => closure.upvals.len(),
                    Function::Native(_) => 0,
                };
                FUNCTION_BYTES.saturating_add(ups.saturating_mul(16))
            })
            .fold(0, usize::saturating_add);
        let protos: usize = self
            .protos
            .iter_live()
            .map(|p| proto_bytes(p))
            .fold(0, usize::saturating_add);
        tables
            .saturating_add(strings)
            .saturating_add(functions)
            .saturating_add(self.upvals.live().saturating_mul(UPVAL_BYTES))
            .saturating_add(protos)
            .saturating_add(self.external_bytes)
    }

    fn mark_value(&mut self, value: Value, gray: &mut Vec<Value>, gray_protos: &mut Vec<ProtoId>) {
        match value {
            Value::String(id) => {
                self.strings.mark(id.0);
            }
            Value::Table(id) => {
                if self.tables.mark(id.0) {
                    self.tables.get(id.0).trace(|v| gray.push(v));
                }
            }
            Value::Function(id) => {
                if self.functions.mark(id.0) {
                    match self.functions.get(id.0) {
                        Function::Lua(closure) => {
                            gray.extend(closure.upvals.iter().copied());
                            gray.push(Value::Table(closure.env));
                            gray_protos.push(closure.proto_id);
                        }
                        Function::Native(native) => native.func.trace(|v| gray.push(v)),
                    }
                }
            }
            Value::UpRef(id) => {
                if self.upvals.mark(id.0) {
                    // An open upvalue's value is a stack slot, which is a root.
                    if let Upval::Closed(v) = self.upval(id) {
                        gray.push(v);
                    }
                }
            }
            Value::Nil
            | Value::Boolean(_)
            | Value::Number(_)
            | Value::Vector(_)
            | Value::LightUserdata(..) => {}
        }
    }
}
