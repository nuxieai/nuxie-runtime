//! The base library (C++ `lbaselib.cpp`), Rive profile: no `print`, no
//! `newproxy`. `print`, `loadstring` and `collectgarbage` come from the host
//! (the conformance harness registers them).
//!
//! The builtins' fast bodies (`fastcall.rs`, C++ `luauF_type` and the
//! rest) sit beside their library functions. Two take a different path
//! from the library on purpose, as upstream's do: `tostring` renders
//! `nil`, booleans, numbers, integers and strings without looking for
//! `__tostring`, and `tonumber` gives `nil` for an integer.

use crate::call::{CallMode, Catch, Continuation, Outcome, Resume, Saved};
use crate::error::{ErrorKind, VmResult};
use crate::heap::Function;
use crate::meta::Meta;
use crate::native::{Args, plain, resumable};
use crate::value::Value;
use crate::vm::{MULTRET, Core};

pub(crate) fn open(vm: &mut Core) -> VmResult<()> {
    let globals = vm.globals;
    vm.raw_set_str(globals, "_G", Value::Table(globals))?;
    let version = vm.heap.intern(b"Luau")?;
    vm.raw_set_str(globals, "_VERSION", Value::String(version))?;
    vm.register_builtins(
        globals,
        &[
            ("assert", plain(assert)),
            ("error", plain(error)),
            ("gcinfo", plain(gcinfo)),
            ("getfenv", plain(getfenv)),
            ("getmetatable", plain(getmetatable)),
            ("next", plain(next)),
            ("rawequal", plain(rawequal)),
            ("rawget", plain(rawget)),
            ("rawset", plain(rawset)),
            ("rawlen", plain(rawlen)),
            ("select", plain(select)),
            ("setfenv", plain(setfenv)),
            ("setmetatable", plain(setmetatable)),
            ("tonumber", plain(tonumber)),
            ("tostring", resumable(tostring)),
            ("type", plain(type_)),
            ("typeof", plain(typeof_)),
            ("pcall", resumable(pcall)),
            ("xpcall", resumable(xpcall)),
        ],
    )?;
    // C++ `auxopen`: the iterator is a separate, nameless closure.
    let inext_fn = vm.new_builtin("", plain(inext), &[])?;
    let ipairs_fn = vm.new_builtin("ipairs", plain(ipairs), &[inext_fn])?;
    vm.raw_set_str(globals, "ipairs", ipairs_fn)?;
    let next_fn = vm.new_builtin("", plain(next), &[])?;
    let pairs_fn = vm.new_builtin("pairs", plain(pairs), &[next_fn])?;
    vm.raw_set_str(globals, "pairs", pairs_fn)?;
    Ok(())
}

/// `luauF_assert`: a true value with no results wanted.
pub(crate) fn fast_assert(
    _: &mut Core,
    _: usize,
    nresults: i32,
    [a0, ..]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    (nparams >= 1 && nresults == 0 && a0.truthy()).then_some(0)
}

fn assert(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_any(args, 1)?;
    if !v.truthy() {
        let message = match vm.opt_string(args, 2)? {
            Some(id) => crate::vm::c_lossy(vm.heap.str_bytes(id)),
            None => "assertion failed!".to_owned(),
        };
        return Err(vm.lib_error(message));
    }
    Ok(args.nargs)
}

fn error(vm: &mut Core, args: Args) -> VmResult<usize> {
    let level = vm.opt_integer(args, 2, 1)?;
    let message = vm.arg_or_nil(args, 1);
    let value = match message {
        Value::String(_) | Value::Number(_) if level > 0 => {
            let mut text = vm.where_prefix(usize::try_from(level).unwrap_or(0));
            if let Some(id) = vm.coerce_to_str(message)? {
                text.extend_from_slice(vm.heap.str_bytes(id));
            }
            Value::String(vm.heap.intern(&text)?)
        }
        other => other,
    };
    Err(crate::error::Throw {
        value,
        kind: ErrorKind::Runtime,
    })
}

fn gcinfo(vm: &mut Core, _args: Args) -> VmResult<usize> {
    let kb = vm.heap.total_bytes() / 1024;
    vm.ret(&[Value::Number(kb as f64)])
}

/// C++ `getfunc`: the function argument 1 names, by value or by level.
fn getfunc(vm: &mut Core, args: Args, optional: bool) -> VmResult<Value> {
    if let Some(f @ Value::Function(_)) = vm.arg(args, 1) {
        return Ok(f);
    }
    let level = if optional {
        vm.opt_integer(args, 1, 1)?
    } else {
        vm.check_integer(args, 1)?
    };
    let Ok(level) = usize::try_from(level) else {
        return Err(vm.arg_error(1, "level must be non-negative"));
    };
    let closure = vm.frames_observed().at_level(level).map(|(_, f)| f.closure);
    match closure {
        Some(closure) => Ok(Value::Function(closure)),
        None => Err(vm.arg_error(1, "invalid level")),
    }
}

fn getfenv(vm: &mut Core, args: Args) -> VmResult<usize> {
    let f = getfunc(vm, args, true)?;
    let env = match f {
        Value::Function(id) => match vm.heap.function(id) {
            Function::Lua(c) => c.env,
            Function::Native(_) => vm.globals,
        },
        _ => vm.globals,
    };
    vm.set_safeenv(env, false);
    vm.ret(&[Value::Table(env)])
}

fn setfenv(vm: &mut Core, args: Args) -> VmResult<usize> {
    let table = vm.check_table(args, 2)?;
    let f = getfunc(vm, args, false)?;
    vm.set_safeenv(table, false);
    // Upstream: `lua_isnumber(L, 1) && lua_tonumber(L, 1) == 0`, so a
    // numeric string counts.
    if vm.arg(args, 1).and_then(|v| vm.to_number(v)) == Some(0.0) {
        vm.globals = table;
        return Ok(0);
    }
    if let Value::Function(id) = f
        && let Function::Lua(c) = vm.heap.function_mut(id)
    {
        c.env = table;
        vm.deoptimize();
        return vm.ret(&[f]);
    }
    Err(vm.lib_error("'setfenv' cannot change environment of given object"))
}

/// What `getmetatable(v)` gives: the metatable's `__metatable` field if
/// set, else the metatable, else nil.
fn metatable_view(vm: &Core, v: Value) -> Value {
    let Some(mt) = vm.metatable(v) else {
        return Value::Nil;
    };
    let protected = vm.heap.table(mt).get_str(vm.tm(crate::vm::Tm::Metatable));
    if protected.is_nil() { Value::Table(mt) } else { protected }
}

fn getmetatable(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_any(args, 1)?;
    let r = metatable_view(vm, v);
    vm.ret(&[r])
}

pub(crate) fn fast_getmetatable(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, ..]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    if nparams < 1 || nresults > 1 {
        return None;
    }
    let r = metatable_view(vm, a0);
    vm.set(ra, r);
    Some(1)
}

fn setmetatable(vm: &mut Core, args: Args) -> VmResult<usize> {
    let mt_arg = vm.arg(args, 2);
    let t = vm.check_table(args, 1)?;
    let mt = match mt_arg {
        Some(Value::Nil) => None,
        Some(Value::Table(mt)) => Some(mt),
        _ => return Err(vm.arg_type_error(args, 2, "nil or table")),
    };
    if let Some(old) = vm.heap.table(t).meta
        && !vm
            .heap
            .table(old)
            .get_str(vm.tm(crate::vm::Tm::Metatable))
            .is_nil()
    {
        return Err(vm.lib_error("cannot change a protected metatable"));
    }
    if vm.heap.table(t).readonly {
        return Err(vm.runtime_error("attempt to modify a readonly table"));
    }
    vm.heap.table_mut(t).meta = mt;
    vm.ret(&[Value::Table(t)])
}

/// `luauF_setmetatable`: a table without a metatable (replacing one needs
/// the `__metatable` check) that is not read-only.
pub(crate) fn fast_setmetatable(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, a1, _]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    let (Value::Table(t), Value::Table(mt)) = (a0, a1) else { return None };
    if nparams < 2 || nresults > 1 {
        return None;
    }
    let table = vm.heap.table(t);
    if table.readonly || table.meta.is_some() {
        return None;
    }
    // `table_mut` passes the write barrier (C++ `luaC_objbarrier`).
    vm.heap.table_mut(t).meta = Some(mt);
    vm.set(ra, Value::Table(t));
    Some(1)
}

fn next(vm: &mut Core, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let key = vm.arg_or_nil(args, 2);
    let table = vm.heap.table(t);
    let Some(cursor) = table.cursor_after(key) else {
        return Err(vm.runtime_error("invalid key to 'next'"));
    };
    match table.next_from(cursor) {
        Some((_, k, v)) => vm.ret(&[k, v]),
        None => vm.ret(&[Value::Nil]),
    }
}

fn pairs(vm: &mut Core, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let f = vm.builtin_upval(0);
    vm.ret(&[f, Value::Table(t), Value::Nil])
}

fn ipairs(vm: &mut Core, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let f = vm.builtin_upval(0);
    vm.ret(&[f, Value::Table(t), Value::Number(0.0)])
}

fn inext(vm: &mut Core, args: Args) -> VmResult<usize> {
    let i = vm.check_integer(args, 2)?;
    let t = vm.check_table(args, 1)?;
    let i = i.wrapping_add(1);
    let v = vm.heap.table(t).get_int(i64::from(i));
    if v.is_nil() {
        Ok(0)
    } else {
        vm.ret(&[Value::Number(f64::from(i)), v])
    }
}

fn rawequal(vm: &mut Core, args: Args) -> VmResult<usize> {
    let a = vm.check_any(args, 1)?;
    let b = vm.check_any(args, 2)?;
    vm.ret(&[Value::Boolean(a.raw_eq(b))])
}

pub(crate) fn fast_rawequal(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, a1, _]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    if nparams < 2 || nresults > 1 {
        return None;
    }
    vm.set(ra, Value::Boolean(a0.raw_eq(a1)));
    Some(1)
}

fn rawget(vm: &mut Core, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let key = vm.check_any(args, 2)?;
    let v = vm.heap.table(t).get(key);
    vm.ret(&[v])
}

pub(crate) fn fast_rawget(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, a1, _]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    let Value::Table(t) = a0 else { return None };
    if nparams < 2 || nresults > 1 {
        return None;
    }
    let v = vm.heap.table(t).get(a1);
    vm.set(ra, v);
    Some(1)
}

fn rawset(vm: &mut Core, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let key = vm.check_any(args, 2)?;
    let value = vm.check_any(args, 3)?;
    if vm.heap.table(t).readonly {
        return Err(vm.runtime_error("attempt to modify a readonly table"));
    }
    vm.raw_set(t, key, value)?;
    vm.ret(&[Value::Table(t)])
}

/// `luauF_rawset`: falls through for a key the store refuses (nil, NaN, a
/// vector with a NaN) and for a read-only table, so the library raises.
pub(crate) fn fast_rawset(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, key, value]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    let Value::Table(t) = a0 else { return None };
    if nparams < 3 || nresults > 1 {
        return None;
    }
    match key {
        Value::Nil => return None,
        Value::Number(n) if n.is_nan() => return None,
        Value::Vector(v) if v.iter().any(|c| c.is_nan()) => return None,
        _ => {}
    }
    if vm.heap.table(t).readonly {
        return None;
    }
    vm.raw_set(t, key, value).ok()?;
    vm.set(ra, Value::Table(t));
    Some(1)
}

fn rawlen(vm: &mut Core, args: Args) -> VmResult<usize> {
    let n = match vm.arg(args, 1) {
        Some(Value::Table(t)) => vm.heap.table(t).len(),
        Some(Value::String(s)) => vm.heap.str_bytes(s).len(),
        _ => return Err(vm.arg_error(1, "table or string expected")),
    };
    vm.ret(&[Value::Number(n as f64)])
}

pub(crate) fn fast_rawlen(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, ..]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    if nparams < 1 || nresults > 1 {
        return None;
    }
    let n = match a0 {
        Value::Table(t) => vm.heap.table(t).len(),
        Value::String(s) => vm.heap.str_bytes(s).len(),
        _ => return None,
    };
    vm.set(ra, Value::Number(n as f64));
    Some(1)
}

fn select(vm: &mut Core, args: Args) -> VmResult<usize> {
    let n = i64::try_from(args.nargs).unwrap_or(i64::MAX);
    if let Some(Value::String(s)) = vm.arg(args, 1)
        && vm.heap.str_bytes(s).first() == Some(&b'#')
    {
        return vm.ret(&[Value::Number(n.saturating_sub(1) as f64)]);
    }
    let i = i64::from(vm.check_integer(args, 1)?);
    let i = if i < 0 {
        n.saturating_add(i)
    } else if i > n {
        n
    } else {
        i
    };
    if i < 1 {
        return Err(vm.arg_error(1, "index out of range"));
    }
    Ok(usize::try_from(n.saturating_sub(i)).unwrap_or(0))
}

/// `luauF_select`, for `select(x, ...)` wanting one result: reads the
/// running Lua function's varargs; falls through for anything but an
/// index in range or `'#'`.
pub(crate) fn fast_select(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, ..]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    if nparams != 1 || nresults != 1 {
        return None;
    }
    let (first, n) = vm.running_varargs()?;
    let r = match a0 {
        Value::Number(x) => {
            let i = (x as i32).wrapping_sub(1) as u32;
            if usize::try_from(i).ok()? >= n {
                return None;
            }
            vm.get(first.saturating_add(i as usize))
        }
        Value::String(s) if vm.heap.str_bytes(s).first() == Some(&b'#') => Value::Number(n as f64),
        _ => return None,
    };
    vm.set(ra, r);
    Some(1)
}

/// C++ `strtoull` for `tonumber(s, base)`: optional space and sign, an
/// optional `0x` for base 16, digits; wraps on a minus sign and saturates
/// on overflow.
fn strtoull(s: &[u8], base: u32) -> Option<(u64, usize)> {
    let mut i = s
        .iter()
        .position(|c| !matches!(c, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'))
        .unwrap_or(s.len());
    let negative = match s.get(i) {
        Some(b'-') => {
            i = i.saturating_add(1);
            true
        }
        Some(b'+') => {
            i = i.saturating_add(1);
            false
        }
        _ => false,
    };
    if base == 16
        && s.get(i) == Some(&b'0')
        && matches!(s.get(i.saturating_add(1)), Some(b'x' | b'X'))
        && s.get(i.saturating_add(2))
            .is_some_and(|c| char::from(*c).is_ascii_hexdigit())
    {
        i = i.saturating_add(2);
    }
    let start = i;
    let mut value: u64 = 0;
    let mut overflow = false;
    while let Some(d) = s.get(i).and_then(|c| char::from(*c).to_digit(base)) {
        match value
            .checked_mul(u64::from(base))
            .and_then(|v| v.checked_add(u64::from(d)))
        {
            Some(v) => value = v,
            None => overflow = true,
        }
        i = i.saturating_add(1);
    }
    if i == start {
        return None;
    }
    if overflow {
        return Some((u64::MAX, i));
    }
    Some((
        if negative {
            value.wrapping_neg()
        } else {
            value
        },
        i,
    ))
}

/// `luauF_tonumber`: one argument; a number, a string that reads as one,
/// else nil (an integer too).
pub(crate) fn fast_tonumber(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, ..]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    if nparams != 1 || nresults > 1 {
        return None;
    }
    let r = match a0 {
        Value::Number(_) | Value::String(_) => vm.to_number(a0).map_or(Value::Nil, Value::Number),
        _ => Value::Nil,
    };
    vm.set(ra, r);
    Some(1)
}

fn tonumber(vm: &mut Core, args: Args) -> VmResult<usize> {
    let base = vm.opt_integer(args, 2, 10)?;
    if base == 10 {
        if let Some(n) = vm.arg(args, 1).and_then(|v| vm.to_number(v)) {
            return vm.ret(&[Value::Number(n)]);
        }
        vm.check_any(args, 1)?;
    } else {
        let s = vm.check_string(args, 1)?;
        let Ok(base) = u32::try_from(base) else {
            return Err(vm.arg_error(2, "base out of range"));
        };
        if !(2..=36).contains(&base) {
            return Err(vm.arg_error(2, "base out of range"));
        }
        let bytes = vm.heap.str_rc(s);
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let text = bytes.get(..end).unwrap_or_default();
        if let Some((n, used)) = strtoull(text, base)
            && text
                .get(used..)
                .unwrap_or_default()
                .iter()
                .all(|c| matches!(c, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'))
        {
            return vm.ret(&[Value::Number(n as f64)]);
        }
    }
    vm.ret(&[Value::Nil])
}

fn tostring(vm: &mut Resume<'_>, args: Args) -> VmResult<usize> {
    let v = vm.check_any(args, 1)?;
    match vm.tostring_step(v)? {
        Meta::Done(s) => vm.ret(&[s]),
        Meta::Call(call) => {
            // C++ `luaL_callmeta`: `lua_call`, counted and not yieldable.
            let at = vm.top;
            vm.ensure_stack(at.saturating_add(2).saturating_add(crate::vm::MIN_STACK))?;
            vm.set(at, call.f);
            vm.set(at.saturating_add(1), v);
            vm.top = at.saturating_add(2);
            vm.request_call(at, 1, 1, CallMode::Counted, Box::new(ToStringCont { at }))
        }
    }
}

/// `tostring` after `__tostring` returned to `at`.
struct ToStringCont {
    at: usize,
}

impl Continuation for ToStringCont {
    fn resume(self: Box<Self>, vm: &mut Resume<'_>, _: Args, _: Outcome) -> VmResult<usize> {
        let v = vm.get(self.at);
        vm.top = self.at;
        let s = vm.apply_post(crate::meta::Post::ToString, v)?;
        vm.ret(&[s])
    }
}

/// `luauF_tostring`: `nil`, booleans, numbers, integers and strings, with
/// no `__tostring` lookup; a number or integer falls through when the
/// collector is due (C++ `luaC_needsGC`), as it makes a string.
pub(crate) fn fast_tostring(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, ..]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    if nparams < 1 || nresults > 1 {
        return None;
    }
    let s = match a0 {
        Value::String(s) => s,
        Value::Nil => vm.type_name_str(a0),
        Value::Boolean(_) => vm.tostring_plain(a0).ok()?,
        Value::Number(_) | Value::Integer(_) => {
            if vm.needs_gc() {
                return None;
            }
            vm.tostring_plain(a0).ok()?
        }
        _ => return None,
    };
    vm.set(ra, Value::String(s));
    Some(1)
}

fn type_(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_any(args, 1)?;
    let name = vm.type_name_str(v);
    vm.ret(&[Value::String(name)])
}

pub(crate) fn fast_type(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, ..]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    if nparams < 1 || nresults > 1 {
        return None;
    }
    let name = vm.type_name_str(a0);
    vm.set(ra, Value::String(name));
    Some(1)
}

fn typeof_(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_any(args, 1)?;
    let name = vm.obj_type_name_str(v);
    vm.ret(&[Value::String(name)])
}

pub(crate) fn fast_typeof(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, ..]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    if nparams < 1 || nresults > 1 {
        return None;
    }
    let name = vm.obj_type_name_str(a0);
    vm.set(ra, Value::String(name));
    Some(1)
}

/// How `pcall` and `xpcall` call: C++ `lua_pcallyieldable` schedules the
/// call uncounted from a yieldable thread (after checking the limit) and
/// counts it otherwise.
fn protected_mode(vm: &Core) -> CallMode {
    if vm.yieldable() {
        CallMode::Checked
    } else {
        CallMode::CountedYieldable
    }
}

fn pcall(vm: &mut Resume<'_>, args: Args) -> VmResult<usize> {
    vm.check_any(args, 1)?;
    let func = args.base;
    let saved = vm.saved_calls();
    let mode = protected_mode(vm);
    let cont = Box::new(Protected {
        callee: func,
        handler: None,
        saved,
    });
    vm.request_call(func, args.nargs.saturating_sub(1), MULTRET, mode, cont)
}

fn xpcall(vm: &mut Resume<'_>, args: Args) -> VmResult<usize> {
    let Some(handler @ Value::Function(_)) = vm.arg(args, 2) else {
        return Err(vm.arg_type_error(args, 2, "function"));
    };
    // C++ swaps the slots: the handler moves to 1, the function to 2.
    let f = vm.arg_or_nil(args, 1);
    let base = args.base;
    let func = base.saturating_add(1);
    vm.set(base, handler);
    vm.set(func, f);
    let saved = vm.saved_calls();
    let mode = protected_mode(vm);
    let cont = Box::new(Protected {
        callee: func,
        handler: Some(base),
        saved,
    });
    vm.request_call(func, args.nargs.saturating_sub(2), MULTRET, mode, cont)
}

/// `pcall`/`xpcall` waiting on the protected call (C++ `luaB_pcallcont`,
/// `luaB_xpcallcont`). The handler, if any, ran in the driver's catch
/// (`HostedVm::catch`) before the unwind.
struct Protected {
    callee: usize,
    handler: Option<usize>,
    saved: Saved,
}

impl Continuation for Protected {
    fn resume(self: Box<Self>, vm: &mut Resume<'_>, args: Args, outcome: Outcome) -> VmResult<usize> {
        let base = args.base;
        match (outcome, self.handler) {
            (Outcome::Ok, None) => {
                let func = self.callee;
                let n = vm.top.saturating_sub(func);
                let top = vm.top;
                vm.ensure_stack(top.saturating_add(1))?;
                vm.copy_slots(func, func.saturating_add(1), n);
                vm.set(func, Value::Boolean(true));
                vm.top = func.saturating_add(n).saturating_add(1);
                Ok(n.saturating_add(1))
            }
            (Outcome::Ok, Some(_)) => {
                vm.set(base, Value::Boolean(true));
                Ok(vm.top.saturating_sub(base))
            }
            (Outcome::Err(throw), _) => {
                vm.ensure_stack(base.saturating_add(2))?;
                vm.set(base, Value::Boolean(false));
                vm.set(base.saturating_add(1), throw.value);
                vm.top = base.saturating_add(2);
                Ok(2)
            }
        }
    }

    fn catch(&self) -> Option<Catch> {
        Some(Catch {
            callee: self.callee,
            handler: self.handler,
            saved: self.saved,
        })
    }
}
