//! The table library (C++ `ltablib.cpp`), every function upstream
//! registers.

use crate::error::VmResult;
use crate::libs::append;
use crate::native::Args;
use crate::table::{MAXSIZE, TableError};
use crate::value::{TableId, Value};
use crate::vm::{MIN_STACK, Tm, Vm};

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
            ("foreach", foreach),
            ("foreachi", foreachi),
            ("getn", getn),
            ("maxn", maxn),
            ("move", tmove),
            ("create", create),
            ("find", find),
            ("clear", clear),
            ("freeze", freeze),
            ("isfrozen", isfrozen),
            ("clone", clone),
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
    // The element count minus one, guarded before adding one (C++).
    let n = (e as u32).wrapping_sub(i as u32);
    let n = usize::try_from(n).unwrap_or(usize::MAX);
    if n >= i32::MAX as usize || !vm.check_stack(args, n.saturating_add(1)) {
        return Err(vm.lib_error("too many results to unpack"));
    }
    let n = n.saturating_add(1);
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

fn check_function(vm: &mut Vm, args: Args, n: usize) -> VmResult<Value> {
    match vm.arg(args, n) {
        Some(f @ Value::Function(_)) => Ok(f),
        _ => Err(vm.arg_type_error(args, n, "function")),
    }
}

/// C++ `luaL_getmetafield(L, 1, "__metatable")` as a test: does the
/// table's metatable protect it?
fn has_protected_metatable(vm: &Vm, t: TableId) -> bool {
    vm.heap
        .table(t)
        .meta
        .is_some_and(|mt| !vm.heap.table(mt).get_str(vm.tm(Tm::Metatable)).is_nil())
}

fn readonly_error(vm: &mut Vm) -> crate::error::Throw {
    vm.runtime_error("attempt to modify a readonly table")
}

/// Calls `f(a, b)` from a builtin with the call placed at `at`, leaving its
/// one result in `at` and the stack top just above it.
fn call2(vm: &mut Vm, at: usize, f: Value, a: Value, b: Value) -> VmResult<Value> {
    vm.ensure_stack(at.saturating_add(3).saturating_add(MIN_STACK))?;
    vm.set(at, f);
    vm.set(at.saturating_add(1), a);
    vm.set(at.saturating_add(2), b);
    vm.top = at.saturating_add(3);
    vm.call_value(at, 2, 1)?;
    vm.top = at.saturating_add(1);
    Ok(vm.get(at))
}

/// C++ `foreach`: calls `f(k, v)` for each entry, in `next` order, and
/// returns the first non-nil result.
fn foreach(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let f = check_function(vm, args, 2)?;
    // The key lives in a stack slot so it stays rooted across the call and
    // `next` can look it up again (C++ keeps it on the stack the same way).
    let key_slot = vm.scratch();
    vm.ensure_stack(key_slot.saturating_add(1))?;
    vm.set(key_slot, Value::Nil);
    loop {
        let key = vm.get(key_slot);
        let table = vm.heap.table(t);
        let Some(cursor) = table.cursor_after(key) else {
            return Err(vm.runtime_error("invalid key to 'next'"));
        };
        let Some((_, k, v)) = table.next_from(cursor) else {
            break;
        };
        vm.set(key_slot, k);
        let at = key_slot.saturating_add(1);
        let result = call2(vm, at, f, k, v)?;
        if !result.is_nil() {
            return Ok(1);
        }
    }
    vm.top = key_slot;
    Ok(0)
}

/// C++ `foreachi`: calls `f(i, t[i])` for `i` in `1..=#t`, `#t` taken once.
fn foreachi(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let f = check_function(vm, args, 2)?;
    let n = len_of(vm, t);
    let at = vm.scratch();
    for i in 1..=n {
        let v = vm.heap.table(t).get_int(i64::from(i));
        let result = call2(vm, at, f, Value::Number(f64::from(i)), v)?;
        if !result.is_nil() {
            return Ok(1);
        }
        vm.top = at;
    }
    Ok(0)
}

fn getn(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let n = len_of(vm, t);
    vm.ret(&[Value::Number(f64::from(n))])
}

/// C++ `maxn`: the largest positive numeric key, scanning the array part
/// and then the hash part.
fn maxn(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let table = vm.heap.table(t);
    let mut max = 0.0f64;
    for (i, v) in table.array.iter().enumerate() {
        if !v.is_nil() {
            max = i.saturating_add(1) as f64;
        }
    }
    for node in &table.nodes {
        if let Value::Number(k) = node.key
            && !node.val.is_nil()
            && k > max
        {
            max = k;
        }
    }
    vm.ret(&[Value::Number(max)])
}

/// C++ `tcreate`.
fn create(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let size = vm.check_integer(args, 1)?;
    let Ok(size) = usize::try_from(size) else {
        return Err(vm.arg_error(1, "size out of range"));
    };
    if size > MAXSIZE {
        return Err(vm.table_error(TableError::Overflow));
    }
    let t = vm.new_table(size, 0)?;
    if let Some(v) = vm.arg(args, 2).filter(|v| !v.is_nil()) {
        vm.heap.table_mut(t).array.fill(v);
    }
    vm.ret(&[Value::Table(t)])
}

/// C++ `tfind`: the first index from `init` whose value equals `v`
/// (with `__eq`), scanning until the first nil.
fn find(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let needle = vm.check_any(args, 2)?;
    let init = vm.opt_integer(args, 3, 1)?;
    if init < 1 {
        return Err(vm.arg_error(3, "index out of range"));
    }
    let mut i = init;
    loop {
        let e = vm.heap.table(t).get_int(i64::from(i));
        if e.is_nil() {
            break;
        }
        if vm.equal(needle, e)? {
            return vm.ret(&[Value::Number(f64::from(i))]);
        }
        // C++ increments past `INT_MAX` (undefined); there is no index
        // beyond it, so stop (the vendored VM does the same).
        if i == i32::MAX {
            break;
        }
        i = i.saturating_add(1);
    }
    vm.ret(&[Value::Nil])
}

fn clear(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    if vm.heap.table(t).readonly {
        return Err(readonly_error(vm));
    }
    vm.heap.table_mut(t).clear();
    Ok(0)
}

fn freeze(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    if vm.heap.table(t).readonly {
        return Err(vm.arg_error(1, "table is already frozen"));
    }
    if has_protected_metatable(vm, t) {
        return Err(vm.arg_error(1, "table has a protected metatable"));
    }
    vm.heap.table_mut(t).readonly = true;
    vm.ret(&[Value::Table(t)])
}

fn isfrozen(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let frozen = vm.heap.table(t).readonly;
    vm.ret(&[Value::Boolean(frozen)])
}

fn clone(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    if has_protected_metatable(vm, t) {
        return Err(vm.arg_error(1, "table has a protected metatable"));
    }
    // Checked before the copy is made; `new_table` charges it.
    vm.heap.check_alloc(vm.heap.table(t).capacity_bytes())?;
    let copy = vm.heap.table(t).clone_unfrozen();
    let id = vm.heap.new_table(copy)?;
    vm.ret(&[Value::Table(id)])
}

/// C++ `tmove`: `a2[t..] = a1[f..=e]`, returning `a2` (default `a1`).
fn tmove(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let src = vm.check_table(args, 1)?;
    let f = i64::from(vm.check_integer(args, 2)?);
    let e = i64::from(vm.check_integer(args, 3)?);
    let t = i64::from(vm.check_integer(args, 4)?);
    let tt = if vm.arg_or_nil(args, 5).is_nil() { 1 } else { 5 };
    let dst = vm.check_table(args, tt)?;
    if e >= f {
        // All of these are `int` values in C++; in `i64` none of the
        // arithmetic below can overflow, so saturation never applies.
        let int_max = i64::from(i32::MAX);
        if !(f > 0 || e < int_max.saturating_add(f)) {
            return Err(vm.arg_error(3, "too many elements to move"));
        }
        let n = e.saturating_sub(f).saturating_add(1);
        if t > int_max.saturating_sub(n).saturating_add(1) {
            return Err(vm.arg_error(4, "destination wrap around"));
        }
        if vm.heap.table(dst).readonly {
            return Err(readonly_error(vm));
        }
        let max_elems = vm.heap.table(src).slot_count().max(vm.heap.table(dst).slot_count());
        let sparse = n > 32 && usize::try_from(n / 2).is_ok_and(|half| half > max_elems);
        let size = i64::try_from(vm.heap.table(dst).array.len()).unwrap_or(i64::MAX);
        let t1 = t.saturating_sub(1);
        let grown = t1.saturating_add(n);
        if t > 0 && t1 <= size && grown > size {
            vm.resize_array(dst, usize::try_from(grown).unwrap_or(usize::MAX))?;
        }
        move_between(vm, src, dst, Range { f, n, t }, sparse)?;
    }
    vm.ret(&[Value::Table(dst)])
}

/// A `table.move` request: `n >= 1` elements from `f` to `t`, all within
/// `int` range (checked by `tmove`).
#[derive(Clone, Copy)]
struct Range {
    f: i64,
    n: i64,
    t: i64,
}

impl Range {
    fn last(self) -> i64 {
        self.f.saturating_add(self.n).saturating_sub(1)
    }

    /// The `k`th (source, destination) index pair, in copy order.
    fn pair(self, k: i64, backwards: bool) -> (i64, i64) {
        let k = if backwards {
            self.n.saturating_sub(1).saturating_sub(k)
        } else {
            k
        };
        (self.f.saturating_add(k), self.t.saturating_add(k))
    }
}

/// C++ `moveelements` (with `LuauTableMoveTimeoutFix`, which the product
/// profile turns on): the array fast path, the sparse path for ranges much
/// larger than both tables, and the element loop otherwise. The fast path
/// and the loop store the same values, so one loop serves both.
fn move_between(vm: &mut Vm, src: TableId, dst: TableId, r: Range, sparse: bool) -> VmResult<()> {
    let backwards = !(r.t > r.last() || r.t <= r.f || src != dst);
    let src_size = i64::try_from(vm.heap.table(src).array.len()).unwrap_or(i64::MAX);
    let dst_size = i64::try_from(vm.heap.table(dst).array.len()).unwrap_or(i64::MAX);
    let in_arrays = r.f >= 1
        && r.t >= 1
        && r.f.saturating_sub(1).saturating_add(r.n) <= src_size
        && r.t.saturating_sub(1).saturating_add(r.n) <= dst_size;
    if sparse && !in_arrays {
        return move_sparse(vm, src, dst, r);
    }
    for k in 0..r.n {
        let (from, to) = r.pair(k, backwards);
        let v = vm.heap.table(src).get_int(from);
        vm.raw_set(dst, Value::Number(to as f64), v)?;
    }
    Ok(())
}

/// An integer key in `first..=last` (C++ `tovalidintkey`).
fn valid_int_key(key: Value, first: i64, last: i64) -> Option<i64> {
    let Value::Number(x) = key else {
        return None;
    };
    if x < first as f64 || x > last as f64 {
        return None;
    }
    let i = x as i64;
    (i as f64 == x).then_some(i)
}

/// The sparse `table.move` path: iterate the entries that exist instead of
/// the range. As in C++, the source entries in range are first copied into
/// a temporary table (charged like any table, and rooted in a stack slot),
/// so overlapping moves within one table see the original values.
fn move_sparse(vm: &mut Vm, src: TableId, dst: TableId, r: Range) -> VmResult<()> {
    let te = r.t.saturating_add(r.n).saturating_sub(1);
    let saved_top = vm.top;
    let tmp = vm.new_table(0, 0)?;
    vm.push(Value::Table(tmp))?;
    let mut cursor = 0;
    while let Some((next, k, v)) = vm.heap.table(src).next_from(cursor) {
        if let Some(i) = valid_int_key(k, r.f, r.last()) {
            vm.raw_set(tmp, Value::Number(i as f64), v)?;
        }
        cursor = next;
    }
    // Clear the destination range while iterating: storing nil over an
    // existing entry never grows or moves the table, so the cursor stays
    // valid.
    let mut cursor = 0;
    while let Some((next, k, _)) = vm.heap.table(dst).next_from(cursor) {
        if let Some(i) = valid_int_key(k, r.t, te) {
            vm.raw_set(dst, Value::Number(i as f64), Value::Nil)?;
        }
        cursor = next;
    }
    let mut cursor = 0;
    while let Some((next, k, v)) = vm.heap.table(tmp).next_from(cursor) {
        if let Value::Number(i) = k {
            let to = (i as i64).saturating_sub(r.f).saturating_add(r.t);
            vm.raw_set(dst, Value::Number(to as f64), v)?;
        }
        cursor = next;
    }
    vm.top = saved_top;
    Ok(())
}
