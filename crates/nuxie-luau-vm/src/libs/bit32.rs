//! The `bit32` library (C++ `lbitlib.cpp`) and its builtins' fast bodies
//! (`fastcall.rs`, C++ `luauF_band` and the rest), which compute what the
//! library functions compute on the inputs they accept and fall through on
//! the others (a shift count outside `0..32`, a bad field).

use crate::error::VmResult;
use crate::fastcall::{arg, fast_number1, fast_number2};
use crate::native::{Args, NativeFn, plain};
use crate::value::Value;
use crate::vm::Core;

const NBITS: i32 = 32;

/// C++ `luai_num2unsigned`: through `int64` to `uint32`.
#[inline]
pub(crate) fn num2unsigned(x: f64) -> u32 {
    (x as i64) as u32
}

impl Core {
    /// C++ `luaL_checkunsigned`: a number (or numeric string) converted
    /// through `int64` to `uint32` (`luai_num2unsigned`).
    pub(crate) fn check_unsigned(&mut self, args: Args, n: usize) -> VmResult<u32> {
        Ok(num2unsigned(self.check_number(args, n)?))
    }
}

fn ret_u32(vm: &mut Core, r: u32) -> VmResult<usize> {
    vm.ret(&[Value::Number(f64::from(r))])
}

fn andaux(vm: &mut Core, args: Args) -> VmResult<u32> {
    let mut r = u32::MAX;
    for i in 1..=args.nargs {
        r &= vm.check_unsigned(args, i)?;
    }
    Ok(r)
}

fn band(vm: &mut Core, args: Args) -> VmResult<usize> {
    let r = andaux(vm, args)?;
    ret_u32(vm, r)
}

fn btest(vm: &mut Core, args: Args) -> VmResult<usize> {
    let r = andaux(vm, args)?;
    vm.ret(&[Value::Boolean(r != 0)])
}

fn bor(vm: &mut Core, args: Args) -> VmResult<usize> {
    let mut r = 0u32;
    for i in 1..=args.nargs {
        r |= vm.check_unsigned(args, i)?;
    }
    ret_u32(vm, r)
}

fn bxor(vm: &mut Core, args: Args) -> VmResult<usize> {
    let mut r = 0u32;
    for i in 1..=args.nargs {
        r ^= vm.check_unsigned(args, i)?;
    }
    ret_u32(vm, r)
}

fn bnot(vm: &mut Core, args: Args) -> VmResult<usize> {
    let r = !vm.check_unsigned(args, 1)?;
    ret_u32(vm, r)
}

/// C++ `b_shift`: a negative count shifts right; `|i| >= 32` gives 0.
fn shift(r: u32, i: i32) -> u32 {
    if i < 0 {
        r.checked_shr(i.unsigned_abs()).unwrap_or(0)
    } else {
        r.checked_shl(i.unsigned_abs()).unwrap_or(0)
    }
}

fn lshift(vm: &mut Core, args: Args) -> VmResult<usize> {
    let r = vm.check_unsigned(args, 1)?;
    let i = vm.check_integer(args, 2)?;
    ret_u32(vm, shift(r, i))
}

fn rshift(vm: &mut Core, args: Args) -> VmResult<usize> {
    let r = vm.check_unsigned(args, 1)?;
    let i = vm.check_integer(args, 2)?;
    ret_u32(vm, shift(r, i.wrapping_neg()))
}

fn arshift(vm: &mut Core, args: Args) -> VmResult<usize> {
    let r = vm.check_unsigned(args, 1)?;
    let i = vm.check_integer(args, 2)?;
    if i < 0 || r & (1 << 31) == 0 {
        return ret_u32(vm, shift(r, i.wrapping_neg()));
    }
    // An arithmetic shift of a "negative" number.
    let r = if i >= NBITS {
        u32::MAX
    } else {
        let i = i.unsigned_abs();
        (r >> i) | !(u32::MAX >> i)
    };
    ret_u32(vm, r)
}

/// C++ `b_rot`: the count is taken mod 32.
fn rot(vm: &mut Core, args: Args, i: i32) -> VmResult<usize> {
    let r = vm.check_unsigned(args, 1)?;
    ret_u32(vm, r.rotate_left((i & (NBITS - 1)).unsigned_abs()))
}

fn lrotate(vm: &mut Core, args: Args) -> VmResult<usize> {
    let i = vm.check_integer(args, 2)?;
    rot(vm, args, i)
}

fn rrotate(vm: &mut Core, args: Args) -> VmResult<usize> {
    let i = vm.check_integer(args, 2)?;
    rot(vm, args, i.wrapping_neg())
}

/// C++ `fieldargs`: the field position at `farg` and its width at
/// `farg + 1` (default 1).
fn fieldargs(vm: &mut Core, args: Args, farg: usize) -> VmResult<(u32, u32)> {
    let f = vm.check_integer(args, farg)?;
    let w = vm.opt_integer(args, farg.saturating_add(1), 1)?;
    if f < 0 {
        return Err(vm.arg_error(farg, "field cannot be negative"));
    }
    if w <= 0 {
        return Err(vm.arg_error(farg.saturating_add(1), "width must be positive"));
    }
    if i64::from(f).saturating_add(i64::from(w)) > i64::from(NBITS) {
        return Err(vm.lib_error("trying to access non-existent bits"));
    }
    Ok((f.unsigned_abs(), w.unsigned_abs()))
}

/// C++ `mask(w)`: the low `w` bits, for `1 <= w <= 32`.
fn mask(w: u32) -> u32 {
    u32::MAX.checked_shr(NBITS.unsigned_abs().saturating_sub(w)).unwrap_or(0)
}

fn extract(vm: &mut Core, args: Args) -> VmResult<usize> {
    let r = vm.check_unsigned(args, 1)?;
    let (f, w) = fieldargs(vm, args, 2)?;
    ret_u32(vm, r.checked_shr(f).unwrap_or(0) & mask(w))
}

fn replace(vm: &mut Core, args: Args) -> VmResult<usize> {
    let r = vm.check_unsigned(args, 1)?;
    let v = vm.check_unsigned(args, 2)?;
    let (f, w) = fieldargs(vm, args, 3)?;
    let m = mask(w);
    let shifted_mask = m.checked_shl(f).unwrap_or(0);
    let r = (r & !shifted_mask) | (v & m).checked_shl(f).unwrap_or(0);
    ret_u32(vm, r)
}

fn countlz(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_unsigned(args, 1)?;
    ret_u32(vm, v.leading_zeros())
}

fn countrz(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_unsigned(args, 1)?;
    ret_u32(vm, v.trailing_zeros())
}

fn byteswap(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_unsigned(args, 1)?;
    ret_u32(vm, v.swap_bytes())
}

// ---- fast bodies -------------------------------------------------------

fn number(r: u32) -> Value {
    Value::Number(f64::from(r))
}

/// `luauF_band`, `bor`, `bxor`, `btest`: two or more numbers folded with
/// `op`, the result made by `out`.
fn fast_fold(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    args: [Value; 3],
    nparams: usize,
    op: fn(u32, u32) -> u32,
    out: fn(u32) -> Value,
) -> Option<usize> {
    let [Value::Number(a1), Value::Number(a2), _] = args else { return None };
    if nparams < 2 || nresults > 1 {
        return None;
    }
    let mut r = op(num2unsigned(a1), num2unsigned(a2));
    for i in 2..nparams {
        let Value::Number(x) = arg(vm, ra, &args, i) else { return None };
        r = op(r, num2unsigned(x));
    }
    vm.set(ra, out(r));
    Some(1)
}

pub(crate) fn fast_band(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_fold(vm, ra, nresults, args, nparams, |a, b| a & b, number)
}

pub(crate) fn fast_bor(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_fold(vm, ra, nresults, args, nparams, |a, b| a | b, number)
}

pub(crate) fn fast_bxor(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_fold(vm, ra, nresults, args, nparams, |a, b| a ^ b, number)
}

pub(crate) fn fast_btest(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_fold(vm, ra, nresults, args, nparams, |a, b| a & b, |r| Value::Boolean(r != 0))
}

fast_number1!(fast_bnot, |x| number(!num2unsigned(x)));
fast_number1!(fast_countlz, |x| number(num2unsigned(x).leading_zeros()));
fast_number1!(fast_countrz, |x| number(num2unsigned(x).trailing_zeros()));
fast_number1!(fast_byteswap, |x| number(num2unsigned(x).swap_bytes()));
fast_number2!(fast_lrotate, |x, s| number(num2unsigned(x).rotate_left(s as i32 as u32)));
fast_number2!(fast_rrotate, |x, s| number(num2unsigned(x).rotate_right(s as i32 as u32)));

/// `luauF_lshift`, `rshift`, `arshift`: a count in `0..32` (read as
/// unsigned, so a negative one falls through), applied by `op`.
fn fast_shift(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, a1, _]: [Value; 3],
    nparams: usize,
    op: fn(u32, u32) -> u32,
) -> Option<usize> {
    let (Value::Number(x), Value::Number(s)) = (a0, a1) else { return None };
    if nparams < 2 || nresults > 1 {
        return None;
    }
    let s = s as i32 as u32;
    if s >= 32 {
        return None;
    }
    vm.set(ra, number(op(num2unsigned(x), s)));
    Some(1)
}

pub(crate) fn fast_lshift(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_shift(vm, ra, nresults, args, nparams, |u, s| u.wrapping_shl(s))
}

pub(crate) fn fast_rshift(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_shift(vm, ra, nresults, args, nparams, |u, s| u.wrapping_shr(s))
}

pub(crate) fn fast_arshift(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_shift(vm, ra, nresults, args, nparams, |u, s| (u as i32).wrapping_shr(s) as u32)
}

/// `!(0xfffffffe << (w - 1))`: the low `w` bits for `1 <= w <= 32`.
fn fast_mask(w: i32) -> u32 {
    !0xffff_fffe_u32.wrapping_shl(w.wrapping_sub(1) as u32)
}

/// A field `f` of width `w` passes `luauF_extract`'s and `replace`'s test.
fn field_ok(f: i32, w: i32) -> bool {
    f >= 0 && w > 0 && i64::from(f).saturating_add(i64::from(w)) <= 32
}

/// `luauF_extract`: the width defaults to 1; a bad field falls through, so
/// the library raises.
pub(crate) fn fast_extract(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, a1, a2]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    let (Value::Number(x), Value::Number(f)) = (a0, a1) else { return None };
    if nparams < 2 || nresults > 1 {
        return None;
    }
    let n = num2unsigned(x);
    let f = f as i32;
    let r = if nparams == 2 {
        if f as u32 >= 32 {
            return None;
        }
        n.wrapping_shr(f as u32) & 1
    } else {
        let Value::Number(w) = a2 else { return None };
        let w = w as i32;
        if !field_ok(f, w) {
            return None;
        }
        n.wrapping_shr(f as u32) & fast_mask(w)
    };
    vm.set(ra, number(r));
    Some(1)
}

/// `luauF_extractk`: the field and width packed by the compiler into one
/// constant, `f | (w - 1) << 5`.
pub(crate) fn fast_extractk(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, a1, _]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    let (Value::Number(x), Value::Number(fw)) = (a0, a1) else { return None };
    if nparams < 2 || nresults > 1 {
        return None;
    }
    let fw = fw as i32;
    let f = (fw & 31) as u32;
    let w1 = (fw >> 5) as u32;
    let m = !0xffff_fffe_u32.wrapping_shl(w1);
    vm.set(ra, number(num2unsigned(x).wrapping_shr(f) & m));
    Some(1)
}

/// `luauF_replace`: the width defaults to 1; a bad field falls through.
pub(crate) fn fast_replace(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    args: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    let [Value::Number(x), Value::Number(v), Value::Number(f)] = args else { return None };
    if nparams < 3 || nresults > 1 {
        return None;
    }
    let n = num2unsigned(x);
    let v = num2unsigned(v);
    let f = f as i32;
    let m = if nparams == 3 {
        if f as u32 >= 32 {
            return None;
        }
        1
    } else {
        let Value::Number(w) = arg(vm, ra, &args, 3) else { return None };
        let w = w as i32;
        if !field_ok(f, w) {
            return None;
        }
        fast_mask(w)
    };
    let f = f as u32;
    let r = (n & !m.wrapping_shl(f)) | (v & m).wrapping_shl(f);
    vm.set(ra, number(r));
    Some(1)
}

pub(crate) fn open(vm: &mut Core) -> VmResult<()> {
    let functions: &[(&str, NativeFn)] = &[
        ("arshift", plain(arshift)),
        ("band", plain(band)),
        ("bnot", plain(bnot)),
        ("bor", plain(bor)),
        ("bxor", plain(bxor)),
        ("btest", plain(btest)),
        ("extract", plain(extract)),
        ("lrotate", plain(lrotate)),
        ("lshift", plain(lshift)),
        ("replace", plain(replace)),
        ("rrotate", plain(rrotate)),
        ("rshift", plain(rshift)),
        ("countlz", plain(countlz)),
        ("countrz", plain(countrz)),
        ("byteswap", plain(byteswap)),
    ];
    vm.open_lib("bit32", functions)?;
    Ok(())
}
