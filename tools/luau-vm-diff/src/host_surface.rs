//! The host surface the corpus gets (Phase 2 lane 6): host functions, a
//! userdata type, the interrupt and the memory limit, registered through the
//! embedding API on `nuxie-luau-vm`. The snapshots of the cases that use
//! them (`tests/luau-vm-diff/host_*.luau`) were recorded with the same
//! surface registered on the vendored VM, so those cases test the embedding
//! API differentially.
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
//! - `hosttable(k1, k2, ...)`: a table the host creates and fills with
//!   `raw_set(k_i, i)` in argument order.
//! - `hostkeys(t)`: a sequence of `t`'s keys in the host API's `pairs`
//!   order (docs/luau-safe-vm-pairs-order.md, 3.3 item 7).
//!
//! An unknown-field store raises `attempt to set unknown field '<k>' on
//! userdata` without a position: the vendored VM's dispatcher was a Lua
//! closure that called `error(..., 0)`, and `nuxie-luau-vm` raises the same
//! text raw.

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
    let make = vm.create_host_function("hosttable", |ctx| {
        let table = ctx.create_table().map_err(HostError::from)?;
        for i in 1..=ctx.top() {
            let key = ctx.value(i as isize)?;
            table
                .raw_set(ctx, key, i as f64)
                .map_err(HostError::from)?;
        }
        ctx.push(table)?;
        Ok(1)
    })?;
    globals.raw_set(vm, "hosttable", make)?;
    let keys = vm.create_host_function("hostkeys", |ctx| {
        let table: nuxie_luau_vm::Table = ctx.get(1)?;
        let pairs = table
            .pairs::<nuxie_luau_vm::Value, nuxie_luau_vm::Value>(ctx)
            .map_err(HostError::from)?;
        let keys = ctx
            .create_sequence_from(pairs.into_iter().map(|(k, _)| k))
            .map_err(HostError::from)?;
        ctx.push(keys)?;
        Ok(1)
    })?;
    globals.raw_set(vm, "hostkeys", keys)?;
    Ok(())
}
