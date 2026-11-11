//! The embedding API without bytecode (the crate has no compiler): handles,
//! userdata, host functions and continuations, threads, registry, app data
//! and the memory budget. Scripted checks (the interrupt, userdata types
//! from Lua, host yields under coroutines) are in `tools/luau-vm-diff`.
//!
//! The tests return `Result` instead of unwrapping: the crate denies the
//! panic lints in tests too.

use std::cell::Cell;
use std::rc::Rc;

use crate::{
    AnyUserData, CallStatus, Error, ErrorKind, Function, HostError, Limits, MultiValue, Table,
    ThreadState, Value, Vm,
};

type TestResult = Result<(), Error>;

fn fail(message: &str) -> Error {
    Error::Host(message.to_owned())
}

fn check(condition: bool, message: &str) -> TestResult {
    if condition { Ok(()) } else { Err(fail(message)) }
}

/// Counts its drops: a userdata destructor.
struct Tracked(Rc<Cell<u32>>);

impl Drop for Tracked {
    fn drop(&mut self) {
        self.0.set(self.0.get().saturating_add(1));
    }
}

#[test]
fn a_handle_keeps_its_object_alive_and_releases_it_on_drop() -> TestResult {
    let mut vm = Vm::new()?;
    vm.collect();
    let baseline = vm.memory_bytes();
    let t = vm.create_table()?;
    t.set(&mut vm, "x", 42.0)?;
    let inner = vm.create_table()?;
    t.set(&mut vm, "inner", &inner)?;
    drop(inner);
    for _ in 0..3 {
        let stats = vm.collect();
        check(stats.tables_freed == 0, "a rooted table was freed")?;
    }
    let x: f64 = t.get(&mut vm, "x")?;
    check(x == 42.0, "the field survived")?;
    let inner: Table = t.get(&mut vm, "inner")?;
    check(inner.raw_len(&vm)? == 0, "the inner table is readable")?;
    drop(inner);
    drop(t);
    let stats = vm.collect();
    check(stats.tables_freed == 2, "both tables freed after the drop")?;
    check(vm.live_handles() == 0, "no live handles")?;
    check(vm.memory_bytes() <= baseline.saturating_add(64), "memory back to baseline")?;
    check(vm.accounting_drift() == 0, "no drift")
}

#[test]
fn a_userdata_destructor_runs_exactly_once() -> TestResult {
    let drops = Rc::new(Cell::new(0));
    let mut vm = Vm::new()?;
    let ud = vm.create_userdata(Tracked(Rc::clone(&drops)))?;
    vm.collect();
    check(drops.get() == 0, "alive while held")?;
    check(ud.is::<Tracked>(&vm)?, "type check")?;
    check(!ud.is::<u32>(&vm)?, "wrong type")?;
    drop(ud);
    let stats = vm.collect();
    check(stats.userdata_freed == 1, "freed by the collector")?;
    check(drops.get() == 1, "destructor ran")?;
    vm.collect();
    check(drops.get() == 1, "ran once")?;

    // Alive at VM drop: the VM's drop runs it, once.
    let _held = vm.create_userdata(Tracked(Rc::clone(&drops)))?;
    check(vm.accounting_drift() == 0, "no drift")?;
    drop(vm);
    check(drops.get() == 2, "destructor at VM drop")
}

#[test]
fn userdata_borrows_conflict_with_errors_not_panics() -> TestResult {
    let mut vm = Vm::new()?;
    let ud = vm.create_userdata(5u32)?;
    let cell = ud.cell::<u32>(&vm)?;
    let guard = cell.borrow_mut();
    let shared = ud.borrow(&vm, |v: &u32| *v);
    check(shared.is_err(), "conflicting borrow is an error")?;
    drop(guard);
    ud.borrow_mut(&vm, |v: &mut u32| *v = 9)?;
    check(ud.get_copy::<u32>(&vm)? == 9, "write through")?;
    check(ud.cell::<f64>(&vm).is_err(), "wrong type is an error")
}

#[test]
fn userdata_memory_is_charged_and_credited() -> TestResult {
    let mut vm = Vm::new()?;
    vm.collect();
    let before = vm.memory_bytes();
    let ud = vm.create_userdata_with_size(Vec::<u8>::new(), 4096)?;
    check(vm.memory_bytes() >= before.saturating_add(4096), "charged")?;
    drop(ud);
    vm.collect();
    check(vm.memory_bytes() <= before.saturating_add(64), "credited")?;
    check(vm.accounting_drift() == 0, "no drift")
}

#[test]
fn tagged_userdata_get_the_tag_metatable() -> TestResult {
    let mut vm = Vm::new()?;
    let mt = vm.create_table()?;
    mt.raw_set(&mut vm, "__type", "Mat4")?;
    vm.set_tag_metatable(62, Some(&mt))?;
    let ud = vm.create_userdata_tagged([0f32; 16], 62)?;
    check(ud.tag(&vm)? == 62, "tag")?;
    let got = ud.metatable(&mut vm)?.ok_or_else(|| fail("metatable"))?;
    check(vm.raw_equal(&Value::Table(got), &Value::Table(mt))?, "the tag's metatable")?;
    check(vm.create_userdata_tagged(0u8, 200).is_err(), "tag limit")
}

#[test]
fn typed_host_functions_convert_and_report_argument_errors() -> TestResult {
    let mut vm = Vm::new()?;
    let add = vm.create_function("add", |_, (a, b): (f64, f64)| Ok(a + b))?;
    let sum: f64 = add.call(&mut vm, (1.5, 2.0))?;
    check(sum == 3.5, "sum")?;
    let sum: f64 = add.call(&mut vm, ("4", 1.0))?;
    check(sum == 5.0, "numeric strings convert")?;
    let err = add.call::<_, f64>(&mut vm, (1.0, "x"));
    match err {
        Err(Error::Lua { kind: ErrorKind::Runtime, message }) => check(
            String::from_utf8_lossy(&message)
                .ends_with("invalid argument #2 to 'add' (number expected, got string)"),
            "luaL_typeerror text",
        )?,
        other => return Err(fail(&format!("expected an argument error, got {other:?}"))),
    }
    let err = add.call::<_, f64>(&mut vm, (1.0,));
    match err {
        Err(e) => check(
            e.message()
                .ends_with("missing argument #2 to 'add' (number expected)"),
            "missing argument text",
        )?,
        Ok(_) => return Err(fail("a missing argument converted")),
    }
    check(vm.accounting_drift() == 0, "no drift")
}

#[test]
fn host_errors_come_back_as_errors_with_their_status() -> TestResult {
    let mut vm = Vm::new()?;
    let raw = vm.create_host_function("raw", |_| Err(HostError::raw("plain")))?;
    match raw.call::<_, ()>(&mut vm, ()) {
        Err(Error::Lua { kind: ErrorKind::Runtime, message }) => {
            check(message == b"plain", "raw text")?
        }
        other => return Err(fail(&format!("{other:?}"))),
    }
    let oom = vm.create_host_function("oom", |_| Err(HostError::memory()))?;
    match oom.call::<_, ()>(&mut vm, ()) {
        Err(e) => check(e.is_memory(), "memory status kept")?,
        Ok(()) => return Err(fail("no error")),
    }
    let thrower = vm.create_function("thrower", move |_, (): ()| -> Result<(), HostError> {
        Err(HostError::from_value(Value::Number(7.0)))
    })?;
    match thrower.call::<_, ()>(&mut vm, ()) {
        Err(Error::Lua { message, .. }) => check(message == b"7", "value error")?,
        other => return Err(fail(&format!("{other:?}"))),
    }
    check(vm.accounting_drift() == 0, "no drift")
}

#[test]
fn reentry_past_the_limit_is_an_error() -> TestResult {
    let mut vm = Vm::with_limits(Limits {
        max_reentry: 8,
        ..Limits::default()
    })?;
    let depth = Rc::new(Cell::new(0u32));
    let seen = Rc::clone(&depth);
    let recurse = vm.create_host_function("recurse", move |ctx| {
        seen.set(seen.get().saturating_add(1));
        let me: Function = ctx.get(1)?;
        me.call::<_, ()>(ctx, &me)?;
        Ok(0)
    })?;
    match recurse.call::<_, ()>(&mut vm, &recurse) {
        Err(e) => check(e.message().contains("C stack overflow"), "overflow text")?,
        Ok(()) => return Err(fail("unbounded recursion returned")),
    }
    check(depth.get() == 8, "stopped at max_reentry")?;
    // The VM is usable afterwards.
    let ok = vm.create_function("ok", |_, (): ()| Ok(1.0))?;
    check(ok.call::<_, f64>(&mut vm, ())? == 1.0, "usable")?;
    check(vm.accounting_drift() == 0, "no drift")
}

#[test]
fn a_host_callback_that_yields_where_it_cannot_gets_an_error() -> TestResult {
    let mut vm = Vm::new()?;
    let y = vm.create_host_function("y", |ctx| {
        ctx.push(1.0)?;
        ctx.yield_values(1)
    })?;
    match y.call::<_, ()>(&mut vm, ()) {
        Err(e) => check(
            e.message()
                .contains("attempt to yield across metamethod/C-call boundary"),
            "yield error text",
        )?,
        Ok(()) => return Err(fail("a yield from a host call returned")),
    }
    check(vm.accounting_drift() == 0, "no drift")
}

#[test]
fn a_thread_yields_from_a_host_function_and_continues() -> TestResult {
    let mut vm = Vm::new()?;
    // Yields `base + 1`, then on resume returns `base + resume value`.
    let body = vm.create_host_function_k(
        "body",
        |ctx| {
            let base = ctx.check_number(1)?;
            ctx.push(base + 1.0)?;
            ctx.yield_values(1)
        },
        |ctx, status| {
            if status != CallStatus::Ok {
                return Err(HostError::raw("unexpected status"));
            }
            let base = ctx.check_number(1)?;
            let resumed: f64 = ctx.get(-1)?;
            ctx.push(base + resumed)?;
            Ok(1)
        },
    )?;
    let co = vm.create_thread(&body)?;
    check(co.status(&vm)? == ThreadState::Resumable, "fresh")?;
    let first: f64 = co.resume(&mut vm, 10.0)?;
    check(first == 11.0, "yielded value")?;
    check(co.status(&vm)? == ThreadState::Resumable, "suspended")?;
    let second: f64 = co.resume(&mut vm, 5.0)?;
    check(second == 15.0, "continuation result")?;
    check(co.status(&vm)? == ThreadState::Finished, "finished")?;
    match co.resume::<_, ()>(&mut vm, ()) {
        Err(e) => check(e.message().contains("cannot resume dead coroutine"), "dead")?,
        Ok(()) => return Err(fail("resumed a dead thread")),
    }
    check(vm.accounting_drift() == 0, "no drift")
}

#[test]
fn a_failing_thread_reports_its_error_and_status() -> TestResult {
    let mut vm = Vm::new()?;
    let body = vm.create_host_function("body", |_| Err(HostError::raw("boom")))?;
    let co = vm.create_thread(&body)?;
    match co.resume::<_, ()>(&mut vm, ()) {
        Err(Error::Lua { kind: ErrorKind::Runtime, message }) => {
            check(message == b"boom", "message")?
        }
        other => return Err(fail(&format!("{other:?}"))),
    }
    check(co.status(&vm)? == ThreadState::Error, "error status")
}

#[test]
fn call_yieldable_runs_the_continuation_with_the_results() -> TestResult {
    let mut vm = Vm::new()?;
    let double = vm.create_function("double", |_, x: f64| Ok(x * 2.0))?;
    let outer = vm.create_host_function_k(
        "outer",
        |ctx| {
            ctx.push_copy(1)?;
            ctx.push_copy(2)?;
            ctx.call_yieldable(1, 1)
        },
        |ctx, status| {
            check_status(status, CallStatus::Ok)?;
            let r: f64 = ctx.get(-1)?;
            ctx.push(r + 1.0)?;
            Ok(1)
        },
    )?;
    let r: f64 = outer.call(&mut vm, (&double, 20.0))?;
    check(r == 41.0, "continuation saw the result")?;
    check(vm.accounting_drift() == 0, "no drift")
}

fn check_status(got: CallStatus, want: CallStatus) -> Result<(), HostError> {
    if got == want {
        Ok(())
    } else {
        Err(HostError::raw(format!("status {got:?}, expected {want:?}")))
    }
}

#[test]
fn pcall_yieldable_hands_the_error_to_the_continuation() -> TestResult {
    let mut vm = Vm::new()?;
    let fails = vm.create_host_function("fails", |_| Err(HostError::raw("bad")))?;
    let outer = vm.create_host_function_k(
        "outer",
        |ctx| {
            ctx.push(7.0)?; // state at index 2
            ctx.push_copy(1)?;
            ctx.pcall_yieldable(0, 1)
        },
        |ctx, status| {
            check_status(status, CallStatus::Error(ErrorKind::Runtime))?;
            let state: f64 = ctx.get(2)?;
            let message: String = ctx.get(-1)?;
            if ctx.top() != 3 {
                return Err(HostError::raw(format!("top {}", ctx.top())));
            }
            ctx.push(format!("{state}:{message}"))?;
            Ok(1)
        },
    )?;
    let r: String = outer.call(&mut vm, &fails)?;
    check(r == "7:bad", "state and error")?;
    check(vm.accounting_drift() == 0, "no drift")
}

#[test]
fn the_stack_api_matches_the_c_api() -> TestResult {
    let mut vm = Vm::new()?;
    let f = vm.create_host_function("f", |ctx| {
        // [1, 2, 3]
        ctx.push(4.0)?;
        ctx.insert(1)?; // [4, 1, 2, 3]
        ctx.remove(2)?; // [4, 2, 3]
        ctx.push(9.0)?;
        ctx.replace(3)?; // [4, 2, 9]
        ctx.set_top(4)?; // [4, 2, 9, nil]
        ctx.push_copy(-4)?; // [4, 2, 9, nil, 4]
        Ok(ctx.top())
    })?;
    let out: MultiValue = f.call(&mut vm, (1.0, 2.0, 3.0))?;
    let got: Vec<Option<f64>> = out.0.iter().map(Value::as_number).collect();
    check(
        got == [Some(4.0), Some(2.0), Some(9.0), None, Some(4.0)],
        &format!("stack {got:?}"),
    )
}

#[test]
fn tables_honor_metamethods_through_handles() -> TestResult {
    let mut vm = Vm::new()?;
    let t = vm.create_table()?;
    let mt = vm.create_table()?;
    let index = vm.create_function("index", |_, (_t, k): (Table, String)| Ok(format!("<{k}>")))?;
    mt.raw_set(&mut vm, "__index", &index)?;
    t.set_metatable(&mut vm, Some(&mt))?;
    let v: String = t.get(&mut vm, "abc")?;
    check(v == "<abc>", "__index ran")?;
    let raw: Value = t.raw_get(&mut vm, "abc")?;
    check(raw.is_nil(), "raw get skips it")?;
    t.raw_set(&mut vm, 1.0, "a")?;
    t.raw_push(&mut vm, "b")?;
    check(t.raw_len(&vm)? == 2, "border")?;
    let seq: Vec<String> = t.sequence_values(&mut vm)?;
    check(seq == ["a", "b"], "sequence")?;
    let pairs: Vec<(Value, Value)> = t.pairs(&mut vm)?;
    check(pairs.len() == 2, "pairs")?;
    t.set_readonly(&mut vm, true)?;
    check(t.raw_set(&mut vm, "z", 1.0).is_err(), "readonly refuses")?;
    check(t.is_readonly(&vm)?, "readonly")?;
    check(vm.accounting_drift() == 0, "no drift")
}

#[test]
fn handles_are_vm_scoped() -> TestResult {
    let mut a = Vm::new()?;
    let mut b = Vm::new()?;
    let t = a.create_table()?;
    match t.set(&mut b, "x", 1.0) {
        Err(Error::Host(m)) => check(m.contains("other than the one"), "wrong-VM text")?,
        other => return Err(fail(&format!("{other:?}"))),
    }
    check(a.state_identity() != b.state_identity(), "identities differ")?;
    // A handle made on a coroutine stays valid on the main thread.
    let made = Rc::new(Cell::new(false));
    let flag = Rc::clone(&made);
    let maker = a.create_host_function("maker", move |ctx| {
        let t = ctx.create_table()?;
        t.raw_set(ctx, "from", "coroutine")?;
        ctx.set_named_registry_value("made", &t)?;
        flag.set(true);
        Ok(0)
    })?;
    let co = a.create_thread(&maker)?;
    co.resume::<_, ()>(&mut a, ())?;
    drop(co);
    a.collect();
    let t: Table = a.named_registry_value("made")?;
    let from: String = t.raw_get(&mut a, "from")?;
    check(made.get() && from == "coroutine", "read from the main thread")
}

#[test]
fn registry_keys_and_app_data() -> TestResult {
    let mut vm = Vm::new()?;
    let t = vm.create_table()?;
    t.raw_set(&mut vm, 1.0, "kept")?;
    let key = vm.create_registry_value(&t)?;
    drop(t);
    vm.collect();
    let t: Table = vm.registry_value(&key)?;
    let v: String = t.raw_get(&mut vm, 1.0)?;
    check(v == "kept", "registry value survived")?;
    drop(t);
    vm.remove_registry_value(key)?;
    let stats = vm.collect();
    check(stats.tables_freed == 1, "freed after unref")?;

    check(vm.set_app_data(Cell::new(3u32)).is_none(), "first set")?;
    let data = vm.app_data::<Cell<u32>>().ok_or_else(|| fail("app data"))?;
    data.set(4);
    let again = vm.app_data::<Cell<u32>>().ok_or_else(|| fail("app data"))?;
    check(again.get() == 4, "shared")?;
    check(vm.remove_app_data::<Cell<u32>>().is_some(), "removed")?;
    check(vm.app_data::<Cell<u32>>().is_none(), "gone")
}

#[test]
fn the_memory_limit_is_an_error_with_no_drift() -> TestResult {
    let mut vm = Vm::new()?;
    vm.collect();
    let used = vm.memory_bytes();
    vm.set_memory_limit(Some(used.saturating_add(16 * 1024)));
    let holder = vm.create_table()?;
    let mut hit = None;
    for i in 0..100_000u32 {
        let t = match vm.create_table_with_capacity(16, 0) {
            Ok(t) => t,
            Err(e) => {
                hit = Some(e);
                break;
            }
        };
        if let Err(e) = holder.raw_set(&mut vm, f64::from(i) + 1.0, t) {
            hit = Some(e);
            break;
        }
    }
    let error = hit.ok_or_else(|| fail("the limit never hit"))?;
    check(error.is_memory(), "a memory error")?;
    check(error.message() == "not enough memory", "its text")?;
    check(vm.memory_exhausted(), "the flag is set")?;
    check(vm.accounting_drift() == 0, "no drift at the limit")?;
    drop(holder);
    vm.collect();
    check(vm.accounting_drift() == 0, "no drift after collection")?;
    vm.clear_memory_exhausted();
    check(!vm.memory_exhausted(), "cleared")?;
    check(vm.create_table().is_ok(), "allocates again")?;
    vm.set_memory_limit(None);
    Ok(())
}

#[test]
fn values_coerce_as_the_c_api_does() -> TestResult {
    let mut vm = Vm::new()?;
    let s = vm
        .coerce_string(&Value::Number(0.1))?
        .ok_or_else(|| fail("number to string"))?;
    check(s.to_str(&vm)? == "0.1", "%.14g")?;
    check(vm.coerce_string(&Value::Boolean(true))?.is_none(), "no bool")?;
    let s = vm.create_string("12.5")?;
    check(vm.coerce_number(&Value::String(s))? == Some(12.5), "string to number")?;
    let b = vm.create_buffer_from(b"abcd")?;
    check(vm.objlen(&Value::Buffer(b.clone()))? == 4, "buffer objlen")?;
    b.with_bytes_mut(&mut vm, |bytes| {
        if let Some(first) = bytes.first_mut() {
            *first = b'z';
        }
    })?;
    check(b.read_bytes(&vm, 0, 2)? == b"zb", "in place")?;
    check(b.read_bytes(&vm, 3, 2).is_err(), "out of bounds")?;
    let ud: AnyUserData = vm.create_userdata(1u8)?;
    check(vm.to_pointer(&Value::UserData(ud.clone()))? != 0, "identity")?;
    let t = vm.create_table()?;
    let s = vm.tostring(&Value::Table(t))?.to_str(&vm)?;
    check(s.starts_with("table: "), "tostring")
}

/// Review round 1: a converted key must stay rooted while the value
/// converts. Converting a large string under a tight limit runs the
/// collector; an unrooted key string was swept and its slot reused.
#[test]
fn a_key_stays_rooted_while_its_value_converts() -> TestResult {
    let mut vm = Vm::new()?;
    let t = vm.create_table()?;
    let big = "v".repeat(110 * 1024);
    for i in 0..16 {
        vm.collect();
        vm.set_memory_limit(Some(vm.used_memory().saturating_add(200 * 1024)));
        let key = format!("rooted_key_{i}");
        t.raw_set(&mut vm, key.as_str(), big.as_str())?;
        t.set(&mut vm, format!("set_{key}").as_str(), big.as_str())?;
        let _other = vm.create_string(format!("unrelated_{i}"))?;
        let raw: Option<String> = t.raw_get(&mut vm, key.as_str())?;
        check(raw.map(|s| s.len()) == Some(big.len()), "raw_set key survives")?;
        let via_set: Option<String> = t.raw_get(&mut vm, format!("set_{key}").as_str())?;
        check(via_set.map(|s| s.len()) == Some(big.len()), "set key survives")?;
        t.raw_set(&mut vm, key.as_str(), Value::Nil)?;
        t.raw_set(&mut vm, format!("set_{key}").as_str(), Value::Nil)?;
        vm.set_memory_limit(None);
    }
    vm.collect();
    check(vm.accounting_drift() == 0, "no drift")
}

/// Review round 1: without a continuation, `call_yieldable`'s results are
/// the function's results even when the callee sat in an argument slot.
#[test]
fn call_yieldable_without_a_continuation_returns_the_call_results() -> TestResult {
    let mut vm = Vm::new()?;
    let double = vm.create_function("double", |_, x: f64| Ok(x * 2.0))?;
    let relay = vm.create_host_function("relay", |ctx| ctx.call_yieldable(1, -1))?;
    let got: f64 = relay.call(&mut vm, (double.clone(), 21.0))?;
    check(got == 42.0, "relay returns the callee's result")?;
    let relay_pcall = vm.create_host_function("relay_pcall", |ctx| ctx.pcall_yieldable(1, -1))?;
    let got: (f64, Value) = relay_pcall.call(&mut vm, (double, 4.0))?;
    check(got.0 == 8.0 && got.1.is_nil(), "pcall relay returns one result")?;
    let fails = vm.create_host_function("fails", |_| Err(HostError::raw("boom")))?;
    let got: String = relay_pcall.call(&mut vm, (fails, 0.0))?;
    check(got == "boom", "a failed protected call returns the error value")
}

/// Review round 1: native integers convert exactly; out-of-range numbers
/// are errors, including the bound `MAX as f64` rounds up to.
#[test]
fn integer_conversions_are_exact_and_range_checked() -> TestResult {
    use crate::FromLua;
    let mut vm = Vm::new()?;
    let exact = i64::from_lua(Value::Integer(9_007_199_254_740_993), &mut vm)?;
    check(exact == 9_007_199_254_740_993, "i64 from an integer is exact")?;
    check(
        u8::from_lua(Value::Integer(256), &mut vm).is_err(),
        "u8 from integer 256 fails",
    )?;
    check(
        i64::from_lua(Value::Number(2f64.powi(63)), &mut vm).is_err(),
        "i64 from 2^63 fails",
    )?;
    check(
        i64::from_lua(Value::Number(-(2f64.powi(63))), &mut vm)? == i64::MIN,
        "i64 from -2^63 is MIN",
    )?;
    check(
        u64::from_lua(Value::Number(2f64.powi(64)), &mut vm).is_err(),
        "u64 from 2^64 fails",
    )?;
    check(u8::from_lua(Value::Number(255.9), &mut vm)? == 255, "u8 truncates")?;
    check(u8::from_lua(Value::Number(-0.5), &mut vm)? == 0, "-0.5 truncates to 0")?;
    check(
        u8::from_lua(Value::Number(-1.0), &mut vm).is_err(),
        "u8 from -1 fails",
    )?;
    check(
        i32::from_lua(Value::Number(f64::NAN), &mut vm).is_err(),
        "NaN fails",
    )
}
