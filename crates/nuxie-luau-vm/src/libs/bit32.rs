//! The `bit32` library (C++ `lbitlib.cpp`).
//!
//! Every builtin's fast body (`luauF_band` and the rest) computes what its
//! library function computes on the inputs it accepts, so the library
//! functions alone serve both paths (`libs/fast.rs`).

use crate::error::VmResult;
use crate::native::{Args, NativeFn, plain};
use crate::value::Value;
use crate::vm::Core;

const NBITS: i32 = 32;

impl Core {
    /// C++ `luaL_checkunsigned`: a number (or numeric string) converted
    /// through `int64` to `uint32` (`luai_num2unsigned`).
    pub(crate) fn check_unsigned(&mut self, args: Args, n: usize) -> VmResult<u32> {
        Ok((self.check_number(args, n)? as i64) as u32)
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
