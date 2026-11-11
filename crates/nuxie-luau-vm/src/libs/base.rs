//! The base library (C++ `lbaselib.cpp`), Rive profile: no `print`, no
//! `newproxy`. `print`, `loadstring` and `collectgarbage` come from the host
//! (the conformance harness registers them).

use crate::error::{ErrorKind, VmResult};
use crate::heap::Function;
use crate::native::Args;
use crate::value::Value;
use crate::vm::{MULTRET, Vm};

pub(crate) fn open(vm: &mut Vm) -> VmResult<()> {
    let globals = vm.globals;
    vm.raw_set_str(globals, "_G", Value::Table(globals))?;
    let version = vm.heap.intern(b"Luau")?;
    vm.raw_set_str(globals, "_VERSION", Value::String(version))?;
    vm.register_builtins(
        globals,
        &[
            ("assert", assert),
            ("error", error),
            ("gcinfo", gcinfo),
            ("getfenv", getfenv),
            ("getmetatable", getmetatable),
            ("next", next),
            ("rawequal", rawequal),
            ("rawget", rawget),
            ("rawset", rawset),
            ("rawlen", rawlen),
            ("select", select),
            ("setfenv", setfenv),
            ("setmetatable", setmetatable),
            ("tonumber", tonumber),
            ("tostring", tostring),
            ("type", type_),
            ("typeof", typeof_),
            ("pcall", pcall),
            ("xpcall", xpcall),
        ],
    )?;
    // C++ `auxopen`: the iterator is a separate, nameless closure.
    let inext_fn = vm.new_builtin("", inext, &[])?;
    let ipairs_fn = vm.new_builtin("ipairs", ipairs, &[inext_fn])?;
    vm.raw_set_str(globals, "ipairs", ipairs_fn)?;
    let next_fn = vm.new_builtin("", next, &[])?;
    let pairs_fn = vm.new_builtin("pairs", pairs, &[next_fn])?;
    vm.raw_set_str(globals, "pairs", pairs_fn)?;
    Ok(())
}

fn assert(vm: &mut Vm, args: Args) -> VmResult<usize> {
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

fn error(vm: &mut Vm, args: Args) -> VmResult<usize> {
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

fn gcinfo(vm: &mut Vm, _args: Args) -> VmResult<usize> {
    let kb = vm.heap.total_bytes() / 1024;
    vm.ret(&[Value::Number(kb as f64)])
}

/// C++ `getfunc`: the function argument 1 names, by value or by level.
fn getfunc(vm: &mut Vm, args: Args, optional: bool) -> VmResult<Value> {
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
    let index = vm
        .frames
        .len()
        .checked_sub(1)
        .and_then(|top| top.checked_sub(level));
    match index.and_then(|i| vm.frames.get(i)) {
        Some(frame) => Ok(Value::Function(frame.closure)),
        None => Err(vm.arg_error(1, "invalid level")),
    }
}

fn getfenv(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let f = getfunc(vm, args, true)?;
    let env = match f {
        Value::Function(id) => match vm.heap.function(id) {
            Function::Lua(c) => c.env,
            Function::Native(_) => vm.globals,
        },
        _ => vm.globals,
    };
    vm.heap.table_mut(env).safeenv = false;
    vm.ret(&[Value::Table(env)])
}

fn setfenv(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let table = vm.check_table(args, 2)?;
    let f = getfunc(vm, args, false)?;
    vm.heap.table_mut(table).safeenv = false;
    // Upstream: `lua_isnumber(L, 1) && lua_tonumber(L, 1) == 0`, so a
    // numeric string counts.
    if vm.arg(args, 1).and_then(|v| vm.to_number(v)) == Some(0.0) {
        vm.globals = table;
        return Ok(0);
    }
    if let Value::Function(id) = f
        && let Function::Lua(c) = vm.heap.functions.get_mut(id.0)
    {
        c.env = table;
        return vm.ret(&[f]);
    }
    Err(vm.lib_error("'setfenv' cannot change environment of given object"))
}

fn getmetatable(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let v = vm.check_any(args, 1)?;
    let Some(mt) = vm.metatable(v) else {
        return vm.ret(&[Value::Nil]);
    };
    let protected = vm.heap.table(mt).get_str(vm.tm(crate::vm::Tm::Metatable));
    if protected.is_nil() {
        vm.ret(&[Value::Table(mt)])
    } else {
        vm.ret(&[protected])
    }
}

fn setmetatable(vm: &mut Vm, args: Args) -> VmResult<usize> {
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

fn next(vm: &mut Vm, args: Args) -> VmResult<usize> {
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

fn pairs(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let f = vm.builtin_upval(0);
    vm.ret(&[f, Value::Table(t), Value::Nil])
}

fn ipairs(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let f = vm.builtin_upval(0);
    vm.ret(&[f, Value::Table(t), Value::Number(0.0)])
}

fn inext(vm: &mut Vm, args: Args) -> VmResult<usize> {
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

fn rawequal(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let a = vm.check_any(args, 1)?;
    let b = vm.check_any(args, 2)?;
    vm.ret(&[Value::Boolean(a.raw_eq(b))])
}

fn rawget(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let key = vm.check_any(args, 2)?;
    let v = vm.heap.table(t).get(key);
    vm.ret(&[v])
}

fn rawset(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let key = vm.check_any(args, 2)?;
    let value = vm.check_any(args, 3)?;
    if vm.heap.table(t).readonly {
        return Err(vm.runtime_error("attempt to modify a readonly table"));
    }
    vm.raw_set(t, key, value)?;
    vm.ret(&[Value::Table(t)])
}

fn rawlen(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let n = match vm.arg(args, 1) {
        Some(Value::Table(t)) => vm.heap.table(t).len(),
        Some(Value::String(s)) => vm.heap.str_bytes(s).len(),
        _ => return Err(vm.arg_error(1, "table or string expected")),
    };
    vm.ret(&[Value::Number(n as f64)])
}

fn select(vm: &mut Vm, args: Args) -> VmResult<usize> {
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

fn tonumber(vm: &mut Vm, args: Args) -> VmResult<usize> {
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

fn tostring(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let v = vm.check_any(args, 1)?;
    let s = vm.tostring_value(v)?;
    vm.ret(&[Value::String(s)])
}

fn type_(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let v = vm.check_any(args, 1)?;
    vm.ret_str(v.type_name().as_bytes())
}

fn typeof_(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let v = vm.check_any(args, 1)?;
    let name = vm.obj_type_name(v);
    vm.ret_str(name.as_bytes())
}

fn pcall(vm: &mut Vm, args: Args) -> VmResult<usize> {
    vm.check_any(args, 1)?;
    let func = args.base;
    let depth = vm.frames.len();
    let c_calls = vm.c_calls;
    match vm.call_value(func, args.nargs.saturating_sub(1), MULTRET) {
        Ok(()) => {
            let n = vm.top.saturating_sub(func);
            vm.ensure_stack(vm.top.saturating_add(1))?;
            vm.copy_slots(func, func.saturating_add(1), n);
            vm.set(func, Value::Boolean(true));
            vm.top = func.saturating_add(n).saturating_add(1);
            Ok(n.saturating_add(1))
        }
        Err(throw) => {
            vm.unwind(depth, func, c_calls);
            vm.set(func, Value::Boolean(false));
            vm.set(func.saturating_add(1), throw.value);
            vm.top = func.saturating_add(2);
            Ok(2)
        }
    }
}

fn xpcall(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let Some(handler @ Value::Function(_)) = vm.arg(args, 2) else {
        return Err(vm.arg_type_error(args, 2, "function"));
    };
    // C++ swaps the slots: the handler moves to 1, the function to 2.
    let f = vm.arg_or_nil(args, 1);
    let base = args.base;
    let func = base.saturating_add(1);
    vm.set(base, handler);
    vm.set(func, f);
    let depth = vm.frames.len();
    let c_calls = vm.c_calls;
    match vm.call_value(func, args.nargs.saturating_sub(2), MULTRET) {
        Ok(()) => {
            vm.set(base, Value::Boolean(true));
            Ok(vm.top.saturating_sub(base))
        }
        Err(throw) => {
            // The handler runs before unwinding, so it sees the stack.
            let at = vm.scratch();
            vm.ensure_stack(at.saturating_add(2).saturating_add(crate::vm::MIN_STACK))?;
            vm.set(at, vm.get(base));
            vm.set(at.saturating_add(1), throw.value);
            vm.top = at.saturating_add(2);
            let value = match vm.call_value(at, 1, 1) {
                Ok(()) => vm.get(at),
                Err(second)
                    if throw.kind == ErrorKind::Memory && second.kind == ErrorKind::Memory =>
                {
                    second.value
                }
                Err(_) => vm.error_in_handler().value,
            };
            vm.unwind(depth, func, c_calls);
            vm.set(base, Value::Boolean(false));
            vm.set(func, value);
            vm.top = base.saturating_add(2);
            Ok(2)
        }
    }
}
