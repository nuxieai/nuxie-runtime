//! Part of the table library (C++ `ltablib.cpp`): `pack`, `unpack`,
//! `concat`, `insert`, `remove`. Lane 2 adds the rest (`create`, `clear`,
//! `move`, `sort`, `find`, `freeze`, `clone`, ...).

use crate::error::VmResult;
use crate::native::Args;
use crate::value::{TableId, Value};
use crate::vm::Vm;

pub(crate) fn open(vm: &mut Vm) -> VmResult<()> {
    vm.open_lib(
        "table",
        &[
            ("pack", pack),
            ("unpack", unpack),
            ("concat", concat),
            ("insert", insert),
            ("remove", remove),
        ],
    )?;
    Ok(())
}

fn len_of(vm: &Vm, t: TableId) -> i32 {
    i32::try_from(vm.heap.table(t).len()).unwrap_or(i32::MAX)
}

/// C++ `lua_rawseti`, with its read-only check.
fn rawseti(vm: &mut Vm, t: TableId, i: i32, v: Value) -> VmResult<()> {
    if vm.heap.table(t).readonly {
        return Err(vm.runtime_error("attempt to modify a readonly table"));
    }
    vm.raw_set(t, Value::Number(f64::from(i)), v)
}

fn pack(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let n = args.nargs;
    let t = vm.new_table(n, 1)?;
    for i in 0..n {
        let v = vm.get(args.base.saturating_add(i));
        if let Some(slot) = vm.heap.table_mut(t).array.get_mut(i) {
            *slot = v;
        }
    }
    vm.raw_set_str(t, "n", Value::Number(n as f64))?;
    vm.ret(&[Value::Table(t)])
}

fn unpack(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let i = vm.opt_integer(args, 2, 1)?;
    let default_end = len_of(vm, t);
    let e = vm.opt_integer(args, 3, default_end)?;
    if i > e {
        return Ok(0);
    }
    let n = (e as u32).wrapping_sub(i as u32);
    let fits = usize::try_from(n)
        .ok()
        .filter(|&n| n < i32::MAX as usize && n < vm.limits.max_stack_slots);
    let Some(n) = fits else {
        return Err(vm.lib_error("too many results to unpack"));
    };
    let n = n.saturating_add(1);
    if vm.ensure_stack(vm.top.saturating_add(n)).is_err() {
        return Err(vm.lib_error("too many results to unpack"));
    }
    let mut k = i;
    for _ in 0..n {
        let v = vm.heap.table(t).get_int(i64::from(k));
        vm.push(v)?;
        k = k.wrapping_add(1);
    }
    Ok(n)
}

fn concat(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let sep = match vm.opt_string(args, 2)? {
        Some(id) => vm.heap.str_bytes(id).to_vec(),
        None => Vec::new(),
    };
    let t = vm.check_table(args, 1)?;
    let i = vm.opt_integer(args, 3, 1)?;
    let default_last = len_of(vm, t);
    let last = vm.opt_integer(args, 4, default_last)?;
    let mut out = Vec::new();
    let mut k = i;
    while k <= last {
        let v = vm.heap.table(t).get_int(i64::from(k));
        match v {
            Value::String(id) => out.extend_from_slice(vm.heap.str_bytes(id)),
            Value::Number(n) => out.extend_from_slice(crate::num::num2str(n).as_bytes()),
            other => {
                let tn = vm.obj_type_name(other);
                return Err(vm.lib_error(format!(
                    "invalid value ({tn}) at index {k} in table for 'concat'"
                )));
            }
        }
        if k == last {
            break;
        }
        out.extend_from_slice(&sep);
        k = k.saturating_add(1);
    }
    vm.ret_str(&out)
}

/// C++ `moveelements` within one table: `t[f..=e]` to `t[to..]`.
fn move_elements(vm: &mut Vm, t: TableId, f: i32, e: i32, to: i32) -> VmResult<()> {
    if vm.heap.table(t).readonly {
        return Err(vm.runtime_error("attempt to modify a readonly table"));
    }
    if f > e {
        return Ok(());
    }
    let count = i64::from(e).saturating_sub(i64::from(f));
    let step = |k: i64| -> (i64, i64) {
        (
            i64::from(f).saturating_add(k),
            i64::from(to).saturating_add(k),
        )
    };
    let order: Box<dyn Iterator<Item = i64>> = if to > f {
        Box::new((0..=count).rev())
    } else {
        Box::new(0..=count)
    };
    for k in order {
        let (src, dst) = step(k);
        let v = vm.heap.table(t).get_int(src);
        vm.raw_set(t, Value::Number(dst as f64), v)?;
    }
    Ok(())
}

fn insert(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let n = len_of(vm, t);
    let pos = match args.nargs {
        2 => n.saturating_add(1),
        3 => {
            let pos = vm.check_integer(args, 2)?;
            if 1 <= pos && pos <= n {
                move_elements(vm, t, pos, n, pos.saturating_add(1))?;
            }
            pos
        }
        _ => return Err(vm.lib_error("wrong number of arguments to 'insert'")),
    };
    let v = vm.get(args.base.saturating_add(args.nargs).saturating_sub(1));
    rawseti(vm, t, pos, v)?;
    Ok(0)
}

fn remove(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let n = len_of(vm, t);
    let pos = vm.opt_integer(args, 2, n)?;
    if !(1 <= pos && pos <= n) {
        return Ok(0);
    }
    let v = vm.heap.table(t).get_int(i64::from(pos));
    vm.push(v)?;
    move_elements(vm, t, pos.saturating_add(1), n, pos)?;
    rawseti(vm, t, n, Value::Nil)?;
    Ok(1)
}
