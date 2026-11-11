//! The embedding API (phase 2 lane 6): what `nuxie-scripting` holds and
//! calls. Every seam row the seam doc (`docs/luau-safe-vm-seam.md`) marks
//! lift or reimplement maps to an item here; the evidence doc
//! (`docs/evidence/luau-safe-vm-phase2-lane6-2026-11-11.md`) has the table.
//!
//! **Handles** (`handle.rs`) are the host's references: rooted while held,
//! VM-scoped (valid on every thread), dropped without touching the VM.
//!
//! **Where the API runs.** At the host's top level (no host callback on
//! the Rust stack) and inside callbacks (through [`HostCtx`], which derefs
//! to the [`Vm`]). Every operation that can run Lua is protected: an error
//! unwinds what the operation pushed and comes back as [`Error`], as C++
//! `lua_pcall` from the host does. At top level, every operation ends at a
//! safepoint, after its results were turned into handles. The one place
//! the API refuses to run is on top of a main thread parked in a yield
//! ([`Vm::start`]); it returns [`Error::Host`] there, as C++ forbids a call
//! on a yielded thread.
//!
//! **Calls** from the host run on the running thread, counted as one C
//! call and not yieldable (C++ `lua_pcall`), each one Rust-stack level
//! bounded by `Limits::max_reentry`. A host callback that yields, calls a
//! function that yields, or re-enters too deep gets an error, never an
//! abort.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::call::{CallMode, Catch, Continuation, Outcome, Request};
use crate::convert::{FromLua, FromLuaMulti, IntoLua, IntoLuaMulti, conversion};
use crate::error::{Error, ErrorKind, Throw, VmResult};
use crate::handle::{
    AnyUserData, Buffer, Function, Handle, LightUserData, LuaString, MultiValue, Table, Thread,
    Value,
};
use crate::heap::{Function as HeapFunction, NativeClosure};
use crate::host::{CallStatus, HostCtx, HostError, HostErrorKind};
use crate::native::{Args, HostContFn, HostFn, NativeFn};
use crate::thread::{CoStatus, ThreadStatus};
use crate::userdata::{UTAG_LIMIT, Userdata};
use crate::value::{TableId, UdId, Value as RawValue};
use crate::vm::{InterruptFn, MIN_STACK, MULTRET, Vm};

/// A thread's status as the running thread sees it (C++ `lua_costatus`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThreadState {
    /// Not started, or yielded: `resume` continues it.
    Resumable,
    /// The running thread.
    Running,
    /// Resumed another thread and waits for it.
    Normal,
    /// Returned.
    Finished,
    /// Died with an error.
    Error,
}

/// One frame of the call stack, as `inspect_stack` reports it (C++
/// `lua_Debug` after `lua_getinfo(level, "sl")`).
#[derive(Clone, Debug, PartialEq)]
pub struct StackInfo {
    /// The chunk name, verbatim (`ar.source`); `"=[C]"` for a native.
    pub source: Vec<u8>,
    /// The current line, or -1 for a native.
    pub current_line: i32,
    /// Whether the frame runs a host or builtin function.
    pub is_native: bool,
}

/// Direct-field getters for one userdata tag: key bytes to getter.
pub(crate) struct DirectFields {
    getters: HashMap<Vec<u8>, DirectGetter>,
}

type DirectGetter = Rc<dyn Fn(&dyn Any) -> Option<f64>>;

/// One direct field: its key and its getter on the payload.
pub type DirectField<'a, T> = (&'a str, fn(&T) -> f64);

// ---- raw <-> handle -------------------------------------------------------

impl Vm {
    /// The handle form of a raw value (objects get a rooted handle).
    pub(crate) fn wrap(&mut self, v: RawValue) -> Result<Value, Error> {
        let object = |vm: &mut Vm, v| vm.handles.insert(v);
        Ok(match v {
            RawValue::Nil | RawValue::UpRef(_) => Value::Nil,
            RawValue::Boolean(b) => Value::Boolean(b),
            RawValue::Number(n) => Value::Number(n),
            RawValue::Integer(i) => Value::Integer(i),
            RawValue::Vector(v) => Value::Vector(v),
            RawValue::LightUserdata(ptr, tag) => Value::LightUserData(LightUserData { ptr, tag }),
            RawValue::String(_) => Value::String(LuaString(object(self, v)?)),
            RawValue::Table(_) => Value::Table(Table(object(self, v)?)),
            RawValue::Function(_) => Value::Function(Function(object(self, v)?)),
            RawValue::Thread(_) => Value::Thread(Thread(object(self, v)?)),
            RawValue::Buffer(_) => Value::Buffer(Buffer(object(self, v)?)),
            RawValue::Userdata(_) => Value::UserData(AnyUserData(object(self, v)?)),
        })
    }

    /// The raw value of a handle value; an error for another VM's handle.
    pub(crate) fn lower(&self, v: &Value) -> Result<RawValue, Error> {
        Ok(match v {
            Value::Nil => RawValue::Nil,
            Value::Boolean(b) => RawValue::Boolean(*b),
            Value::Number(n) => RawValue::Number(*n),
            Value::Integer(i) => RawValue::Integer(*i),
            Value::Vector(v) => RawValue::Vector(*v),
            Value::LightUserData(l) => RawValue::LightUserdata(l.ptr, l.tag),
            Value::String(h) => self.handles.get(&h.0)?,
            Value::Table(h) => self.handles.get(&h.0)?,
            Value::Function(h) => self.handles.get(&h.0)?,
            Value::Thread(h) => self.handles.get(&h.0)?,
            Value::UserData(h) => self.handles.get(&h.0)?,
            Value::Buffer(h) => self.handles.get(&h.0)?,
        })
    }

    /// Converts and lowers `v`. The returned `Value` holds the handle that
    /// roots the raw value: keep it alive until the raw value is stored or
    /// on the stack, since any later conversion or allocation can collect.
    fn lower_into(&mut self, v: impl IntoLua) -> Result<(RawValue, Value), Error> {
        let v = v.into_lua(self)?;
        Ok((self.lower(&v)?, v))
    }

    fn table_id(&self, h: &Handle) -> Result<TableId, Error> {
        match self.handles.get(h)? {
            RawValue::Table(id) => Ok(id),
            other => Err(Error::Host(format!("expected a table, got {}", other.type_name()))),
        }
    }

    fn ud_id(&self, h: &Handle) -> Result<UdId, Error> {
        match self.handles.get(h)? {
            RawValue::Userdata(id) => Ok(id),
            other => Err(Error::Host(format!(
                "expected a userdata, got {}",
                other.type_name()
            ))),
        }
    }

    /// The type name of a value, as `typeof` sees it (`__type` honored for
    /// userdata), for conversion errors.
    pub(crate) fn value_type_name(&self, v: &Value) -> String {
        match self.lower(v) {
            Ok(raw) => self.obj_type_name(raw),
            Err(_) => v.type_name().to_owned(),
        }
    }

    // ---- protection ----------------------------------------------------

    /// Refuses the API on top of a main thread parked in a yield.
    fn check_entry(&self) -> Result<(), Error> {
        if self.host_depth == 0 && (!self.frames.is_empty() || self.current != self.main_thread)
        {
            return Err(Error::Host(
                "the main thread is suspended in a yield; resume it before calling into the VM"
                    .to_owned(),
            ));
        }
        Ok(())
    }

    /// Runs `op` as a protected call from the host: an error unwinds the
    /// frames and stack it left and is rendered.
    pub(crate) fn protect<R>(
        &mut self,
        op: impl FnOnce(&mut Vm) -> VmResult<R>,
    ) -> Result<R, Error> {
        self.check_entry()?;
        let (depth, top, saved) = (self.frames.len(), self.top, self.saved_calls());
        match op(self) {
            Ok(r) => Ok(r),
            Err(throw) => {
                self.unwind(depth, top, saved);
                self.top = top;
                if depth == 0 {
                    self.request = None;
                }
                Err(self.render_error(throw))
            }
        }
    }

    /// The safepoint at the end of a top-level operation (nothing on the
    /// stack is live there; handles are roots).
    pub(crate) fn settle(&mut self) {
        if self.host_depth == 0 && self.frames.is_empty() && self.current == self.main_thread {
            self.top = 0;
            self.safepoint();
        }
    }

    /// `protect`, then the result as a handle value, then `settle`.
    fn protect_value(
        &mut self,
        op: impl FnOnce(&mut Vm) -> VmResult<RawValue>,
    ) -> Result<Value, Error> {
        let raw = self.protect(op);
        let out = raw.and_then(|raw| self.wrap(raw));
        self.settle();
        out
    }

    /// Calls `f` with `args` and returns every result (C++ `lua_pcall`
    /// with `LUA_MULTRET`).
    pub(crate) fn call_raw(&mut self, f: RawValue, args: &[RawValue]) -> Result<MultiValue, Error> {
        let top = self.top;
        let at = self.scratch();
        let called = self.protect(|vm| {
            vm.ensure_stack(
                at.saturating_add(1)
                    .saturating_add(args.len())
                    .saturating_add(MIN_STACK),
            )?;
            vm.set(at, f);
            for (i, &v) in args.iter().enumerate() {
                vm.set(at.saturating_add(1).saturating_add(i), v);
            }
            vm.top = at.saturating_add(1).saturating_add(args.len());
            vm.call_value(at, args.len(), MULTRET)
        });
        let out = called.and_then(|()| {
            let end = self.top;
            let mut values = Vec::with_capacity(end.saturating_sub(at));
            for slot in at..end {
                let v = self.get(slot);
                values.push(self.wrap(v)?);
            }
            Ok(MultiValue(values))
        });
        self.top = top;
        self.settle();
        out
    }

    // ---- creating values -----------------------------------------------

    /// The running thread's global table (C++ `LUA_GLOBALSINDEX`).
    pub fn globals(&mut self) -> Result<Table, Error> {
        let globals = self.globals;
        self.handles.insert(RawValue::Table(globals)).map(Table)
    }

    /// A new empty table.
    pub fn create_table(&mut self) -> Result<Table, Error> {
        self.create_table_with_capacity(0, 0)
    }

    /// A new table with room for `narray` sequence and `nhash` other
    /// entries (C++ `lua_createtable`).
    pub fn create_table_with_capacity(&mut self, narray: usize, nhash: usize) -> Result<Table, Error> {
        let raw = self.protect(|vm| vm.new_table(narray, nhash).map(RawValue::Table))?;
        let out = self.handles.insert(raw).map(Table);
        self.settle();
        out
    }

    /// A sequence table of `values`.
    pub fn create_sequence_from<T: IntoLua>(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> Result<Table, Error> {
        let table = self.create_table()?;
        for (i, v) in values.into_iter().enumerate() {
            table.raw_set(self, i.saturating_add(1) as f64, v)?;
        }
        Ok(table)
    }

    /// An interned string (C++ `lua_pushlstring`).
    pub fn create_string(&mut self, bytes: impl AsRef<[u8]>) -> Result<LuaString, Error> {
        let bytes = bytes.as_ref();
        let raw = self.protect(|vm| {
            if bytes.len() > vm.heap.string_limit {
                return Err(vm.heap.memory_error());
            }
            vm.heap.intern(bytes).map(RawValue::String)
        })?;
        let out = self.handles.insert(raw).map(LuaString);
        self.settle();
        out
    }

    /// A zero-filled buffer of `len` bytes (C++ `lua_newbuffer`).
    pub fn create_buffer(&mut self, len: usize) -> Result<Buffer, Error> {
        let raw = self.protect(|vm| vm.new_buffer(len).map(RawValue::Buffer))?;
        let out = self.handles.insert(raw).map(Buffer);
        self.settle();
        out
    }

    /// A buffer holding a copy of `bytes`.
    pub fn create_buffer_from(&mut self, bytes: &[u8]) -> Result<Buffer, Error> {
        let buffer = self.create_buffer(bytes.len())?;
        buffer.write_bytes(self, 0, bytes)?;
        Ok(buffer)
    }

    fn new_native(&mut self, name: &str, func: NativeFn) -> Result<Function, Error> {
        let raw = self.protect(|vm| {
            vm.heap
                .new_function(HeapFunction::Native(NativeClosure {
                    func,
                    name: Rc::from(name),
                }))
                .map(RawValue::Function)
        })?;
        let out = self.handles.insert(raw).map(Function);
        self.settle();
        out
    }

    /// A host function working on its stack (C++ `lua_pushcclosurek`
    /// without a continuation). `name` is its debug name, which argument
    /// errors print (`invalid argument #1 to 'name' ...`).
    pub fn create_host_function(
        &mut self,
        name: &str,
        f: impl Fn(&mut HostCtx<'_>) -> Result<usize, HostError> + 'static,
    ) -> Result<Function, Error> {
        let host: Rc<HostFn> = Rc::new(f);
        self.new_native(name, NativeFn::Host(host))
    }

    /// A host function with a continuation (C++ `lua_pushcclosurek`): `k`
    /// runs when a call it requested with [`HostCtx::call_yieldable`] or
    /// [`HostCtx::pcall_yieldable`] finishes, or when a yield it made with
    /// [`HostCtx::yield_values`] is resumed.
    pub fn create_host_function_k(
        &mut self,
        name: &str,
        f: impl Fn(&mut HostCtx<'_>) -> Result<usize, HostError> + 'static,
        k: impl Fn(&mut HostCtx<'_>, CallStatus) -> Result<usize, HostError> + 'static,
    ) -> Result<Function, Error> {
        let host: Rc<HostFn> = Rc::new(f);
        let cont: Rc<HostContFn> = Rc::new(k);
        self.new_native(name, NativeFn::HostK(host, cont))
    }

    /// A typed host function: the arguments convert to `A` (a conversion
    /// failure is `luaL_typeerror` for that argument), the result `R`
    /// converts back. `name` is the debug name errors print.
    pub fn create_function<A, R, F>(&mut self, name: &str, f: F) -> Result<Function, Error>
    where
        A: FromLuaMulti,
        R: IntoLuaMulti,
        F: Fn(&mut HostCtx<'_>, A) -> Result<R, HostError> + 'static,
    {
        self.create_host_function(name, move |ctx| {
            let args = ctx.args_from::<A>(1)?;
            let results = f(ctx, args)?;
            ctx.push_results(results)
        })
    }

    /// Loads a chunk under its name verbatim (C++ `luau_load` with env 0;
    /// seam item 1, `load_bytecode_as`): the function runs with the
    /// running thread's globals as its environment. Nothing runs.
    pub fn load(&mut self, chunk_name: &[u8], bytecode: &[u8]) -> Result<Function, Error> {
        let env = self.globals;
        self.load_in(chunk_name, bytecode, env)
    }

    /// `load` with `env` as the chunk's environment.
    pub fn load_with_env(
        &mut self,
        chunk_name: &[u8],
        bytecode: &[u8],
        env: &Table,
    ) -> Result<Function, Error> {
        let env = self.table_id(&env.0)?;
        self.load_in(chunk_name, bytecode, env)
    }

    fn load_in(&mut self, chunk_name: &[u8], bytecode: &[u8], env: TableId) -> Result<Function, Error> {
        self.check_entry()?;
        let loaded = crate::bytecode::load::load(self, chunk_name, bytecode, env);
        let out = loaded.and_then(|f| self.handles.insert(RawValue::Function(f)).map(Function));
        self.settle();
        out
    }

    /// A new coroutine running `f` (C++ `lua_newthread` + `lua_xpush`),
    /// with the running thread's globals.
    pub fn create_thread(&mut self, f: &Function) -> Result<Thread, Error> {
        let f = self.handles.get(&f.0)?;
        let raw = self.protect(|vm| vm.new_coroutine(f).map(RawValue::Thread))?;
        let out = self.handles.insert(raw).map(Thread);
        self.settle();
        out
    }

    /// A module thread (luaur-rt `create_module_thread`): a coroutine
    /// running `f` whose globals are `env`, so chunks loaded on it
    /// (`HostCtx::load`) see `env`.
    pub fn create_thread_with_env(&mut self, f: &Function, env: &Table) -> Result<Thread, Error> {
        let env = self.table_id(&env.0)?;
        let thread = self.create_thread(f)?;
        if let RawValue::Thread(id) = self.handles.get(&thread.0)? {
            self.heap.thread_mut(id).exec.globals = env;
        }
        Ok(thread)
    }

    // ---- userdata ------------------------------------------------------

    /// A userdata holding `value`, with the metatable registered for `T`
    /// ([`Vm::register_userdata_type`] or [`Vm::set_userdata_metatable`]).
    /// It costs its `size_of` against the memory budget; `T`'s `Drop` runs
    /// once, when the collector frees it or the VM drops.
    pub fn create_userdata<T: 'static>(&mut self, value: T) -> Result<AnyUserData, Error> {
        self.create_userdata_with_size(value, std::mem::size_of::<T>())
    }

    /// `create_userdata` charging `size` bytes (for a value owning heap
    /// memory the budget should see).
    pub fn create_userdata_with_size<T: 'static>(
        &mut self,
        value: T,
        size: usize,
    ) -> Result<AnyUserData, Error> {
        let meta = self.type_metas.get(&TypeId::of::<T>()).copied();
        self.new_userdata_handle(Userdata::new(value, size, 0, meta))
    }

    /// A userdata with tag `tag` (C++ `lua_newuserdatatagged`): its
    /// metatable is the tag's ([`Vm::set_tag_metatable`]), else `T`'s.
    pub fn create_userdata_tagged<T: 'static>(
        &mut self,
        value: T,
        tag: u8,
    ) -> Result<AnyUserData, Error> {
        if tag >= UTAG_LIMIT {
            return Err(Error::Host(format!(
                "userdata tag {tag} is not below {UTAG_LIMIT}"
            )));
        }
        let meta = self
            .tag_metas
            .get(usize::from(tag))
            .copied()
            .flatten()
            .or_else(|| self.type_metas.get(&TypeId::of::<T>()).copied());
        self.new_userdata_handle(Userdata::new(value, std::mem::size_of::<T>(), tag, meta))
    }

    fn new_userdata_handle(&mut self, ud: Userdata) -> Result<AnyUserData, Error> {
        let raw = self.protect(|vm| vm.heap.new_userdata(ud).map(RawValue::Userdata))?;
        let out = self.handles.insert(raw).map(AnyUserData);
        self.settle();
        out
    }

    /// The metatable every later `create_userdata::<T>` gets (`None`
    /// removes it). Existing userdata keep theirs.
    pub fn set_userdata_metatable<T: 'static>(&mut self, mt: Option<&Table>) -> Result<(), Error> {
        match mt {
            Some(t) => {
                let id = self.table_id(&t.0)?;
                self.type_metas.insert(TypeId::of::<T>(), id);
            }
            None => {
                self.type_metas.remove(&TypeId::of::<T>());
            }
        }
        Ok(())
    }

    /// C++ `lua_setuserdatametatable(tag)`: the metatable every later
    /// userdata with `tag` gets.
    pub fn set_tag_metatable(&mut self, tag: u8, mt: Option<&Table>) -> Result<(), Error> {
        if tag >= UTAG_LIMIT {
            return Err(Error::Host(format!(
                "userdata tag {tag} is not below {UTAG_LIMIT}"
            )));
        }
        let id = match mt {
            Some(t) => Some(self.table_id(&t.0)?),
            None => None,
        };
        let index = usize::from(tag);
        if self.tag_metas.len() <= index {
            self.tag_metas.resize(index.saturating_add(1), None);
        }
        if let Some(slot) = self.tag_metas.get_mut(index) {
            *slot = id;
        }
        Ok(())
    }

    /// C++ `lua_registeruserdatadirectfieldget` for every field: `GETTABLEKS`
    /// on a userdata with `tag` holding a `T` reads `key` through the
    /// getter without calling `__index` (seam rows R29, 3.4). Replaces the
    /// tag's previous set.
    pub fn register_direct_fields<T: 'static>(
        &mut self,
        tag: u8,
        fields: &[DirectField<T>],
    ) -> Result<(), Error> {
        if tag >= UTAG_LIMIT {
            return Err(Error::Host(format!(
                "userdata tag {tag} is not below {UTAG_LIMIT}"
            )));
        }
        let mut getters: HashMap<Vec<u8>, DirectGetter> = HashMap::new();
        for &(key, getter) in fields {
            let read: DirectGetter = Rc::new(move |payload: &dyn Any| {
                let cell = payload.downcast_ref::<RefCell<T>>()?;
                let value = cell.try_borrow().ok()?;
                Some(getter(&value))
            });
            getters.insert(key.as_bytes().to_vec(), read);
        }
        let index = usize::from(tag);
        if self.direct_fields.len() <= index {
            self.direct_fields.resize(index.saturating_add(1), None);
        }
        if let Some(slot) = self.direct_fields.get_mut(index) {
            *slot = Some(Rc::new(DirectFields { getters }));
        }
        Ok(())
    }

    /// The `GETTABLEKS` vector fast path (C++ `lvmexecute.cpp`): a
    /// one-letter key `x`, `y` or `z`, either case, is that component.
    #[inline]
    pub(crate) fn vector_component(&self, v: [f32; 3], key: RawValue) -> Option<f64> {
        let RawValue::String(key) = key else {
            return None;
        };
        let index = match self.heap.str_bytes(key) {
            [c] => usize::from((c | b' ').checked_sub(b'x')?),
            _ => return None,
        };
        v.get(index).map(|&c| f64::from(c))
    }

    /// The `GETTABLEKS` direct-field read: `Some` when `ud`'s tag has a
    /// getter for `key` and the payload is its type.
    #[inline]
    pub(crate) fn direct_field(&self, ud: UdId, key: RawValue) -> Option<RawValue> {
        if self.direct_fields.is_empty() {
            return None;
        }
        let RawValue::String(key) = key else {
            return None;
        };
        let data = self.heap.userdata(ud);
        let fields = self.direct_fields.get(usize::from(data.tag))?.as_ref()?;
        let getter = fields.getters.get(self.heap.str_bytes(key))?;
        let payload = data.payload.as_ref()?;
        getter(payload.as_ref()).map(RawValue::Number)
    }

    /// Builds `spec`'s metatable, makes it `T`'s (every later
    /// `create_userdata::<T>` gets it) and returns it, writable until the
    /// host makes it read-only (seam item 3).
    pub fn register_userdata_type<T: 'static>(
        &mut self,
        spec: UserDataType<T>,
    ) -> Result<Table, Error> {
        let mt = self.create_table()?;
        let methods = self.create_table()?;
        for (name, f) in spec.methods {
            let f = self.new_native(&name, NativeFn::Host(f))?;
            methods.raw_set(self, name.as_str(), f)?;
        }
        for (name, f) in spec.meta {
            let f = self.new_native(&name, NativeFn::Host(f))?;
            mt.raw_set(self, name.as_str(), f)?;
        }
        if let Some(name) = &spec.type_name {
            mt.raw_set(self, "__type", name.as_str())?;
        }
        if spec.getters.is_empty() && spec.setters.is_empty() {
            mt.raw_set(self, "__index", &methods)?;
        } else {
            let getters = self.create_table()?;
            for (name, f) in spec.getters {
                let f = self.new_native(&name, NativeFn::Host(f))?;
                getters.raw_set(self, name.as_str(), f)?;
            }
            let setters = self.create_table()?;
            for (name, f) in spec.setters {
                let f = self.new_native(&name, NativeFn::Host(f))?;
                setters.raw_set(self, name.as_str(), f)?;
            }
            // The handles stay alive (and rooted) across the allocations.
            let ups = [
                RawValue::Table(self.table_id(&getters.0)?),
                RawValue::Table(self.table_id(&methods.0)?),
            ];
            let index = self.protect(|vm| vm.new_builtin("__index", dispatch_index, &ups))?;
            let index = self.wrap(index)?;
            mt.raw_set(self, "__index", index)?;
            let ups = [RawValue::Table(self.table_id(&setters.0)?)];
            let newindex =
                self.protect(|vm| vm.new_builtin("__newindex", dispatch_newindex, &ups))?;
            let newindex = self.wrap(newindex)?;
            mt.raw_set(self, "__newindex", newindex)?;
        }
        self.set_userdata_metatable::<T>(Some(&mt))?;
        self.settle();
        Ok(mt)
    }

    // ---- registry and app data -------------------------------------------

    /// C++ `lua_setfield(LUA_REGISTRYINDEX, name)`.
    pub fn set_named_registry_value(&mut self, name: &str, value: impl IntoLua) -> Result<(), Error> {
        let (v, _root) = self.lower_into(value)?;
        let registry = self.registry;
        let out = self.protect(|vm| vm.raw_set_str(registry, name, v));
        self.settle();
        out
    }

    /// C++ `lua_getfield(LUA_REGISTRYINDEX, name)`.
    pub fn named_registry_value<T: FromLua>(&mut self, name: &str) -> Result<T, Error> {
        let registry = self.registry;
        let v = self.protect_value(|vm| {
            let key = vm.heap.intern(name.as_bytes())?;
            Ok(vm.heap.table(registry).get_str(key))
        })?;
        T::from_lua(v, self)
    }

    pub fn unset_named_registry_value(&mut self, name: &str) -> Result<(), Error> {
        self.set_named_registry_value(name, Value::Nil)
    }

    /// A reference to `value` that keeps it alive while held (C++
    /// `lua_ref`; seam item 2). Valid from any thread; dropping it never
    /// allocates or enters the VM.
    pub fn create_registry_value(&mut self, value: impl IntoLua) -> Result<RegistryKey, Error> {
        let (v, _root) = self.lower_into(value)?;
        self.handles.insert(v).map(RegistryKey)
    }

    /// The value a registry key refers to.
    pub fn registry_value<T: FromLua>(&mut self, key: &RegistryKey) -> Result<T, Error> {
        let raw = self.handles.get(&key.0)?;
        let v = self.wrap(raw)?;
        T::from_lua(v, self)
    }

    /// C++ `lua_unref`: the same as dropping the key.
    pub fn remove_registry_value(&mut self, key: RegistryKey) -> Result<(), Error> {
        self.handles.get(&key.0).map(drop)
    }

    /// Stores per-VM host state of type `T` (luaur-rt app data), replacing
    /// and returning any earlier one. Values are shared (`Rc`): reading one
    /// does not hold the VM borrowed, so a callback can keep it across
    /// calls into the VM. Use a `RefCell` or `Cell` inside for mutation.
    pub fn set_app_data<T: 'static>(&mut self, value: T) -> Option<Rc<T>> {
        let old = self.app_data.insert(TypeId::of::<T>(), Rc::new(value));
        old.and_then(|old| old.downcast::<T>().ok())
    }

    /// The app data of type `T`.
    pub fn app_data<T: 'static>(&self) -> Option<Rc<T>> {
        let v = self.app_data.get(&TypeId::of::<T>())?;
        Rc::clone(v).downcast::<T>().ok()
    }

    pub fn remove_app_data<T: 'static>(&mut self) -> Option<Rc<T>> {
        let old = self.app_data.remove(&TypeId::of::<T>())?;
        old.downcast::<T>().ok()
    }

    // ---- callbacks ---------------------------------------------------------

    /// Installs the interrupt (C++ `lua_callbacks()->interrupt`, GC steps
    /// excluded) and returns the previous one, so a caller can scope one
    /// (luaur-rt `scoped_interrupt`) by restoring it afterwards. It fires
    /// at `CALL`, `CALLFB`, `RETURN`, `JUMPBACK`, `JUMPX`, `FORNLOOP`,
    /// `FORGLOOP` and every string pattern step, as luaur-vm does (seam
    /// 3.1); an error it returns is raised on the running thread.
    pub fn set_interrupt(
        &mut self,
        f: impl FnMut() -> Result<(), HostError> + 'static,
    ) -> Option<Box<InterruptFn>> {
        let boxed: Box<InterruptFn> = Box::new(f);
        self.replace_interrupt(Some(boxed))
    }

    /// Removes the interrupt and returns it.
    pub fn remove_interrupt(&mut self) -> Option<Box<InterruptFn>> {
        self.replace_interrupt(None)
    }

    /// Installs `f` (or none) and returns the previous interrupt.
    pub fn replace_interrupt(&mut self, f: Option<Box<InterruptFn>>) -> Option<Box<InterruptFn>> {
        let new = f.map(|f| Rc::new(RefCell::new(f)));
        let old = std::mem::replace(&mut self.interrupt, new)?;
        // The string matcher may hold a clone while it runs; it never runs
        // across this call (the host is not inside a match here), so the
        // slot is unique and its box comes back.
        Rc::try_unwrap(old).ok().map(RefCell::into_inner)
    }

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
    /// [`Vm::clear_memory_exhausted`] (seam item 10; the audio engine's
    /// `oom` flag).
    pub fn memory_exhausted(&self) -> bool {
        self.heap.budget.exhausted()
    }

    pub fn clear_memory_exhausted(&self) {
        self.heap.budget.clear_exhausted();
    }

    /// `lua_gc(LUA_GCSTEP, kb)` from the host; true when a cycle finished.
    pub fn gc_step_kb(&mut self, kb: i32) -> bool {
        self.gc_step(kb)
    }

    // ---- threads -----------------------------------------------------------

    /// The running thread.
    pub fn current_thread(&mut self) -> Result<Thread, Error> {
        let current = self.current;
        self.handles.insert(RawValue::Thread(current)).map(Thread)
    }

    /// The main thread.
    pub fn main_thread(&mut self) -> Result<Thread, Error> {
        let main = self.main_thread;
        self.handles.insert(RawValue::Thread(main)).map(Thread)
    }

    /// An opaque id of the running thread, unique across VMs in this
    /// process (luaur-rt patch 7c9291ae).
    pub fn state_identity(&self) -> u64 {
        (self.handles.vm_id() << 32) | u64::from(self.current.0)
    }

    fn resume_function(&mut self) -> VmResult<RawValue> {
        if let Some(f) = self.resume_fn {
            return Ok(RawValue::Function(f));
        }
        let f = self.new_builtin("resume", crate::libs::coroutine::resume_builtin, &[])?;
        if let RawValue::Function(id) = f {
            self.resume_fn = Some(id);
            self.pinned.push(f);
        }
        Ok(f)
    }

    // ---- values -------------------------------------------------------------

    /// `lua_tolstring`: a string, or a number rendered with `%.14g`
    /// (seam row R31); `None` for anything else.
    pub fn coerce_string(&mut self, value: &Value) -> Result<Option<LuaString>, Error> {
        if let Value::String(s) = value {
            return Ok(Some(s.clone()));
        }
        let raw = self.lower(value)?;
        let out = self.protect_value(|vm| {
            Ok(vm
                .coerce_to_str(raw)?
                .map_or(RawValue::Nil, RawValue::String))
        })?;
        Ok(match out {
            Value::String(s) => Some(s),
            _ => None,
        })
    }

    /// `lua_tonumberx`: a number, or a string that reads as one.
    pub fn coerce_number(&mut self, value: &Value) -> Result<Option<f64>, Error> {
        Ok(match value {
            Value::Number(n) => Some(*n),
            Value::Integer(i) => Some(*i as f64),
            Value::String(_) => {
                let raw = self.lower(value)?;
                self.to_number(raw)
            }
            _ => None,
        })
    }

    /// `lua_objlen`: string and buffer byte length, a table's border,
    /// a userdata's declared size, 0 otherwise (seam row R32).
    pub fn objlen(&self, value: &Value) -> Result<usize, Error> {
        Ok(match self.lower(value)? {
            RawValue::String(id) => self.heap.str_bytes(id).len(),
            RawValue::Buffer(id) => self.heap.buffer(id).len(),
            RawValue::Table(id) => self.heap.table(id).len(),
            RawValue::Userdata(id) => self.heap.userdata(id).size,
            _ => 0,
        })
    }

    /// `lua_rawequal`.
    pub fn raw_equal(&self, a: &Value, b: &Value) -> Result<bool, Error> {
        Ok(self.lower(a)?.raw_eq(self.lower(b)?))
    }

    /// `lua_topointer`: an identity for objects (stable while the object
    /// lives, unique per object kind), 0 for other values.
    pub fn to_pointer(&self, value: &Value) -> Result<u64, Error> {
        let raw = self.lower(value)?;
        Ok(match raw {
            RawValue::String(_)
            | RawValue::Table(_)
            | RawValue::Function(_)
            | RawValue::Thread(_)
            | RawValue::Buffer(_)
            | RawValue::Userdata(_) => self.pseudo_address(raw),
            RawValue::LightUserdata(p, _) => p,
            _ => 0,
        })
    }

    /// `tostring(value)` (`__tostring` honored).
    pub fn tostring(&mut self, value: &Value) -> Result<LuaString, Error> {
        let raw = self.lower(value)?;
        let v = self.protect_value(|vm| vm.tostring_value(raw).map(RawValue::String))?;
        match v {
            Value::String(s) => Ok(s),
            other => Err(conversion(&other, self, "string")),
        }
    }

    /// The method name of the last `NAMECALL` dispatched through a
    /// `__namecall` metamethod (C++ `lua_namecallatom`'s string; seam item
    /// 4), whatever the metamethod's debug name. As in C++ it is only
    /// meaningful inside that metamethod; outside a host function it is
    /// `None`.
    pub fn namecall_method(&self) -> Option<Vec<u8>> {
        let frame = self.frames.last()?;
        let HeapFunction::Native(_) = self.heap.function(frame.closure) else {
            return None;
        };
        self.namecall.map(|id| self.heap.str_bytes(id).to_vec())
    }

    /// C++ `lua_getinfo(level, "sl")`: level 0 is the running function
    /// (inside a callback, the callback), 1 its caller.
    pub fn inspect_stack(&self, level: usize) -> Option<StackInfo> {
        let index = self.frames.len().checked_sub(1)?.checked_sub(level)?;
        let frame = self.frames.get(index)?;
        match &frame.proto {
            Some(proto) => {
                let savedpc = if index.saturating_add(1) == self.frames.len() {
                    self.pc
                } else {
                    frame.savedpc
                };
                Some(StackInfo {
                    source: proto
                        .source
                        .map(|s| self.heap.str_bytes(s).to_vec())
                        .unwrap_or_default(),
                    current_line: proto.line(savedpc.saturating_sub(1)),
                    is_native: false,
                })
            }
            None => Some(StackInfo {
                source: b"=[C]".to_vec(),
                current_line: -1,
                is_native: true,
            }),
        }
    }

    /// The metatable shared by all vectors (luaur-rt
    /// `set_type_metatable::<Vector>`).
    pub fn set_vector_metatable(&mut self, mt: Option<&Table>) -> Result<(), Error> {
        self.vector_meta = match mt {
            Some(t) => Some(self.table_id(&t.0)?),
            None => None,
        };
        Ok(())
    }

    /// Live handles (tests).
    #[doc(hidden)]
    pub fn live_handles(&self) -> usize {
        self.handles.live_count()
    }
}

/// A reference to a value the registry keeps (luaur-rt `RegistryKey`, C++
/// `lua_ref`). A handle under another name.
#[derive(Clone, Debug)]
pub struct RegistryKey(pub(crate) Handle);

// ---- the userdata dispatchers ---------------------------------------------

/// The `__index` of a userdata type with fields (luaur-rt's dispatcher):
/// a getter for the key, else the method table's entry.
fn dispatch_index(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let ud = vm.arg_or_nil(args, 1);
    let key = vm.arg_or_nil(args, 2);
    let (RawValue::Table(getters), RawValue::Table(methods)) =
        (vm.builtin_upval(0), vm.builtin_upval(1))
    else {
        return Err(vm.runtime_error("internal error: a userdata dispatcher without tables"));
    };
    let getter = vm.heap.table(getters).get(key);
    if let RawValue::Function(_) = getter {
        let func = vm.top;
        vm.push(getter)?;
        vm.push(ud)?;
        return vm.request_call(func, 1, 1, CallMode::Counted, Box::new(Results { from: func }));
    }
    let method = vm.heap.table(methods).get(key);
    vm.ret(&[method])
}

/// The `__newindex` of a userdata type with fields: a setter for the key,
/// else `attempt to set unknown field 'k' on userdata`.
fn dispatch_newindex(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let ud = vm.arg_or_nil(args, 1);
    let key = vm.arg_or_nil(args, 2);
    let value = vm.arg_or_nil(args, 3);
    let RawValue::Table(setters) = vm.builtin_upval(0) else {
        return Err(vm.runtime_error("internal error: a userdata dispatcher without tables"));
    };
    let setter = vm.heap.table(setters).get(key);
    if let RawValue::Function(_) = setter {
        let func = vm.top;
        vm.push(setter)?;
        vm.push(ud)?;
        vm.push(value)?;
        return vm.request_call(func, 2, 0, CallMode::Counted, Box::new(Results { from: func }));
    }
    // luaur-rt's dispatcher formats `tostring(key)` (`USERDATA_DISPATCH`),
    // so any key renders, `__tostring` included.
    // The bytes as they are, up to a NUL (`%s`).
    let id = vm.tostring_value(key)?;
    let bytes = vm.heap.str_bytes(id);
    let name = bytes.split(|&b| b == 0).next().unwrap_or_default();
    let mut text = b"attempt to set unknown field '".to_vec();
    text.extend_from_slice(name);
    text.extend_from_slice(b"' on userdata");
    Err(vm.throw_string(&text, ErrorKind::Runtime))
}

/// Returns the values from `from` up: a requested call's results.
struct Results {
    from: usize,
}

impl Continuation for Results {
    fn resume(self: Box<Self>, vm: &mut Vm, _: Args, outcome: Outcome) -> VmResult<usize> {
        match outcome {
            Outcome::Ok => Ok(vm.top.saturating_sub(self.from)),
            Outcome::Err(throw) => Err(throw),
        }
    }
}

// ---- host continuations ---------------------------------------------------

/// A host function's continuation, waiting on a call it requested or a
/// yield it made.
struct HostCont {
    k: Option<Rc<HostContFn>>,
    /// For `pcall_yieldable`: what the protected call protects.
    catch: Option<Catch>,
    /// The requested call's function slot, where its results land.
    results_from: usize,
}

impl Continuation for HostCont {
    fn resume(self: Box<Self>, vm: &mut Vm, args: Args, outcome: Outcome) -> VmResult<usize> {
        let status = match outcome {
            Outcome::Ok => CallStatus::Ok,
            Outcome::Err(throw) => {
                // C++ `luaD_pcall`'s `seterrorobj` at the function's slot.
                let func = self.catch.map_or(vm.top, |c| c.callee);
                vm.ensure_stack(func.saturating_add(1))?;
                vm.set(func, throw.value);
                vm.top = func.saturating_add(1);
                CallStatus::Error(throw.kind)
            }
        };
        match self.k {
            Some(k) => {
                let body = Rc::clone(&k);
                vm.run_host(args, Some(k), move |ctx| body(ctx, status))
            }
            // No continuation (calls only; a yield without one has no
            // `HostCont`): the function returns the call's results, which
            // start at the callee's slot (the error value after a failed
            // protected call), whatever the arguments were.
            None => Ok(vm.top.saturating_sub(self.results_from)),
        }
    }

    fn catch(&self) -> Option<Catch> {
        self.catch
    }
}

// ---- HostCtx: the stack and calls -----------------------------------------

impl HostCtx<'_> {
    pub(crate) fn throw(&mut self, throw: Throw) -> HostError {
        self.vm.host_errors.push(throw.value);
        HostError(HostErrorKind::Throw(throw))
    }

    /// The absolute slot of stack index `i` (1-based from the first
    /// argument; negative from the top), if it is in the frame.
    fn slot(&self, i: isize) -> Option<usize> {
        let base = self.args.base;
        let top = self.vm.top;
        let slot = if i > 0 {
            base.checked_add(i.unsigned_abs())?.checked_sub(1)?
        } else if i < 0 {
            top.checked_sub(i.unsigned_abs())?
        } else {
            return None;
        };
        (slot >= base && slot < top).then_some(slot)
    }

    fn slot_or_error(&self, i: isize) -> Result<usize, HostError> {
        self.slot(i)
            .ok_or_else(|| HostError::raw(format!("stack index {i} is not in the frame")))
    }

    /// C++ `lua_gettop`: the values in the frame.
    pub fn top(&self) -> usize {
        self.vm.top.saturating_sub(self.args.base)
    }

    /// C++ `lua_settop`: grows with nils or drops values.
    pub fn set_top(&mut self, n: usize) -> Result<(), HostError> {
        let new = self.args.base.saturating_add(n);
        if new > self.vm.top {
            if let Err(t) = self.vm.ensure_stack(new.saturating_add(MIN_STACK)) {
                return Err(self.throw(t));
            }
            let top = self.vm.top;
            self.vm.fill_nil(top, new);
        }
        self.vm.top = new;
        Ok(())
    }

    /// Drops `n` values (C++ `lua_pop`).
    pub fn pop(&mut self, n: usize) -> Result<(), HostError> {
        let top = self.top();
        self.set_top(top.saturating_sub(n))
    }

    /// The value at index `i` (nil when out of range, as C++'s
    /// `LUA_TNONE` reads as nil).
    pub fn value(&mut self, i: isize) -> Result<Value, HostError> {
        let v = self.slot(i).map_or(RawValue::Nil, |s| self.vm.get(s));
        self.vm.wrap(v).map_err(HostError::from)
    }

    /// The value at index `i` converted to `T`; a failed conversion is
    /// `luaL_typeerror` for argument `i`.
    pub fn get<T: FromLua>(&mut self, i: isize) -> Result<T, HostError> {
        let v = self.value(i)?;
        match T::from_lua(v, self.vm) {
            Ok(t) => Ok(t),
            Err(Error::Conversion { to, .. }) => {
                let n = self.slot(i).map_or(i.unsigned_abs(), |s| {
                    s.saturating_sub(self.args.base).saturating_add(1)
                });
                let t = self.vm.arg_type_error(self.frame_args(), n, to);
                Err(self.throw(t))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// The arguments from position `first` on, converted to `A`.
    pub fn args_from<A: FromLuaMulti>(&mut self, first: usize) -> Result<A, HostError> {
        let mut values = Vec::new();
        for n in first..=self.args.nargs {
            let v = self.vm.arg_or_nil(self.args, n);
            values.push(self.vm.wrap(v).map_err(HostError::from)?);
        }
        match A::from_lua_multi(MultiValue(values), self.vm) {
            Ok(a) => Ok(a),
            Err(Error::Conversion { to, position, .. }) => {
                let n = position.unwrap_or(1).saturating_add(first).saturating_sub(1);
                let t = self.vm.arg_type_error(self.args, n, to);
                Err(self.throw(t))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Every argument, converted to `A`.
    pub fn args<A: FromLuaMulti>(&mut self) -> Result<A, HostError> {
        self.args_from(1)
    }

    /// The frame's current values as `Args` (for `luaL` errors after
    /// `set_top`).
    fn frame_args(&self) -> Args {
        Args {
            base: self.args.base,
            nargs: self.top(),
        }
    }

    /// Pushes a value.
    pub fn push(&mut self, value: impl IntoLua) -> Result<(), HostError> {
        let v = value.into_lua(self.vm).map_err(HostError::from)?;
        let raw = self.vm.lower(&v).map_err(HostError::from)?;
        self.vm.push(raw).map_err(|t| self.throw(t))
    }

    /// Pushes several values and returns how many (a host function's
    /// result count).
    pub fn push_results(&mut self, values: impl IntoLuaMulti) -> Result<usize, HostError> {
        let values = values.into_lua_multi(self.vm).map_err(HostError::from)?;
        let n = values.len();
        if n > MIN_STACK
            && let Err(t) = self.vm.ensure_stack(self.vm.top.saturating_add(n))
        {
            return Err(self.throw(t));
        }
        for v in values.0 {
            self.push(v)?;
        }
        Ok(n)
    }

    /// Pushes a copy of the value at index `i` (C++ `lua_pushvalue`).
    pub fn push_copy(&mut self, i: isize) -> Result<(), HostError> {
        let v = self.slot(i).map_or(RawValue::Nil, |s| self.vm.get(s));
        self.vm.push(v).map_err(|t| self.throw(t))
    }

    /// Pops the top value into index `i` (C++ `lua_replace`).
    pub fn replace(&mut self, i: isize) -> Result<(), HostError> {
        let slot = self.slot_or_error(i)?;
        let top = self.slot_or_error(-1)?;
        let v = self.vm.get(top);
        self.vm.set(slot, v);
        self.vm.top = top;
        Ok(())
    }

    /// Moves the top value to index `i`, shifting the values above up
    /// (C++ `lua_insert`).
    pub fn insert(&mut self, i: isize) -> Result<(), HostError> {
        let slot = self.slot_or_error(i)?;
        let top = self.slot_or_error(-1)?;
        let v = self.vm.get(top);
        self.vm
            .copy_slots(slot, slot.saturating_add(1), top.saturating_sub(slot));
        self.vm.set(slot, v);
        Ok(())
    }

    /// Removes the value at index `i`, shifting the values above down
    /// (C++ `lua_remove`).
    pub fn remove(&mut self, i: isize) -> Result<(), HostError> {
        let slot = self.slot_or_error(i)?;
        let top = self.vm.top;
        self.vm.copy_slots(
            slot.saturating_add(1),
            slot,
            top.saturating_sub(slot).saturating_sub(1),
        );
        self.vm.top = top.saturating_sub(1);
        Ok(())
    }

    /// The function at `top - nargs - 1`, checked to be in the frame.
    fn call_slot(&self, nargs: usize) -> Result<usize, HostError> {
        let func = self
            .vm
            .top
            .checked_sub(nargs.saturating_add(1))
            .filter(|&f| f >= self.args.base);
        func.ok_or_else(|| HostError::raw("not enough values on the stack for the call"))
    }

    /// C++ `lua_callyieldable`: calls the function below the top `nargs`
    /// values. Return its result from the host function at once
    /// (`return ctx.call_yieldable(..)`): the call runs after the host
    /// function returns, and the function's continuation then runs with
    /// the `nresults` results on top (all of them for a negative count).
    /// The callee may yield when the thread can. Without a continuation the
    /// call's results are the host function's results.
    pub fn call_yieldable(&mut self, nargs: usize, nresults: i32) -> Result<usize, HostError> {
        let func = self.call_slot(nargs)?;
        let mode = if self.cont.is_some() {
            CallMode::CountedYieldable
        } else {
            CallMode::Counted
        };
        let cont = Box::new(HostCont {
            k: self.cont.clone(),
            catch: None,
            results_from: func,
        });
        self.vm.request = Some(Request::Call {
            func,
            nargs,
            nresults: nresults.max(MULTRET),
            mode,
            cont,
        });
        Ok(0)
    }

    /// C++ `lua_pcallyieldable` without a handler: `call_yieldable`, but an
    /// error stops at this call; the continuation runs with
    /// `CallStatus::Error` and the error value on top in place of the
    /// function and its arguments.
    pub fn pcall_yieldable(&mut self, nargs: usize, nresults: i32) -> Result<usize, HostError> {
        let func = self.call_slot(nargs)?;
        let saved = self.vm.saved_calls();
        let mode = if self.vm.yieldable() {
            CallMode::Checked
        } else {
            CallMode::CountedYieldable
        };
        let cont = Box::new(HostCont {
            k: self.cont.clone(),
            catch: Some(Catch {
                callee: func,
                handler: None,
                saved,
            }),
            results_from: func,
        });
        self.vm.request = Some(Request::Call {
            func,
            nargs,
            nresults: nresults.max(MULTRET),
            mode,
            cont,
        });
        Ok(0)
    }

    /// C++ `lua_yield`: yields the top `n` values. Return its result from
    /// the host function at once. On resume, the continuation runs with
    /// the resume values on top of the frame (the yielded ones are gone);
    /// without one, the resume values are the function's results. Raises
    /// `attempt to yield across metamethod/C-call boundary` where the
    /// thread cannot yield (inside a call from the host, for one).
    pub fn yield_values(&mut self, n: usize) -> Result<usize, HostError> {
        if !self.vm.yieldable() {
            let t = self
                .vm
                .runtime_error("attempt to yield across metamethod/C-call boundary");
            return Err(self.throw(t));
        }
        if n > self.top() {
            return Err(HostError::raw("not enough values on the stack to yield"));
        }
        let cont = self.cont.clone().map(|k| {
            Box::new(HostCont {
                k: Some(k),
                catch: None,
                results_from: 0,
            }) as Box<dyn Continuation>
        });
        self.vm.request = Some(Request::Yield { n, cont });
        Ok(0)
    }

    /// C++ `lua_isyieldable`.
    pub fn is_yieldable(&self) -> bool {
        self.vm.yieldable()
    }

    // ---- luaL checks ---------------------------------------------------

    /// `luaL_checknumber`.
    pub fn check_number(&mut self, n: usize) -> Result<f64, HostError> {
        let args = self.frame_args();
        self.vm.check_number(args, n).map_err(|t| self.throw(t))
    }

    /// `luaL_checkinteger` (a number truncated to `int`).
    pub fn check_integer(&mut self, n: usize) -> Result<i32, HostError> {
        let args = self.frame_args();
        self.vm.check_integer(args, n).map_err(|t| self.throw(t))
    }

    /// `luaL_checkunsigned` (a number converted to `unsigned`, wrapping as
    /// C++'s `luai_num2unsigned` does).
    pub fn check_unsigned(&mut self, n: usize) -> Result<u32, HostError> {
        let x = self.check_number(n)?;
        // C++ `luai_num2unsigned`: through a 64-bit integer, then wrapped.
        Ok((x as i64) as u32)
    }

    /// `luaL_checkboolean`.
    pub fn check_boolean(&mut self, n: usize) -> Result<bool, HostError> {
        let args = self.frame_args();
        match self.vm.arg(args, n) {
            Some(RawValue::Boolean(b)) => Ok(b),
            _ => {
                let t = self.vm.arg_type_error(args, n, "boolean");
                Err(self.throw(t))
            }
        }
    }

    /// `luaL_checkvector`.
    pub fn check_vector(&mut self, n: usize) -> Result<[f32; 3], HostError> {
        let args = self.frame_args();
        self.vm.check_vector(args, n).map_err(|t| self.throw(t))
    }

    /// `luaL_checkany`.
    pub fn check_any(&mut self, n: usize) -> Result<Value, HostError> {
        let args = self.frame_args();
        let v = self.vm.check_any(args, n).map_err(|t| self.throw(t))?;
        self.vm.wrap(v).map_err(HostError::from)
    }

    /// `luaL_checkbuffer`.
    pub fn check_buffer(&mut self, n: usize) -> Result<Buffer, HostError> {
        let args = self.frame_args();
        let id = self
            .vm
            .check_buffer_arg(args, n)
            .map_err(|t| self.throw(t))?;
        match self.vm.wrap(RawValue::Buffer(id)) {
            Ok(Value::Buffer(b)) => Ok(b),
            Ok(_) => Err(HostError::raw("internal error: a buffer handle")),
            Err(e) => Err(e.into()),
        }
    }

    /// Argument `n` as a `T` userdata, else `luaL_typeerror(n, type_name)`.
    /// The cell is the host's: borrowing it does not hold the VM, and a
    /// conflicting borrow is a `try_borrow` error, not a panic.
    pub fn check_userdata<T: 'static>(
        &mut self,
        n: usize,
        type_name: &str,
    ) -> Result<Rc<RefCell<T>>, HostError> {
        let args = self.frame_args();
        if let Some(RawValue::Userdata(id)) = self.vm.arg(args, n)
            && let Some(cell) = self.vm.heap.userdata(id).cell::<T>()
        {
            return Ok(cell);
        }
        let t = self.vm.arg_type_error(args, n, type_name);
        Err(self.throw(t))
    }

    /// Argument `n` as a userdata with tag `tag` holding a `T` (C++
    /// `lua_touserdatatagged`), or `None`.
    pub fn to_userdata_tagged<T: 'static>(&self, n: usize, tag: u8) -> Option<Rc<RefCell<T>>> {
        match self.vm.arg(self.frame_args(), n)? {
            RawValue::Userdata(id) if self.vm.heap.userdata(id).tag == tag => {
                self.vm.heap.userdata(id).cell::<T>()
            }
            _ => None,
        }
    }

    /// A `luaL_error`: the message prefixed with the calling Lua position.
    pub fn error(&self, message: impl Into<String>) -> HostError {
        HostError::new(message)
    }

    /// `luaL_argerror`.
    pub fn arg_error(&mut self, n: usize, message: &str) -> HostError {
        let t = self.vm.arg_error(n, message);
        self.throw(t)
    }

    /// `luaL_typeerror`.
    pub fn type_error(&mut self, n: usize, expected: &str) -> HostError {
        let args = self.frame_args();
        let t = self.vm.arg_type_error(args, n, expected);
        self.throw(t)
    }
}

// ---- handle operations ------------------------------------------------------

impl Table {
    fn id(&self, vm: &Vm) -> Result<TableId, Error> {
        vm.table_id(&self.0)
    }

    /// `t[key]` with metamethods (C++ `lua_gettable`).
    pub fn get<V: FromLua>(&self, vm: &mut Vm, key: impl IntoLua) -> Result<V, Error> {
        let t = self.id(vm)?;
        let (k, _key_root) = vm.lower_into(key)?;
        let v = vm.protect_value(|vm| {
            let step = vm.get_table_step(RawValue::Table(t), k)?;
            vm.meta_now(step)
        })?;
        V::from_lua(v, vm)
    }

    /// `t.name` (luaur-rt patch `get_field`).
    pub fn get_field<V: FromLua>(&self, vm: &mut Vm, name: &str) -> Result<V, Error> {
        self.get(vm, name)
    }

    /// `t[key] = value` with metamethods (C++ `lua_settable`).
    pub fn set(&self, vm: &mut Vm, key: impl IntoLua, value: impl IntoLua) -> Result<(), Error> {
        let t = self.id(vm)?;
        let (k, _key_root) = vm.lower_into(key)?;
        let (v, _value_root) = vm.lower_into(value)?;
        let out = vm.protect(|vm| {
            let step = vm.set_table_step(RawValue::Table(t), k, v)?;
            vm.meta_now(step).map(drop)
        });
        vm.settle();
        out
    }

    /// `rawget(t, key)`.
    pub fn raw_get<V: FromLua>(&self, vm: &mut Vm, key: impl IntoLua) -> Result<V, Error> {
        let t = self.id(vm)?;
        let (k, _key_root) = vm.lower_into(key)?;
        let raw = vm.heap.table(t).get(k);
        let v = vm.wrap(raw)?;
        V::from_lua(v, vm)
    }

    /// `rawset(t, key, value)`; a read-only table refuses.
    pub fn raw_set(&self, vm: &mut Vm, key: impl IntoLua, value: impl IntoLua) -> Result<(), Error> {
        let t = self.id(vm)?;
        let (k, _key_root) = vm.lower_into(key)?;
        let (v, _value_root) = vm.lower_into(value)?;
        let out = vm.protect(|vm| {
            if vm.heap.table(t).readonly {
                return Err(vm.runtime_error("attempt to modify a readonly table"));
            }
            vm.raw_set(t, k, v)
        });
        vm.settle();
        out
    }

    /// The border (`#t` without `__len`, C++ `lua_objlen`).
    pub fn raw_len(&self, vm: &Vm) -> Result<usize, Error> {
        Ok(vm.heap.table(self.id(vm)?).len())
    }

    /// `#t` with `__len`.
    pub fn len(&self, vm: &mut Vm) -> Result<f64, Error> {
        let t = self.id(vm)?;
        let v = vm.protect_value(|vm| {
            let step = vm.length_step(RawValue::Table(t))?;
            vm.meta_now(step)
        })?;
        f64::from_lua(v, vm)
    }

    /// `t[#t + 1] = value`, raw (C++ `lua_rawseti(n + 1)`).
    pub fn raw_push(&self, vm: &mut Vm, value: impl IntoLua) -> Result<(), Error> {
        let n = self.raw_len(vm)?;
        self.raw_set(vm, n.saturating_add(1) as f64, value)
    }

    /// Whether `t[key]` (with metamethods) is not nil.
    pub fn contains_key(&self, vm: &mut Vm, key: impl IntoLua) -> Result<bool, Error> {
        let v: Value = self.get(vm, key)?;
        Ok(!v.is_nil())
    }

    /// Every entry, in `next` order (raw).
    pub fn pairs<K: FromLua, V: FromLua>(&self, vm: &mut Vm) -> Result<Vec<(K, V)>, Error> {
        let t = self.id(vm)?;
        let mut raw = Vec::new();
        let mut cursor = 0usize;
        while let Some((next, k, v)) = vm.heap.table(t).next_from(cursor) {
            raw.push((k, v));
            cursor = next;
        }
        // Root the whole snapshot before any conversion: a conversion can
        // allocate and collect, and a weak table's other entries are held
        // by nothing else.
        let mut rooted = Vec::with_capacity(raw.len());
        for (k, v) in raw {
            rooted.push((vm.wrap(k)?, vm.wrap(v)?));
        }
        let mut out = Vec::with_capacity(rooted.len());
        for (k, v) in rooted {
            out.push((K::from_lua(k, vm)?, V::from_lua(v, vm)?));
        }
        Ok(out)
    }

    /// `t[1]`, `t[2]`, ... up to the first nil (raw).
    pub fn sequence_values<V: FromLua>(&self, vm: &mut Vm) -> Result<Vec<V>, Error> {
        let t = self.id(vm)?;
        let mut out = Vec::new();
        for i in 1i64.. {
            let raw = vm.heap.table(t).get_int(i);
            if raw.is_nil() {
                break;
            }
            let v = vm.wrap(raw)?;
            out.push(V::from_lua(v, vm)?);
        }
        Ok(out)
    }

    /// Removes every entry (raw).
    pub fn clear(&self, vm: &mut Vm) -> Result<(), Error> {
        let keys: Vec<(Value, Value)> = self.pairs(vm)?;
        for (k, _) in keys {
            self.raw_set(vm, k, Value::Nil)?;
        }
        Ok(())
    }

    pub fn metatable(&self, vm: &mut Vm) -> Result<Option<Table>, Error> {
        let t = self.id(vm)?;
        match vm.heap.table(t).meta {
            Some(mt) => vm.handles.insert(RawValue::Table(mt)).map(|h| Some(Table(h))),
            None => Ok(None),
        }
    }

    /// C++ `lua_setmetatable` (no `__metatable` check, as the C API).
    pub fn set_metatable(&self, vm: &mut Vm, mt: Option<&Table>) -> Result<(), Error> {
        let t = self.id(vm)?;
        let mt = match mt {
            Some(m) => Some(m.id(vm)?),
            None => None,
        };
        if vm.heap.table(t).readonly {
            return Err(Error::runtime("attempt to modify a readonly table"));
        }
        vm.heap.table_mut(t).meta = mt;
        Ok(())
    }

    /// C++ `lua_setreadonly`.
    pub fn set_readonly(&self, vm: &mut Vm, readonly: bool) -> Result<(), Error> {
        let t = self.id(vm)?;
        vm.heap.table_mut(t).readonly = readonly;
        Ok(())
    }

    pub fn is_readonly(&self, vm: &Vm) -> Result<bool, Error> {
        Ok(vm.heap.table(self.id(vm)?).readonly)
    }

    /// C++ `lua_setsafeenv`.
    pub fn set_safeenv(&self, vm: &mut Vm, safe: bool) -> Result<(), Error> {
        let t = self.id(vm)?;
        vm.heap.table_mut(t).safeenv = safe;
        Ok(())
    }

    pub fn to_pointer(&self, vm: &Vm) -> Result<u64, Error> {
        vm.to_pointer(&Value::Table(self.clone()))
    }
}

impl Function {
    /// Calls the function (C++ `lua_pcall`): an error comes back as
    /// [`Error`], with the VM as it was before the call.
    pub fn call<A: IntoLuaMulti, R: FromLuaMulti>(&self, vm: &mut Vm, args: A) -> Result<R, Error> {
        let f = vm.handles.get(&self.0)?;
        let args = args.into_lua_multi(vm)?;
        let mut raw = Vec::with_capacity(args.len());
        for a in &args.0 {
            raw.push(vm.lower(a)?);
        }
        let results = vm.call_raw(f, &raw)?;
        R::from_lua_multi(results, vm)
    }

    /// A Lua function's environment (`getfenv`); `None` for a native.
    pub fn environment(&self, vm: &mut Vm) -> Result<Option<Table>, Error> {
        let RawValue::Function(id) = vm.handles.get(&self.0)? else {
            return Ok(None);
        };
        match vm.heap.function(id) {
            HeapFunction::Lua(closure) => {
                let env = closure.env;
                vm.handles.insert(RawValue::Table(env)).map(|h| Some(Table(h)))
            }
            HeapFunction::Native(_) => Ok(None),
        }
    }

    /// Sets a Lua function's environment (`setfenv`); false for a native.
    pub fn set_environment(&self, vm: &mut Vm, env: &Table) -> Result<bool, Error> {
        let env = env.id(vm)?;
        let RawValue::Function(id) = vm.handles.get(&self.0)? else {
            return Ok(false);
        };
        match vm.heap.function_mut(id) {
            HeapFunction::Lua(closure) => {
                closure.env = env;
                Ok(true)
            }
            HeapFunction::Native(_) => Ok(false),
        }
    }

    pub fn to_pointer(&self, vm: &Vm) -> Result<u64, Error> {
        vm.to_pointer(&Value::Function(self.clone()))
    }
}

impl LuaString {
    pub fn as_bytes(&self, vm: &Vm) -> Result<Vec<u8>, Error> {
        match vm.handles.get(&self.0)? {
            RawValue::String(id) => Ok(vm.heap.str_bytes(id).to_vec()),
            other => Err(Error::Host(format!("expected a string, got {}", other.type_name()))),
        }
    }

    /// The bytes as UTF-8, or a conversion error.
    pub fn to_str(&self, vm: &Vm) -> Result<String, Error> {
        String::from_utf8(self.as_bytes(vm)?).map_err(|_| Error::Conversion {
            from: "string".to_owned(),
            to: "UTF-8 string",
            position: None,
        })
    }

    pub fn to_string_lossy(&self, vm: &Vm) -> Result<String, Error> {
        Ok(String::from_utf8_lossy(&self.as_bytes(vm)?).into_owned())
    }

    pub fn to_pointer(&self, vm: &Vm) -> Result<u64, Error> {
        vm.to_pointer(&Value::String(self.clone()))
    }
}

impl Buffer {
    fn id(&self, vm: &Vm) -> Result<crate::value::BufId, Error> {
        match vm.handles.get(&self.0)? {
            RawValue::Buffer(id) => Ok(id),
            other => Err(Error::Host(format!("expected a buffer, got {}", other.type_name()))),
        }
    }

    pub fn len(&self, vm: &Vm) -> Result<usize, Error> {
        Ok(vm.heap.buffer(self.id(vm)?).len())
    }

    pub fn is_empty(&self, vm: &Vm) -> Result<bool, Error> {
        Ok(self.len(vm)? == 0)
    }

    pub fn to_vec(&self, vm: &Vm) -> Result<Vec<u8>, Error> {
        Ok(vm.heap.buffer(self.id(vm)?).to_vec())
    }

    /// `len` bytes from `offset`; out of range is an error.
    pub fn read_bytes(&self, vm: &Vm, offset: usize, len: usize) -> Result<Vec<u8>, Error> {
        let bytes = vm.heap.buffer(self.id(vm)?);
        offset
            .checked_add(len)
            .and_then(|end| bytes.get(offset..end))
            .map(<[u8]>::to_vec)
            .ok_or_else(|| Error::Host("buffer access out of bounds".to_owned()))
    }

    /// Copies `data` in at `offset`; out of range is an error.
    pub fn write_bytes(&self, vm: &mut Vm, offset: usize, data: &[u8]) -> Result<(), Error> {
        let id = self.id(vm)?;
        let bytes = vm.heap.buffer_mut(id);
        let target = offset
            .checked_add(data.len())
            .and_then(|end| bytes.get_mut(offset..end))
            .ok_or_else(|| Error::Host("buffer access out of bounds".to_owned()))?;
        target.copy_from_slice(data);
        Ok(())
    }

    /// The bytes in place (seam item 10): `f` cannot reach the VM.
    pub fn with_bytes_mut<R>(&self, vm: &mut Vm, f: impl FnOnce(&mut [u8]) -> R) -> Result<R, Error> {
        let id = self.id(vm)?;
        Ok(f(vm.heap.buffer_mut(id)))
    }

    pub fn to_pointer(&self, vm: &Vm) -> Result<u64, Error> {
        vm.to_pointer(&Value::Buffer(self.clone()))
    }
}

impl AnyUserData {
    /// Whether it holds a `T`.
    pub fn is<T: 'static>(&self, vm: &Vm) -> Result<bool, Error> {
        Ok(vm.heap.userdata(vm.ud_id(&self.0)?).is::<T>())
    }

    /// The host value's cell. Borrowing it does not hold the VM.
    pub fn cell<T: 'static>(&self, vm: &Vm) -> Result<Rc<RefCell<T>>, Error> {
        let data = vm.heap.userdata(vm.ud_id(&self.0)?);
        data.cell::<T>().ok_or_else(|| Error::Conversion {
            from: data.type_name.to_owned(),
            to: std::any::type_name::<T>(),
            position: None,
        })
    }

    /// Runs `f` on a shared borrow; a conflicting borrow is an error.
    pub fn borrow<T: 'static, R>(&self, vm: &Vm, f: impl FnOnce(&T) -> R) -> Result<R, Error> {
        let cell = self.cell::<T>(vm)?;
        let value = cell
            .try_borrow()
            .map_err(|_| Error::runtime("userdata already mutably borrowed"))?;
        Ok(f(&value))
    }

    /// Runs `f` on an exclusive borrow; a conflicting borrow is an error.
    pub fn borrow_mut<T: 'static, R>(&self, vm: &Vm, f: impl FnOnce(&mut T) -> R) -> Result<R, Error> {
        let cell = self.cell::<T>(vm)?;
        let mut value = cell
            .try_borrow_mut()
            .map_err(|_| Error::runtime("userdata already borrowed"))?;
        Ok(f(&mut value))
    }

    /// A copy of a plain-data payload (seam item 7, `tagged`).
    pub fn get_copy<T: Copy + 'static>(&self, vm: &Vm) -> Result<T, Error> {
        self.borrow(vm, |t: &T| *t)
    }

    /// Replaces a plain-data payload (seam item 7, `set_tagged`).
    pub fn set_copy<T: Copy + 'static>(&self, vm: &Vm, value: T) -> Result<(), Error> {
        self.borrow_mut(vm, |t: &mut T| *t = value)
    }

    pub fn tag(&self, vm: &Vm) -> Result<u8, Error> {
        Ok(vm.heap.userdata(vm.ud_id(&self.0)?).tag)
    }

    /// Its metatable, the shared per-type or per-tag one (seam item 3).
    pub fn metatable(&self, vm: &mut Vm) -> Result<Option<Table>, Error> {
        let id = vm.ud_id(&self.0)?;
        match vm.heap.userdata(id).meta {
            Some(mt) => vm.handles.insert(RawValue::Table(mt)).map(|h| Some(Table(h))),
            None => Ok(None),
        }
    }

    pub fn set_metatable(&self, vm: &mut Vm, mt: Option<&Table>) -> Result<(), Error> {
        let id = vm.ud_id(&self.0)?;
        let mt = match mt {
            Some(m) => Some(m.id(vm)?),
            None => None,
        };
        vm.heap.set_userdata_meta(id, mt);
        Ok(())
    }

    pub fn to_pointer(&self, vm: &Vm) -> Result<u64, Error> {
        vm.to_pointer(&Value::UserData(self.clone()))
    }
}

impl Thread {
    fn id(&self, vm: &Vm) -> Result<crate::value::ThreadId, Error> {
        match vm.handles.get(&self.0)? {
            RawValue::Thread(id) => Ok(id),
            other => Err(Error::Host(format!("expected a thread, got {}", other.type_name()))),
        }
    }

    /// Resumes the thread with `args` (C++ `lua_resume` from the running
    /// thread, as `coroutine.resume` does) and returns what it yielded or
    /// returned. A thread that fails, or cannot be resumed, is an error
    /// with the thread's error status.
    pub fn resume<A: IntoLuaMulti, R: FromLuaMulti>(&self, vm: &mut Vm, args: A) -> Result<R, Error> {
        let co = self.id(vm)?;
        let args = args.into_lua_multi(vm)?;
        let mut raw = Vec::with_capacity(args.len().saturating_add(1));
        raw.push(RawValue::Thread(co));
        for a in &args.0 {
            raw.push(vm.lower(a)?);
        }
        let resume = vm.protect(|vm| vm.resume_function())?;
        let mut results = vm.call_raw(resume, &raw)?.0.into_iter();
        let ok = matches!(results.next(), Some(Value::Boolean(true)));
        if ok {
            return R::from_lua_multi(MultiValue(results.collect()), vm);
        }
        let error = results.next().unwrap_or_default();
        let kind = match vm.heap.thread(co).status {
            ThreadStatus::Failed(kind) => kind,
            _ => ErrorKind::Runtime,
        };
        let message = match &error {
            Value::String(s) => s.as_bytes(vm)?,
            Value::Number(n) => crate::num::num2str(*n).into_bytes(),
            other => format!("(error object is a {} value)", other.type_name()).into_bytes(),
        };
        Err(Error::Lua { kind, message })
    }

    /// Its status as the running thread sees it.
    pub fn status(&self, vm: &Vm) -> Result<ThreadState, Error> {
        let co = self.id(vm)?;
        Ok(match vm.co_status(co) {
            CoStatus::Running => ThreadState::Running,
            CoStatus::Normal => ThreadState::Normal,
            CoStatus::Suspended => ThreadState::Resumable,
            CoStatus::Dead => match vm.heap.thread(co).status {
                ThreadStatus::Failed(_) => ThreadState::Error,
                _ => ThreadState::Finished,
            },
        })
    }

    /// C++ `lua_resetthread` on a thread that is not running.
    pub fn reset(&self, vm: &mut Vm) -> Result<(), Error> {
        let co = self.id(vm)?;
        if matches!(vm.co_status(co), CoStatus::Running | CoStatus::Normal) {
            return Err(Error::Host("cannot reset a running thread".to_owned()));
        }
        vm.reset_thread(co);
        Ok(())
    }

    pub fn to_pointer(&self, vm: &Vm) -> Result<u64, Error> {
        vm.to_pointer(&Value::Thread(self.clone()))
    }
}

// ---- userdata type registration ---------------------------------------------

/// A host userdata type's fields, methods and metamethods (luaur-rt's
/// `UserData` impl), registered with [`Vm::register_userdata_type`].
///
/// The metatable it builds dispatches as luaur-rt's does (seam rows
/// 129–130): with no fields, `__index` is the method table; with fields,
/// `__index` tries a getter, then the method table, and `__newindex` tries
/// a setter, else raises `attempt to set unknown field 'k' on userdata`.
/// No `__namecall` is installed unless the type adds one.
///
/// Methods receive the payload through a `try_borrow`: a method that
/// re-enters Lua which calls a `_mut` method on the same object gets an
/// error, not a panic.
pub struct UserDataType<T: 'static> {
    type_name: Option<String>,
    getters: Vec<(String, Rc<HostFn>)>,
    setters: Vec<(String, Rc<HostFn>)>,
    methods: Vec<(String, Rc<HostFn>)>,
    meta: Vec<(String, Rc<HostFn>)>,
    _type: PhantomData<fn() -> T>,
}

impl<T: 'static> Default for UserDataType<T> {
    fn default() -> Self {
        UserDataType::new()
    }
}

/// The receiver, `self` of a method: argument 1 as a `T`.
fn receiver<T: 'static>(ctx: &mut HostCtx<'_>) -> Result<Rc<RefCell<T>>, HostError> {
    let args = ctx.args;
    if let Some(RawValue::Userdata(id)) = ctx.vm.arg(args, 1)
        && let Some(cell) = ctx.vm.heap.userdata(id).cell::<T>()
    {
        return Ok(cell);
    }
    let name = std::any::type_name::<T>();
    let name = name.rsplit("::").next().unwrap_or(name);
    let t = ctx.vm.arg_type_error(args, 1, name);
    Err(ctx.throw(t))
}

fn borrow_error() -> HostError {
    HostError::new("userdata already mutably borrowed")
}

fn borrow_mut_error() -> HostError {
    HostError::new("userdata already borrowed")
}

impl<T: 'static> UserDataType<T> {
    pub fn new() -> UserDataType<T> {
        UserDataType {
            type_name: None,
            getters: Vec::new(),
            setters: Vec::new(),
            methods: Vec::new(),
            meta: Vec::new(),
            _type: PhantomData,
        }
    }

    /// Sets `__type` (what `typeof` and argument errors print).
    pub fn type_name(mut self, name: &str) -> Self {
        self.type_name = Some(name.to_owned());
        self
    }

    /// A field read through a getter on the payload.
    pub fn add_field_method_get<R, F>(&mut self, name: &str, f: F)
    where
        R: IntoLua,
        F: Fn(&mut HostCtx<'_>, &T) -> Result<R, HostError> + 'static,
    {
        let host: Rc<HostFn> = Rc::new(move |ctx: &mut HostCtx<'_>| {
            let cell = receiver::<T>(ctx)?;
            let value = cell.try_borrow().map_err(|_| borrow_error())?;
            let r = f(ctx, &value)?;
            drop(value);
            ctx.push(r)?;
            Ok(1)
        });
        self.getters.push((name.to_owned(), host));
    }

    /// A field read through a function of the userdata handle.
    pub fn add_field_function_get<R, F>(&mut self, name: &str, f: F)
    where
        R: IntoLua,
        F: Fn(&mut HostCtx<'_>, AnyUserData) -> Result<R, HostError> + 'static,
    {
        let host: Rc<HostFn> = Rc::new(move |ctx: &mut HostCtx<'_>| {
            let ud: AnyUserData = ctx.get(1)?;
            let r = f(ctx, ud)?;
            ctx.push(r)?;
            Ok(1)
        });
        self.getters.push((name.to_owned(), host));
    }

    /// A field written through a setter on the payload.
    pub fn add_field_method_set<V, F>(&mut self, name: &str, f: F)
    where
        V: FromLua,
        F: Fn(&mut HostCtx<'_>, &mut T, V) -> Result<(), HostError> + 'static,
    {
        let host: Rc<HostFn> = Rc::new(move |ctx: &mut HostCtx<'_>| {
            let cell = receiver::<T>(ctx)?;
            let v: V = ctx.args_from(2)?;
            let mut value = cell.try_borrow_mut().map_err(|_| borrow_mut_error())?;
            f(ctx, &mut value, v)?;
            Ok(0)
        });
        self.setters.push((name.to_owned(), host));
    }

    /// A method reading the payload (`obj:name(...)`).
    pub fn add_method<A, R, F>(&mut self, name: &str, f: F)
    where
        A: FromLuaMulti,
        R: IntoLuaMulti,
        F: Fn(&mut HostCtx<'_>, &T, A) -> Result<R, HostError> + 'static,
    {
        let host: Rc<HostFn> = Rc::new(move |ctx: &mut HostCtx<'_>| {
            let cell = receiver::<T>(ctx)?;
            let a: A = ctx.args_from(2)?;
            let value = cell.try_borrow().map_err(|_| borrow_error())?;
            let r = f(ctx, &value, a)?;
            drop(value);
            ctx.push_results(r)
        });
        self.methods.push((name.to_owned(), host));
    }

    /// A method changing the payload.
    pub fn add_method_mut<A, R, F>(&mut self, name: &str, f: F)
    where
        A: FromLuaMulti,
        R: IntoLuaMulti,
        F: Fn(&mut HostCtx<'_>, &mut T, A) -> Result<R, HostError> + 'static,
    {
        let host: Rc<HostFn> = Rc::new(move |ctx: &mut HostCtx<'_>| {
            let cell = receiver::<T>(ctx)?;
            let a: A = ctx.args_from(2)?;
            let mut value = cell.try_borrow_mut().map_err(|_| borrow_mut_error())?;
            let r = f(ctx, &mut value, a)?;
            drop(value);
            ctx.push_results(r)
        });
        self.methods.push((name.to_owned(), host));
    }

    /// A function in the method table (no receiver check).
    pub fn add_function<A, R, F>(&mut self, name: &str, f: F)
    where
        A: FromLuaMulti,
        R: IntoLuaMulti,
        F: Fn(&mut HostCtx<'_>, A) -> Result<R, HostError> + 'static,
    {
        let host: Rc<HostFn> = Rc::new(move |ctx: &mut HostCtx<'_>| {
            let a: A = ctx.args()?;
            let r = f(ctx, a)?;
            ctx.push_results(r)
        });
        self.methods.push((name.to_owned(), host));
    }

    /// A metamethod reading the payload (`__eq`, `__len`, `__tostring`,
    /// arithmetic, `__namecall`, ...). For binary metamethods the receiver
    /// is argument 1, which Lua makes the userdata only when it is the left
    /// operand; use [`UserDataType::add_meta_function`] when either side
    /// may be it.
    pub fn add_meta_method<A, R, F>(&mut self, name: &str, f: F)
    where
        A: FromLuaMulti,
        R: IntoLuaMulti,
        F: Fn(&mut HostCtx<'_>, &T, A) -> Result<R, HostError> + 'static,
    {
        let host: Rc<HostFn> = Rc::new(move |ctx: &mut HostCtx<'_>| {
            let cell = receiver::<T>(ctx)?;
            let a: A = ctx.args_from(2)?;
            let value = cell.try_borrow().map_err(|_| borrow_error())?;
            let r = f(ctx, &value, a)?;
            drop(value);
            ctx.push_results(r)
        });
        self.meta.push((name.to_owned(), host));
    }

    /// A metamethod with no receiver conversion.
    pub fn add_meta_function<A, R, F>(&mut self, name: &str, f: F)
    where
        A: FromLuaMulti,
        R: IntoLuaMulti,
        F: Fn(&mut HostCtx<'_>, A) -> Result<R, HostError> + 'static,
    {
        let host: Rc<HostFn> = Rc::new(move |ctx: &mut HostCtx<'_>| {
            let a: A = ctx.args()?;
            let r = f(ctx, a)?;
            ctx.push_results(r)
        });
        self.meta.push((name.to_owned(), host));
    }

    /// A metamethod working on the stack (`__namecall` dispatchers that
    /// read [`Vm::namecall_method`]).
    pub fn add_meta_host_function(
        &mut self,
        name: &str,
        f: impl Fn(&mut HostCtx<'_>) -> Result<usize, HostError> + 'static,
    ) {
        self.meta.push((name.to_owned(), Rc::new(f)));
    }
}
