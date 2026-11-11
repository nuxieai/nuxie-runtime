//! The host surface both VMs get (Phase 2 lane 6): the same host
//! functions, userdata type, interrupt and memory limit, registered through
//! `luaur-rt` on the vendored VM and through the embedding API on
//! `nuxie-luau-vm`, so a corpus case that uses them is a differential test
//! of the embedding API (`tests/luau-vm-diff/host_*.luau`).
//!
//! Globals:
//!
//! - `hostadd(a, b)`: a typed host function.
//! - `hostcall(f, ...)`: calls `f` from the host and returns its results;
//!   an error comes back as a `luaL_error` (positioned) with its message.
//! - `hosterror(message)`: raises `luaL_error(message)`.
//! - `newcounter(n)`: a `Counter` userdata: field `value` (get and set),
//!   methods `get()` and `add(k)`, metamethods `__len` and `__tostring`.
//!   `drops()` counts destroyed counters.
//! - `interruptcount()`: interrupts fired so far (GC steps excluded).
//! - `setinterruptlimit(n)`: the interrupt raises `interrupted` after `n`
//!   more firings, once.
//! - `memlimit(extra)`: caps memory at what is in use plus `extra` bytes;
//!   `memlimit()` removes the cap.
//!
//! An unknown-field store raises `attempt to set unknown field '<k>' on
//! userdata` without a position on both: `luaur-rt`'s dispatcher is the
//! Lua closure `USERDATA_DISPATCH` (`vendor/luaur-rt-0.1.8/build.rs`,
//! `error(..., 0)`), and `nuxie-luau-vm` raises the same text raw.

use std::cell::Cell;
use std::rc::Rc;

/// Interrupt state shared by the interrupt closure and the globals.
#[derive(Default)]
pub struct Interrupts {
    count: Cell<u64>,
    /// Firings left before the interrupt raises.
    remaining: Cell<Option<u64>>,
}

impl Interrupts {
    /// One firing: `Err` when the limit ran out.
    fn fire(&self) -> Result<(), ()> {
        self.count.set(self.count.get().saturating_add(1));
        match self.remaining.get() {
            Some(0) => {
                self.remaining.set(None);
                Err(())
            }
            Some(n) => {
                self.remaining.set(Some(n.saturating_sub(1)));
                Ok(())
            }
            None => Ok(()),
        }
    }
}

pub struct Counter {
    value: f64,
    drops: Rc<Cell<u32>>,
}

impl Drop for Counter {
    fn drop(&mut self) {
        self.drops.set(self.drops.get().saturating_add(1));
    }
}

fn counter_text(value: f64) -> String {
    format!("Counter({value})")
}

// ---- the vendored VM ----------------------------------------------------

impl luaur_rt::UserData for Counter {
    fn add_fields<F: luaur_rt::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("value", |_, c| Ok(c.value));
        fields.add_field_method_set("value", |_, c, v: f64| {
            c.value = v;
            Ok(())
        });
    }

    fn add_methods<M: luaur_rt::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get", |_, c, ()| Ok(c.value));
        methods.add_method_mut("add", |_, c, k: f64| {
            c.value += k;
            Ok(c.value)
        });
        methods.add_meta_method("__len", |_, c, ()| Ok(c.value));
        methods.add_meta_method("__tostring", |_, c, ()| Ok(counter_text(c.value)));
    }
}

fn reference_text(error: &luaur_rt::Error) -> String {
    use luaur_rt::Error;
    match error {
        Error::RuntimeError(text) | Error::LuaLRuntimeError(text) | Error::MemoryError(text) => {
            text.clone()
        }
        other => other.to_string(),
    }
}

/// Installs the surface on the vendored VM.
pub fn install_reference(lua: &luaur_rt::Lua) -> luaur_rt::Result<()> {
    use luaur_rt::{Error, Function, MultiValue, Value, Variadic, VmState};

    let globals = lua.globals();
    globals.set(
        "hostadd",
        lua.create_function(|_, (a, b): (f64, f64)| Ok(a + b))?,
    )?;
    globals.set(
        "hostcall",
        lua.create_function(|_, (f, args): (Function, Variadic<Value>)| {
            f.call::<MultiValue>(MultiValue::from_vec(args.into_iter().collect()))
                .map_err(|e| Error::lua_l_runtime(reference_text(&e)))
        })?,
    )?;
    globals.set(
        "hosterror",
        lua.create_function(|_, message: String| -> luaur_rt::Result<()> {
            Err(Error::lua_l_runtime(message))
        })?,
    )?;
    let drops = Rc::new(Cell::new(0u32));
    let made = Rc::clone(&drops);
    globals.set(
        "newcounter",
        lua.create_function(move |lua, value: f64| {
            lua.create_userdata(Counter {
                value,
                drops: Rc::clone(&made),
            })
        })?,
    )?;
    globals.set(
        "drops",
        lua.create_function(move |_, ()| Ok(f64::from(drops.get())))?,
    )?;
    let interrupts = Rc::new(Interrupts::default());
    let fired = Rc::clone(&interrupts);
    lua.set_interrupt(move |_| match fired.fire() {
        Ok(()) => Ok(VmState::Continue),
        Err(()) => Err(Error::RuntimeError("interrupted".to_owned())),
    });
    let counted = Rc::clone(&interrupts);
    globals.set(
        "interruptcount",
        lua.create_function(move |_, ()| Ok(counted.count.get() as f64))?,
    )?;
    globals.set(
        "setinterruptlimit",
        lua.create_function(move |_, n: f64| {
            interrupts.remaining.set(Some(n as u64));
            Ok(())
        })?,
    )?;
    globals.set(
        "memlimit",
        lua.create_function(|lua, extra: Option<f64>| {
            let limit = extra.map_or(0, |extra| lua.used_memory().saturating_add(extra as usize));
            lua.set_memory_limit(limit)?;
            Ok(())
        })?,
    )?;
    Ok(())
}

// ---- nuxie-luau-vm ------------------------------------------------------------

/// Installs the surface on `nuxie-luau-vm`.
pub fn install_nuxie(vm: &mut nuxie_luau_vm::Vm) -> Result<(), nuxie_luau_vm::Error> {
    use nuxie_luau_vm::{Function, HostError, MultiValue, UserDataType};

    let globals = vm.globals()?;
    let add = vm.create_function("hostadd", |_, (a, b): (f64, f64)| Ok(a + b))?;
    globals.raw_set(vm, "hostadd", add)?;
    let call = vm.create_host_function("hostcall", |ctx| {
        let f: Function = ctx.get(1)?;
        let mut args = Vec::new();
        for i in 2..=ctx.top() {
            args.push(ctx.value(i as isize)?);
        }
        let results: MultiValue = f
            .call(ctx, MultiValue(args))
            .map_err(|e| HostError::new(e.message()))?;
        ctx.push_results(results)
    })?;
    globals.raw_set(vm, "hostcall", call)?;
    let error = vm.create_function("hosterror", |_, message: String| -> Result<(), HostError> {
        Err(HostError::new(message))
    })?;
    globals.raw_set(vm, "hosterror", error)?;

    let mut counter = UserDataType::<Counter>::new();
    counter.add_field_method_get("value", |_, c| Ok(c.value));
    counter.add_field_method_set("value", |_, c, v: f64| {
        c.value = v;
        Ok(())
    });
    counter.add_method("get", |_, c, ()| Ok(c.value));
    counter.add_method_mut("add", |_, c, k: f64| {
        c.value += k;
        Ok(c.value)
    });
    counter.add_meta_method("__len", |_, c, ()| Ok(c.value));
    counter.add_meta_method("__tostring", |_, c, ()| Ok(counter_text(c.value)));
    vm.register_userdata_type(counter)?;
    let drops = Rc::new(Cell::new(0u32));
    let made = Rc::clone(&drops);
    let new = vm.create_function("newcounter", move |ctx, value: f64| {
        ctx.create_userdata(Counter {
            value,
            drops: Rc::clone(&made),
        })
        .map_err(HostError::from)
    })?;
    globals.raw_set(vm, "newcounter", new)?;
    let count = vm.create_function("drops", move |_, (): ()| Ok(f64::from(drops.get())))?;
    globals.raw_set(vm, "drops", count)?;

    let interrupts = Rc::new(Interrupts::default());
    let fired = Rc::clone(&interrupts);
    vm.set_interrupt(move || {
        fired
            .fire()
            .map_err(|()| HostError::raw("interrupted"))
    });
    let counted = Rc::clone(&interrupts);
    let count = vm.create_function("interruptcount", move |_, (): ()| {
        Ok(counted.count.get() as f64)
    })?;
    globals.raw_set(vm, "interruptcount", count)?;
    let limit = vm.create_function("setinterruptlimit", move |_, n: f64| {
        interrupts.remaining.set(Some(n as u64));
        Ok(())
    })?;
    globals.raw_set(vm, "setinterruptlimit", limit)?;
    let mem = vm.create_function("memlimit", |ctx, extra: Option<f64>| {
        let limit = extra.map(|extra| ctx.used_memory().saturating_add(extra as usize));
        ctx.set_memory_limit(limit);
        Ok(())
    })?;
    globals.raw_set(vm, "memlimit", mem)?;
    Ok(())
}
