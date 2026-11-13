//! [`Lua`]: the VM owner, the lend slot, host callbacks and the VM-wide
//! services (registry, app data, interrupt, memory limit, loading).

use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::{Rc, Weak};

use super::conversion::{FromLua, FromLuaMulti, IntoLua, IntoLuaMulti};
use super::error::{Error, Result};
use super::handles::{Buffer, Function, LuaString, Table, Thread};
use super::multi::MultiValue;
use super::userdata::{AnyUserData, UserData};
use super::value::{self, Value};
use super::vm;

/// The VM and what the facade keeps beside it.
pub(crate) struct Shared {
    /// The VM, when no handle operation has it out. During a host callback
    /// it is here (lent from the callback's frame), see [`lend`].
    slot: RefCell<Option<Box<vm::Vm>>>,
    /// Spare VMs the callback trampoline swaps in while it lends the real
    /// one out. One per nesting level, reused. Boxed: a VM moves between
    /// the slot, a callback frame and this pool by pointer.
    #[allow(clippy::vec_box)]
    spares: RefCell<Vec<Box<vm::Vm>>>,
    /// The old binding crate's app data: per-VM host state by type.
    app_data: RefCell<HashMap<TypeId, Box<dyn Any>>>,
    /// The metatable built for each `UserData` type (`userdata.rs`).
    pub(crate) userdata_metatables: RefCell<HashMap<TypeId, vm::Table>>,
    /// The message `Thread::resume_module` puts on a module thread's top before
    /// its error hook runs, keyed by the thread's identity (the old binding
    /// crate pushes it onto the thread; the safe VM's thread stack is not the
    /// host's to push on).
    pub(crate) module_error_tops: RefCell<HashMap<usize, Value>>,
    /// Set when the last owning [`Lua`] is dropped and the VM with it.
    closed: std::cell::Cell<bool>,
}

impl Shared {
    /// Runs `f` on the VM. The VM is out of the slot while `f` runs, so a
    /// host callback `f` reaches can lend it back (see [`lend`]).
    pub(crate) fn with_vm<R>(&self, f: impl FnOnce(&mut vm::Vm) -> Result<R>) -> Result<R> {
        let taken = match self.slot.try_borrow_mut() {
            Ok(mut slot) => slot.take(),
            Err(_) => None,
        };
        let Some(boxed) = taken else {
            return Err(Error::RuntimeError(if self.closed.get() {
                "the Lua VM was dropped".to_owned()
            } else {
                "the Lua VM is in use by another operation".to_owned()
            }));
        };
        // The guard puts the VM back when `f` returns or unwinds.
        let mut taken = Taken {
            shared: self,
            vm: Some(boxed),
        };
        let Some(vm) = taken.vm.as_deref_mut() else {
            return Err(Error::RuntimeError("the Lua VM was not taken".to_owned()));
        };
        f(vm).map_err(|error| error.recover_error_object(vm))
    }

    /// Drops the VM and what the facade keeps for it (the last owning
    /// [`Lua`] is gone). Handles that outlive it report the VM dropped.
    fn close(&self) {
        self.closed.set(true);
        let vm = self.slot.try_borrow_mut().ok().and_then(|mut slot| slot.take());
        let spares = self.spares.try_borrow_mut().map(|mut s| std::mem::take(&mut *s));
        let app_data = self.app_data.try_borrow_mut().map(|mut a| std::mem::take(&mut *a));
        let metatables = self
            .userdata_metatables
            .try_borrow_mut()
            .map(|mut m| std::mem::take(&mut *m));
        let tops = self
            .module_error_tops
            .try_borrow_mut()
            .map(|mut t| std::mem::take(&mut *t));
        // Dropped outside the borrows: a payload's `Drop` may reach the
        // facade again.
        drop((vm, spares, app_data, metatables, tops));
    }

    fn spare(&self) -> Result<Box<vm::Vm>> {
        let reused = self.spares.try_borrow_mut().ok().and_then(|mut s| s.pop());
        match reused {
            Some(spare) => Ok(spare),
            None => vm::Vm::new().map(Box::new).map_err(Error::from_vm),
        }
    }
}

/// The VM a [`Shared::with_vm`] took out of the slot. Dropping it puts the
/// VM back, on unwinding too, or drops it if the owner went meanwhile.
struct Taken<'a> {
    shared: &'a Shared,
    vm: Option<Box<vm::Vm>>,
}

impl Drop for Taken<'_> {
    fn drop(&mut self) {
        let Some(vm) = self.vm.take() else {
            return;
        };
        if self.shared.closed.get() {
            return;
        }
        if let Ok(mut slot) = self.shared.slot.try_borrow_mut() {
            *slot = Some(vm);
        }
    }
}

/// Runs `f` with the VM that `vm` (a host callback's `&mut HostCtx`) points
/// at parked in the slot, so every handle operation `f` makes reaches it.
/// The callback frame keeps a spare VM in its place until `f` returns.
pub(crate) fn lend<R>(shared: &Shared, vm: &mut vm::Vm, f: impl FnOnce() -> R) -> Result<R> {
    let mut parked = shared.spare()?;
    std::mem::swap(vm, &mut parked);
    let occupied = match shared.slot.try_borrow_mut() {
        Ok(mut slot) => slot.replace(parked),
        Err(_) => Some(parked),
    };
    if let Some(mut back) = occupied {
        // The slot was not free: undo, and refuse.
        std::mem::swap(vm, &mut back);
        let _ = shared.spares.try_borrow_mut().map(|mut s| s.push(back));
        return Err(Error::RuntimeError(
            "the Lua VM is in use by another operation".to_owned(),
        ));
    }
    let result = f();
    let returned = match shared.slot.try_borrow_mut() {
        Ok(mut slot) => slot.take(),
        Err(_) => None,
    };
    match returned {
        Some(mut real) => {
            std::mem::swap(vm, &mut real);
            let _ = shared.spares.try_borrow_mut().map(|mut s| s.push(real));
            Ok(result)
        }
        None => Err(Error::RuntimeError(
            "the Lua VM was not returned by a host callback".to_owned(),
        )),
    }
}

/// A host callback in the old binding crate's shape.
pub(crate) type BoxedCallback = Box<dyn Fn(&Lua, MultiValue) -> Result<MultiValue>>;

/// The state handle. Cloning shares the VM.
///
/// A `Lua` from [`Lua::new`] (and its clones) owns the VM: the VM is dropped
/// with the last of them. The `Lua` a host callback or interrupt receives is
/// borrowed (the old binding crate's trampoline builds a non-owning `LuaInner`
/// the same way): it and the handles made through it reach the VM but do not
/// keep it alive, so a userdata whose payload holds such a handle does not keep
/// its own VM alive.
#[derive(Clone)]
pub struct Lua(pub(crate) Rc<Shared>, Option<Rc<Owner>>);

/// Ownership of the VM: dropping the last one closes the [`Shared`].
pub(crate) struct Owner(Weak<Shared>);

impl Drop for Owner {
    fn drop(&mut self) {
        if let Some(shared) = self.0.upgrade() {
            shared.close();
        }
    }
}

/// A weak handle to a [`Lua`].
#[derive(Clone)]
pub struct WeakLua(Weak<Shared>, Option<Weak<Owner>>);

impl WeakLua {
    pub fn try_upgrade(&self) -> Option<Lua> {
        let shared = self.0.upgrade().filter(|shared| !shared.closed.get())?;
        let owner = match &self.1 {
            Some(owner) => Some(owner.upgrade()?),
            None => None,
        };
        Some(Lua(shared, owner))
    }

    /// The VM, which must still exist; a dropped VM yields a fresh,
    /// unrelated one (the old binding crate panics here).
    pub fn upgrade(&self) -> Lua {
        self.try_upgrade().unwrap_or_default()
    }
}

impl Default for Lua {
    fn default() -> Self {
        Lua::new()
    }
}

impl std::fmt::Debug for Lua {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lua({:p})", Rc::as_ptr(&self.0))
    }
}

/// What an interrupt asks for (the old binding crate's `VmState`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmState {
    Continue,
    /// Accepted for source compatibility and treated as `Continue`: the
    /// product never yields from an interrupt (seam doc section 3.1).
    Yield,
}

/// Restores the interrupt that [`Lua::scoped_interrupt`] replaced.
pub struct InterruptGuard {
    lua: Lua,
    previous: Option<Box<vm::InterruptFn>>,
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        let previous = self.previous.take();
        let _ = self.lua.0.with_vm(|vm| {
            vm.replace_interrupt(previous);
            Ok(())
        });
    }
}

/// A registry reference (the old binding crate's `RegistryKey`).
#[derive(Debug)]
pub struct RegistryKey(pub(crate) vm::RegistryKey);

/// Shared borrow of app data.
pub type AppDataRef<'a, T> = Ref<'a, T>;
/// Exclusive borrow of app data.
pub type AppDataRefMut<'a, T> = RefMut<'a, T>;

/// What `inspect_stack` reports for a frame (the old binding crate's `Debug`).
#[derive(Clone, Debug)]
pub struct Debug {
    what: DebugWhat,
    source: Option<String>,
    short_src: Option<String>,
    current_line: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugWhat {
    Lua,
    Main,
    C,
    Unknown,
}

impl Debug {
    pub fn name(&self) -> Option<&str> {
        None
    }

    pub fn what(&self) -> DebugWhat {
        self.what
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn short_src(&self) -> Option<&str> {
        self.short_src.as_deref()
    }

    pub fn current_line(&self) -> Option<i64> {
        self.current_line
    }

    pub fn line_defined(&self) -> Option<i64> {
        None
    }
}

/// The old binding crate's `LuauDirectFieldGet` switch. The safe VM always takes
/// the direct-field path (seam doc section 3.8), so this only records the
/// request; [`luau_direct_field_get`] reports `true`.
pub fn set_luau_direct_field_get(_value: bool) {}

/// Whether the direct-field path is on: always, on the safe VM.
pub fn luau_direct_field_get() -> bool {
    true
}

impl Lua {
    /// A VM with the standard libraries open. It sets no Luau flag: neither
    /// the VM nor the bytecode validator reads one; only compiling does
    /// ([`compile`], under its own flag profile).
    pub fn new() -> Lua {
        let vm = vm::Vm::new().ok().map(Box::new);
        let shared = Rc::new(Shared {
            slot: RefCell::new(vm),
            spares: RefCell::new(Vec::new()),
            app_data: RefCell::new(HashMap::new()),
            userdata_metatables: RefCell::new(HashMap::new()),
            module_error_tops: RefCell::new(HashMap::new()),
            closed: std::cell::Cell::new(false),
        });
        let owner = Rc::new(Owner(Rc::downgrade(&shared)));
        Lua(shared, Some(owner))
    }

    /// The non-owning `Lua` a callback receives (see [`Lua`]).
    fn borrowed(shared: Rc<Shared>) -> Lua {
        Lua(shared, None)
    }

    /// Runs `f` on the VM (for the code that uses the safe VM's own API,
    /// such as tagged userdata).
    pub fn with_vm<R>(&self, f: impl FnOnce(&mut vm::Vm) -> Result<R>) -> Result<R> {
        self.0.with_vm(f)
    }

    pub fn weak(&self) -> WeakLua {
        WeakLua(Rc::downgrade(&self.0), self.1.as_ref().map(Rc::downgrade))
    }

    /// An opaque id of the running thread (the old binding crate's
    /// `state_identity`).
    pub fn state_identity(&self) -> usize {
        self.0
            .with_vm(|vm| Ok(usize::try_from(vm.state_identity()).unwrap_or(usize::MAX)))
            .unwrap_or(0)
    }

    // ---- creation --------------------------------------------------------

    /// The running thread's globals. A VM error yields a detached handle
    /// whose operations fail (the old binding crate's signature is infallible).
    pub fn globals(&self) -> Table {
        match self.0.with_vm(|vm| vm.globals().map_err(Error::from_vm)) {
            Ok(t) => Table::wrap(self, t),
            Err(e) => Table::failed(self, e),
        }
    }

    pub fn create_table(&self) -> Table {
        match self.create_table_result() {
            Ok(t) => t,
            Err(e) => Table::failed(self, e),
        }
    }

    pub fn create_table_result(&self) -> Result<Table> {
        self.0
            .with_vm(|vm| vm.create_table().map_err(Error::from_vm))
            .map(|t| Table::wrap(self, t))
    }

    pub fn create_string(&self, s: impl AsRef<[u8]>) -> LuaString {
        match self.0.with_vm(|vm| vm.create_string(s.as_ref()).map_err(Error::from_vm)) {
            Ok(h) => LuaString::wrap(self, h),
            Err(e) => LuaString::failed(self, e),
        }
    }

    pub fn create_table_from<K, V, I>(&self, iter: I) -> Result<Table>
    where
        K: IntoLua,
        V: IntoLua,
        I: IntoIterator<Item = (K, V)>,
    {
        let table = self.create_table_result()?;
        for (k, v) in iter {
            table.raw_set(k, v)?;
        }
        Ok(table)
    }

    pub fn create_sequence_from<V, I>(&self, iter: I) -> Result<Table>
    where
        V: IntoLua,
        I: IntoIterator<Item = V>,
    {
        let table = self.create_table_result()?;
        let mut index: i64 = 1;
        for v in iter {
            table.raw_set(index, v)?;
            index = index.saturating_add(1);
        }
        Ok(table)
    }

    pub fn create_buffer(&self, data: impl AsRef<[u8]>) -> Result<Buffer> {
        self.0
            .with_vm(|vm| vm.create_buffer_from(data.as_ref()).map_err(Error::from_vm))
            .map(|b| Buffer::wrap(self, b))
    }

    pub fn create_buffer_with_capacity(&self, size: usize) -> Result<Buffer> {
        self.0
            .with_vm(|vm| vm.create_buffer(size).map_err(Error::from_vm))
            .map(|b| Buffer::wrap(self, b))
    }

    pub fn create_vector(&self, x: f32, y: f32, z: f32) -> value::Vector {
        value::Vector::new(x, y, z)
    }

    pub fn create_function<F, A, R>(&self, func: F) -> Result<Function>
    where
        F: Fn(&Lua, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        let boxed: BoxedCallback = Box::new(move |lua, args| {
            let a = A::from_lua_multi(args, lua)?;
            let r = func(lua, a)?;
            r.into_lua_multi(lua)
        });
        self.create_callback_function("luaur-rt-callback", boxed)
    }

    pub fn create_function_mut<F, A, R>(&self, func: F) -> Result<Function>
    where
        F: FnMut(&Lua, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        let func = RefCell::new(func);
        self.create_function(move |lua, args| {
            let mut borrow = func
                .try_borrow_mut()
                .map_err(|_| Error::RecursiveMutCallback)?;
            (borrow)(lua, args)
        })
    }

    /// A host function named `name` (in argument errors) from a callback.
    pub(crate) fn create_callback_function(
        &self,
        name: &str,
        callback: BoxedCallback,
    ) -> Result<Function> {
        let weak = Rc::downgrade(&self.0);
        let callback = Rc::new(callback);
        self.0
            .with_vm(|vm| {
                vm.create_host_function(name, move |ctx| trampoline(&weak, &callback, ctx))
                    .map_err(Error::from_vm)
            })
            .map(|f| Function::wrap(self, f))
    }

    pub fn create_userdata<T: UserData + 'static>(&self, data: T) -> Result<AnyUserData> {
        super::userdata::create_userdata(self, data)
    }

    pub fn create_thread(&self, func: Function) -> Result<Thread> {
        let raw = func.raw()?;
        self.0
            .with_vm(|vm| vm.create_thread(raw).map_err(Error::from_vm))
            .map(|t| Thread::wrap(self, t))
    }

    /// A thread whose globals are `func`'s environment, marked safe (the
    /// UNIV c5faa1fa patch's `create_module_thread`; thread data is vacuous,
    /// seam doc section 3.3).
    pub fn create_module_thread(&self, func: Function) -> Result<Thread> {
        let raw = func.raw()?;
        self.0
            .with_vm(|vm| {
                let env = raw.environment(vm).map_err(Error::from_vm)?;
                match env {
                    Some(env) => {
                        env.set_safeenv(vm, true).map_err(Error::from_vm)?;
                        vm.create_thread_with_env(raw, &env)
                            .map_err(Error::from_vm)
                    }
                    None => vm.create_thread(raw).map_err(Error::from_vm),
                }
            })
            .map(|t| Thread::wrap(self, t))
    }

    pub fn current_thread(&self) -> Thread {
        match self.0.with_vm(|vm| vm.current_thread().map_err(Error::from_vm)) {
            Ok(t) => Thread::wrap(self, t),
            Err(e) => Thread::failed(self, e),
        }
    }

    // ---- loading ---------------------------------------------------------

    /// Loads bytecode with the old binding crate's chunk name, `"=" + name`.
    pub fn load_bytecode(&self, name: &str, bytecode: &[u8]) -> Result<Function> {
        let chunk_name = format!("={name}");
        self.load_bytecode_as(chunk_name.as_bytes(), bytecode)
    }

    /// Loads bytecode under a verbatim chunk name (seam item 1, rows R1 and
    /// R34). The safe VM validates the chunk (D7).
    pub fn load_bytecode_as(&self, chunk_name: &[u8], bytecode: &[u8]) -> Result<Function> {
        self.0
            .with_vm(|vm| vm.load(chunk_name, bytecode).map_err(Error::from_vm))
            .map(|f| Function::wrap(self, f))
    }

    /// Source compilation (the old binding crate's `compiler` feature): the
    /// vendored compiler with the product flag profile, then
    /// [`Self::load_bytecode`].
    #[cfg(feature = "compiler")]
    pub fn load(&self, source: impl AsRef<str>) -> Chunk {
        Chunk {
            lua: self.clone(),
            source: source.as_ref().as_bytes().to_vec(),
            name: "chunk".to_owned(),
            environment: None,
        }
    }

    #[cfg(feature = "compiler")]
    pub fn load_bytes(&self, source: impl AsRef<[u8]>) -> Chunk {
        Chunk {
            lua: self.clone(),
            source: source.as_ref().to_vec(),
            name: "chunk".to_owned(),
            environment: None,
        }
    }

    // ---- conversions -----------------------------------------------------

    pub fn pack(&self, value: impl IntoLua) -> Result<Value> {
        value.into_lua(self)
    }

    pub fn pack_multi(&self, values: impl IntoLuaMulti) -> Result<MultiValue> {
        values.into_lua_multi(self)
    }

    pub fn unpack_multi<T: FromLuaMulti>(&self, values: MultiValue) -> Result<T> {
        T::from_lua_multi(values, self)
    }

    pub fn unpack<T: FromLua>(&self, value: Value) -> Result<T> {
        T::from_lua(value, self)
    }

    pub fn coerce_number(&self, value: Value) -> Result<Option<f64>> {
        self.0.with_vm(|vm| {
            let raw = value::to_vm(vm, &value)?;
            vm.coerce_number(&raw).map_err(Error::from_vm)
        })
    }

    pub fn coerce_integer(&self, value: Value) -> Result<Option<i64>> {
        Ok(self.coerce_number(value)?.and_then(|n| {
            (n.fract() == 0.0 && n.is_finite() && n >= i64::MIN as f64 && n <= i64::MAX as f64)
                .then_some(n as i64)
        }))
    }

    /// `lua_tolstring` (seam row R31): strings as they are, numbers in
    /// `%.14g`, anything else `None`.
    pub fn coerce_string(&self, value: Value) -> Result<Option<LuaString>> {
        let lua = self.clone();
        self.0.with_vm(|vm| {
            let raw = value::to_vm(vm, &value)?;
            Ok(vm
                .coerce_string(&raw)
                .map_err(Error::from_vm)?
                .map(|s| LuaString::wrap(&lua, s)))
        })
    }

    /// `lua_objlen` of any value (seam row R32).
    pub fn objlen(&self, value: &Value) -> Result<usize> {
        self.0.with_vm(|vm| {
            let raw = value::to_vm(vm, value)?;
            vm.objlen(&raw).map_err(Error::from_vm)
        })
    }

    /// The metatable of `value`, as the deleted vendored bindings read it with
    /// `lua_getmetatable` in a raw hatch (seam rows R10 to R20): a value
    /// without one is the conversion error that hatch returns for the
    /// missing result.
    pub fn getmetatable(&self, value: impl IntoLua) -> Result<Table> {
        let value = value.into_lua(self)?;
        let metatable = match &value {
            Value::UserData(ud) => ud.metatable()?,
            Value::Table(t) => t.try_metatable()?,
            _ => None,
        };
        metatable.ok_or_else(|| Error::FromLuaConversionError {
            from: "nil",
            to: "Table".to_string(),
            message: None,
        })
    }

    /// A safe-VM function of this VM as a facade [`Function`].
    pub fn wrap_vm_function(&self, f: vm::Function) -> Function {
        Function::wrap(self, f)
    }

    /// A facade value as a rooted safe-VM value: a handle that keeps its
    /// object alive and holds no reference to the [`Lua`].
    pub fn to_vm_value(&self, value: &Value) -> Result<vm::Value> {
        self.0.with_vm(|vm| value::to_vm(vm, value))
    }

    /// A safe-VM value of this VM as a facade value.
    pub fn from_vm_value(&self, value: vm::Value) -> Result<Value> {
        let lua = self.clone();
        self.0.with_vm(|vm| value::from_vm(&lua, vm, value))
    }

    /// `tostring(value)`, `__tostring` honored.
    pub(crate) fn value_to_string(&self, value: &Value) -> Result<String> {
        self.0.with_vm(|vm| {
            let raw = value::to_vm(vm, value)?;
            let s = vm.tostring(&raw).map_err(Error::from_vm)?;
            s.to_string_lossy(vm).map_err(Error::from_vm)
        })
    }

    /// The method name of the running `__namecall` (seam item 4).
    pub fn namecall_method(&self) -> Option<Vec<u8>> {
        self.0.with_vm(|vm| Ok(vm.namecall_method())).ok().flatten()
    }

    pub fn inspect_stack(&self, level: usize) -> Option<Debug> {
        let info = self.0.with_vm(|vm| Ok(vm.inspect_stack(level))).ok()??;
        let source = String::from_utf8_lossy(&info.source).into_owned();
        let what = if info.is_native {
            DebugWhat::C
        } else {
            DebugWhat::Lua
        };
        Some(Debug {
            what,
            short_src: Some(short_src(&source)),
            source: Some(source),
            current_line: (info.current_line >= 0).then_some(i64::from(info.current_line)),
        })
    }

    // ---- sandbox, GC, memory -------------------------------------------

    /// `luaL_sandbox` plus `luaL_sandboxthread` (the old binding crate's
    /// `sandbox(true)`). Leaving the sandbox is not supported here; the product
    /// never does.
    pub fn sandbox(&self, enabled: bool) -> Result<()> {
        if !enabled {
            return Err(Error::runtime(
                "leaving the sandbox is not supported on the safe VM",
            ));
        }
        self.0.with_vm(|vm| vm.sandbox().map_err(Error::from_vm))
    }

    pub fn gc_collect(&self) -> Result<()> {
        self.0.with_vm(|vm| {
            vm.collect();
            Ok(())
        })
    }

    pub fn gc_step_kbytes(&self, kbytes: i32) -> Result<bool> {
        self.0.with_vm(|vm| Ok(vm.gc_step_kb(kbytes)))
    }

    pub fn used_memory(&self) -> usize {
        self.0.with_vm(|vm| Ok(vm.used_memory())).unwrap_or(0)
    }

    /// Caps the VM's memory; returns the previous cap. `0` means no cap,
    /// both ways (the old binding crate and mlua).
    pub fn set_memory_limit(&self, limit: usize) -> Result<usize> {
        let limit = (limit != 0).then_some(limit);
        self.0
            .with_vm(|vm| Ok(vm.set_memory_limit(limit).unwrap_or(0)))
    }

    /// The metatable shared by every vector (the old binding crate's
    /// `set_type_metatable::<Vector>`).
    pub fn set_vector_metatable(&self, metatable: Option<Table>) -> Result<()> {
        let raw = match &metatable {
            Some(t) => Some(t.raw()?.clone()),
            None => None,
        };
        self.0.with_vm(|vm| {
            vm.set_vector_metatable(raw.as_ref())
                .map_err(Error::from_vm)
        })
    }

    /// The old binding crate's `set_type_metatable`: the safe VM has one per-type
    /// metatable a host sets, the vector's.
    pub fn set_type_metatable<T: 'static>(&self, metatable: Option<Table>) -> Result<()> {
        if TypeId::of::<T>() != TypeId::of::<value::Vector>() {
            return Err(Error::runtime(
                "set_type_metatable: only the vector type has a metatable on the safe VM",
            ));
        }
        self.set_vector_metatable(metatable)
    }

    /// The old binding crate's `scope`: functions made through the [`Scope`]
    /// stop working when `f` returns or unwinds (a later call is
    /// `CallbackDestructed`). The callbacks here are `'static`; the bindings'
    /// scoped callbacks borrow nothing.
    pub fn scope<R>(&self, f: impl FnOnce(&Scope) -> Result<R>) -> Result<R> {
        let scope = Scope {
            lua: self.clone(),
            callbacks: RefCell::new(Vec::new()),
        };
        // `Scope`'s `Drop` revokes the callbacks, on unwinding too.
        f(&scope)
    }

    // ---- interrupt -------------------------------------------------------

    pub fn set_interrupt<F>(&self, callback: F)
    where
        F: Fn(&Lua) -> Result<VmState> + 'static,
    {
        let interrupt = self.interrupt_fn(callback);
        let _ = self.0.with_vm(|vm| {
            vm.set_interrupt(interrupt);
            Ok(())
        });
    }

    pub fn scoped_interrupt<F>(&self, callback: F) -> InterruptGuard
    where
        F: Fn(&Lua) -> Result<VmState> + 'static,
    {
        let interrupt = self.interrupt_fn(callback);
        let previous = self
            .0
            .with_vm(|vm| Ok(vm.replace_interrupt(Some(interrupt))))
            .ok()
            .flatten();
        InterruptGuard {
            lua: self.clone(),
            previous,
        }
    }

    pub fn remove_interrupt(&self) {
        let _ = self.0.with_vm(|vm| {
            vm.remove_interrupt();
            Ok(())
        });
    }

    /// The interrupt as the VM calls it. It runs inside the interpreter, so
    /// the VM is not reachable from `callback` (a handle operation there is
    /// an error); the product's interrupts only count and check clocks.
    fn interrupt_fn<F>(&self, callback: F) -> Box<vm::InterruptFn>
    where
        F: Fn(&Lua) -> Result<VmState> + 'static,
    {
        let weak = Rc::downgrade(&self.0);
        Box::new(move || {
            let Some(shared) = weak.upgrade() else {
                return Ok(());
            };
            match callback(&Lua::borrowed(shared)) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into_interrupt_error()),
            }
        })
    }

    // ---- registry --------------------------------------------------------

    pub fn create_registry_value(&self, value: impl IntoLua) -> Result<RegistryKey> {
        let value = value.into_lua(self)?;
        self.0.with_vm(|vm| {
            let raw = value::to_vm(vm, &value)?;
            vm.create_registry_value(raw)
                .map(RegistryKey)
                .map_err(Error::from_vm)
        })
    }

    pub fn registry_value<T: FromLua>(&self, key: &RegistryKey) -> Result<T> {
        let lua = self.clone();
        let value = self.0.with_vm(|vm| {
            let raw: vm::Value = vm.registry_value(&key.0).map_err(Error::from_vm)?;
            value::from_vm(&lua, vm, raw)
        })?;
        T::from_lua(value, self)
    }

    pub fn remove_registry_value(&self, key: RegistryKey) -> Result<()> {
        self.0
            .with_vm(|vm| vm.remove_registry_value(key.0).map_err(Error::from_vm))
    }

    pub fn set_named_registry_value(&self, name: &str, value: impl IntoLua) -> Result<()> {
        let value = value.into_lua(self)?;
        self.0.with_vm(|vm| {
            let raw = value::to_vm(vm, &value)?;
            vm.set_named_registry_value(name, raw)
                .map_err(Error::from_vm)
        })
    }

    pub fn named_registry_value<T: FromLua>(&self, name: &str) -> Result<T> {
        let lua = self.clone();
        let value = self.0.with_vm(|vm| {
            let raw: vm::Value = vm.named_registry_value(name).map_err(Error::from_vm)?;
            value::from_vm(&lua, vm, raw)
        })?;
        T::from_lua(value, self)
    }

    pub fn unset_named_registry_value(&self, name: &str) -> Result<()> {
        self.0
            .with_vm(|vm| vm.unset_named_registry_value(name).map_err(Error::from_vm))
    }

    // ---- app data --------------------------------------------------------

    /// Stores `data`. While any app data is borrowed the store cannot
    /// change; then this does nothing (the old binding crate panics).
    pub fn set_app_data<T: 'static>(&self, data: T) {
        let _ = self.try_set_app_data(data);
    }

    pub fn try_set_app_data<T: 'static>(&self, data: T) -> Result<Option<T>> {
        let mut map = self
            .0
            .app_data
            .try_borrow_mut()
            .map_err(|_| Error::runtime("cannot mutably borrow app data container"))?;
        let old = map.insert(TypeId::of::<T>(), Box::new(data));
        Ok(old.and_then(|b| b.downcast::<T>().ok()).map(|b| *b))
    }

    pub fn app_data_ref<T: 'static>(&self) -> Option<AppDataRef<'_, T>> {
        let map = self.0.app_data.try_borrow().ok()?;
        Ref::filter_map(map, |m| {
            m.get(&TypeId::of::<T>())
                .and_then(|b| b.downcast_ref::<T>())
        })
        .ok()
    }

    /// `Ok(None)` when absent and an error while the store is mutably borrowed.
    /// The old binding crate borrows each entry on its own, so there only a
    /// mutable borrow of `T` itself errors; here one of any entry does.
    pub fn try_app_data_ref<T: 'static>(&self) -> Result<Option<AppDataRef<'_, T>>> {
        let map = self
            .0
            .app_data
            .try_borrow()
            .map_err(|_| Error::runtime("app data is currently mutably borrowed"))?;
        Ok(Ref::filter_map(map, |m| {
            m.get(&TypeId::of::<T>())
                .and_then(|b| b.downcast_ref::<T>())
        })
        .ok())
    }

    pub fn app_data_mut<T: 'static>(&self) -> Option<AppDataRefMut<'_, T>> {
        let map = self.0.app_data.try_borrow_mut().ok()?;
        RefMut::filter_map(map, |m| {
            m.get_mut(&TypeId::of::<T>())
                .and_then(|b| b.downcast_mut::<T>())
        })
        .ok()
    }

    pub fn remove_app_data<T: 'static>(&self) -> Option<T> {
        let mut map = self.0.app_data.try_borrow_mut().ok()?;
        map.remove(&TypeId::of::<T>())
            .and_then(|b| b.downcast::<T>().ok())
            .map(|b| *b)
    }
}

/// `luaL`'s `short_src` for a chunk name: `=name` and `@name` lose their
/// marker; source text becomes `[string "..."]`.
fn short_src(source: &str) -> String {
    if let Some(rest) = source.strip_prefix('=').or_else(|| source.strip_prefix('@')) {
        rest.to_owned()
    } else {
        let first_line = source.lines().next().unwrap_or_default();
        format!("[string \"{first_line}\"]")
    }
}

/// The host-function body every facade callback runs in (the old binding crate's
/// `trampoline`): read the arguments, lend the VM, run the callback with panics
/// caught, push the results or raise the error.
fn trampoline(
    weak: &Weak<Shared>,
    callback: &Rc<BoxedCallback>,
    ctx: &mut vm::HostCtx<'_>,
) -> std::result::Result<usize, vm::HostError> {
    let Some(shared) = weak.upgrade() else {
        return Err(vm::HostError::raw("luaur-rt: missing callback upvalue"));
    };
    let lua = Lua::borrowed(shared);
    let raw: vm::MultiValue = ctx.args()?;
    let args = value::many_from_vm(&lua, ctx, raw).map_err(Error::into_host)?;
    let outcome = lend(&lua.0, ctx, || {
        catch_unwind(AssertUnwindSafe(|| callback(&lua, args)))
    })
    .map_err(Error::into_host)?;
    match outcome {
        Ok(Ok(results)) => {
            let raw = value::many_to_vm(ctx, results.iter()).map_err(Error::into_host)?;
            ctx.push_results(raw)
        }
        Ok(Err(error)) => Err(error.raise(ctx)),
        Err(payload) => Err(vm::HostError::raw(format!(
            "rust panic: {}",
            panic_message(payload.as_ref())
        ))),
    }
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

type ScopedSlot = Rc<RefCell<Option<BoxedCallback>>>;

/// What [`Lua::scope`] hands its closure.
pub struct Scope {
    lua: Lua,
    callbacks: RefCell<Vec<ScopedSlot>>,
}

impl Drop for Scope {
    fn drop(&mut self) {
        for slot in self.callbacks.get_mut().drain(..) {
            if let Ok(mut slot) = slot.try_borrow_mut() {
                *slot = None;
            }
        }
    }
}

impl Scope {
    pub fn create_function<F, A, R>(&self, func: F) -> Result<Function>
    where
        F: Fn(&Lua, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        let callback: BoxedCallback = Box::new(move |lua, args| {
            let a = A::from_lua_multi(args, lua)?;
            func(lua, a)?.into_lua_multi(lua)
        });
        let slot: ScopedSlot = Rc::new(RefCell::new(Some(callback)));
        if let Ok(mut callbacks) = self.callbacks.try_borrow_mut() {
            callbacks.push(Rc::clone(&slot));
        }
        self.lua.create_callback_function(
            "luaur-rt-callback",
            Box::new(move |lua, args| match &*slot.try_borrow().map_err(|_| Error::RecursiveMutCallback)? {
                Some(callback) => callback(lua, args),
                None => Err(Error::CallbackDestructed),
            }),
        )
    }
}

/// A source chunk to compile and run (the old binding crate's `Chunk`,
/// `compiler` feature).
#[cfg(feature = "compiler")]
pub struct Chunk {
    lua: Lua,
    source: Vec<u8>,
    name: String,
    environment: Option<Table>,
}

#[cfg(feature = "compiler")]
impl Chunk {
    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn set_environment(mut self, env: Table) -> Self {
        self.environment = Some(env);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn call<R: FromLuaMulti>(self, args: impl IntoLuaMulti) -> Result<R> {
        self.into_function()?.call::<R>(args)
    }

    pub fn into_function(self) -> Result<Function> {
        let bytecode = compile(&self.source)?;
        let func = self.lua.load_bytecode(&self.name, &bytecode)?;
        if let Some(env) = &self.environment {
            func.set_environment(env.clone())?;
        }
        Ok(func)
    }

    pub fn exec(self) -> Result<()> {
        self.into_function()?.call::<()>(())
    }

    /// Evaluates as an expression first (`return <source>`), then as a chunk, as
    /// the old binding crate does. Only the crate's own tests use it; callers
    /// outside the crate use `into_function` and `call`
    /// (tools/pure-runtime-boundary/check.py flags a public `eval` in a runtime
    /// crate).
    #[cfg(test)]
    pub(crate) fn eval<R: FromLuaMulti>(self) -> Result<R> {
        let mut expression = b"return ".to_vec();
        expression.extend_from_slice(&self.source);
        if let Ok(bytecode) = compile(&expression)
            && let Ok(f) = self.lua.load_bytecode(&self.name, &bytecode)
        {
            if let Some(env) = &self.environment {
                f.set_environment(env.clone())?;
            }
            return f.call::<R>(());
        }
        self.into_function()?.call::<R>(())
    }
}

/// Compiles Luau source with the vendored compiler under the product flag
/// profile and the old binding crate's default options (optimization 1, debug
/// 1).
#[cfg(feature = "compiler")]
pub fn compile(source: &[u8]) -> Result<Vec<u8>> {
    use luaur_ast::records::parse_options::ParseOptions;
    use luaur_bytecode::records::bytecode_encoder::BytecodeEncoder;
    use luaur_compiler::functions::compile::compile_bytes;
    use luaur_compiler::records::compile_options::CompileOptions;

    struct NoopEncoder;
    impl BytecodeEncoder for NoopEncoder {
        fn encode(&mut self, _data: &mut [u32]) {}
    }

    luaur_common::set_all_flags(true);
    let mut encoder = NoopEncoder;
    let blob = compile_bytes(
        source,
        &CompileOptions::default(),
        &ParseOptions::default(),
        &mut encoder as *mut dyn BytecodeEncoder,
    );
    let bytes = blob.into_bytes();
    match bytes.split_first() {
        Some((0, message)) => {
            let message = String::from_utf8_lossy(message).into_owned();
            let incomplete_input = message.contains("<eof>");
            Err(Error::SyntaxError {
                message,
                incomplete_input,
            })
        }
        Some(_) => Ok(bytes),
        None => Err(Error::runtime("the Luau compiler returned nothing")),
    }
}
