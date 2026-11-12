//! The `integer` builtins' fast bodies (C++ `luauF_integer*`, ids 94 to
//! 130; `fastcall.rs`).
//!
//! This VM opens no `integer` library: the vendored VM opens one (as
//! `int64`) only under `LuauIntegerLibrary`, and the compiler emits these
//! ids only under `LuauIntegerFastcalls`, which the pinned profiles leave
//! off. The bodies are ported all the same (docs/luau-safe-vm-call-path.md,
//! decision 2); with no library behind them, a body that falls through
//! leaves the `CALL` to raise. No differential case reaches them, so the
//! unit tests below pin each one's results and bail conditions to the
//! vendored body's.

use crate::fastcall::arg;
use crate::value::Value;
use crate::vm::Core;

/// Two integers in, one value out: `nparams >= 2`, `nresults <= 1`; `f`
/// may refuse (`None`).
macro_rules! fast_integer2 {
    ($name:ident, $f:expr) => {
        pub(crate) fn $name(
            vm: &mut Core,
            ra: usize,
            nresults: i32,
            [a0, a1, _]: [Value; 3],
            nparams: usize,
        ) -> Option<usize> {
            let (Value::Integer(a), Value::Integer(b)) = (a0, a1) else { return None };
            if nparams < 2 || nresults > 1 {
                return None;
            }
            let f: fn(i64, i64) -> Option<Value> = $f;
            vm.set(ra, f(a, b)?);
            Some(1)
        }
    };
}

/// One integer in, one value out.
macro_rules! fast_integer1 {
    ($name:ident, $f:expr) => {
        pub(crate) fn $name(
            vm: &mut Core,
            ra: usize,
            nresults: i32,
            [a0, ..]: [Value; 3],
            nparams: usize,
        ) -> Option<usize> {
            let Value::Integer(a) = a0 else { return None };
            if nparams < 1 || nresults > 1 {
                return None;
            }
            let f: fn(i64) -> Value = $f;
            vm.set(ra, f(a));
            Some(1)
        }
    };
}

/// `luauF_integercreate`: a number with an exact `int64` value, else nil.
pub(crate) fn fast_create(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, ..]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    let Value::Number(x) = a0 else { return None };
    if nparams < 1 || nresults > 1 {
        return None;
    }
    let r = if (-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&x)
        && (x as i64) as f64 == x
    {
        Value::Integer(x as i64)
    } else {
        Value::Nil
    };
    vm.set(ra, r);
    Some(1)
}

fn int(i: i64) -> Option<Value> {
    Some(Value::Integer(i))
}

fn boolean(b: bool) -> Option<Value> {
    Some(Value::Boolean(b))
}

fast_integer1!(fast_tonumber, |a| Value::Number(a as f64));
fast_integer1!(fast_neg, |a| Value::Integer(a.wrapping_neg()));
fast_integer1!(fast_bnot, |a| Value::Integer(!a));
fast_integer1!(fast_countrz, |a| Value::Integer(i64::from(a.trailing_zeros())));
fast_integer1!(fast_countlz, |a| Value::Integer(i64::from(a.leading_zeros())));
fast_integer1!(fast_bswap, |a| Value::Integer(a.swap_bytes()));

fast_integer2!(fast_add, |a, b| int(a.wrapping_add(b)));
fast_integer2!(fast_sub, |a, b| int(a.wrapping_sub(b)));
fast_integer2!(fast_mul, |a, b| int(a.wrapping_mul(b)));
// Truncating; division by zero and `MIN / -1` fall through.
fast_integer2!(fast_div, |a, b| a.checked_div(b).map(Value::Integer));
// Flooring; the same two fall through.
fast_integer2!(fast_idiv, |a, b| {
    let q = a.checked_div(b)?;
    let r = a.checked_rem(b)?;
    int(if q < 0 && r != 0 { q.wrapping_sub(1) } else { q })
});
fast_integer2!(fast_udiv, |a, b| (a as u64).checked_div(b as u64).map(|q| Value::Integer(q as i64)));
fast_integer2!(fast_urem, |a, b| (a as u64).checked_rem(b as u64).map(|r| Value::Integer(r as i64)));
// Truncating remainder; `MIN % -1` is 0, a zero divisor falls through.
fast_integer2!(fast_rem, |a, b| {
    if b == 0 {
        return None;
    }
    int(a.checked_rem(b).unwrap_or(0))
});
// Flooring remainder (the sign of the divisor).
fast_integer2!(fast_mod, |a, b| {
    if b == 0 {
        return None;
    }
    let Some(r) = a.checked_rem(b) else { return int(0) };
    int(if r != 0 && (a < 0) != (b < 0) { r.wrapping_add(b) } else { r })
});
fast_integer2!(fast_lt, |a, b| boolean(a < b));
fast_integer2!(fast_le, |a, b| boolean(a <= b));
fast_integer2!(fast_gt, |a, b| boolean(a > b));
fast_integer2!(fast_ge, |a, b| boolean(a >= b));
fast_integer2!(fast_ult, |a, b| boolean((a as u64) < (b as u64)));
fast_integer2!(fast_ule, |a, b| boolean((a as u64) <= (b as u64)));
fast_integer2!(fast_ugt, |a, b| boolean((a as u64) > (b as u64)));
fast_integer2!(fast_uge, |a, b| boolean((a as u64) >= (b as u64)));
// A negative count shifts the other way; past 63 either way gives 0.
fast_integer2!(fast_lshift, |n, i| {
    let n = n as u64;
    int(match i {
        -63..=-1 => n.wrapping_shr(i.unsigned_abs() as u32),
        0..=63 => n.wrapping_shl(i as u32),
        _ => 0,
    } as i64)
});
fast_integer2!(fast_rshift, |n, i| {
    let n = n as u64;
    int(match i {
        -63..=-1 => n.wrapping_shl(i.unsigned_abs() as u32),
        0..=63 => n.wrapping_shr(i as u32),
        _ => 0,
    } as i64)
});
// Arithmetic: past 63 to the right gives the sign, past 63 to the left 0.
fast_integer2!(fast_arshift, |n, i| {
    int(match i {
        -63..=-1 => (n as u64).wrapping_shl(i.unsigned_abs() as u32) as i64,
        0..=63 => n.wrapping_shr(i as u32),
        i64::MIN..=-64 => 0,
        _ => {
            if n < 0 {
                -1
            } else {
                0
            }
        }
    })
});
// The count is taken mod 64 as unsigned.
fast_integer2!(fast_lrotate, |n, s| int((n as u64).rotate_left(((s as u64) % 64) as u32) as i64));
fast_integer2!(fast_rrotate, |n, s| int((n as u64).rotate_right(((s as u64) % 64) as u32) as i64));

/// `luauF_integerclamp`: unordered bounds fall through.
pub(crate) fn fast_clamp(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    args: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    let [Value::Integer(a), Value::Integer(lo), Value::Integer(hi)] = args else { return None };
    if nparams < 3 || nresults > 1 || lo > hi {
        return None;
    }
    vm.set(ra, Value::Integer(a.clamp(lo, hi)));
    Some(1)
}

/// `luauF_integermin` and `max`: two or more integers.
fn fast_fold(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    args: [Value; 3],
    nparams: usize,
    keep: fn(i64, i64) -> bool,
) -> Option<usize> {
    let [Value::Integer(a1), Value::Integer(a2), _] = args else { return None };
    if nparams < 2 || nresults > 1 {
        return None;
    }
    let mut r = if keep(a2, a1) { a2 } else { a1 };
    for i in 2..nparams {
        let Value::Integer(x) = arg(vm, ra, &args, i) else { return None };
        if keep(x, r) {
            r = x;
        }
    }
    vm.set(ra, Value::Integer(r));
    Some(1)
}

pub(crate) fn fast_min(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_fold(vm, ra, nresults, args, nparams, |x, r| x < r)
}

pub(crate) fn fast_max(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_fold(vm, ra, nresults, args, nparams, |x, r| x > r)
}

/// `luauF_integerband`, `bor`, `bxor`, `btest`: one or more integers.
fn fast_bits(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    args: [Value; 3],
    nparams: usize,
    op: fn(u64, u64) -> u64,
    out: fn(u64) -> Value,
) -> Option<usize> {
    let [Value::Integer(a0), ..] = args else { return None };
    if nparams < 1 || nresults > 1 {
        return None;
    }
    let mut r = a0 as u64;
    for i in 1..nparams {
        let Value::Integer(x) = arg(vm, ra, &args, i) else { return None };
        r = op(r, x as u64);
    }
    vm.set(ra, out(r));
    Some(1)
}

fn integer(r: u64) -> Value {
    Value::Integer(r as i64)
}

pub(crate) fn fast_band(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_bits(vm, ra, nresults, args, nparams, |a, b| a & b, integer)
}

pub(crate) fn fast_bor(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_bits(vm, ra, nresults, args, nparams, |a, b| a | b, integer)
}

pub(crate) fn fast_bxor(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_bits(vm, ra, nresults, args, nparams, |a, b| a ^ b, integer)
}

pub(crate) fn fast_btest(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_bits(vm, ra, nresults, args, nparams, |a, b| a & b, |r| Value::Boolean(r != 0))
}

/// `luauF_integerextract`: field `f` (0 to 63) of width `w` (default 1,
/// at most 64 with `f + w <= 64`); a non-integer width falls through
/// before anything else.
pub(crate) fn fast_extract(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, a1, a2]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    if nparams >= 3 && !matches!(a2, Value::Integer(_)) {
        return None;
    }
    let (Value::Integer(n), Value::Integer(f)) = (a0, a1) else { return None };
    if nparams < 2 || nresults > 1 {
        return None;
    }
    let w = match a2 {
        Value::Integer(w) if nparams >= 3 => w,
        _ => 1,
    };
    if !(0..=63).contains(&f) || !(1..=64).contains(&w) || f.saturating_add(w) > 64 {
        return None;
    }
    // `w` is 1 to 64, so the shift is 0 to 63.
    let mask = u64::MAX.wrapping_shr(64u32.wrapping_sub(w as u32));
    vm.set(ra, Value::Integer(((n as u64).wrapping_shr(f as u32) & mask) as i64));
    Some(1)
}

#[cfg(test)]
mod tests {
    use crate::fastcall::entry;
    use crate::limits::Limits;
    use crate::value::Value;
    use crate::vm::Core;

    /// Runs fast body `id` on `args` wanting one result: its result, or
    /// `None` when it falls through.
    fn call(id: u8, args: &[Value]) -> Option<Value> {
        let mut vm = Core::with_limits_bare(Limits::default()).ok()?;
        let mut three = [Value::Nil; 3];
        for (slot, &v) in three.iter_mut().zip(args) {
            *slot = v;
        }
        // Arguments past the third sit in the registers above `ra + 3`.
        vm.ensure_stack(16).ok()?;
        for (i, &v) in args.iter().enumerate().skip(3) {
            vm.set(i.checked_add(1)?, v);
        }
        let f = entry(id)?.f;
        let n = f(&mut vm, 0, 1, three, args.len())?;
        assert_eq!(n, 1);
        Some(vm.get(0))
    }

    /// `Value` has no `PartialEq` (NaN); its `Debug` form tells every
    /// result here apart.
    #[track_caller]
    fn same(got: Option<Value>, want: Option<Value>) {
        assert_eq!(format!("{got:?}"), format!("{want:?}"));
    }

    fn i(v: i64) -> Value {
        Value::Integer(v)
    }

    fn b(v: bool) -> Value {
        Value::Boolean(v)
    }

    #[test]
    fn arithmetic() {
        same(call(97, &[i(i64::MAX), i(1)]), Some(i(i64::MIN)));
        same(call(98, &[i(i64::MIN), i(1)]), Some(i(i64::MAX)));
        same(call(99, &[i(1 << 62), i(4)]), Some(i(0)));
        same(call(96, &[i(i64::MIN)]), Some(i(i64::MIN)));
        same(call(100, &[i(-7), i(2)]), Some(i(-3)));
        same(call(100, &[i(1), i(0)]), None);
        same(call(100, &[i(i64::MIN), i(-1)]), None);
        same(call(104, &[i(-7), i(2)]), Some(i(-4)));
        same(call(104, &[i(7), i(-2)]), Some(i(-4)));
        same(call(104, &[i(6), i(-2)]), Some(i(-3)));
        same(call(104, &[i(i64::MIN), i(-1)]), None);
        same(call(103, &[i(-7), i(2)]), Some(i(-1)));
        same(call(103, &[i(i64::MIN), i(-1)]), Some(i(0)));
        same(call(103, &[i(1), i(0)]), None);
        same(call(107, &[i(-7), i(2)]), Some(i(1)));
        same(call(107, &[i(7), i(-2)]), Some(i(-1)));
        same(call(107, &[i(i64::MIN), i(-1)]), Some(i(0)));
        same(call(107, &[i(1), i(0)]), None);
        same(call(105, &[i(-1), i(2)]), Some(i(i64::MAX)));
        same(call(106, &[i(-1), i(10)]), Some(i(5)));
        same(call(105, &[i(1), i(0)]), None);
        same(call(97, &[i(1), Value::Number(1.0)]), None);
        same(call(97, &[i(1)]), None);
    }

    #[test]
    fn conversions_and_bounds() {
        same(call(94, &[Value::Number(3.0)]), Some(i(3)));
        same(call(94, &[Value::Number(3.5)]), Some(Value::Nil));
        same(call(94, &[Value::Number(9_223_372_036_854_775_808.0)]), Some(Value::Nil));
        same(call(94, &[Value::Number(-9_223_372_036_854_775_808.0)]), Some(i(i64::MIN)));
        same(call(94, &[Value::Number(f64::NAN)]), Some(Value::Nil));
        same(call(94, &[i(3)]), None);
        same(call(95, &[i(-5)]), Some(Value::Number(-5.0)));
        same(call(108, &[i(5), i(0), i(3)]), Some(i(3)));
        same(call(108, &[i(-5), i(0), i(3)]), Some(i(0)));
        same(call(108, &[i(5), i(3), i(0)]), None);
        same(call(101, &[i(5), i(3), i(9), i(-2)]), Some(i(-2)));
        same(call(102, &[i(5), i(3), i(9), i(-2)]), Some(i(9)));
        same(call(102, &[i(5), i(3), Value::Number(9.0)]), None);
        same(call(102, &[i(5)]), None);
    }

    #[test]
    fn comparisons() {
        same(call(113, &[i(-1), i(1)]), Some(b(true)));
        same(call(115, &[i(-1), i(1)]), Some(b(false)));
        same(call(114, &[i(1), i(1)]), Some(b(true)));
        same(call(116, &[i(1), i(-1)]), Some(b(true)));
        same(call(117, &[i(1), i(-1)]), Some(b(true)));
        same(call(119, &[i(1), i(-1)]), Some(b(false)));
        same(call(118, &[i(1), i(1)]), Some(b(true)));
        same(call(120, &[i(-1), i(-1)]), Some(b(true)));
    }

    #[test]
    fn bits() {
        same(call(109, &[i(0b1100)]), Some(i(0b1100)));
        same(call(109, &[i(0b1100), i(0b1010), i(0b1000)]), Some(i(0b1000)));
        same(call(110, &[i(1), i(2), i(4), i(8)]), Some(i(15)));
        same(call(112, &[i(3), i(5)]), Some(i(6)));
        same(call(127, &[i(3), i(4)]), Some(b(false)));
        same(call(127, &[i(3)]), Some(b(true)));
        same(call(109, &[i(3), Value::Number(1.0)]), None);
        same(call(111, &[i(0)]), Some(i(-1)));
        same(call(128, &[i(0)]), Some(i(64)));
        same(call(129, &[i(1)]), Some(i(63)));
        same(call(130, &[i(0x0102_0304_0506_0708)]), Some(i(0x0807_0605_0403_0201)));
    }

    #[test]
    fn shifts_and_rotates() {
        same(call(121, &[i(1), i(63)]), Some(i(i64::MIN)));
        same(call(121, &[i(1), i(64)]), Some(i(0)));
        same(call(121, &[i(-1), i(-63)]), Some(i(1)));
        same(call(122, &[i(-1), i(63)]), Some(i(1)));
        same(call(122, &[i(1), i(-2)]), Some(i(4)));
        same(call(122, &[i(1), i(-64)]), Some(i(0)));
        same(call(123, &[i(-8), i(2)]), Some(i(-2)));
        same(call(123, &[i(-8), i(64)]), Some(i(-1)));
        same(call(123, &[i(8), i(64)]), Some(i(0)));
        same(call(123, &[i(1), i(-1)]), Some(i(2)));
        same(call(123, &[i(1), i(-64)]), Some(i(0)));
        same(call(124, &[i(i64::MIN), i(1)]), Some(i(1)));
        same(call(124, &[i(1), i(-1)]), Some(i(i64::MIN)));
        same(call(125, &[i(1), i(1)]), Some(i(i64::MIN)));
        same(call(125, &[i(1), i(64)]), Some(i(1)));
    }

    #[test]
    fn extract() {
        same(call(126, &[i(0b1010), i(1)]), Some(i(1)));
        same(call(126, &[i(0b1010), i(1), i(3)]), Some(i(0b101)));
        same(call(126, &[i(-1), i(0), i(64)]), Some(i(-1)));
        same(call(126, &[i(-1), i(63), i(1)]), Some(i(1)));
        same(call(126, &[i(-1), i(63), i(2)]), None);
        same(call(126, &[i(-1), i(64)]), None);
        same(call(126, &[i(-1), i(-1)]), None);
        same(call(126, &[i(-1), i(0), i(0)]), None);
        same(call(126, &[i(-1), i(0), Value::Number(1.0)]), None);
    }
}
