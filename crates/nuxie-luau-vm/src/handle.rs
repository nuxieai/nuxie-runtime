//! Handles: VM objects the host holds across calls (phase 2 lane 6).
//!
//! A handle is an `Rc` to a marker owning one core [`Root`]
//! (`roots.rs`), so the object lives at least as long as some clone of the
//! handle. Dropping the last clone pushes the root onto a release queue
//! the lane-6 host (`Compat`) shares with its markers; `Compat` releases
//! the queued roots at its drain points (`Vm::drain_freed`), so a dropped
//! handle's object is collectable from the next drain on. Dropping never
//! touches the VM and is valid anywhere (in a userdata destructor, after
//! the coroutine that made the handle died, or after the VM itself is
//! gone, when the queue is gone too); it does not allocate either, since
//! the queue reserves room for every live handle when one is made.
//!
//! Handles are VM-scoped, not thread-scoped: a handle made on one
//! coroutine is valid on every thread of the same VM (seam doc 3.9, the
//! UNIV-1764 bug class). A handle's root carries its VM's id; using it
//! with another VM is [`Error::Host`], never a wrong object.
//!
//! The typed handles ([`Table`], [`Function`], [`LuaString`],
//! [`AnyUserData`], [`Thread`], [`Buffer`]) take the VM as an argument to
//! every operation. A host callback's [`HostCtx`](crate::HostCtx) derefs
//! to the [`Vm`], so `table.get(ctx, "x")` works inside callbacks too.
//!
//! Handle memory is host memory, not charged to the script's budget: a
//! handle exists only while the host keeps it.

use std::cell::Cell;
use std::rc::{Rc, Weak};

use crate::roots::Root;

/// Roots whose handles were dropped, waiting for `Compat` to release them.
#[derive(Default)]
pub(crate) struct ReleaseQueue(Cell<Vec<Root>>);

impl ReleaseQueue {
    fn push(&self, root: Root) {
        let mut queued = self.0.take();
        queued.push(root);
        self.0.set(queued);
    }

    /// Makes room for `n` more roots, so `n` drops push without allocating.
    pub(crate) fn reserve(&self, n: usize) {
        let mut queued = self.0.take();
        queued.reserve(n);
        self.0.set(queued);
    }

    /// The queued roots, in a vector to hand back with [`Self::restore`].
    pub(crate) fn take(&self) -> Vec<Root> {
        self.0.take()
    }

    /// Puts an emptied vector back, keeping its capacity.
    pub(crate) fn restore(&self, mut emptied: Vec<Root>) {
        emptied.clear();
        let queued = self.0.take();
        emptied.extend(queued);
        self.0.set(emptied);
    }

    pub(crate) fn len(&self) -> usize {
        let queued = self.0.take();
        let n = queued.len();
        self.0.set(queued);
        n
    }
}

#[derive(Debug)]
pub(crate) struct Marker {
    root: Root,
    queue: Weak<ReleaseQueue>,
}

impl Drop for Marker {
    fn drop(&mut self) {
        if let Some(queue) = self.queue.upgrade() {
            queue.push(self.root);
        }
    }
}

impl std::fmt::Debug for ReleaseQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ReleaseQueue")
    }
}

/// A rooted reference to a VM object. Cloning shares the root.
#[derive(Clone, Debug)]
pub(crate) struct Handle(Rc<Marker>);

impl Handle {
    /// A handle owning `root`, released through `queue` when dropped.
    pub(crate) fn new(root: Root, queue: &Rc<ReleaseQueue>) -> Handle {
        Handle(Rc::new(Marker {
            root,
            queue: Rc::downgrade(queue),
        }))
    }

    /// The root this handle owns.
    #[inline]
    pub(crate) fn root(&self) -> Root {
        self.0.root
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
