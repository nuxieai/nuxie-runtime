//! The reference handles (luaur-rt `Table`, `Function`, `LuaString`,
//! `Buffer`, `Thread`): a safe-VM handle plus the [`Lua`] it belongs to, so
//! operations take no VM argument.
//!
//! `luaur-rt` gives several operations infallible signatures (it aborts on
//! the failures they could meet). Here such an operation returns a neutral
//! value instead (an empty string, length 0, no metatable), and a handle
//! the VM could not create is a *failed* handle whose fallible operations
//! return the creation error. Neither path panics.

use std::ffi::c_void;
use std::rc::Rc;
use std::fmt;

use super::conversion::{FromLua, FromLuaMulti, IntoLua, IntoLuaMulti};
use super::error::{Error, NON_STRING_ERROR, Result};
use super::multi::MultiValue;
use super::state::Lua;
use super::value::{self, Value};
use super::vm;

macro_rules! handle {
    ($(#[$doc:meta])* $name:ident, $raw:ident, $variant:ident) => {
        $(#[$doc])*
        #[derive(Clone)]
        pub struct $name {
            pub(crate) lua: Lua,
            raw: std::result::Result<vm::$raw, Error>,
        }

        impl $name {
            pub(crate) fn wrap(lua: &Lua, raw: vm::$raw) -> Self {
                $name { lua: lua.clone(), raw: Ok(raw) }
            }

            #[allow(dead_code)]
            pub(crate) fn failed(lua: &Lua, error: Error) -> Self {
                $name { lua: lua.clone(), raw: Err(error) }
            }

            /// The safe-VM handle, or the error that kept it from existing.
            pub(crate) fn raw(&self) -> Result<&vm::$raw> {
                self.raw.as_ref().map_err(Clone::clone)
            }

            /// The owning [`Lua`].
            pub fn lua(&self) -> Lua {
                self.lua.clone()
            }

            /// The object's identity (`lua_topointer`); null for a failed
            /// handle.
            pub fn to_pointer(&self) -> *const c_void {
                let address = self
                    .lua
                    .0
                    .with_vm(|vm| vm.to_pointer(&self.vm_value()?).map_err(Error::from_vm))
                    .unwrap_or(0);
                std::ptr::without_provenance(usize::try_from(address).unwrap_or(usize::MAX))
            }

            pub(crate) fn vm_value(&self) -> Result<vm::Value> {
                Ok(vm::Value::$variant(self.raw()?.clone()))
            }
        }

    };
}

/// Identity, as `luaur-rt` compares reference handles: the same object of
/// the same VM (the safe VM's addresses are VM-local).
macro_rules! identity_eq {
    ($($name:ident),*) => {$(
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                Rc::ptr_eq(&self.lua.0, &other.lua.0) && self.to_pointer() == other.to_pointer()
            }
        }
    )*};
}

handle!(
    /// A table handle.
    Table,
    Table,
    Table
);
handle!(
    /// A function handle.
    Function,
    Function,
    Function
);
handle!(
    /// A string handle.
    LuaString,
    LuaString,
    String
);
handle!(
    /// A buffer handle.
    Buffer,
    Buffer,
    Buffer
);
handle!(
    /// A thread (coroutine) handle.
    Thread,
    Thread,
    Thread
);

identity_eq!(Table, Function, Buffer, Thread);

/// Strings compare by content, as `luaur-rt`'s `String` does, consistent
/// with its `Eq`, `Ord` and `Hash`.
impl PartialEq for LuaString {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

// ---- Table ------------------------------------------------------------------

impl Table {
    fn with<R>(&self, f: impl FnOnce(&mut vm::Vm, &vm::Table) -> Result<R>) -> Result<R> {
        let raw = self.raw()?;
        self.lua.0.with_vm(|vm| f(vm, raw))
    }

    fn lift(&self, vm: &mut vm::Vm, v: vm::Value) -> Result<Value> {
        value::from_vm(&self.lua, vm, v)
    }

    /// `table[key] = value`, metamethods honored.
    pub fn set<K: IntoLua, V: IntoLua>(&self, key: K, value: V) -> Result<()> {
        let key = key.into_lua(&self.lua)?;
        let value = value.into_lua(&self.lua)?;
        self.with(|vm, t| {
            let k = value::to_vm(vm, &key)?;
            let v = value::to_vm(vm, &value)?;
            t.set(vm, k, v).map_err(Error::from_vm)
        })
    }

    /// `table[key]`, metamethods honored.
    pub fn get<V: FromLua>(&self, key: impl IntoLua) -> Result<V> {
        let key = key.into_lua(&self.lua)?;
        let value = self.with(|vm, t| {
            let k = value::to_vm(vm, &key)?;
            let v: vm::Value = t.get(vm, k).map_err(Error::from_vm)?;
            self.lift(vm, v)
        })?;
        V::from_lua(value, &self.lua)
    }

    pub fn get_field<V: FromLua>(&self, field: &str) -> Result<V> {
        self.get(field)
    }

    pub fn raw_set<K: IntoLua, V: IntoLua>(&self, key: K, value: V) -> Result<()> {
        let key = key.into_lua(&self.lua)?;
        let value = value.into_lua(&self.lua)?;
        self.with(|vm, t| {
            let k = value::to_vm(vm, &key)?;
            let v = value::to_vm(vm, &value)?;
            t.raw_set(vm, k, v).map_err(Error::from_vm)
        })
    }

    pub fn raw_get<V: FromLua>(&self, key: impl IntoLua) -> Result<V> {
        let key = key.into_lua(&self.lua)?;
        let value = self.with(|vm, t| {
            let k = value::to_vm(vm, &key)?;
            let v: vm::Value = t.raw_get(vm, k).map_err(Error::from_vm)?;
            self.lift(vm, v)
        })?;
        V::from_lua(value, &self.lua)
    }

    pub fn raw_push<V: IntoLua>(&self, value: V) -> Result<()> {
        let value = value.into_lua(&self.lua)?;
        self.with(|vm, t| {
            let v = value::to_vm(vm, &value)?;
            t.raw_push(vm, v).map_err(Error::from_vm)
        })
    }

    /// `table[#table + 1] = value` (luaur-rt `push`, metamethods honored).
    pub fn push<V: IntoLua>(&self, value: V) -> Result<()> {
        let next = self.len()?.saturating_add(1);
        self.set(i64::try_from(next).unwrap_or(i64::MAX), value)
    }

    pub fn contains_key<K: IntoLua>(&self, key: K) -> Result<bool> {
        let v: Value = self.get(key)?;
        Ok(!v.is_nil())
    }

    /// The border length; 0 for a failed handle (luaur-rt: infallible).
    pub fn raw_len(&self) -> usize {
        self.with(|vm, t| t.raw_len(vm).map_err(Error::from_vm))
            .unwrap_or(0)
    }

    /// `#table`, `__len` honored.
    pub fn len(&self) -> Result<usize> {
        let n = self.with(|vm, t| t.len(vm).map_err(Error::from_vm))?;
        Ok(if n.is_finite() && n > 0.0 {
            // `#` of a table is an integer border, or `__len`'s number,
            // truncated as luaur-rt reads it through an `i64`.
            n.trunc().min(usize::MAX as f64) as usize
        } else {
            0
        })
    }

    pub fn is_empty(&self) -> bool {
        self.try_is_empty().unwrap_or(true)
    }

    pub fn try_is_empty(&self) -> Result<bool> {
        self.with(|vm, t| {
            let pairs: Vec<(vm::Value, vm::Value)> = t.pairs(vm).map_err(Error::from_vm)?;
            Ok(pairs.is_empty())
        })
    }

    /// Every pair, read at the call (the safe VM snapshots a traversal).
    pub fn pairs<K: FromLua, V: FromLua>(&self) -> TablePairs<K, V> {
        let snapshot = self.with(|vm, t| {
            let pairs: Vec<(vm::Value, vm::Value)> = t.pairs(vm).map_err(Error::from_vm)?;
            let mut out = Vec::with_capacity(pairs.len());
            for (k, v) in pairs {
                out.push((self.lift(vm, k)?, self.lift(vm, v)?));
            }
            Ok(out)
        });
        TablePairs {
            lua: self.lua.clone(),
            pairs: match snapshot {
                Ok(pairs) => Ok(pairs.into_iter()),
                Err(e) => Err(Some(e)),
            },
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn pairs_vec<K: FromLua, V: FromLua>(&self) -> Result<Vec<(K, V)>> {
        self.pairs().collect()
    }

    pub fn sequence_values<V: FromLua>(&self) -> TableSequence<V> {
        TableSequence {
            table: self.clone(),
            index: 1,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn for_each<K: FromLua, V: FromLua>(
        &self,
        mut f: impl FnMut(K, V) -> Result<()>,
    ) -> Result<()> {
        for pair in self.pairs::<K, V>() {
            let (k, v) = pair?;
            f(k, v)?;
        }
        Ok(())
    }

    pub fn for_each_value<V: FromLua>(&self, mut f: impl FnMut(V) -> Result<()>) -> Result<()> {
        for v in self.sequence_values::<V>() {
            f(v?)?;
        }
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        self.with(|vm, t| t.clear(vm).map_err(Error::from_vm))
    }

    /// The metatable; `None` for a failed handle (luaur-rt: infallible).
    pub fn metatable(&self) -> Option<Table> {
        self.try_metatable().ok().flatten()
    }

    pub fn try_metatable(&self) -> Result<Option<Table>> {
        let lua = self.lua.clone();
        self.with(|vm, t| {
            Ok(t.metatable(vm)
                .map_err(Error::from_vm)?
                .map(|m| Table::wrap(&lua, m)))
        })
    }

    pub fn set_metatable(&self, metatable: Option<Table>) -> Result<()> {
        let mt = match &metatable {
            Some(m) => Some(m.raw()?.clone()),
            None => None,
        };
        self.with(|vm, t| t.set_metatable(vm, mt.as_ref()).map_err(Error::from_vm))
    }

    pub fn is_readonly(&self) -> bool {
        self.with(|vm, t| t.is_readonly(vm).map_err(Error::from_vm))
            .unwrap_or(false)
    }

    /// Sets the read-only flag; a failed handle is left as it is
    /// (luaur-rt: infallible).
    pub fn set_readonly(&self, enabled: bool) {
        let _ = self.with(|vm, t| t.set_readonly(vm, enabled).map_err(Error::from_vm));
    }

    /// Marks the table a safe environment (`lua_setsafeenv`).
    pub fn set_safeenv(&self, enabled: bool) -> Result<()> {
        self.with(|vm, t| t.set_safeenv(vm, enabled).map_err(Error::from_vm))
    }

    /// Looks up `table[key]` and calls it with `args` for no results;
    /// `false` when it is not a function (luaur-rt `call_function_unit`).
    pub fn call_function_unit(&self, key: impl IntoLua, args: impl IntoLuaMulti) -> Result<bool> {
        let f: Value = self.get(key)?;
        match f {
            Value::Function(f) => {
                f.call::<()>(args)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Looks up `table[key]`, calls it for one result and reads that
    /// result's truthiness; `false` when it is not a function.
    pub fn call_function_truthy(&self, key: impl IntoLua, args: impl IntoLuaMulti) -> Result<bool> {
        let f: Value = self.get(key)?;
        match f {
            Value::Function(f) => {
                let r: Value = f.call(args)?;
                Ok(!matches!(r, Value::Nil | Value::Boolean(false)))
            }
            _ => Ok(false),
        }
    }
}

impl fmt::Debug for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Table(len={})", self.raw_len())
    }
}

impl<T> PartialEq<[T]> for Table
where
    T: FromLua + PartialEq + Clone,
{
    fn eq(&self, other: &[T]) -> bool {
        let mut iter = self.sequence_values::<T>();
        for expected in other.iter() {
            match iter.next() {
                Some(Ok(got)) if &got == expected => {}
                _ => return false,
            }
        }
        iter.next().is_none()
    }
}

impl<T, const N: usize> PartialEq<[T; N]> for Table
where
    T: FromLua + PartialEq + Clone,
{
    fn eq(&self, other: &[T; N]) -> bool {
        self == other.as_slice()
    }
}

impl<T> PartialEq<&[T]> for Table
where
    T: FromLua + PartialEq + Clone,
{
    fn eq(&self, other: &&[T]) -> bool {
        self == *other
    }
}

/// A table's sequence `1..`, read raw up to the first nil.
pub struct TableSequence<V> {
    table: Table,
    index: i64,
    _phantom: std::marker::PhantomData<V>,
}

impl<V: FromLua> Iterator for TableSequence<V> {
    type Item = Result<V>;

    fn next(&mut self) -> Option<Self::Item> {
        let value: Value = match self.table.raw_get(self.index) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        if value.is_nil() {
            return None;
        }
        self.index = self.index.saturating_add(1);
        Some(V::from_lua(value, &self.table.lua))
    }
}

/// A table's pairs, snapshotted when [`Table::pairs`] was called.
pub struct TablePairs<K, V> {
    lua: Lua,
    pairs: std::result::Result<std::vec::IntoIter<(Value, Value)>, Option<Error>>,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K: FromLua, V: FromLua> Iterator for TablePairs<K, V> {
    type Item = Result<(K, V)>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.pairs {
            Ok(iter) => {
                let (k, v) = iter.next()?;
                Some(
                    K::from_lua(k, &self.lua)
                        .and_then(|k| V::from_lua(v, &self.lua).map(|v| (k, v))),
                )
            }
            Err(error) => error.take().map(Err),
        }
    }
}

// ---- Function ---------------------------------------------------------------

impl Function {
    /// Calls the function, every result kept (`lua_pcall`, `LUA_MULTRET`).
    pub fn call<R: FromLuaMulti>(&self, args: impl IntoLuaMulti) -> Result<R> {
        let args = args.into_lua_multi(&self.lua)?;
        let raw = self.raw()?;
        let lua = self.lua.clone();
        let results = self.lua.0.with_vm(|vm| {
            let args = value::many_to_vm(vm, args.iter())?;
            let results: vm::MultiValue = raw.call(vm, args).map_err(Error::from_vm)?;
            value::many_from_vm(&lua, vm, results)
        })?;
        R::from_lua_multi(results, &self.lua)
    }

    /// A function that prepends `args` to its arguments and calls `self`.
    pub fn bind(&self, args: impl IntoLuaMulti) -> Result<Function> {
        let bound = args.into_lua_multi(&self.lua)?;
        let target = self.clone();
        self.lua
            .create_function(move |_, rest: MultiValue| {
                let mut all = bound.clone();
                all.extend(rest);
                target.call::<MultiValue>(all)
            })
    }

    /// The environment of a Lua function; `None` for a host function or a
    /// failed handle (luaur-rt: infallible).
    pub fn environment(&self) -> Option<Table> {
        self.try_environment().ok().flatten()
    }

    pub fn try_environment(&self) -> Result<Option<Table>> {
        let raw = self.raw()?;
        let lua = self.lua.clone();
        self.lua.0.with_vm(|vm| {
            Ok(raw
                .environment(vm)
                .map_err(Error::from_vm)?
                .map(|t| Table::wrap(&lua, t)))
        })
    }

    pub fn set_environment(&self, env: Table) -> Result<bool> {
        let raw = self.raw()?;
        let env = env.raw()?.clone();
        self.lua
            .0
            .with_vm(|vm| raw.set_environment(vm, &env).map_err(Error::from_vm))
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Function({:p})", self.to_pointer())
    }
}

// ---- LuaString --------------------------------------------------------------

impl LuaString {
    /// The bytes; empty for a failed handle (luaur-rt: infallible).
    pub fn as_bytes(&self) -> Vec<u8> {
        self.raw()
            .and_then(|raw| self.lua.0.with_vm(|vm| raw.as_bytes(vm).map_err(Error::from_vm)))
            .unwrap_or_default()
    }

    pub fn to_str(&self) -> Result<String> {
        let raw = self.raw()?;
        let bytes = self
            .lua
            .0
            .with_vm(|vm| raw.as_bytes(vm).map_err(Error::from_vm))?;
        String::from_utf8(bytes).map_err(|e| Error::FromLuaConversionError {
            from: "string",
            to: "String".to_string(),
            message: Some(format!("invalid utf-8: {e}")),
        })
    }

    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.as_bytes()).into_owned()
    }

    pub fn as_bytes_with_nul(&self) -> Vec<u8> {
        let mut v = self.as_bytes();
        v.push(0);
        v
    }

    pub fn display(&self) -> LuaStringDisplay {
        LuaStringDisplay(self.to_string_lossy())
    }
}

/// [`LuaString::display`].
pub struct LuaStringDisplay(String);

impl fmt::Display for LuaStringDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for LuaString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.as_bytes();
        match std::str::from_utf8(&bytes) {
            Ok(s) => write!(f, "{s:?}"),
            Err(_) => {
                f.write_str("b\"")?;
                for &b in &bytes {
                    match b {
                        b'\0' => f.write_str("\\0")?,
                        b'\r' => f.write_str("\\r")?,
                        b'\n' => f.write_str("\\n")?,
                        b'\t' => f.write_str("\\t")?,
                        b'\\' => f.write_str("\\\\")?,
                        b'"' => f.write_str("\\\"")?,
                        0x20..=0x7e => write!(f, "{}", char::from(b))?,
                        _ => write!(f, "\\x{b:02x}")?,
                    }
                }
                f.write_str("\"")
            }
        }
    }
}

impl Eq for LuaString {}

impl PartialOrd for LuaString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LuaString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_bytes().cmp(&other.as_bytes())
    }
}

impl std::hash::Hash for LuaString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl PartialEq<str> for LuaString {
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<&str> for LuaString {
    fn eq(&self, other: &&str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<String> for LuaString {
    fn eq(&self, other: &String) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<[u8]> for LuaString {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_bytes() == other
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for LuaString {
    fn eq(&self, other: &&[u8; N]) -> bool {
        self.as_bytes() == other.as_slice()
    }
}

// ---- Buffer -----------------------------------------------------------------

impl Buffer {
    /// The contents; empty for a failed handle (luaur-rt: infallible).
    pub fn to_vec(&self) -> Vec<u8> {
        self.raw()
            .and_then(|raw| self.lua.0.with_vm(|vm| raw.to_vec(vm).map_err(Error::from_vm)))
            .unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.raw()
            .and_then(|raw| self.lua.0.with_vm(|vm| raw.len(vm).map_err(Error::from_vm)))
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `N` bytes at `offset`; zeros where the range is out of bounds
    /// (luaur-rt panics there).
    pub fn read_bytes<const N: usize>(&self, offset: usize) -> [u8; N] {
        let mut out = [0u8; N];
        let read = self
            .raw()
            .and_then(|raw| {
                self.lua
                    .0
                    .with_vm(|vm| raw.read_bytes(vm, offset, N).map_err(Error::from_vm))
            })
            .unwrap_or_default();
        for (slot, byte) in out.iter_mut().zip(read) {
            *slot = byte;
        }
        out
    }

    /// Writes `bytes` at `offset`; an out-of-bounds write does nothing
    /// (luaur-rt panics there).
    pub fn write_bytes(&self, offset: usize, bytes: &[u8]) {
        if let Ok(raw) = self.raw() {
            let _ = self
                .lua
                .0
                .with_vm(|vm| raw.write_bytes(vm, offset, bytes).map_err(Error::from_vm));
        }
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Buffer(len={})", self.len())
    }
}

// ---- Thread -----------------------------------------------------------------

/// A thread's status (luaur-rt `ThreadStatus`).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ThreadStatus {
    Resumable,
    Running,
    Normal,
    Finished,
    Error,
}

impl Thread {
    /// The status; `Error` for a failed handle (luaur-rt: infallible).
    pub fn status(&self) -> ThreadStatus {
        let state = self
            .raw()
            .and_then(|raw| self.lua.0.with_vm(|vm| raw.status(vm).map_err(Error::from_vm)));
        match state {
            Ok(vm::ThreadState::Resumable) => ThreadStatus::Resumable,
            Ok(vm::ThreadState::Running) => ThreadStatus::Running,
            Ok(vm::ThreadState::Normal) => ThreadStatus::Normal,
            Ok(vm::ThreadState::Finished) => ThreadStatus::Finished,
            Ok(vm::ThreadState::Error) | Err(_) => ThreadStatus::Error,
        }
    }

    pub fn is_resumable(&self) -> bool {
        self.status() == ThreadStatus::Resumable
    }

    pub fn is_running(&self) -> bool {
        self.status() == ThreadStatus::Running
    }

    pub fn is_normal(&self) -> bool {
        self.status() == ThreadStatus::Normal
    }

    pub fn is_finished(&self) -> bool {
        self.status() == ThreadStatus::Finished
    }

    pub fn is_error(&self) -> bool {
        self.status() == ThreadStatus::Error
    }

    pub fn resume<R: FromLuaMulti>(&self, args: impl IntoLuaMulti) -> Result<R> {
        let args = args.into_lua_multi(&self.lua)?;
        if self.status() != ThreadStatus::Resumable {
            return Err(Error::CoroutineUnresumable);
        }
        let results = self.resume_raw(args)?;
        R::from_lua_multi(results, &self.lua)
    }

    fn resume_raw(&self, args: MultiValue) -> Result<MultiValue> {
        let raw = self.raw()?;
        let lua = self.lua.clone();
        self.lua.0.with_vm(|vm| {
            let args = value::many_to_vm(vm, args.iter())?;
            let results: vm::MultiValue = raw.resume(vm, args).map_err(Error::from_vm)?;
            value::many_from_vm(&lua, vm, results)
        })
    }

    /// A value on the thread's stack as C++ leaves it (luaur-rt
    /// `stack_value`): 1 is the bottom, negative counts from the top, nil
    /// outside the stack. Inside a module error hook, the top is the
    /// message `resume_module` reports.
    pub fn stack_value(&self, index: i32) -> Result<Value> {
        if index == -1 {
            let key = self.to_pointer().addr();
            let top = self
                .lua
                .0
                .module_error_tops
                .try_borrow()
                .ok()
                .and_then(|tops| tops.get(&key).cloned());
            if let Some(top) = top {
                return Ok(top);
            }
        }
        let raw = self.raw()?;
        let lua = self.lua.clone();
        self.lua.0.with_vm(|vm| {
            let v = raw.stack_value(vm, index).map_err(Error::from_vm)?;
            value::from_vm(&lua, vm, v)
        })
    }

    /// Runs `on_error` with `message` as the thread's reported top.
    fn with_error_top(&self, message: &str, on_error: impl FnOnce(&Thread)) {
        let key = self.to_pointer().addr();
        let top = Value::String(self.lua.create_string(message));
        if let Ok(mut tops) = self.lua.0.module_error_tops.try_borrow_mut() {
            tops.insert(key, top);
        }
        on_error(self);
        if let Ok(mut tops) = self.lua.0.module_error_tops.try_borrow_mut() {
            tops.remove(&key);
        }
    }

    /// Runs a module body to its end (the UNIV c5faa1fa patch's
    /// `resume_module`): it must return one table or function, and must not
    /// yield. `begin` runs right before the resume and its guard drops
    /// right after; `on_error` sees the thread when the body failed. The
    /// messages are the patch's, prefixed `"{display}:1: "`.
    pub fn resume_module<R: FromLuaMulti, G>(
        &self,
        display: &str,
        begin: impl FnOnce() -> Result<G>,
        on_error: impl FnOnce(&Thread),
    ) -> Result<R> {
        if self.status() != ThreadStatus::Resumable {
            return Err(Error::CoroutineUnresumable);
        }
        let execution = begin()?;
        let outcome = self.resume_raw(MultiValue::new());
        drop(execution);
        let results = match outcome {
            Ok(results) => results,
            // A non-string error object: the hook sees C++'s text, and a
            // structured host error keeps its cause (luaur-rt `host_cause`).
            Err(error)
                if matches!(&error, Error::CallbackError { .. })
                    || matches!(&error, Error::RuntimeError(m) if m == NON_STRING_ERROR) =>
            {
                let message = format!("{display}:1: unknown error while running module");
                self.with_error_top(&message, on_error);
                return Err(match error {
                    Error::CallbackError { .. } => error,
                    _ => Error::RuntimeError(message),
                });
            }
            Err(error) => {
                // The thread died with its error on top (`stack_value`).
                on_error(self);
                return Err(error);
            }
        };
        if self.status() == ThreadStatus::Resumable {
            let message = format!("{display}:1: module can not yield");
            self.with_error_top(&message, on_error);
            return Err(Error::RuntimeError(message));
        }
        let Some(last) = results.iter().last().cloned() else {
            return Err(Error::RuntimeError(format!(
                "{display}:1: module must return a value"
            )));
        };
        if !matches!(last, Value::Table(_) | Value::Function(_)) {
            return Err(Error::RuntimeError(format!(
                "{display}:1: module must return a table or function"
            )));
        }
        R::from_lua_multi(MultiValue::from_vec(vec![last]), &self.lua)
    }
}

impl fmt::Debug for Thread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Thread({:p})", self.to_pointer())
    }
}
