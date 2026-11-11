//! The core with a host of its own, no `Compat` (docs/luau-safe-vm-
//! embedding-core.md, section 5): roots, the freed lists, the budget for
//! declared payload sizes and the re-entry bound, as a host other than the
//! lane-6 API sees them.

use crate::error::VmResult;
use crate::heap::{Function, NativeClosure};
use crate::host::{HostError, HostedCtx};
use crate::hosted::{Host, HostEntry, HostedVm};
use crate::limits::Limits;
use crate::native::{HostFnId, NativeFn};
use crate::roots::Root;
use crate::userdata::Userdata;
use crate::value::{FnId, Value};
use crate::vm::Core;

type TestResult = Result<(), String>;

fn check(ok: bool, what: &str) -> TestResult {
    if ok { Ok(()) } else { Err(what.to_owned()) }
}

/// A host with one function entry that calls itself back.
#[derive(Default)]
struct Probe {
    /// The function itself, rooted.
    me: Option<Root>,
    depth: u32,
    deepest: u32,
}

impl Host for Probe {}

fn hosted(limits: Limits) -> Result<HostedVm<Probe>, String> {
    let core = Core::with_limits_bare(limits).map_err(|_| "no memory for the core")?;
    Ok(HostedVm::from_core(core, Probe::default()))
}

fn host_native(vm: &mut HostedVm<Probe>, id: HostFnId, data: u32) -> VmResult<FnId> {
    vm.core.heap.new_function(Function::Native(NativeClosure {
        func: NativeFn::Host(id, data),
        name: Box::from("probe"),
        upvals: Box::default(),
    }))
}

fn collect(vm: &mut HostedVm<Probe>) {
    vm.core.full_gc();
}

#[test]
fn a_root_keeps_its_value_until_released() -> TestResult {
    let mut vm = hosted(Limits::default())?;
    let t = vm.core.heap.new_table(0, 0).map_err(|_| "table")?;
    let root = vm.core.roots.insert(Value::Table(t)).map_err(|e| e.to_string())?;
    let stats = vm.core.full_gc();
    check(stats.tables_freed == 0, "a rooted table was freed")?;
    check(vm.core.roots.release(root), "released")?;
    let stats = vm.core.full_gc();
    check(stats.tables_freed == 1, "the released table was freed")?;
    check(vm.core.roots.get(root).is_err(), "the root is stale")
}

#[test]
fn a_freed_userdata_key_and_owned_native_are_listed_exactly_once() -> TestResult {
    let mut vm = hosted(Limits::default())?;
    let owned = vm
        .add_host_fn(HostEntry::Full { f: |_| Ok(0), k: None }, true)
        .map_err(|e| e.to_string())?;
    let plain = vm
        .add_host_fn(HostEntry::Full { f: |_| Ok(0), k: None }, false)
        .map_err(|e| e.to_string())?;
    host_native(&mut vm, owned, 42).map_err(|_| "native")?;
    host_native(&mut vm, plain, 7).map_err(|_| "native")?;
    vm.core
        .heap
        .new_userdata(Userdata {
            key: 99,
            meta: None,
            tag: 0,
            size: 16,
        })
        .map_err(|_| "userdata")?;
    collect(&mut vm);
    let natives = vm.core.heap.take_freed_natives();
    let keys = vm.core.heap.take_freed_userdata();
    check(natives == [(owned, 42)], "only the owning entry's native is listed")?;
    check(keys == [99], "the userdata's key is listed")?;
    collect(&mut vm);
    check(
        vm.core.heap.take_freed_natives().is_empty() && vm.core.heap.take_freed_userdata().is_empty(),
        "nothing is listed twice",
    )
}

#[test]
fn a_declared_payload_size_is_charged_and_credited() -> TestResult {
    let limits = Limits {
        max_memory_bytes: Some(256 * 1024),
        ..Limits::default()
    };
    let mut vm = hosted(limits)?;
    collect(&mut vm);
    let before = vm.core.heap.total_bytes();
    let ud = |size| Userdata {
        key: 1,
        meta: None,
        tag: 0,
        size,
    };
    check(
        vm.core.heap.new_userdata(ud(1 << 20)).is_err(),
        "a payload over the budget is a memory error",
    )?;
    check(vm.core.heap.total_bytes() == before, "the refused payload was not charged")?;
    vm.core.heap.new_userdata(ud(64 * 1024)).map_err(|_| "userdata")?;
    check(
        vm.core.heap.total_bytes() >= before.saturating_add(64 * 1024),
        "the declared size is charged",
    )?;
    collect(&mut vm);
    check(vm.core.heap.total_bytes() == before, "and credited when collected")?;
    check(vm.core.heap.recount() == vm.core.heap.total_bytes(), "no drift")
}

/// Calls itself back through the driver until the core refuses.
fn reenter(ctx: &mut HostedCtx<'_, Probe>) -> Result<usize, HostError> {
    let host = &mut ctx.vm.host;
    host.depth = host.depth.saturating_add(1);
    host.deepest = host.deepest.max(host.depth);
    let me = host.me.ok_or_else(|| HostError::new("no function"))?;
    let f = ctx.vm.core.roots.get(me).map_err(HostError::from)?;
    let result = ctx.vm.call_function(f, &[], 0);
    ctx.vm.host.depth = ctx.vm.host.depth.saturating_sub(1);
    match result {
        Ok(_) => Ok(0),
        Err(_) => Err(HostError::new("C stack overflow")),
    }
}

#[test]
fn reentry_from_a_host_function_is_bounded_and_unwinds() -> TestResult {
    let limits = Limits {
        max_reentry: 8,
        ..Limits::default()
    };
    let mut vm = hosted(limits)?;
    let id = vm
        .add_host_fn(HostEntry::Full { f: reenter, k: None }, false)
        .map_err(|e| e.to_string())?;
    let f = host_native(&mut vm, id, 0).map_err(|_| "native")?;
    vm.host.me = Some(vm.core.roots.insert(Value::Function(f)).map_err(|e| e.to_string())?);
    let result = vm.call_function(Value::Function(f), &[], 0);
    check(result.is_err(), "the recursion ends in an error")?;
    check(vm.host.deepest == 8, "each nesting up to the limit ran")?;
    check(vm.host.depth == 0, "every level returned")?;
    check(vm.core.reentry == 0, "the re-entry count is back to zero")?;
    check(vm.core.host_depth == 0, "the host depth is back to zero")
}
