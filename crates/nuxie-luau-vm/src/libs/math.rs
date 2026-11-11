//! The math library (C++ `lmathlib.cpp`, as vendored in the Rive profile:
//! `fround` and the `isnan`/`isinf`/`isfinite` predicates included).
//!
//! `random` is upstream's PCG32 with its seeding, so a seeded sequence is
//! the vendored VM's. The unseeded state differs: upstream seeds from the
//! state's address, this VM from a fixed constant (`Core::rng_state`).

use crate::error::VmResult;
use crate::native::{Args, NativeFn, plain};
use crate::value::Value;
use crate::vm::Core;

macro_rules! unary {
    ($name:ident, $f:expr) => {
        fn $name(vm: &mut Core, args: Args) -> VmResult<usize> {
            let x = vm.check_number(args, 1)?;
            let f: fn(f64) -> f64 = $f;
            vm.ret(&[Value::Number(f(x))])
        }
    };
}

unary!(abs, f64::abs);
unary!(ceil, f64::ceil);
unary!(floor, f64::floor);
unary!(sqrt, f64::sqrt);
unary!(sin, f64::sin);
unary!(cos, f64::cos);
unary!(tan, f64::tan);
unary!(asin, f64::asin);
unary!(acos, f64::acos);
unary!(exp, f64::exp);
unary!(log10, f64::log10);
unary!(sinh, f64::sinh);
unary!(cosh, f64::cosh);
unary!(tanh, f64::tanh);
// Upstream divides by RADIANS_PER_DEGREE; multiplying by its inverse rounds
// differently.
unary!(deg, |x| x / (std::f64::consts::PI / 180.0));
unary!(rad, |x| x * (std::f64::consts::PI / 180.0));
unary!(sign, |x| if x > 0.0 {
    1.0
} else if x < 0.0 {
    -1.0
} else {
    0.0
});
// C++ `math.round`: half away from zero.
unary!(round, f64::round);

fn atan(vm: &mut Core, args: Args) -> VmResult<usize> {
    let y = vm.check_number(args, 1)?;
    vm.ret(&[Value::Number(y.atan())])
}

fn atan2(vm: &mut Core, args: Args) -> VmResult<usize> {
    let y = vm.check_number(args, 1)?;
    let x = vm.check_number(args, 2)?;
    vm.ret(&[Value::Number(y.atan2(x))])
}

fn pow(vm: &mut Core, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    let y = vm.check_number(args, 2)?;
    vm.ret(&[Value::Number(x.powf(y))])
}

fn fmod(vm: &mut Core, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    let y = vm.check_number(args, 2)?;
    vm.ret(&[Value::Number(x % y)])
}

fn modf(vm: &mut Core, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    // The library body (`math_modf`): `x - trunc(x)`, so an infinity has a
    // NaN fraction and a negative integer a `+0` one. A compiled
    // `math.modf(x)` under safeenv runs the fast body in `interp.rs`
    // instead, as upstream does.
    let ip = x.trunc();
    let fp = x - ip;
    vm.ret(&[Value::Number(ip), Value::Number(fp)])
}

fn log(vm: &mut Core, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    let r = if vm.arg_or_nil(args, 2).is_nil() {
        x.ln()
    } else {
        let base = vm.check_number(args, 2)?;
        if base == 2.0 {
            x.log2()
        } else if base == 10.0 {
            x.log10()
        } else {
            x.ln() / base.ln()
        }
    };
    vm.ret(&[Value::Number(r)])
}

fn min(vm: &mut Core, args: Args) -> VmResult<usize> {
    let mut m = vm.check_number(args, 1)?;
    for i in 2..=args.nargs {
        let x = vm.check_number(args, i)?;
        if x < m {
            m = x;
        }
    }
    vm.ret(&[Value::Number(m)])
}

fn max(vm: &mut Core, args: Args) -> VmResult<usize> {
    let mut m = vm.check_number(args, 1)?;
    for i in 2..=args.nargs {
        let x = vm.check_number(args, i)?;
        if x > m {
            m = x;
        }
    }
    vm.ret(&[Value::Number(m)])
}

fn clamp(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_number(args, 1)?;
    let lo = vm.check_number(args, 2)?;
    let hi = vm.check_number(args, 3)?;
    // Upstream checks `min <= max`, so a NaN bound is refused too.
    let ordered = lo <= hi;
    if !ordered {
        return Err(vm.arg_error(3, "max must be greater than or equal to min"));
    }
    let r = if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    };
    vm.ret(&[Value::Number(r)])
}

fn frexp(vm: &mut Core, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    let (m, e) = frexp_parts(x);
    vm.ret(&[Value::Number(m), Value::Number(f64::from(e))])
}

/// C `frexp`: `x = m * 2^e` with `0.5 <= |m| < 1`; zero, infinities and
/// NaN come back unchanged with `e = 0`. The library body and the fast body
/// (`luauF_frexp`) compute the same pair.
pub(crate) fn frexp_parts(x: f64) -> (f64, i32) {
    if !x.is_finite() || x == 0.0 {
        return (x, 0);
    }
    let bits = x.to_bits();
    let mut exponent = ((bits >> 52) & 0x7ff) as i32;
    let mut mantissa = bits & 0x000f_ffff_ffff_ffff;
    if exponent == 0 {
        // Subnormal: normalize by 2^54 first.
        let norm = (x * 18_014_398_509_481_984.0).to_bits();
        exponent = (((norm >> 52) & 0x7ff) as i32).saturating_sub(54);
        mantissa = norm & 0x000f_ffff_ffff_ffff;
    }
    let sign = bits & (1 << 63);
    (
        f64::from_bits(sign | (0x3fe << 52) | mantissa),
        exponent.saturating_sub(1022),
    )
}

fn ldexp(vm: &mut Core, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    let e = vm.check_integer(args, 2)?;
    // The vendored library body is `x * 2^e` through `powi`, which
    // overflows (or flushes to zero) for `|e| > 1023` even when the result
    // is representable. A compiled `math.ldexp` under safeenv runs the
    // exact fast body (`ldexp_fast`) instead.
    vm.ret(&[Value::Number(x * 2.0f64.powi(e))])
}

/// `luauF_ldexp`'s body: C `ldexp`, scaling in at most three steps so an
/// intermediate never overflows or loses a subnormal's bits early.
pub(crate) fn ldexp_fast(mut value: f64, mut exponent: i32) -> f64 {
    const MAX_EXPONENT: i32 = 1023;
    const MIN_EXPONENT: i32 = -1022;
    let max_scale = f64::from_bits(0x7fe0_0000_0000_0000);
    let min_scale = f64::from_bits(0x0010_0000_0000_0000) * f64::from_bits(0x4340_0000_0000_0000);
    // -MIN_EXPONENT - 53
    const ADJUSTMENT: i32 = 969;
    if exponent > MAX_EXPONENT {
        value *= max_scale;
        exponent = exponent.saturating_sub(MAX_EXPONENT);
        if exponent > MAX_EXPONENT {
            value *= max_scale;
            exponent = exponent.saturating_sub(MAX_EXPONENT).min(MAX_EXPONENT);
        }
    } else if exponent < MIN_EXPONENT {
        value *= min_scale;
        exponent = exponent.saturating_add(ADJUSTMENT);
        if exponent < MIN_EXPONENT {
            value *= min_scale;
            exponent = exponent.saturating_add(ADJUSTMENT).max(MIN_EXPONENT);
        }
    }
    // `exponent` is in MIN_EXPONENT..=MAX_EXPONENT, so the biased exponent
    // is in 1..=2046.
    let biased = u64::try_from(exponent.saturating_add(1023)).unwrap_or(1);
    value * f64::from_bits(biased << 52)
}

fn fround(vm: &mut Core, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    vm.ret(&[Value::Number(f64::from(x as f32))])
}

fn map(vm: &mut Core, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    let inmin = vm.check_number(args, 2)?;
    let inmax = vm.check_number(args, 3)?;
    let outmin = vm.check_number(args, 4)?;
    let outmax = vm.check_number(args, 5)?;
    let r = outmin + (x - inmin) * (outmax - outmin) / (inmax - inmin);
    vm.ret(&[Value::Number(r)])
}

fn lerp(vm: &mut Core, args: Args) -> VmResult<usize> {
    let a = vm.check_number(args, 1)?;
    let b = vm.check_number(args, 2)?;
    let t = vm.check_number(args, 3)?;
    // The fast body (`luauF_lerp`) is the same expression.
    let r = if t == 1.0 { b } else { a + (b - a) * t };
    vm.ret(&[Value::Number(r)])
}

fn isnan(vm: &mut Core, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    vm.ret(&[Value::Boolean(x.is_nan())])
}

fn isinf(vm: &mut Core, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    vm.ret(&[Value::Boolean(x.is_infinite())])
}

fn isfinite(vm: &mut Core, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    vm.ret(&[Value::Boolean(x.is_finite())])
}

// ---- random (PCG32, upstream `pcg32_random` / `pcg32_seed`) -----------

const PCG32_INC: u64 = 105;

pub(crate) fn pcg32_random(state: &mut u64) -> u32 {
    let old = *state;
    *state = old
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(PCG32_INC | 1);
    let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
    let rot = (old >> 59) as u32;
    xorshifted.rotate_right(rot)
}

pub(crate) fn pcg32_seed(state: &mut u64, seed: u64) {
    *state = 0;
    pcg32_random(state);
    *state = state.wrapping_add(seed);
    pcg32_random(state);
}

fn random(vm: &mut Core, args: Args) -> VmResult<usize> {
    let r = match args.nargs {
        0 => {
            let lo = u64::from(pcg32_random(&mut vm.rng_state));
            let hi = u64::from(pcg32_random(&mut vm.rng_state));
            // ldexp(double(lo | hi << 32), -64)
            (lo | (hi << 32)) as f64 * 2.0f64.powi(-64)
        }
        1 => {
            let u = vm.check_integer(args, 1)?;
            if u < 1 {
                return Err(vm.arg_error(1, "interval is empty"));
            }
            let x = u64::from(u as u32).wrapping_mul(u64::from(pcg32_random(&mut vm.rng_state)));
            f64::from((x >> 32).wrapping_add(1) as i32)
        }
        2 => {
            let l = vm.check_integer(args, 1)?;
            let u = vm.check_integer(args, 2)?;
            if l > u {
                return Err(vm.arg_error(2, "interval is empty"));
            }
            let ul = (u as u32).wrapping_sub(l as u32);
            if ul == u32::MAX {
                return Err(vm.arg_error(2, "interval is too large"));
            }
            let x = u64::from(ul)
                .wrapping_add(1)
                .wrapping_mul(u64::from(pcg32_random(&mut vm.rng_state)));
            f64::from(i64::from(l).wrapping_add((x >> 32) as i64) as i32)
        }
        _ => return Err(vm.lib_error("wrong number of arguments")),
    };
    vm.ret(&[Value::Number(r)])
}

fn randomseed(vm: &mut Core, args: Args) -> VmResult<usize> {
    // `(unsigned long long)luaL_checkinteger`: an `int`, sign-extended.
    let seed = i64::from(vm.check_integer(args, 1)?) as u64;
    pcg32_seed(&mut vm.rng_state, seed);
    Ok(0)
}

// ---- noise (upstream's Perlin) ----------------------------------------

#[rustfmt::skip]
const PERLIN_HASH: [u8; 256] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148,
    247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54,
    65, 25, 63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173, 186, 3, 64,
    52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213,
    119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104,
    218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157,
    184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
];

/// Upstream's 512-entry table is this one twice; indices are taken mod 256.
fn perm(i: i32) -> i32 {
    i32::from(PERLIN_HASH.get((i & 255) as usize).copied().unwrap_or(0))
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn plerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

fn grad(hash: i32, x: f32, y: f32, z: f32) -> f32 {
    const GRAD: [[f32; 3]; 16] = [
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [1.0, -1.0, 0.0],
        [-1.0, -1.0, 0.0],
        [1.0, 0.0, 1.0],
        [-1.0, 0.0, 1.0],
        [1.0, 0.0, -1.0],
        [-1.0, 0.0, -1.0],
        [0.0, 1.0, 1.0],
        [0.0, -1.0, 1.0],
        [0.0, 1.0, -1.0],
        [0.0, -1.0, -1.0],
        [1.0, 1.0, 0.0],
        [0.0, -1.0, 1.0],
        [-1.0, 1.0, 0.0],
        [0.0, -1.0, -1.0],
    ];
    let [gx, gy, gz] = GRAD.get((hash & 15) as usize).copied().unwrap_or_default();
    gx * x + gy * y + gz * z
}

fn perlin(x: f32, y: f32, z: f32) -> f32 {
    let (xflr, yflr, zflr) = (x.floor(), y.floor(), z.floor());
    let xi = (xflr as i32) & 255;
    let yi = (yflr as i32) & 255;
    let zi = (zflr as i32) & 255;
    let (xf, yf, zf) = (x - xflr, y - yflr, z - zflr);
    let (u, v, w) = (fade(xf), fade(yf), fade(zf));
    // Every operand is at most 256 + 255, so the sums cannot overflow.
    let a = perm(xi).wrapping_add(yi) & 255;
    let aa = perm(a).wrapping_add(zi) & 255;
    let ab = perm(a.wrapping_add(1)).wrapping_add(zi) & 255;
    let b = perm(xi.wrapping_add(1)).wrapping_add(yi) & 255;
    let ba = perm(b).wrapping_add(zi) & 255;
    let bb = perm(b.wrapping_add(1)).wrapping_add(zi) & 255;
    let la = plerp(u, grad(perm(aa), xf, yf, zf), grad(perm(ba), xf - 1.0, yf, zf));
    let lb = plerp(
        u,
        grad(perm(ab), xf, yf - 1.0, zf),
        grad(perm(bb), xf - 1.0, yf - 1.0, zf),
    );
    let la1 = plerp(
        u,
        grad(perm(aa.wrapping_add(1)), xf, yf, zf - 1.0),
        grad(perm(ba.wrapping_add(1)), xf - 1.0, yf, zf - 1.0),
    );
    let lb1 = plerp(
        u,
        grad(perm(ab.wrapping_add(1)), xf, yf - 1.0, zf - 1.0),
        grad(perm(bb.wrapping_add(1)), xf - 1.0, yf - 1.0, zf - 1.0),
    );
    plerp(w, plerp(v, la, lb), plerp(v, la1, lb1))
}

/// `FixMathNoisePrecision` (on in every profile the harnesses run): reduce
/// a coordinate mod 256 into `[0, 256)` before the `f32` conversion.
fn noise_wrap(x: f64) -> f64 {
    let m = x % 256.0;
    if m < 0.0 { m + 256.0 } else { m }
}

fn noise(vm: &mut Core, args: Args) -> VmResult<usize> {
    let coord = |vm: &Core, n: usize| vm.arg(args, n).and_then(|v| vm.to_number(v));
    let x = coord(vm, 1);
    let y = coord(vm, 2);
    let z = coord(vm, 3);
    let Some(x) = x else {
        return Err(vm.arg_type_error(args, 1, "number"));
    };
    if y.is_none() && !vm.arg_or_nil(args, 2).is_nil() {
        return Err(vm.arg_type_error(args, 2, "number"));
    }
    if z.is_none() && !vm.arg_or_nil(args, 3).is_nil() {
        return Err(vm.arg_type_error(args, 3, "number"));
    }
    let (x, y, z) = (
        noise_wrap(x),
        noise_wrap(y.unwrap_or(0.0)),
        noise_wrap(z.unwrap_or(0.0)),
    );
    let r = perlin(x as f32, y as f32, z as f32);
    vm.ret(&[Value::Number(f64::from(r))])
}

pub(crate) fn open(vm: &mut Core) -> VmResult<()> {
    let functions: &[(&str, NativeFn)] = &[
        ("abs", plain(abs)),
        ("ceil", plain(ceil)),
        ("floor", plain(floor)),
        ("sqrt", plain(sqrt)),
        ("sin", plain(sin)),
        ("cos", plain(cos)),
        ("tan", plain(tan)),
        ("asin", plain(asin)),
        ("acos", plain(acos)),
        ("atan", plain(atan)),
        ("atan2", plain(atan2)),
        ("exp", plain(exp)),
        ("log", plain(log)),
        ("log10", plain(log10)),
        ("pow", plain(pow)),
        ("fmod", plain(fmod)),
        ("modf", plain(modf)),
        ("sinh", plain(sinh)),
        ("cosh", plain(cosh)),
        ("tanh", plain(tanh)),
        ("deg", plain(deg)),
        ("rad", plain(rad)),
        ("sign", plain(sign)),
        ("round", plain(round)),
        ("min", plain(min)),
        ("max", plain(max)),
        ("clamp", plain(clamp)),
        ("frexp", plain(frexp)),
        ("fround", plain(fround)),
        ("ldexp", plain(ldexp)),
        ("random", plain(random)),
        ("randomseed", plain(randomseed)),
        ("noise", plain(noise)),
        ("map", plain(map)),
        ("lerp", plain(lerp)),
        ("isnan", plain(isnan)),
        ("isinf", plain(isinf)),
        ("isfinite", plain(isfinite)),
    ];
    let t = vm.open_lib("math", functions)?;
    vm.raw_set_str(t, "pi", Value::Number(std::f64::consts::PI))?;
    vm.raw_set_str(t, "huge", Value::Number(f64::INFINITY))?;
    vm.raw_set_str(t, "nan", Value::Number(f64::NAN))?;
    vm.raw_set_str(t, "e", Value::Number(std::f64::consts::E))?;
    vm.raw_set_str(t, "phi", Value::Number(1.618_033_988_749_895))?;
    vm.raw_set_str(t, "sqrt2", Value::Number(std::f64::consts::SQRT_2))?;
    vm.raw_set_str(t, "tau", Value::Number(std::f64::consts::TAU))?;
    Ok(())
}
