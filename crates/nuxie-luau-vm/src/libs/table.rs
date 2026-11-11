//! The table library (C++ `ltablib.cpp`), every function upstream
//! registers.

use crate::call::{CallMode, Continuation, Outcome};
use crate::error::VmResult;
use crate::libs::append;
use crate::meta::{Meta, Post};
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
            ("sort", crate::libs::sort::sort),
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

/// Places `f(a, b)` at `at` and requests it as a counted call (C++
/// `lua_call`: not yieldable), continuing in `cont` with the one result in
/// `at`. The call is a frame, not Rust recursion (lane 4).
fn request_call2(
    vm: &mut Vm,
    at: usize,
    [f, a, b]: [Value; 3],
    cont: Box<dyn Continuation>,
) -> VmResult<usize> {
    vm.ensure_stack(at.saturating_add(3).saturating_add(MIN_STACK))?;
    vm.set(at, f);
    vm.set(at.saturating_add(1), a);
    vm.set(at.saturating_add(2), b);
    vm.top = at.saturating_add(3);
    vm.request_call(at, 2, 1, CallMode::Counted, cont)
}

/// C++ `foreach`: calls `f(k, v)` for each entry, in `next` order, and
/// returns the first non-nil result.
fn foreach(vm: &mut Vm, args: Args) -> VmResult<usize> {
    vm.check_table(args, 1)?;
    check_function(vm, args, 2)?;
    // The key and value live in stack slots below the call, as C++ keeps
    // `lua_next`'s pair: both stay rooted across the callback (a weak-value
    // entry survives a callback that drops its argument and collects), and
    // `next` can look the key up again.
    let key_slot = vm.scratch();
    vm.ensure_stack(key_slot.saturating_add(2))?;
    vm.set(key_slot, Value::Nil);
    foreach_next(vm, args, key_slot)
}

/// `foreach` from the entry after the key in `key_slot`.
fn foreach_next(vm: &mut Vm, args: Args, key_slot: usize) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let f = vm.arg_or_nil(args, 2);
    let key = vm.get(key_slot);
    let table = vm.heap.table(t);
    let Some(cursor) = table.cursor_after(key) else {
        return Err(vm.runtime_error("invalid key to 'next'"));
    };
    let Some((_, k, v)) = table.next_from(cursor) else {
        vm.top = key_slot;
        return Ok(0);
    };
    vm.set(key_slot, k);
    vm.set(key_slot.saturating_add(1), v);
    let at = key_slot.saturating_add(2);
    request_call2(vm, at, [f, k, v], Box::new(ForeachCont { key_slot }))
}

struct ForeachCont {
    key_slot: usize,
}

impl Continuation for ForeachCont {
    fn resume(self: Box<Self>, vm: &mut Vm, args: Args, _: Outcome) -> VmResult<usize> {
        let at = self.key_slot.saturating_add(2);
        if !vm.get(at).is_nil() {
            vm.top = at.saturating_add(1);
            return Ok(1);
        }
        foreach_next(vm, args, self.key_slot)
    }
}

/// C++ `foreachi`: calls `f(i, t[i])` for `i` in `1..=#t`, `#t` taken once.
fn foreachi(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    check_function(vm, args, 2)?;
    let n = len_of(vm, t);
    let at = vm.scratch();
    foreachi_from(vm, args, ForeachiCont { at, i: 1, n })
}

/// `foreachi` from index `state.i`.
fn foreachi_from(vm: &mut Vm, args: Args, state: ForeachiCont) -> VmResult<usize> {
    if state.i > state.n {
        vm.top = state.at;
        return Ok(0);
    }
    let t = vm.check_table(args, 1)?;
    let f = vm.arg_or_nil(args, 2);
    let v = vm.heap.table(t).get_int(i64::from(state.i));
    let i = Value::Number(f64::from(state.i));
    request_call2(vm, state.at, [f, i, v], Box::new(state))
}

struct ForeachiCont {
    at: usize,
    i: i32,
    n: i32,
}

impl Continuation for ForeachiCont {
    fn resume(self: Box<Self>, vm: &mut Vm, args: Args, _: Outcome) -> VmResult<usize> {
        if !vm.get(self.at).is_nil() {
            vm.top = self.at.saturating_add(1);
            return Ok(1);
        }
        let next = ForeachiCont {
            i: self.i.saturating_add(1),
            ..*self
        };
        foreachi_from(vm, args, next)
    }
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
    let max = vm.heap.table(t).max_number_key();
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
        vm.heap.table_mut(t).array_mut().fill(v);
    }
    vm.ret(&[Value::Table(t)])
}

/// C++ `tfind`: the first index from `init` whose value equals `v`
/// (with `__eq`), scanning until the first nil. An `__eq` call is a frame
/// (`FindCont`), not Rust recursion.
fn find(vm: &mut Vm, args: Args) -> VmResult<usize> {
    vm.check_table(args, 1)?;
    vm.check_any(args, 2)?;
    let init = vm.opt_integer(args, 3, 1)?;
    if init < 1 {
        return Err(vm.arg_error(3, "index out of range"));
    }
    find_from(vm, args, init)
}

/// `find` from index `i`.
fn find_from(vm: &mut Vm, args: Args, mut i: i32) -> VmResult<usize> {
    let t = vm.check_table(args, 1)?;
    let needle = vm.arg_or_nil(args, 2);
    loop {
        let e = vm.heap.table(t).get_int(i64::from(i));
        if e.is_nil() {
            break;
        }
        let equal = match vm.equal_step(needle, e) {
            Meta::Done(v) => v.truthy(),
            Meta::Call(call) => {
                let at = vm.scratch();
                let [a, b, _] = call.args;
                let cont = Box::new(FindCont {
                    at,
                    i,
                    post: call.post,
                });
                return request_call2(vm, at, [call.f, a, b], cont);
            }
        };
        if equal {
            return vm.ret(&[Value::Number(f64::from(i))]);
        }
        match next_find_index(i) {
            Some(next) => i = next,
            None => break,
        }
    }
    vm.ret(&[Value::Nil])
}

/// C++ increments past `INT_MAX` (undefined); there is no index beyond it,
/// so the scan stops (the vendored VM does the same).
fn next_find_index(i: i32) -> Option<i32> {
    (i != i32::MAX).then(|| i.saturating_add(1))
}

struct FindCont {
    at: usize,
    i: i32,
    post: Post,
}

impl Continuation for FindCont {
    fn resume(self: Box<Self>, vm: &mut Vm, args: Args, _: Outcome) -> VmResult<usize> {
        let result = vm.get(self.at);
        vm.top = self.at;
        if vm.apply_post(self.post, result)?.truthy() {
            return vm.ret(&[Value::Number(f64::from(self.i))]);
        }
        match next_find_index(self.i) {
            Some(next) => find_from(vm, args, next),
            None => vm.ret(&[Value::Nil]),
        }
    }
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
    // C++ `luaH_clone`: the same parts, contents and metatable, neither
    // read-only nor `safeenv`. `clone_table` checks and charges the copy.
    let id = vm.heap.clone_table(t)?;
    let copy = vm.heap.table_mut(id);
    copy.readonly = false;
    copy.safeenv = false;
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
        let size = i64::try_from(vm.heap.table(dst).array().len()).unwrap_or(i64::MAX);
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
    let src_size = i64::try_from(vm.heap.table(src).array().len()).unwrap_or(i64::MAX);
    let dst_size = i64::try_from(vm.heap.table(dst).array().len()).unwrap_or(i64::MAX);
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
