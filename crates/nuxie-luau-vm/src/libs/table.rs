//! Part of the table library (C++ `ltablib.cpp`): `pack`, `unpack`,
//! `concat`, `insert`, `remove`, `sort` (the bench support code calls it).
//! Lane 2 adds the rest (`create`, `clear`, `move`, `find`, `freeze`,
//! `clone`, ...).

use crate::error::VmResult;
use crate::libs::append;
use crate::native::Args;
use crate::value::{TableId, Value};
use crate::vm::Vm;

pub(crate) fn open(vm: &mut Vm) -> VmResult<()> {
    let t = vm.open_lib(
        "table",
        &[
            ("pack", pack),
            ("unpack", unpack),
            ("concat", concat),
            ("insert", insert),
            ("remove", remove),
            ("sort", sort),
        ],
    )?;
    // Lua 5.1 compatibility: the global `unpack` is `table.unpack`.
    let name = vm.heap.intern(b"unpack")?;
    let unpack = vm.heap.table(t).get_str(name);
    let globals = vm.globals;
    vm.raw_set_str(globals, "unpack", unpack)
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
        if let Some(slot) = vm.heap.table_mut(t).array_mut().get_mut(i) {
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
            Value::String(id) => append(vm, &mut out, vm.heap.str_bytes(id))?,
            Value::Number(n) => append(vm, &mut out, crate::num::num2str(n).as_bytes())?,
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
        append(vm, &mut out, &sep)?;
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

/// C++ `tsort`: introsort over the array part, quicksort with a
/// median-of-three pivot falling back to heapsort past `1.5 log2 n`
/// partitions. The comparison order and the errors are C++'s, so a script
/// observes the same predicate calls.
fn sort(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let n = vm.heap.table(t).len();
    if vm.heap.table(t).readonly {
        return Err(vm.runtime_error("attempt to modify a readonly table"));
    }
    let pred = match vm.arg(args, 2) {
        None | Some(Value::Nil) => None,
        Some(f @ Value::Function(_)) => Some(f),
        Some(_) => return Err(vm.arg_type_error(args, 2, "function")),
    };
    if n == 0 {
        return Ok(0);
    }
    // C++ reads the border's elements straight from the array part; a border
    // that reaches into the hash part (C++ asserts there) moves them over.
    if vm.heap.table(t).array().len() < n {
        vm.resize_array(t, n)?;
    }
    let mut sorter = Sorter {
        t,
        size: vm.heap.table(t).array().len(),
        pred,
    };
    sorter.sort_rec(vm, 0, n.saturating_sub(1), n)?;
    Ok(0)
}

struct Sorter {
    t: TableId,
    /// The array size when sorting started; a predicate that resizes the
    /// table is an error.
    size: usize,
    pred: Option<Value>,
}

impl Sorter {
    fn element(&self, vm: &mut Vm, i: usize) -> VmResult<Value> {
        match vm.heap.table(self.t).array().get(i) {
            Some(&v) => Ok(v),
            None => Err(vm.lib_error("table modified during sorting")),
        }
    }

    /// C++ `sort_less`: is `a[i] < a[j]`?
    fn less(&self, vm: &mut Vm, i: usize, j: usize) -> VmResult<bool> {
        let a = self.element(vm, i)?;
        let b = self.element(vm, j)?;
        let result = match self.pred {
            None => vm.less_than(a, b)?,
            Some(f) => {
                let at = vm.scratch();
                vm.ensure_stack(at.saturating_add(3).saturating_add(crate::vm::MIN_STACK))?;
                vm.set(at, f);
                vm.set(at.saturating_add(1), a);
                vm.set(at.saturating_add(2), b);
                let saved = vm.top;
                vm.top = at.saturating_add(3);
                vm.call_value(at, 2, 1)?;
                let v = vm.get(at);
                vm.top = saved;
                v.truthy()
            }
        };
        if vm.heap.table(self.t).array().len() != self.size {
            return Err(vm.lib_error("table modified during sorting"));
        }
        Ok(result)
    }

    fn swap(&self, vm: &mut Vm, i: usize, j: usize) {
        let array = vm.heap.table_mut(self.t).array_mut();
        if i < array.len() && j < array.len() {
            array.swap(i, j);
        }
    }

    /// C++ `sort_rec` over `a[l..=u]`.
    fn sort_rec(
        &mut self,
        vm: &mut Vm,
        mut l: usize,
        mut u: usize,
        mut limit: usize,
    ) -> VmResult<()> {
        while l < u {
            if limit == 0 {
                return self.sort_heap(vm, l, u);
            }
            if self.less(vm, u, l)? {
                self.swap(vm, u, l);
            }
            if u.saturating_sub(l) == 1 {
                break;
            }
            let m = l.saturating_add(u.saturating_sub(l) >> 1);
            if self.less(vm, m, l)? {
                self.swap(vm, m, l);
            } else if self.less(vm, u, m)? {
                self.swap(vm, m, u);
            }
            if u.saturating_sub(l) == 2 {
                break;
            }
            let p = u.saturating_sub(1);
            self.swap(vm, m, p);
            let mut i = l;
            let mut j = p;
            loop {
                loop {
                    i = i.saturating_add(1);
                    if !self.less(vm, i, p)? {
                        break;
                    }
                    if i >= u {
                        return Err(vm.lib_error("invalid order function for sorting"));
                    }
                }
                loop {
                    j = j.saturating_sub(1);
                    if !self.less(vm, p, j)? {
                        break;
                    }
                    if j <= l {
                        return Err(vm.lib_error("invalid order function for sorting"));
                    }
                }
                if j < i {
                    break;
                }
                self.swap(vm, i, j);
            }
            self.swap(vm, p, i);
            limit = (limit >> 1).saturating_add(limit >> 2);
            if i.saturating_sub(l) < u.saturating_sub(i) {
                if let Some(upper) = i.checked_sub(1) {
                    self.sort_rec(vm, l, upper, limit)?;
                }
                l = i.saturating_add(1);
            } else {
                self.sort_rec(vm, i.saturating_add(1), u, limit)?;
                match i.checked_sub(1) {
                    Some(upper) => u = upper,
                    None => break,
                }
            }
        }
        Ok(())
    }

    /// C++ `sort_heap` over `a[l..=u]`.
    fn sort_heap(&mut self, vm: &mut Vm, l: usize, u: usize) -> VmResult<()> {
        let count = u.saturating_sub(l).saturating_add(1);
        let mut root = (count / 2).checked_sub(1);
        while let Some(r) = root {
            self.sift(vm, l, u, r)?;
            root = r.checked_sub(1);
        }
        let mut i = count.saturating_sub(1);
        while i > 0 {
            self.swap(vm, l, l.saturating_add(i));
            self.sift(vm, l, l.saturating_add(i).saturating_sub(1), 0)?;
            i = i.saturating_sub(1);
        }
        Ok(())
    }

    /// C++ `sort_siftheap`.
    fn sift(&mut self, vm: &mut Vm, l: usize, u: usize, mut root: usize) -> VmResult<()> {
        let count = u.saturating_sub(l).saturating_add(1);
        while root.saturating_mul(2).saturating_add(2) < count {
            let left = root.saturating_mul(2).saturating_add(1);
            let right = left.saturating_add(1);
            let mut next = root;
            if self.less(vm, l.saturating_add(next), l.saturating_add(left))? {
                next = left;
            }
            if self.less(vm, l.saturating_add(next), l.saturating_add(right))? {
                next = right;
            }
            if next == root {
                break;
            }
            self.swap(vm, l.saturating_add(root), l.saturating_add(next));
            root = next;
        }
        let last_left = root.saturating_mul(2).saturating_add(1);
        if last_left == count.saturating_sub(1)
            && self.less(vm, l.saturating_add(root), l.saturating_add(last_left))?
        {
            self.swap(vm, l.saturating_add(root), l.saturating_add(last_left));
        }
        Ok(())
    }
}
