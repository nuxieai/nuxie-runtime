//! The lane-6 API's host (docs/luau-safe-vm-embedding-core.md, section
//! 5.8): everything of lane 6 the core cannot hold as plain data.
//! [`Vm`] is the core hosted by [`Compat`]: the API `nuxie-scripting` holds.
//!
//! - **Host closures** live in a slot store; a closure's native is
//!   `NativeFn::HostFull(CALL or CALL_K, slot)`, and three fixed table entries
//!   (trampolines) find the closure by its slot and call it.
//! - **Userdata payloads** (`Rc<RefCell<T>>`, erased, with the type's id
//!   and name) live in a store keyed by the userdata's `key`: a slot and a
//!   generation, so a key outliving its payload finds nothing.
//! - **Per-type metatables, app data and direct-field getters**, keyed by
//!   `TypeId` or tag. Each metatable is held by a core root while here.
//! - **Handles** (`handle.rs`): each owns a core root; a dropped handle
//!   queues its root on the release queue, which `drain_freed` empties.
//!   The error object kept for `take_error_object` is a handle value.
//!
//! When the collector frees a native or a userdata, the sweep lists its
//! slot or key, and `Compat` drains the lists before it allocates a slot,
//! at the start and end of every trampoline call and at the end of every
//! protected host operation: captured state and payloads drop there,
//! outside the collector, and before any host code can look (decision 3).

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::DirectFields;
use super::handle::{Handle, ReleaseQueue};
use crate::error::{Error, VmResult};
use crate::heap::{Function, NativeClosure};
use super::HostCtx;
use crate::host::{CallStatus, HostError};
use crate::hosted::{Host, HostEntry, HostedVm, LoanFamily, NoLoan};
use crate::native::{HostFnId, NativeFn};
use crate::roots::Root;
use crate::value::{FnId, TableId, Value};
use crate::vm::Core;

/// A host closure registered through the public API.
pub type HostFn<L = NoLoan> = dyn Fn(&mut HostCtx<'_, L>) -> Result<usize, HostError>;

/// A host function's continuation (C++ `lua_Continuation`): runs when a
/// call it requested returns (or, for a protected call, fails), or when a
/// yield it made is resumed.
pub type HostContFn<L = NoLoan> = dyn Fn(&mut HostCtx<'_, L>, CallStatus) -> Result<usize, HostError>;

/// The host's interrupt callback (C++ `lua_callbacks()->interrupt`, GC
/// steps excluded): asked at every VM safepoint and pattern step; an error
/// stops the script with that error (`Host::interrupt`).
pub type InterruptFn = dyn FnMut() -> Result<(), HostError>;

/// The VM the lane-6 API offers: the core hosted by [`Compat`], lending
/// what `L` names to the calls made with a loan ([`NoLoan`] by default).
pub type Vm<L = NoLoan> = HostedVm<Compat<L>>;

/// The entry of a closure without a continuation.
const CALL: u32 = 0;
/// The entry of a closure with a continuation (`CONT`).
const CALL_K: u32 = 1;
/// The entry of the continuation of a `CALL_K` closure.
const CONT: u32 = 2;

struct Closure<L: LoanFamily> {
    f: Rc<HostFn<L>>,
    k: Option<Rc<HostContFn<L>>>,
}

/// A userdata's host value.
pub(crate) struct Payload {
    /// `Rc<RefCell<T>>`, erased.
    pub(crate) value: Rc<dyn Any>,
    pub(crate) type_id: TypeId,
    /// `std::any::type_name::<T>()`, for errors.
    pub(crate) type_name: &'static str,
}

impl Payload {
    pub(crate) fn new<T: 'static>(value: T) -> Payload {
        Payload {
            value: Rc::new(RefCell::new(value)),
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
        }
    }

    /// The value as `Rc<RefCell<T>>`, if it holds a `T`.
    pub(crate) fn cell<T: 'static>(&self) -> Option<Rc<RefCell<T>>> {
        Rc::clone(&self.value).downcast::<RefCell<T>>().ok()
    }

    pub(crate) fn is<T: 'static>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }
}

#[derive(Default)]
struct PayloadSlot {
    generation: u32,
    payload: Option<Payload>,
}

/// The lane-6 API's host state.
pub struct Compat<L: LoanFamily = NoLoan> {
    closures: Vec<Option<Closure<L>>>,
    free: Vec<u32>,
    payloads: Vec<PayloadSlot>,
    free_payloads: Vec<u32>,
    /// The metatable each host userdata type gets at creation, and the
    /// root keeping it alive while it is here.
    pub(crate) type_metas: HashMap<TypeId, (TableId, Root)>,
    /// Per-VM host state (the old binding crate's app data), by type.
    pub(crate) app_data: HashMap<TypeId, Rc<dyn Any>>,
    /// Direct-field getters per userdata tag.
    pub(crate) direct_fields: Vec<Option<Rc<DirectFields>>>,
    /// The interrupt; the core's `interrupt_on` says whether there is one.
    pub(crate) interrupt: Option<Box<InterruptFn>>,
    /// Roots of dropped handles, released at the next drain.
    release_queue: Rc<ReleaseQueue>,
    /// Roots owned by handles, queued ones included.
    handle_roots: usize,
    /// The last error object that reached the host as `(error object is a
    /// <type> value)`, until taken
    /// ([`Vm::take_error_object`](crate::Vm::take_error_object)).
    pub(crate) error_object: Option<super::handle::Value>,
}

impl<L: LoanFamily> Default for Compat<L> {
    fn default() -> Self {
        Compat {
            closures: Vec::new(),
            free: Vec::new(),
            payloads: Vec::new(),
            free_payloads: Vec::new(),
            type_metas: HashMap::new(),
            app_data: HashMap::new(),
            direct_fields: Vec::new(),
            interrupt: None,
            release_queue: Rc::default(),
            handle_roots: 0,
            error_object: None,
        }
    }
}

impl<L: LoanFamily> Host for Compat<L> {
    type Loans = L;

    fn interrupt(&mut self) -> Result<(), HostError> {
        match &mut self.interrupt {
            Some(f) => f(),
            None => Ok(()),
        }
    }

    fn direct_field(&mut self, tag: u8, key: u64, field: &[u8]) -> Option<f64> {
        let fields = self.direct_fields.get(usize::from(tag))?.as_ref()?;
        let payload = self.payload(key)?;
        fields.get(field, payload.value.as_ref())
    }
}

impl<L: LoanFamily> Compat<L> {
    fn closure(&self, slot: u32) -> Result<&Closure<L>, HostError> {
        self.closures
            .get(slot as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| HostError::raw("internal error: a released host closure ran"))
    }

    fn insert(&mut self, closure: Closure<L>) -> Option<u32> {
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

    /// Stores `payload` under a new key (never 0): slot plus one in the low
    /// half, the slot's generation in the high half.
    fn insert_payload(&mut self, payload: Payload) -> Option<u64> {
        let slot = match self.free_payloads.pop() {
            Some(slot) => slot,
            None => {
                let slot = u32::try_from(self.payloads.len())
                    .ok()
                    .filter(|&s| s < u32::MAX)?;
                self.payloads.push(PayloadSlot::default());
                slot
            }
        };
        let entry = self.payloads.get_mut(slot as usize)?;
        entry.payload = Some(payload);
        let low = u64::from(slot).checked_add(1)?;
        Some(u64::from(entry.generation).checked_shl(32)? | low)
    }

    fn payload_slot(&self, key: u64) -> Option<(usize, &PayloadSlot)> {
        let low = u32::try_from(key & u64::from(u32::MAX)).ok()?;
        let index = low.checked_sub(1)? as usize;
        let generation = u32::try_from(key.checked_shr(32)?).ok()?;
        let entry = self.payloads.get(index)?;
        (entry.generation == generation).then_some((index, entry))
    }

    /// The payload under `key`, if it is still there.
    pub(crate) fn payload(&self, key: u64) -> Option<&Payload> {
        self.payload_slot(key)?.1.payload.as_ref()
    }

    /// Drops the payload under `key` (the collector freed its userdata).
    fn release_payload(&mut self, key: u64) {
        let Some((index, _)) = self.payload_slot(key) else {
            return;
        };
        if let Some(entry) = self.payloads.get_mut(index)
            && entry.payload.take().is_some()
        {
            entry.generation = entry.generation.wrapping_add(1);
            if let Ok(slot) = u32::try_from(index) {
                self.free_payloads.push(slot);
            }
        }
    }
}

fn call<L: LoanFamily>(ctx: &mut HostCtx<'_, L>) -> Result<usize, HostError> {
    ctx.vm.drain_freed();
    let f = Rc::clone(&ctx.vm.host.closure(ctx.data)?.f);
    let result = f(ctx);
    ctx.vm.drain_freed();
    result
}

fn cont<L: LoanFamily>(ctx: &mut HostCtx<'_, L>, status: CallStatus) -> Result<usize, HostError> {
    ctx.vm.drain_freed();
    let k = ctx
        .vm
        .host
        .closure(ctx.data)?
        .k
        .clone()
        .ok_or_else(|| HostError::raw("internal error: a host closure has no continuation"))?;
    let result = k(ctx, status);
    ctx.vm.drain_freed();
    result
}

impl<L: LoanFamily> Vm<L> {
    /// `core` hosted by an empty closure store.
    pub(crate) fn with_compat(core: Core) -> Result<Vm<L>, Error> {
        let mut vm = HostedVm::from_core(core, Compat::default());
        let cont_id = vm.compat_entry(CONT);
        let ids = (
            vm.add_host_fn(HostEntry::Full { f: call, k: None }, true)?,
            vm.add_host_fn(
                HostEntry::Full {
                    f: call,
                    k: Some(cont_id),
                },
                true,
            )?,
            vm.add_host_fn(HostEntry::Cont(cont), false)?,
        );
        let fixed = (
            vm.compat_entry(CALL),
            vm.compat_entry(CALL_K),
            vm.compat_entry(CONT),
        );
        if ids != fixed {
            return Err(Error::Host("internal error: host entries out of place".to_owned()));
        }
        Ok(vm)
    }

    /// Releases the closures and payloads of the natives and userdata the
    /// collector freed, then the roots of dropped handles (a payload's
    /// destructor may drop handles, so those go in the same drain).
    pub(crate) fn drain_freed(&mut self) {
        let (call, call_k) = (self.compat_entry(CALL), self.compat_entry(CALL_K));
        for (id, slot) in self.core.heap.take_freed_natives() {
            if id == call || id == call_k {
                self.host.release(slot);
            }
        }
        for key in self.core.heap.take_freed_userdata() {
            self.host.release_payload(key);
        }
        self.release_dropped_handles();
    }

    /// The layer's safepoint at a host boundary: dropped handles' roots go
    /// first, so the collection sees what the host let go, and what it frees is
    /// released after, so destructors have run when the host operation returns
    /// (the old binding crate's timing). Every safepoint of the layer is this
    /// one; a bare `Core::safepoint` here would keep a dropped handle's object
    /// alive through the cycle.
    pub(crate) fn host_safepoint(&mut self) {
        self.drain_freed();
        self.core.safepoint();
        self.drain_freed();
    }

    /// Releases the roots of dropped handles (allocation-free).
    fn release_dropped_handles(&mut self) {
        let released = self.host.release_queue.take();
        for &root in &released {
            self.core.roots.release(root);
        }
        self.host.handle_roots = self.host.handle_roots.saturating_sub(released.len());
        self.host.release_queue.restore(released);
    }

    /// The id of `Compat`'s fixed entry `index` in this VM's table.
    fn compat_entry(&self, index: u32) -> HostFnId {
        HostFnId::new(self.core.roots.vm_id(), index)
    }

    /// A new handle keeping `value` alive while some clone of it is held.
    pub(crate) fn new_handle(&mut self, value: Value) -> Result<Handle, Error> {
        // Dropped handles' slots first: an API that makes handles outside
        // any drain point (`globals`, `registry_value`, ...) would otherwise
        // grow the root table with every handle made and dropped. Nothing
        // collects between this and the insert, so `value` stays alive.
        self.release_dropped_handles();
        let root = self.core.roots.insert(value)?;
        self.host.handle_roots = self.host.handle_roots.saturating_add(1);
        // Room for every handle's drop, so a drop never allocates.
        self.host.release_queue.reserve(self.host.handle_roots);
        Ok(Handle::new(root, &self.host.release_queue))
    }

    /// The value `handle` keeps; an error for another VM's handle.
    pub(crate) fn handle_value(&self, handle: &Handle) -> Result<Value, Error> {
        self.core.roots.get(handle.root())
    }

    /// Handles held by the host (dropped ones waiting for release excluded).
    pub(crate) fn live_handle_count(&self) -> usize {
        self.host
            .handle_roots
            .saturating_sub(self.host.release_queue.len())
    }

    /// A new userdata holding `payload`, with metatable `meta`.
    pub(crate) fn new_userdata(
        &mut self,
        payload: Payload,
        size: usize,
        tag: u8,
        meta: Option<TableId>,
    ) -> VmResult<crate::value::UdId> {
        self.drain_freed();
        let Some(key) = self.host.insert_payload(payload) else {
            return Err(self.core.heap.memory_error());
        };
        let ud = crate::userdata::Userdata {
            key,
            meta,
            tag,
            size,
        };
        match self.core.heap.new_userdata(ud) {
            Ok(id) => Ok(id),
            Err(throw) => {
                self.host.release_payload(key);
                Err(throw)
            }
        }
    }

    /// The payload of userdata `id`.
    pub(crate) fn ud_payload(&self, id: crate::value::UdId) -> Option<&Payload> {
        self.host.payload(self.core.heap.userdata(id).key)
    }

    /// Makes `meta` the metatable every later userdata of type `type_id`
    /// gets (`None` removes it), keeping it rooted while it is here.
    pub(crate) fn set_type_meta(
        &mut self,
        type_id: TypeId,
        meta: Option<TableId>,
    ) -> Result<(), Error> {
        let old = match meta {
            Some(id) => {
                let root = self.core.roots.insert(Value::Table(id))?;
                self.host.type_metas.insert(type_id, (id, root))
            }
            None => self.host.type_metas.remove(&type_id),
        };
        if let Some((_, root)) = old {
            self.core.roots.release(root);
        }
        Ok(())
    }

    /// A native function for the closure `f`, with continuation `k`.
    pub(crate) fn new_host_native(
        &mut self,
        name: &str,
        f: Rc<HostFn<L>>,
        k: Option<Rc<HostContFn<L>>>,
    ) -> VmResult<FnId> {
        self.drain_freed();
        let id = self.compat_entry(if k.is_some() { CALL_K } else { CALL });
        let Some(slot) = self.host.insert(Closure { f, k }) else {
            return Err(self.core.heap.memory_error());
        };
        let native = Function::Native(NativeClosure {
            func: NativeFn::HostFull(id, slot),
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

#[cfg(test)]
mod tests {
    use super::{Compat, Payload};
    use crate::hosted::NoLoan;

    #[test]
    fn a_released_payload_key_finds_nothing_once_its_slot_is_reused() -> Result<(), &'static str> {
        let mut compat = Compat::<NoLoan>::default();
        let old = compat.insert_payload(Payload::new(1u32)).ok_or("insert")?;
        compat.release_payload(old);
        // A second release of the same key is a no-op: the slot is not
        // freed twice.
        compat.release_payload(old);
        let new = compat.insert_payload(Payload::new(2u32)).ok_or("insert")?;
        let other = compat.insert_payload(Payload::new(3u32)).ok_or("insert")?;
        if new == old || other == old || new == other {
            return Err("a key was handed out twice");
        }
        if compat.payload(old).is_some() {
            return Err("a stale key found a payload");
        }
        if compat.payload(0).is_some() {
            return Err("key 0 found a payload");
        }
        let value = compat
            .payload(new)
            .and_then(Payload::cell::<u32>)
            .and_then(|cell| cell.try_borrow().ok().map(|v| *v));
        if value != Some(2) {
            return Err("the reused slot holds the wrong payload");
        }
        Ok(())
    }
}
