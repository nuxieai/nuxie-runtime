//! Builtin fast bodies (C++ `lbuiltins.cpp`, `luauF_*`) that compute a
//! different result from their library function.
//!
//! Under `safeenv`, a compiled call to a builtin runs its fast body from
//! the `FASTCALL*` instruction and skips the `CALL` (`interp.rs`). Most
//! fast bodies compute what the library function computes, so this VM
//! falls through to the `CALL` for them. The ones listed here differ on
//! some inputs; each was found by reading the vendored `luau_f_*` body
//! against its library function, and each has a `tests/luau-vm-diff` case
//! that calls the builtin both ways:
//!
//! - `math.modf` (`fastcall.luau`): an integral argument has a signed-zero
//!   fraction (the library gives `+0` for negatives and NaN for infinities).
//! - `math.ldexp` (`mathlib.luau`): the fast body is C `ldexp`; the library
//!   body multiplies by `powi(2, e)`, which overflows or flushes for
//!   `|e| > 1023`.
//! - `vector.magnitude` and `vector.dot` (`vectorlib.luau`): the fast
//!   bodies sum with fused multiply-adds, the library bodies do not.
//!
//! A fast body that does not apply (wrong types or counts) returns `None`
//! and the `CALL` runs, as upstream's `-1` does.

use crate::libs::math::ldexp_fast;
use crate::libs::vector::{dot_fast, magnitude_fast};
use crate::value::Value;

/// `LBF_MATH_LDEXP`.
const MATH_LDEXP: u8 = 15;
/// `LBF_MATH_MODF`.
const MATH_MODF: u8 = 20;
/// `LBF_VECTOR_MAGNITUDE`.
const VECTOR_MAGNITUDE: u8 = 78;
/// `LBF_VECTOR_DOT`.
const VECTOR_DOT: u8 = 81;

/// Whether builtin `id` has a fast body here.
#[inline]
pub(crate) fn has_fast_body(id: u8) -> bool {
    matches!(id, MATH_LDEXP | MATH_MODF | VECTOR_MAGNITUDE | VECTOR_DOT)
}

/// Runs builtin `id`'s fast body on `arg0` and the following arguments
/// (`rest`, valid up to `nparams`), for a call that wants `nresults`
/// (`-1` for all). Returns the results and their count.
pub(crate) fn run(
    id: u8,
    arg0: Value,
    rest: [Value; 2],
    nparams: usize,
    nresults: i32,
) -> Option<([Value; 2], usize)> {
    match id {
        MATH_MODF => {
            let Value::Number(x) = arg0 else { return None };
            if nparams < 1 || nresults > 2 {
                return None;
            }
            let ip = x.trunc();
            let fp = if x == ip { 0.0f64.copysign(x) } else { x - ip };
            Some(([Value::Number(ip), Value::Number(fp)], 2))
        }
        MATH_LDEXP => {
            let (Value::Number(x), [Value::Number(e), _]) = (arg0, rest) else {
                return None;
            };
            if nparams < 2 || nresults > 1 {
                return None;
            }
            Some(([Value::Number(ldexp_fast(x, e as i32)), Value::Nil], 1))
        }
        VECTOR_MAGNITUDE => {
            let Value::Vector(v) = arg0 else { return None };
            if nparams < 1 || nresults > 1 {
                return None;
            }
            Some(([Value::Number(magnitude_fast(v)), Value::Nil], 1))
        }
        VECTOR_DOT => {
            let (Value::Vector(a), [Value::Vector(b), _]) = (arg0, rest) else {
                return None;
            };
            if nparams < 2 || nresults > 1 {
                return None;
            }
            Some(([Value::Number(dot_fast(a, b)), Value::Nil], 1))
        }
        _ => None,
    }
}
