//! The `utf8` library (C++ `lutf8lib.cpp`).
//!
//! C reads strings through their NUL terminator; here a read past the end
//! yields 0, which is the same byte.
//!
//! Allocating builtins: `char` (each code point is encoded into a 4-byte
//! scratch, then checked with `check_buffer` before the result grows) and `codes` (a step
//! closure, charged by `Heap::new_function`).

use crate::error::VmResult;
use crate::libs::check_buffer;
use crate::native::{Args, NativeFn, plain};
use crate::value::Value;
use crate::vm::Core;

const MAXUNICODE: i32 = 0x10_FFFF;
/// C++ `LUAI_MAXCSTACK`: the most values `luaL_checkstack` allows.
const MAX_C_STACK: usize = 8000;
/// `utf8.charpattern`, `"[\0-\x7F\xC2-\xF4][\x80-\xBF]*"`.
const CHARPATTERN: &[u8] = b"[\0-\x7F\xC2-\xF4][\x80-\xBF]*";

/// The byte at `i`, or the terminating NUL past the end.
fn at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

fn iscont(s: &[u8], i: usize) -> bool {
    at(s, i) & 0xC0 == 0x80
}

/// C++ `u_posrelat`: a negative position counts from the end.
fn posrelat(pos: i32, len: usize) -> i64 {
    let pos = i64::from(pos);
    let len = len as i64;
    if pos >= 0 {
        pos
    } else if pos.saturating_neg() > len {
        0
    } else {
        len.saturating_add(pos).saturating_add(1)
    }
}

/// C++ `utf8_decode`: the code point at `i` and the index after it, or
/// `None` for an invalid sequence (overlong, surrogate, past U+10FFFF).
fn decode(s: &[u8], i: usize) -> Option<(u32, usize)> {
    const LIMITS: [u32; 4] = [0xFF, 0x7F, 0x7FF, 0xFFFF];
    let c = u32::from(at(s, i));
    if c < 0x80 {
        return Some((c, i.saturating_add(1)));
    }
    let mut res: u32 = 0;
    let mut count = 0usize;
    let mut cv = c;
    while cv & 0x40 != 0 {
        count = count.saturating_add(1);
        let cc = u32::from(at(s, i.saturating_add(count)));
        if cc & 0xC0 != 0x80 {
            return None;
        }
        res = (res << 6) | (cc & 0x3F);
        cv <<= 1;
    }
    let shift = u32::try_from(count.saturating_mul(5)).unwrap_or(u32::MAX);
    res |= (cv & 0x7F).checked_shl(shift).unwrap_or(0);
    let limit = LIMITS.get(count)?;
    if res > MAXUNICODE as u32 || res <= *limit || res.wrapping_sub(0xD800) < 0x800 {
        return None;
    }
    Some((res, i.saturating_add(count).saturating_add(1)))
}

/// C++ `luaO_utf8esc`: the UTF-8 bytes of `x` (surrogates included).
fn encode(x: u32, out: &mut Vec<u8>) {
    if x < 0x80 {
        out.push(x as u8);
        return;
    }
    let mut tail = [0u8; 4];
    let mut n = 0usize;
    let mut x = x;
    let mut mfb: u32 = 0x3f;
    loop {
        if let Some(slot) = tail.get_mut(n) {
            *slot = 0x80 | (x & 0x3f) as u8;
        }
        n = n.saturating_add(1);
        x >>= 6;
        mfb >>= 1;
        if x <= mfb {
            break;
        }
    }
    out.push(((!mfb << 1) | x) as u8);
    out.extend(tail.iter().take(n).rev());
}

fn len(vm: &mut Core, args: Args) -> VmResult<usize> {
    let s = vm.check_string(args, 1)?;
    let bytes = vm.heap.str_rc(s);
    let l = bytes.len();
    let posi = posrelat(vm.opt_integer(args, 2, 1)?, l);
    let posj = posrelat(vm.opt_integer(args, 3, -1)?, l);
    if !(1 <= posi && posi <= (l as i64).saturating_add(1)) {
        return Err(vm.arg_error(2, "initial position out of string"));
    }
    let posj = posj.saturating_sub(1);
    if posj >= l as i64 {
        return Err(vm.arg_error(3, "final position out of string"));
    }
    let mut pos = posi.saturating_sub(1);
    let mut n = 0i32;
    while pos <= posj {
        match decode(&bytes, pos as usize) {
            Some((_, next)) => pos = next as i64,
            None => {
                return vm.ret(&[Value::Nil, Value::Number(pos.saturating_add(1) as f64)]);
            }
        }
        n = n.saturating_add(1);
    }
    vm.ret(&[Value::Number(f64::from(n))])
}

fn codepoint(vm: &mut Core, args: Args) -> VmResult<usize> {
    let s = vm.check_string(args, 1)?;
    let bytes = vm.heap.str_rc(s);
    let l = bytes.len();
    let posi = posrelat(vm.opt_integer(args, 2, 1)?, l);
    let default = i32::try_from(posi).unwrap_or(i32::MAX);
    let pose = posrelat(vm.opt_integer(args, 3, default)?, l);
    if posi < 1 {
        return Err(vm.arg_error(2, "out of range"));
    }
    if pose > l as i64 {
        return Err(vm.arg_error(3, "out of range"));
    }
    if posi > pose {
        return Ok(0);
    }
    let n = pose.saturating_sub(posi).saturating_add(1) as usize;
    if n > MAX_C_STACK || args.nargs.saturating_add(n) > MAX_C_STACK {
        return Err(vm.lib_error("stack overflow (string slice too long)"));
    }
    vm.ensure_stack(vm.exec.top.saturating_add(n))?;
    let end = pose as usize;
    let mut i = posi.saturating_sub(1) as usize;
    let mut count = 0usize;
    while i < end {
        let Some((code, next)) = decode(&bytes, i) else {
            return Err(vm.lib_error("invalid UTF-8 code"));
        };
        vm.push(Value::Number(f64::from(code)))?;
        count = count.saturating_add(1);
        i = next;
    }
    Ok(count)
}

/// C++ `buffutfchar`: argument `n` as a code point, appended to `out`.
fn utfchar_arg(vm: &mut Core, args: Args, n: usize, out: &mut Vec<u8>) -> VmResult<()> {
    let code = vm.check_integer(args, n)?;
    if !(0..=MAXUNICODE).contains(&code) {
        return Err(vm.arg_error(n, "value out of range"));
    }
    encode(code.unsigned_abs(), out);
    Ok(())
}

fn char_(vm: &mut Core, args: Args) -> VmResult<usize> {
    let mut out = Vec::new();
    let mut one = Vec::with_capacity(4);
    for i in 1..=args.nargs {
        one.clear();
        utfchar_arg(vm, args, i, &mut one)?;
        check_buffer(vm, out.len().saturating_add(one.len()))?;
        out.extend_from_slice(&one);
    }
    vm.ret_str(&out)
}

fn offset(vm: &mut Core, args: Args) -> VmResult<usize> {
    let s = vm.check_string(args, 1)?;
    let bytes = vm.heap.str_rc(s);
    let l = bytes.len() as i64;
    let mut n = i64::from(vm.check_integer(args, 2)?);
    let default = if n >= 0 { 1 } else { l.saturating_add(1) };
    let default = i32::try_from(default).unwrap_or(i32::MAX);
    let posi = posrelat(vm.opt_integer(args, 3, default)?, bytes.len());
    if !(1 <= posi && posi <= l.saturating_add(1)) {
        return Err(vm.arg_error(3, "position out of range"));
    }
    let mut posi = posi.saturating_sub(1);
    let cont = |p: i64| iscont(&bytes, p as usize);
    if n == 0 {
        while posi > 0 && cont(posi) {
            posi = posi.saturating_sub(1);
        }
    } else {
        if cont(posi) {
            return Err(vm.lib_error("initial position is a continuation byte"));
        }
        if n < 0 {
            while n < 0 && posi > 0 {
                loop {
                    posi = posi.saturating_sub(1);
                    if !(posi > 0 && cont(posi)) {
                        break;
                    }
                }
                n = n.saturating_add(1);
            }
        } else {
            n = n.saturating_sub(1);
            while n > 0 && posi < l {
                loop {
                    posi = posi.saturating_add(1);
                    if !cont(posi) {
                        break;
                    }
                }
                n = n.saturating_sub(1);
            }
        }
    }
    if n == 0 {
        vm.ret(&[Value::Number(posi.saturating_add(1) as f64)])
    } else {
        vm.ret(&[Value::Nil])
    }
}

/// C++ `iter_aux`: the step function `codes` returns.
fn iter_aux(vm: &mut Core, args: Args) -> VmResult<usize> {
    let s = vm.check_string(args, 1)?;
    let bytes = vm.heap.str_rc(s);
    let l = bytes.len() as i64;
    // `lua_tointeger`: 0 for a non-number.
    let control = vm
        .arg(args, 2)
        .and_then(|v| vm.to_number(v))
        .map_or(0, |x| x as i32);
    let mut n = i64::from(control).saturating_sub(1);
    if n < 0 {
        n = 0;
    } else if n < l {
        n = n.saturating_add(1);
        while iscont(&bytes, n as usize) {
            n = n.saturating_add(1);
        }
    }
    if n >= l {
        return Ok(0);
    }
    match decode(&bytes, n as usize) {
        Some((code, next)) if !iscont(&bytes, next) => vm.ret(&[
            Value::Number(n.saturating_add(1) as f64),
            Value::Number(f64::from(code)),
        ]),
        _ => Err(vm.lib_error("invalid UTF-8 code")),
    }
}

fn codes(vm: &mut Core, args: Args) -> VmResult<usize> {
    let s = vm.check_string(args, 1)?;
    // A new step closure per call, as C++ `lua_pushcfunction` makes.
    let f = vm.new_builtin("", plain(iter_aux), &[])?;
    vm.ret(&[f, Value::String(s), Value::Number(0.0)])
}

pub(crate) fn open(vm: &mut Core) -> VmResult<()> {
    let functions: &[(&str, NativeFn)] = &[
        ("offset", plain(offset)),
        ("codepoint", plain(codepoint)),
        ("char", plain(char_)),
        ("len", plain(len)),
        ("codes", plain(codes)),
    ];
    let t = vm.open_lib("utf8", functions)?;
    let pattern = vm.heap.intern(CHARPATTERN)?;
    vm.raw_set_str(t, "charpattern", Value::String(pattern))?;
    Ok(())
}
