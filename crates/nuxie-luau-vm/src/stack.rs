//! The host's stack (docs/luau-safe-vm-embedding-core.md, sections 2.1
//! and 5.7): C++'s `lua_*` stack functions over one frame of the core's
//! stack, for a host of its own (the lane-6 API has its own, `HostCtx`).
//!
//! A [`Stack`] is a host function's frame, its arguments and then what it
//! pushed, or a frame at the top level ([`HostedVm::enter`](crate::hosted::HostedVm::enter)).
//! Index 1 is the frame's first value; negative indices count from the
//! top. Every object the host reaches through it is on the stack or in a
//! [`Root`], so it stays alive (plan decision D2): no operation hands out
//! an object reference a collection could free.
//!
//! A failed operation is a [`HostError`] that raises the VM's error (a
//! memory error, a bad key, a read-only table); its value is kept alive
//! until the host function returns.

use crate::error::Throw;
use crate::host::{HostError, HostErrorKind};
use crate::roots::Root;
use crate::value::Value;
use crate::vm::{Core, MIN_STACK};

/// The basic type of a value (C++ `lua_Type`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Nil,
    Boolean,
    LightUserdata,
    Number,
    Integer,
    Vector,
    String,
    Table,
    Function,
    Userdata,
    Thread,
    Buffer,
}

impl Type {
    fn of(value: Value) -> Type {
        match value {
            Value::Nil | Value::UpRef(_) => Type::Nil,
            Value::Boolean(_) => Type::Boolean,
            Value::LightUserdata(..) => Type::LightUserdata,
            Value::Number(_) => Type::Number,
            Value::Integer(_) => Type::Integer,
            Value::Vector(_) => Type::Vector,
            Value::String(_) => Type::String,
            Value::Table(_) => Type::Table,
            Value::Function(_) => Type::Function,
            Value::Userdata(_) => Type::Userdata,
            Value::Thread(_) => Type::Thread,
            Value::Buffer(_) => Type::Buffer,
        }
    }
}

impl Core {
    /// `throw` as a host error; its value stays alive until the running
    /// host function returns (`host_errors`).
    pub(crate) fn host_throw(&mut self, throw: Throw) -> HostError {
        self.host_errors.push(throw.value);
        HostError(HostErrorKind::Throw(throw))
    }
}

/// One frame of the core's stack, as the host sees it.
pub struct Stack<'a> {
    pub(crate) core: &'a mut Core,
    /// The absolute slot of index 1.
    pub(crate) base: usize,
}

impl Stack<'_> {
    /// The absolute slot of index `i`, if it is in the frame.
    pub(crate) fn slot(&self, i: isize) -> Option<usize> {
        let top = self.core.top;
        let slot = if i > 0 {
            self.base.checked_add(i.unsigned_abs())?.checked_sub(1)?
        } else if i < 0 {
            top.checked_sub(i.unsigned_abs())?
        } else {
            return None;
        };
        (slot >= self.base && slot < top).then_some(slot)
    }

    fn slot_or_error(&self, i: isize) -> Result<usize, HostError> {
        self.slot(i)
            .ok_or_else(|| HostError::raw(format!("stack index {i} is not in the frame")))
    }

    /// The value at `i`; nil when out of the frame (C++'s `LUA_TNONE`).
    fn value(&self, i: isize) -> Value {
        self.slot(i).map_or(Value::Nil, |s| self.core.get(s))
    }

    fn fail(&mut self, throw: Throw) -> HostError {
        self.core.host_throw(throw)
    }

    pub(crate) fn push(&mut self, value: Value) -> Result<(), HostError> {
        self.core.push(value).map_err(|t| self.core.host_throw(t))
    }

    /// Pops the top value.
    fn take(&mut self) -> Result<Value, HostError> {
        let slot = self.slot_or_error(-1)?;
        let v = self.core.get(slot);
        self.core.top = slot;
        Ok(v)
    }

    /// The table at `i`, writable (C++'s `api_check` that it is not
    /// read-only, as an error).
    fn writable_table(&mut self, i: isize) -> Result<crate::value::TableId, HostError> {
        let Value::Table(t) = self.value(i) else {
            return Err(HostError::raw(format!("stack index {i} is not a table")));
        };
        if self.core.heap.table(t).readonly {
            let throw = self.core.runtime_error("attempt to modify a readonly table");
            return Err(self.fail(throw));
        }
        Ok(t)
    }

    // ---- size ----------------------------------------------------------

    /// C++ `lua_gettop`: the values in the frame.
    pub fn top(&self) -> usize {
        self.core.top.saturating_sub(self.base)
    }

    /// C++ `lua_settop`: grows with nils or drops values.
    pub fn set_top(&mut self, n: usize) -> Result<(), HostError> {
        let new = self.base.saturating_add(n);
        if new > self.core.top {
            if let Err(t) = self.core.ensure_stack(new.saturating_add(MIN_STACK)) {
                return Err(self.fail(t));
            }
            let top = self.core.top;
            self.core.fill_nil(top, new);
        }
        self.core.top = new;
        Ok(())
    }

    /// Drops `n` values (C++ `lua_pop`); at most the frame's.
    pub fn pop(&mut self, n: usize) -> Result<(), HostError> {
        let top = self.top();
        self.set_top(top.saturating_sub(n))
    }

    // ---- reading ---------------------------------------------------------

    /// C++ `lua_type`; `None` out of the frame (`LUA_TNONE`).
    pub fn type_of(&self, i: isize) -> Option<Type> {
        self.slot(i).map(|s| Type::of(self.core.get(s)))
    }

    /// C++ `lua_toboolean`: Lua truthiness.
    pub fn to_boolean(&self, i: isize) -> bool {
        self.value(i).truthy()
    }

    /// C++ `lua_tonumberx`: a number, or a string that reads as one.
    pub fn to_number(&self, i: isize) -> Option<f64> {
        self.core.to_number(self.value(i))
    }

    /// The bytes of the string at `i` (`lua_tolstring` without the
    /// number-to-string conversion); `None` for anything else.
    pub fn string(&self, i: isize) -> Option<&[u8]> {
        match self.value(i) {
            Value::String(id) => Some(self.core.heap.str_bytes(id)),
            _ => None,
        }
    }

    /// The bytes of the buffer at `i` (C++ `lua_tobuffer`).
    pub fn buffer(&self, i: isize) -> Option<&[u8]> {
        match self.value(i) {
            Value::Buffer(id) => Some(self.core.heap.buffer(id)),
            _ => None,
        }
    }

    /// The bytes of the buffer at `i`, writable.
    pub fn buffer_mut(&mut self, i: isize) -> Option<&mut [u8]> {
        match self.value(i) {
            Value::Buffer(id) => Some(self.core.heap.buffer_mut(id)),
            _ => None,
        }
    }

    /// C++ `lua_topointer`: an identity for objects (stable while the
    /// object lives, unique per object kind), 0 for other values.
    pub fn to_pointer(&self, i: isize) -> u64 {
        self.core.pointer_of(self.value(i))
    }

    // ---- pushing -----------------------------------------------------------

    pub fn push_nil(&mut self) -> Result<(), HostError> {
        self.push(Value::Nil)
    }

    pub fn push_boolean(&mut self, b: bool) -> Result<(), HostError> {
        self.push(Value::Boolean(b))
    }

    pub fn push_number(&mut self, n: f64) -> Result<(), HostError> {
        self.push(Value::Number(n))
    }

    /// C++ `lua_pushlstring`.
    pub fn push_string(&mut self, bytes: &[u8]) -> Result<(), HostError> {
        match self.core.heap.intern(bytes) {
            Ok(s) => self.push(Value::String(s)),
            Err(t) => Err(self.fail(t)),
        }
    }

    /// C++ `lua_pushvalue`: a copy of the value at `i` (nil out of the
    /// frame).
    pub fn push_copy(&mut self, i: isize) -> Result<(), HostError> {
        let v = self.value(i);
        self.push(v)
    }

    /// C++ `lua_createtable`.
    pub fn push_table(&mut self, narray: usize, nhash: usize) -> Result<(), HostError> {
        match self.core.new_table(narray, nhash) {
            Ok(t) => self.push(Value::Table(t)),
            Err(t) => Err(self.fail(t)),
        }
    }

    /// C++ `lua_newbuffer`: pushes a zero-filled buffer of `len` bytes and
    /// returns them, writable.
    pub fn push_buffer(&mut self, len: usize) -> Result<&mut [u8], HostError> {
        let id = match self.core.new_buffer(len) {
            Ok(id) => id,
            Err(t) => return Err(self.fail(t)),
        };
        self.push(Value::Buffer(id))?;
        Ok(self.core.heap.buffer_mut(id))
    }

    /// The running thread's global table (C++ `LUA_GLOBALSINDEX`).
    pub fn push_globals(&mut self) -> Result<(), HostError> {
        let globals = self.core.globals;
        self.push(Value::Table(globals))
    }

    /// Pushes the value `root` keeps; a stale or foreign root is an error.
    pub fn push_root(&mut self, root: Root) -> Result<(), HostError> {
        let v = self.core.roots.get(root)?;
        self.push(v)
    }

    /// A new root for the value at `i` (C++ `lua_ref`), until the host
    /// releases it (`Core::release_root`).
    pub fn root(&mut self, i: isize) -> Result<Root, HostError> {
        let v = self.value(i);
        Ok(self.core.roots.insert(v)?)
    }

    // ---- tables ------------------------------------------------------------

    /// C++ `lua_rawset`: pops a value and a key and stores them in the
    /// table at `t`.
    pub fn raw_set(&mut self, t: isize) -> Result<(), HostError> {
        let table = self.writable_table(t)?;
        let v = self.take()?;
        let k = self.take()?;
        self.core.raw_set(table, k, v).map_err(|t| self.core.host_throw(t))
    }

    /// C++ `lua_setfield` on a table without `__newindex`: pops a value
    /// and stores it under the string `name`, raw.
    pub fn raw_set_field(&mut self, t: isize, name: &[u8]) -> Result<(), HostError> {
        let table = self.writable_table(t)?;
        let key = match self.core.heap.intern(name) {
            Ok(s) => Value::String(s),
            Err(t) => return Err(self.fail(t)),
        };
        let v = self.take()?;
        self.core.raw_set(table, key, v).map_err(|t| self.core.host_throw(t))
    }

    /// C++ `lua_rawseti`: pops a value and stores it at integer key `n`.
    pub fn raw_set_index(&mut self, t: isize, n: usize) -> Result<(), HostError> {
        let table = self.writable_table(t)?;
        let v = self.take()?;
        self.core
            .raw_set(table, Value::Number(n as f64), v)
            .map_err(|t| self.core.host_throw(t))
    }

    /// C++ `lua_setmetatable` for a table: pops a table (or nil) and makes
    /// it the metatable of the table at `i`.
    pub fn set_metatable(&mut self, i: isize) -> Result<(), HostError> {
        let table = self.writable_table(i)?;
        let meta = match self.take()? {
            Value::Nil => None,
            Value::Table(mt) => Some(mt),
            _ => return Err(HostError::raw("a metatable must be a table or nil")),
        };
        self.core.heap.table_mut(table).meta = meta;
        Ok(())
    }

    /// C++ `lua_next`: pops a key and pushes the next key and its value in
    /// the table at `t`; false, with nothing pushed, after the last.
    pub fn next(&mut self, t: isize) -> Result<bool, HostError> {
        let Value::Table(table) = self.value(t) else {
            return Err(HostError::raw(format!("stack index {t} is not a table")));
        };
        let key = self.take()?;
        let entries = self.core.heap.table(table);
        let Some(cursor) = entries.cursor_after(key) else {
            let throw = self.core.runtime_error("invalid key to 'next'");
            return Err(self.fail(throw));
        };
        match entries.next_from(cursor) {
            Some((_, k, v)) => {
                self.push(k)?;
                self.push(v)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}
