//! The core with a host of its own, no `Compat` (docs/luau-safe-vm-
//! embedding-core.md, section 5): roots, the freed lists, the budget for
//! declared payload sizes and the re-entry bound, as a host other than the
//! lane-6 API sees them.

use crate::error::VmResult;
use crate::heap::{Function, NativeClosure};
use crate::host::{HostError, HostedCtx};
use crate::hosted::PlainCtx;
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
    plain_calls: u32,
}

impl Host for Probe {}

fn hosted(limits: Limits) -> Result<HostedVm<Probe>, String> {
    let core = Core::with_limits_bare(limits).map_err(|_| "no memory for the core")?;
    Ok(HostedVm::from_core(core, Probe::default()))
}

fn host_native(vm: &mut HostedVm<Probe>, id: HostFnId, data: u32) -> VmResult<FnId> {
    vm.core.heap.new_function(Function::Native(NativeClosure {
        func: NativeFn::HostFull(id, data),
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

/// Pushes its data plus its argument count; fails when its data is 0.
fn plain_probe(ctx: &mut PlainCtx<'_, Probe>) -> Result<usize, HostError> {
    let host = ctx.host();
    host.plain_calls = host.plain_calls.saturating_add(1);
    if ctx.data() == 0 {
        return Err(HostError::new("plain probe failed"));
    }
    let n = f64::from(ctx.data()) + ctx.arg_count() as f64;
    ctx.push(Value::Number(n)).map_err(|_| HostError::memory())?;
    Ok(1)
}

#[test]
fn a_plain_host_function_runs_with_its_data_and_its_host() -> TestResult {
    let mut vm = hosted(Limits::default())?;
    let id = vm
        .add_host_fn(HostEntry::Plain(plain_probe), false)
        .map_err(|e| e.to_string())?;
    let f = vm.host_function("probe", id, 40).map_err(|e| e.to_string())?;
    let f = vm.core.roots.get(f).map_err(|e| e.to_string())?;
    let args = [Value::Number(1.0), Value::Number(2.0)];
    let result = vm.call_function(f, &args, 1).map_err(|_| "the call failed")?;
    check(matches!(result, Value::Number(n) if n == 42.0), "data plus argument count")?;
    let failing = vm.host_function("probe", id, 0).map_err(|e| e.to_string())?;
    let failing = vm.core.roots.get(failing).map_err(|e| e.to_string())?;
    check(
        vm.call_function(failing, &[], 1).is_err(),
        "a host error is a Lua error",
    )?;
    check(vm.host.plain_calls == 2, "both calls reached the host")
}

#[test]
fn a_host_function_needs_a_function_entry_of_its_host() -> TestResult {
    let mut vm = hosted(Limits::default())?;
    let k = vm
        .add_host_fn(HostEntry::Cont(|_, _| Ok(0)), false)
        .map_err(|e| e.to_string())?;
    check(vm.host_function("k", k, 0).is_err(), "a continuation entry is refused")?;
    check(
        vm.host_function("none", HostFnId::new(vm.roots.vm_id(), 7), 0)
            .is_err(),
        "an entry this host did not add is refused",
    )
}

fn nothing(_: &mut HostedCtx<'_, Probe>) -> Result<usize, HostError> {
    Ok(0)
}

#[test]
fn a_host_function_id_of_another_vm_is_refused() -> TestResult {
    let mut a = with_libs()?;
    let mut b = with_libs()?;
    let own = a
        .add_host_fn(HostEntry::Plain(plain_probe), false)
        .map_err(|e| e.to_string())?;
    let foreign = b
        .add_host_fn(HostEntry::Plain(plain_probe), false)
        .map_err(|e| e.to_string())?;
    check(own.index() == foreign.index(), "both are entry 0 of their table")?;
    check(
        a.host_function("b", foreign, 0).is_err(),
        "host_function refuses another VM's entry at the same index",
    )?;
    check(
        a.enter(|ctx| ctx.push_host_function("b", foreign, 0)).is_err(),
        "push_host_function refuses it",
    )?;
    let k = b
        .add_host_fn(HostEntry::Cont(|_, _| Ok(0)), false)
        .map_err(|e| e.to_string())?;
    check(
        a.add_host_fn(HostEntry::Full { f: nothing, k: Some(k) }, false)
            .is_err(),
        "a continuation of another VM is refused",
    )?;
    check(a.host_function("a", own, 0).is_ok(), "its own entry is accepted")
}

#[test]
fn a_custom_host_drains_its_freed_natives() -> TestResult {
    let mut vm = hosted(Limits::default())?;
    let owning = vm
        .add_host_fn(HostEntry::Plain(plain_probe), true)
        .map_err(|e| e.to_string())?;
    let root = vm
        .host_function("owned", owning, 41)
        .map_err(|e| e.to_string())?;
    collect(&mut vm);
    let mut freed = Vec::new();
    vm.drain_freed_natives(|_, id, data| freed.push((id, data)));
    check(freed.is_empty(), "a rooted native is not freed")?;
    check(vm.release_root(root), "the root is released")?;
    collect(&mut vm);
    vm.drain_freed_natives(|host, id, data| {
        host.plain_calls += 1;
        freed.push((id, data));
    });
    check(freed == vec![(owning, 41)], "the freed native is handed out")?;
    check(vm.host.plain_calls == 1, "with the host's state")?;
    vm.drain_freed_natives(|_, id, data| freed.push((id, data)));
    check(freed.len() == 1, "exactly once")
}

// ---- the stack API (stack.rs) and the driver's frame API -------------------

fn with_libs() -> Result<HostedVm<Probe>, String> {
    HostedVm::with_host(Limits::default(), Probe::default()).map_err(|e| e.to_string())
}

#[test]
fn a_frame_builds_tables_and_reads_them_back() -> TestResult {
    let mut vm = with_libs()?;
    let top = vm.core.top;
    let (root, pointer) = vm
        .enter(|ctx| {
            let mut s = ctx.stack();
            s.push_table(2, 2)?;
            s.push_number(0.5)?;
            s.raw_set_field(-2, b"gain")?;
            s.push_string(b"x")?;
            s.push_boolean(true)?;
            s.raw_set(-3)?;
            s.push_string(b"first")?;
            s.raw_set_index(-2, 1)?;
            let mut seen = Vec::new();
            s.push_nil()?;
            while s.next(1)? {
                seen.push((s.type_of(-2), s.type_of(-1)));
                s.pop(1)?;
            }
            if seen.len() != 3 || s.top() != 1 {
                return Err(HostError::raw("next saw the wrong entries"));
            }
            Ok((s.root(1)?, s.to_pointer(1)))
        })
        .map_err(|e| e.to_string())?;
    check(vm.core.top == top, "the frame is dropped")?;
    check(pointer != 0, "a table has an identity")?;
    let gain = vm
        .enter(|ctx| {
            let mut s = ctx.stack();
            s.push_root(root)?;
            check(s.to_pointer(1) == pointer, "the same table").map_err(HostError::raw)?;
            s.push_globals()?;
            s.pop(1)?;
            s.push_string(b"gain")?;
            s.push_copy(-1)?;
            s.pop(1)?;
            Ok(s.string(-1).map(<[u8]>::to_vec))
        })
        .map_err(|e| e.to_string())?;
    check(gain.as_deref() == Some(b"gain".as_slice()), "a string reads back")?;
    check(vm.core.release_root(root), "released")?;
    check(
        vm.enter(|ctx| ctx.stack().push_root(root)).is_err(),
        "a released root is refused",
    )
}

#[test]
fn a_buffer_is_written_and_read_in_place() -> TestResult {
    let mut vm = with_libs()?;
    let bytes = vm
        .enter(|ctx| {
            let mut s = ctx.stack();
            s.push_buffer(4)?.copy_from_slice(&[1, 2, 3, 4]);
            if let Some(first) = s.buffer_mut(1).and_then(|b| b.first_mut()) {
                *first = 9;
            }
            Ok(s.buffer(-1).map(<[u8]>::to_vec))
        })
        .map_err(|e| e.to_string())?;
    check(bytes == Some(vec![9, 2, 3, 4]), "the bytes are the buffer's")
}

#[test]
fn a_call_from_a_frame_runs_and_its_error_unwinds() -> TestResult {
    let mut vm = with_libs()?;
    let caught = vm
        .enter(|ctx| {
            ctx.stack().push_globals()?;
            ctx.push_field(-1, b"error")?;
            ctx.stack().push_string(b"boom")?;
            let status = ctx.pcall_top(1, 0)?;
            let message = ctx.stack().string(-1).map(<[u8]>::to_vec);
            // The error value replaced the function and its argument.
            let top = ctx.stack().top();
            ctx.push_field(1, b"tostring")?;
            ctx.stack().push_number(7.0)?;
            ctx.call_top(1, 1)?;
            let seven = ctx.stack().string(-1).map(<[u8]>::to_vec);
            Ok((status, message, top, seven))
        })
        .map_err(|e| e.to_string())?;
    check(
        matches!(caught.0, crate::host::CallStatus::Error(crate::error::ErrorKind::Runtime)),
        "pcall_top reports the error",
    )?;
    // `error` called from the host has no Lua position (`where_prefix`).
    check(
        caught.1.as_ref().is_some_and(|m| m.ends_with(b"boom")),
        "its value on top",
    )?;
    check(caught.2 == 2, "in place of the function and its argument")?;
    check(caught.3.as_deref() == Some(b"7".as_slice()), "a call returns its result")?;
    let failed = vm.enter(|ctx| {
        ctx.stack().push_globals()?;
        ctx.push_field(-1, b"error")?;
        ctx.stack().push_string(b"again")?;
        ctx.call_top(1, 0)?;
        Ok(())
    });
    check(
        matches!(&failed, Err(crate::error::Error::Lua { message, .. }) if message.ends_with(b"again")),
        "an uncaught error comes back from enter",
    )?;
    check(vm.core.frames.is_empty() && vm.core.reentry == 0, "and unwound")
}

#[test]
fn sandboxed_globals_refuse_a_raw_store() -> TestResult {
    let mut vm = with_libs()?;
    vm.core.sandbox_globals();
    let result = vm.enter(|ctx| {
        let mut s = ctx.stack();
        s.push_globals()?;
        s.push_number(1.0)?;
        s.raw_set_field(-2, b"x")
    });
    check(
        matches!(&result, Err(e) if e.message().contains("readonly")),
        "a read-only table is an error, not a store",
    )
}

/// A plain host function that returns a table with a method bound to its
/// data (as the audio engine binds channel methods).
fn make_counter(ctx: &mut PlainCtx<'_, Probe>) -> Result<usize, HostError> {
    let data = ctx.data();
    ctx.stack().push_table(0, 1)?;
    let id = HostFnId::new(ctx.roots.vm_id(), data);
    ctx.push_host_function("count", id, 5)?;
    ctx.stack().raw_set_field(-2, b"count")?;
    Ok(1)
}

#[test]
fn a_plain_host_function_makes_host_functions() -> TestResult {
    let mut vm = with_libs()?;
    let probe = vm
        .add_host_fn(HostEntry::Plain(plain_probe), false)
        .map_err(|e| e.to_string())?;
    let maker = vm
        .add_host_fn(HostEntry::Plain(make_counter), false)
        .map_err(|e| e.to_string())?;
    let n = vm
        .enter(|ctx| {
            ctx.push_host_function("make", maker, u32::try_from(probe.index()).unwrap_or(0))?;
            ctx.call_top(0, 1)?;
            ctx.push_field(-1, b"count")?;
            ctx.stack().push_number(0.0)?;
            ctx.call_top(1, 1)?;
            Ok(ctx.stack().to_number(-1))
        })
        .map_err(|e| e.to_string())?;
    check(n == Some(6.0), "the method ran with its data")?;
    check(
        vm.enter(|ctx| ctx.push_host_function("k", HostFnId::new(ctx.vm.core.roots.vm_id(), 99), 0))
            .is_err(),
        "an entry the host did not add is refused",
    )
}

/// The collector steps on the audio thread every block (`collect_step`):
/// a steady cycle keeps the root set's and the gray and weak lists'
/// storage instead of dropping and reallocating it (review round 1; the
/// allocator calls themselves are counted by a probe outside the crate,
/// which forbids the `unsafe` a counting allocator needs).
#[test]
fn a_steady_collection_cycle_keeps_the_collectors_storage() -> TestResult {
    let mut vm = with_libs()?;
    let cycle = |vm: &mut HostedVm<Probe>| -> TestResult {
        for _ in 0..100_000 {
            if vm.collect_step(0) {
                return Ok(());
            }
        }
        Err("a cycle finishes".to_owned())
    };
    cycle(&mut vm)?;
    cycle(&mut vm)?;
    let warm = vm.core.gc_storage();
    check(
        warm.first().is_some_and(|&(_, cap)| cap > 0),
        "the root set's storage is kept between steps",
    )?;
    for _ in 0..3 {
        cycle(&mut vm)?;
    }
    check(vm.core.gc_storage() == warm, "a steady cycle reallocates nothing")
}
