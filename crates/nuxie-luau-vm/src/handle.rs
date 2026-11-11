//! Handles: VM objects the host holds across calls (phase 2 lane 6).
//!
//! A handle is an `Rc` to a small marker naming a slot of the VM's handle
//! table ([`HandleTable`]). The slot holds the object's value and a `Weak`
//! to the marker. The collector roots every slot whose marker is alive
//! (`Vm::gc_roots`), so the object lives at least as long as some clone of
//! the handle. Dropping the last clone only decrements a reference count:
//! it never allocates, never touches the VM and is valid anywhere (in a
//! userdata destructor, after the coroutine that made the handle died, or
//! after the VM itself is gone). The dead slot is reclaimed the next time
//! the table is scanned: at every collection, and when the table has
//! doubled since the last scan.
//!
//! Handles are VM-scoped, not thread-scoped: a handle made on one
//! coroutine is valid on every thread of the same VM (seam doc 3.9, the
//! UNIV-1764 bug class). A handle carries its VM's id; using it with
//! another VM is [`Error::Host`], never a wrong object.
//!
//! The typed handles ([`Table`], [`Function`], [`LuaString`],
//! [`AnyUserData`], [`Thread`], [`Buffer`]) take the VM as an argument to
//! every operation. A host callback's [`HostCtx`](crate::HostCtx) derefs
//! to the [`Vm`], so `table.get(ctx, "x")` works inside callbacks too.
//!
//! Handle memory is host memory, not charged to the script's budget: a
//! handle exists only while the host keeps it.

use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::Error;
use crate::value::Value as RawValue;

/// Distinguishes VMs, so a handle used with the wrong VM is caught.
static NEXT_VM_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) fn next_vm_id() -> u64 {
    NEXT_VM_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug)]
pub(crate) struct Marker {
    vm: u64,
    slot: u32,
}

/// A rooted reference to a VM object. Cloning shares the root.
#[derive(Clone, Debug)]
pub(crate) struct Handle(Rc<Marker>);

struct Slot {
    value: RawValue,
    owner: Weak<Marker>,
}

/// The VM's table of host roots.
pub(crate) struct HandleTable {
    vm: u64,
    slots: Vec<Slot>,
    free: Vec<u32>,
    /// Scan for dead slots when the table reaches this length.
    reclaim_at: usize,
}

const MIN_RECLAIM: usize = 64;

impl HandleTable {
    pub(crate) fn new(vm: u64) -> HandleTable {
        HandleTable {
            vm,
            slots: Vec::new(),
            free: Vec::new(),
            reclaim_at: MIN_RECLAIM,
        }
    }

    pub(crate) fn vm_id(&self) -> u64 {
        self.vm
    }

    /// A new handle rooting `value`.
    pub(crate) fn insert(&mut self, value: RawValue) -> Result<Handle, Error> {
        if self.free.is_empty() && self.slots.len() >= self.reclaim_at {
            self.reclaim();
            self.reclaim_at = self.live().saturating_mul(2).max(MIN_RECLAIM);
        }
        let slot = match self.free.pop() {
            Some(slot) => slot,
            None => {
                let slot = u32::try_from(self.slots.len())
                    .map_err(|_| Error::Host("too many live handles".to_owned()))?;
                self.slots.push(Slot {
                    value: RawValue::Nil,
                    owner: Weak::new(),
                });
                slot
            }
        };
        let marker = Rc::new(Marker { vm: self.vm, slot });
        if let Some(entry) = self.slots.get_mut(slot as usize) {
            entry.value = value;
            entry.owner = Rc::downgrade(&marker);
        }
        Ok(Handle(marker))
    }

    /// The value `handle` roots, or an error for another VM's handle.
    pub(crate) fn get(&self, handle: &Handle) -> Result<RawValue, Error> {
        let marker = &handle.0;
        if marker.vm != self.vm {
            return Err(Error::Host(
                "a handle was used with a VM other than the one that made it".to_owned(),
            ));
        }
        match self.slots.get(marker.slot as usize) {
            Some(entry) if std::ptr::eq(entry.owner.as_ptr(), Rc::as_ptr(marker)) => {
                Ok(entry.value)
            }
            _ => Err(Error::Host("a stale handle".to_owned())),
        }
    }

    fn live(&self) -> usize {
        self.slots
            .iter()
            .filter(|s| s.owner.strong_count() > 0)
            .count()
    }

    /// Frees every slot whose handles are all dropped.
    fn reclaim(&mut self) {
        self.free.clear();
        for (index, entry) in self.slots.iter_mut().enumerate() {
            if entry.owner.strong_count() == 0 {
                entry.value = RawValue::Nil;
                entry.owner = Weak::new();
                if let Ok(index) = u32::try_from(index) {
                    self.free.push(index);
                }
            }
        }
    }

    /// Reclaims dead slots and appends the live values (GC roots).
    pub(crate) fn roots(&mut self, out: &mut Vec<RawValue>) {
        self.reclaim();
        out.extend(
            self.slots
                .iter()
                .filter(|s| s.owner.strong_count() > 0)
                .map(|s| s.value),
        );
    }

    /// Live handles (tests and the drift check).
    pub(crate) fn live_count(&self) -> usize {
        self.live()
    }
}

macro_rules! typed_handle {
    ($(#[$doc:meta])* $name:ident, $kind:literal) => {
        $(#[$doc])*
        #[derive(Clone, Debug)]
        pub struct $name(pub(crate) Handle);

        impl $name {
            /// The Luau type name.
            pub const TYPE_NAME: &'static str = $kind;
        }
    };
}

typed_handle!(
    /// A table.
    Table,
    "table"
);
typed_handle!(
    /// A function: a Lua closure or a host function.
    Function,
    "function"
);
typed_handle!(
    /// An interned string.
    LuaString,
    "string"
);
typed_handle!(
    /// A full userdata of any host type (`userdata.rs`).
    AnyUserData,
    "userdata"
);
typed_handle!(
    /// A thread (coroutine).
    Thread,
    "thread"
);
typed_handle!(
    /// A buffer.
    Buffer,
    "buffer"
);

/// A light userdata: an opaque word and a tag (C++
/// `lua_pushlightuserdatatagged`). It references nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LightUserData {
    pub ptr: u64,
    pub tag: u8,
}

/// A value the host holds. Objects are handles, rooted while held.
#[derive(Clone, Debug, Default)]
pub enum Value {
    #[default]
    Nil,
    Boolean(bool),
    Number(f64),
    /// The integer type (`1i`).
    Integer(i64),
    Vector([f32; 3]),
    LightUserData(LightUserData),
    String(LuaString),
    Table(Table),
    Function(Function),
    Thread(Thread),
    UserData(AnyUserData),
    Buffer(Buffer),
}

impl Value {
    /// The Luau type name (`type()`, without `__type`).
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Boolean(_) => "boolean",
            Value::Number(_) => "number",
            Value::Integer(_) => "integer",
            Value::Vector(_) => "vector",
            Value::LightUserData(_) | Value::UserData(_) => "userdata",
            Value::String(_) => "string",
            Value::Table(_) => "table",
            Value::Function(_) => "function",
            Value::Thread(_) => "thread",
            Value::Buffer(_) => "buffer",
        }
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    pub fn as_table(&self) -> Option<&Table> {
        match self {
            Value::Table(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_function(&self) -> Option<&Function> {
        match self {
            Value::Function(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_userdata(&self) -> Option<&AnyUserData> {
        match self {
            Value::UserData(u) => Some(u),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

/// Several values: a function's arguments or results.
#[derive(Clone, Debug, Default)]
pub struct MultiValue(pub Vec<Value>);

impl MultiValue {
    pub fn new() -> MultiValue {
        MultiValue(Vec::new())
    }

    pub fn into_vec(self) -> Vec<Value> {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<Value>> for MultiValue {
    fn from(values: Vec<Value>) -> MultiValue {
        MultiValue(values)
    }
}
