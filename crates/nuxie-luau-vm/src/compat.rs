//! The lane-6 API's host (docs/luau-safe-vm-embedding-core.md, section
//! 5.8): host functions are Rust closures, which the core cannot hold as
//! plain data. [`Compat`] keeps them in a slot store; a closure's native is
//! `NativeFn::Host(CALL or CALL_K, slot)`, and three fixed table entries
//! (trampolines) find the closure by its slot and call it. [`Vm`] is the
//! core hosted by `Compat`: the API `nuxie-scripting` holds.
//!
//! A slot is released when the collector has freed its native: the sweep
//! lists the native (hosted.rs), and `Compat` drains the list before it
//! allocates a slot and at the end of every protected host operation, so a
//! closure's captured state drops there, outside the collector.

use std::rc::Rc;

use crate::error::{Error, VmResult};
use crate::heap::{Function, NativeClosure};
use crate::host::{CallStatus, HostCtx, HostError};
use crate::hosted::{Host, HostEntry, HostedVm};
use crate::native::{HostFnId, NativeFn};
use crate::value::FnId;
use crate::vm::Core;

/// A host closure registered through the public API.
pub type HostFn = dyn Fn(&mut HostCtx<'_>) -> Result<usize, HostError>;

/// A host function's continuation (C++ `lua_Continuation`): runs when a
/// call it requested returns (or, for a protected call, fails), or when a
/// yield it made is resumed.
pub type HostContFn = dyn Fn(&mut HostCtx<'_>, CallStatus) -> Result<usize, HostError>;

/// The VM the lane-6 API offers: the core hosted by [`Compat`].
pub type Vm = HostedVm<Compat>;

/// A closure without a continuation.
const CALL: HostFnId = HostFnId(0);
/// A closure with a continuation (`CONT`).
const CALL_K: HostFnId = HostFnId(1);
/// The continuation of a `CALL_K` closure.
const CONT: HostFnId = HostFnId(2);

struct Closure {
    f: Rc<HostFn>,
    k: Option<Rc<HostContFn>>,
}

/// The lane-6 API's host state: the closure store.
#[derive(Default)]
pub struct Compat {
    closures: Vec<Option<Closure>>,
    free: Vec<u32>,
}

impl Host for Compat {}

impl Compat {
    fn closure(&self, slot: u32) -> Result<&Closure, HostError> {
        self.closures
            .get(slot as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| HostError::raw("internal error: a released host closure ran"))
    }

    fn insert(&mut self, closure: Closure) -> Option<u32> {
        if let Some(slot) = self.free.pop() {
            if let Some(entry) = self.closures.get_mut(slot as usize) {
                *entry = Some(closure);
                return Some(slot);
            }
            return None;
        }
        let slot = u32::try_from(self.closures.len()).ok()?;
        self.closures.push(Some(closure));
        Some(slot)
    }

    fn release(&mut self, slot: u32) {
        if let Some(entry) = self.closures.get_mut(slot as usize)
            && entry.take().is_some()
        {
            self.free.push(slot);
        }
    }
}

fn call(ctx: &mut HostCtx<'_>) -> Result<usize, HostError> {
    let f = Rc::clone(&ctx.vm.host.closure(ctx.data)?.f);
    f(ctx)
}

fn cont(ctx: &mut HostCtx<'_>, status: CallStatus) -> Result<usize, HostError> {
    let k = ctx
        .vm
        .host
        .closure(ctx.data)?
        .k
        .clone()
        .ok_or_else(|| HostError::raw("internal error: a host closure has no continuation"))?;
    k(ctx, status)
}

impl Vm {
    /// `core` hosted by an empty closure store.
    pub(crate) fn with_compat(core: Core) -> Result<Vm, Error> {
        let mut vm = HostedVm::from_core(core, Compat::default());
        let ids = (
            vm.add_host_fn(HostEntry::Full { f: call, k: None }, true)?,
            vm.add_host_fn(
                HostEntry::Full {
                    f: call,
                    k: Some(CONT),
                },
                true,
            )?,
            vm.add_host_fn(HostEntry::Cont(cont), false)?,
        );
        if ids != (CALL, CALL_K, CONT) {
            return Err(Error::Host("internal error: host entries out of place".to_owned()));
        }
        Ok(vm)
    }

    /// Releases the closures of natives the collector freed.
    pub(crate) fn drain_freed(&mut self) {
        for (id, slot) in self.core.heap.take_freed_natives() {
            if id == CALL || id == CALL_K {
                self.host.release(slot);
            }
        }
    }

    /// A native function for the closure `f`, with continuation `k`.
    pub(crate) fn new_host_native(
        &mut self,
        name: &str,
        f: Rc<HostFn>,
        k: Option<Rc<HostContFn>>,
    ) -> VmResult<FnId> {
        self.drain_freed();
        let id = if k.is_some() { CALL_K } else { CALL };
        let Some(slot) = self.host.insert(Closure { f, k }) else {
            return Err(self.core.heap.memory_error());
        };
        let native = Function::Native(NativeClosure {
            func: NativeFn::Host(id, slot),
            name: Box::from(name),
            upvals: Box::default(),
        });
        match self.core.heap.new_function(native) {
            Ok(fid) => Ok(fid),
            Err(throw) => {
                self.host.release(slot);
                Err(throw)
            }
        }
    }
}
