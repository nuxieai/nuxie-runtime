//! The `vector` library (C++ `lveclib.cpp`, `LUA_VECTOR_SIZE` 3) and the
//! metatable shared by all vectors, whose `__index` reads `x`/`y`/`z`.
//!
//! The builtins' fast bodies (`fastcall.rs`, C++ `luauF_vector*`) sit
//! beside their library functions. Those of `magnitude` and `dot` sum with
//! fused multiply-adds, where the library bodies multiply and add; both run
//! here as upstream runs them. The other fast bodies compute what the
//! library bodies compute.
//!
//! The Rive `Vector` builtins (`Vector.distance` and the rest, ids 245 to
//! 255) have fast bodies only: the compiler knows them, and the vendored
//! VM runs their bodies under `safeenv` whether or not a `Vector` global
//! exists; without one the fallback `CALL` raises, as on the vendored VM.
//!
//! One deliberate deviation: `vector.angle` with no second argument reads a
//! null pointer in the vendored transliteration; upstream C++ checks it
//! (`luaL_checkvector`), and so does this.

use crate::error::VmResult;
use crate::fastcall::arg;
use crate::native::{Args, NativeFn, plain};
use crate::value::Value;
use crate::vm::Core;

type V3 = [f32; 3];

impl Core {
    /// C++ `luaL_checkvector`.
    pub(crate) fn check_vector(&mut self, args: Args, n: usize) -> VmResult<V3> {
        match self.arg(args, n) {
            Some(Value::Vector(v)) => Ok(v),
            _ => Err(self.arg_type_error(args, n, "vector")),
        }
    }
}

fn ret_v(vm: &mut Core, v: V3) -> VmResult<usize> {
    vm.ret(&[Value::Vector(v)])
}

fn map(v: V3, f: impl Fn(f32) -> f32) -> V3 {
    [f(v[0]), f(v[1]), f(v[2])]
}

fn create(vm: &mut Core, args: Args) -> VmResult<usize> {
    let x = vm.check_number(args, 1)?;
    let y = vm.check_number(args, 2)?;
    let z = if args.nargs >= 3 { vm.check_number(args, 3)? } else { 0.0 };
    ret_v(vm, [x as f32, y as f32, z as f32])
}

fn magnitude(vm: &mut Core, args: Args) -> VmResult<usize> {
    let [x, y, z] = vm.check_vector(args, 1)?;
    vm.ret(&[Value::Number(f64::from((x * x + y * y + z * z).sqrt()))])
}

/// `luauF_vectormagnitude`.
pub(crate) fn magnitude_fast([x, y, z]: V3) -> f64 {
    let xy = x.mul_add(x, y * y);
    f64::from(z.mul_add(z, xy).sqrt())
}

fn normalize(vm: &mut Core, args: Args) -> VmResult<usize> {
    let [x, y, z] = vm.check_vector(args, 1)?;
    let inv = 1.0 / (x * x + y * y + z * z).sqrt();
    ret_v(vm, [x * inv, y * inv, z * inv])
}

fn cross3(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn cross(vm: &mut Core, args: Args) -> VmResult<usize> {
    let a = vm.check_vector(args, 1)?;
    let b = vm.check_vector(args, 2)?;
    ret_v(vm, cross3(a, b))
}

fn dot(vm: &mut Core, args: Args) -> VmResult<usize> {
    let a = vm.check_vector(args, 1)?;
    let b = vm.check_vector(args, 2)?;
    let d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    vm.ret(&[Value::Number(f64::from(d))])
}

/// `luauF_vectordot`.
pub(crate) fn dot_fast(a: V3, b: V3) -> f64 {
    let xy = a[0].mul_add(b[0], a[1] * b[1]);
    f64::from(a[2].mul_add(b[2], xy))
}

fn angle(vm: &mut Core, args: Args) -> VmResult<usize> {
    let a = vm.check_vector(args, 1)?;
    let b = vm.check_vector(args, 2)?;
    let axis = if vm.arg_or_nil(args, 3).is_nil() {
        None
    } else {
        Some(vm.check_vector(args, 3)?)
    };
    let c = cross3(a, b);
    let sin_a = f64::from(c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    let cos_a = f64::from(a[0] * b[0] + a[1] * b[1] + a[2] * b[2]);
    let mut r = sin_a.atan2(cos_a);
    if let Some(axis) = axis
        && c[0] * axis[0] + c[1] * axis[1] + c[2] * axis[2] < 0.0
    {
        r = -r;
    }
    vm.ret(&[Value::Number(r)])
}

fn floor(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_vector(args, 1)?;
    ret_v(vm, map(v, f32::floor))
}

fn ceil(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_vector(args, 1)?;
    ret_v(vm, map(v, f32::ceil))
}

fn abs(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_vector(args, 1)?;
    ret_v(vm, map(v, f32::abs))
}

fn signf(v: f32) -> f32 {
    if v > 0.0 {
        1.0
    } else if v < 0.0 {
        -1.0
    } else {
        0.0
    }
}

fn sign(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_vector(args, 1)?;
    ret_v(vm, map(v, signf))
}

fn clampf(v: f32, min: f32, max: f32) -> f32 {
    let r = if v < min { min } else { v };
    if r > max { max } else { r }
}

fn clamp(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_vector(args, 1)?;
    let lo = vm.check_vector(args, 2)?;
    let hi = vm.check_vector(args, 3)?;
    for (i, axis) in ["x", "y", "z"].iter().enumerate() {
        let ordered = matches!((lo.get(i), hi.get(i)), (Some(l), Some(h)) if l <= h);
        if !ordered {
            return Err(vm.arg_error(
                3,
                &format!("max.{axis} must be greater than or equal to min.{axis}"),
            ));
        }
    }
    ret_v(
        vm,
        [
            clampf(v[0], lo[0], hi[0]),
            clampf(v[1], lo[1], hi[1]),
            clampf(v[2], lo[2], hi[2]),
        ],
    )
}

fn fold(vm: &mut Core, args: Args, keep: fn(f32, f32) -> bool) -> VmResult<usize> {
    let mut r = vm.check_vector(args, 1)?;
    for i in 2..=args.nargs {
        let b = vm.check_vector(args, i)?;
        for (r, b) in r.iter_mut().zip(b) {
            if keep(b, *r) {
                *r = b;
            }
        }
    }
    ret_v(vm, r)
}

fn max(vm: &mut Core, args: Args) -> VmResult<usize> {
    fold(vm, args, |b, r| b > r)
}

fn min(vm: &mut Core, args: Args) -> VmResult<usize> {
    fold(vm, args, |b, r| b < r)
}

/// `luai_lerpf`: exact at `t == 1`, otherwise a fused `(b - a) * t + a`.
fn lerpf(a: f32, b: f32, t: f32) -> f32 {
    if t == 1.0 { b } else { (b - a).mul_add(t, a) }
}

fn lerp(vm: &mut Core, args: Args) -> VmResult<usize> {
    let a = vm.check_vector(args, 1)?;
    let b = vm.check_vector(args, 2)?;
    let t = vm.check_number(args, 3)? as f32;
    ret_v(
        vm,
        [
            lerpf(a[0], b[0], t),
            lerpf(a[1], b[1], t),
            lerpf(a[2], b[2], t),
        ],
    )
}

// ---- fast bodies -----------------------------------------------------------

/// One vector in, one value out: `nparams >= 1`, `nresults <= 1`.
macro_rules! fast_vector1 {
    ($name:ident, $f:expr) => {
        pub(crate) fn $name(
            vm: &mut Core,
            ra: usize,
            nresults: i32,
            [a0, ..]: [Value; 3],
            nparams: usize,
        ) -> Option<usize> {
            let Value::Vector(v) = a0 else { return None };
            if nparams < 1 || nresults > 1 {
                return None;
            }
            let f: fn(V3) -> Value = $f;
            vm.set(ra, f(v));
            Some(1)
        }
    };
}

/// Two vectors in, one value out: `nparams >= 2`, `nresults <= 1`.
macro_rules! fast_vector2 {
    ($name:ident, $f:expr) => {
        pub(crate) fn $name(
            vm: &mut Core,
            ra: usize,
            nresults: i32,
            [a0, a1, _]: [Value; 3],
            nparams: usize,
        ) -> Option<usize> {
            let (Value::Vector(a), Value::Vector(b)) = (a0, a1) else { return None };
            if nparams < 2 || nresults > 1 {
                return None;
            }
            let f: fn(V3, V3) -> Value = $f;
            vm.set(ra, f(a, b));
            Some(1)
        }
    };
}

/// Two vectors and a number in, a vector out: `nparams >= 3`.
macro_rules! fast_vector_vvn {
    ($name:ident, $f:expr) => {
        pub(crate) fn $name(
            vm: &mut Core,
            ra: usize,
            nresults: i32,
            [a0, a1, a2]: [Value; 3],
            nparams: usize,
        ) -> Option<usize> {
            let (Value::Vector(a), Value::Vector(b), Value::Number(t)) = (a0, a1, a2) else {
                return None;
            };
            if nparams < 3 || nresults > 1 {
                return None;
            }
            let f: fn(V3, V3, f32) -> V3 = $f;
            vm.set(ra, Value::Vector(f(a, b, t as f32)));
            Some(1)
        }
    };
}

/// `luauF_vector` (`vector.create`, `Vector.xy`): two or three numbers.
pub(crate) fn fast_create(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    [a0, a1, a2]: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    let (Value::Number(x), Value::Number(y)) = (a0, a1) else { return None };
    if nparams < 2 || nresults > 1 {
        return None;
    }
    let z = if nparams >= 3 {
        let Value::Number(z) = a2 else { return None };
        z
    } else {
        0.0
    };
    vm.set(ra, Value::Vector([x as f32, y as f32, z as f32]));
    Some(1)
}

fast_vector1!(fast_magnitude, |v| Value::Number(magnitude_fast(v)));
fast_vector1!(fast_normalize, |[x, y, z]| {
    let inv = 1.0 / (x * x + y * y + z * z).sqrt();
    Value::Vector([x * inv, y * inv, z * inv])
});
fast_vector2!(fast_cross, |a, b| Value::Vector(cross3(a, b)));
fast_vector2!(fast_dot, |a, b| Value::Number(dot_fast(a, b)));
fast_vector1!(fast_floor, |v| Value::Vector(map(v, f32::floor)));
fast_vector1!(fast_ceil, |v| Value::Vector(map(v, f32::ceil)));
fast_vector1!(fast_abs, |v| Value::Vector(map(v, f32::abs)));
fast_vector1!(fast_sign, |v| Value::Vector(map(v, signf)));
fast_vector_vvn!(fast_lerp, |a, b, t| [
    lerpf(a[0], b[0], t),
    lerpf(a[1], b[1], t),
    lerpf(a[2], b[2], t),
]);

/// `luauF_vectorclamp`: falls through for unordered bounds, so the
/// library raises.
pub(crate) fn fast_clamp(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    args: [Value; 3],
    nparams: usize,
) -> Option<usize> {
    let [Value::Vector(v), Value::Vector(lo), Value::Vector(hi)] = args else { return None };
    if nparams < 3 || nresults > 1 {
        return None;
    }
    if !(lo[0] <= hi[0] && lo[1] <= hi[1] && lo[2] <= hi[2]) {
        return None;
    }
    let r = [
        clampf(v[0], lo[0], hi[0]),
        clampf(v[1], lo[1], hi[1]),
        clampf(v[2], lo[2], hi[2]),
    ];
    vm.set(ra, Value::Vector(r));
    Some(1)
}

/// `luauF_vectormin` and `max`: two or more vectors, componentwise.
fn fast_fold(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    args: [Value; 3],
    nparams: usize,
    keep: fn(f32, f32) -> bool,
) -> Option<usize> {
    let [Value::Vector(mut r), Value::Vector(b), _] = args else { return None };
    if nparams < 2 || nresults > 1 {
        return None;
    }
    let mut fold = |b: V3| {
        for (r, b) in r.iter_mut().zip(b) {
            if keep(b, *r) {
                *r = b;
            }
        }
    };
    fold(b);
    for i in 2..nparams {
        let Value::Vector(c) = arg(vm, ra, &args, i) else { return None };
        fold(c);
    }
    vm.set(ra, Value::Vector(r));
    Some(1)
}

pub(crate) fn fast_min(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_fold(vm, ra, nresults, args, nparams, |b, r| b < r)
}

pub(crate) fn fast_max(vm: &mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize> {
    fast_fold(vm, ra, nresults, args, nparams, |b, r| b > r)
}

// The Rive `Vector` builtins (`luauF_vectordistance` and the rest).

/// `a . a` (or the squared distance) with two fused multiply-adds.
fn length_squared_fused([x, y, z]: V3) -> f32 {
    z.mul_add(z, x.mul_add(x, y * y))
}

fn difference(a: V3, b: V3) -> V3 {
    [b[0] - a[0], b[1] - a[1], b[2] - a[2]]
}

fast_vector2!(fast_rive_distance, |a, b| {
    Value::Number(f64::from(length_squared_fused(difference(a, b)).sqrt()))
});
fast_vector2!(fast_rive_distance_squared, |a, b| {
    Value::Number(f64::from(length_squared_fused(difference(a, b))))
});
fast_vector1!(fast_rive_length_squared, |v| Value::Number(f64::from(length_squared_fused(v))));
fast_vector1!(fast_rive_normalize, |v| {
    let len_sq = length_squared_fused(v);
    let inv = if len_sq > 0.0 { 1.0 / len_sq.sqrt() } else { 1.0 };
    Value::Vector(map(v, |c| c * inv))
});
fast_vector2!(fast_rive_cross, |a, b| Value::Number(f64::from(a[0].mul_add(b[1], -(a[1] * b[0])))));
fast_vector_vvn!(fast_rive_scale_and_add, |a, b, s| [
    b[0].mul_add(s, a[0]),
    b[1].mul_add(s, a[1]),
    b[2].mul_add(s, a[2]),
]);
fast_vector_vvn!(fast_rive_scale_and_sub, |a, b, s| [
    b[0].mul_add(-s, a[0]),
    b[1].mul_add(-s, a[1]),
    b[2].mul_add(-s, a[2]),
]);

/// `luauF_vectororigin` (`Vector.origin()`): any arguments.
pub(crate) fn fast_rive_origin(
    vm: &mut Core,
    ra: usize,
    nresults: i32,
    _: [Value; 3],
    _: usize,
) -> Option<usize> {
    if nresults > 1 {
        return None;
    }
    vm.set(ra, Value::Vector([0.0; 3]));
    Some(1)
}

/// The vector metatable's `__index` (C++ `vector_index`).
fn index(vm: &mut Core, args: Args) -> VmResult<usize> {
    let v = vm.check_vector(args, 1)?;
    let name = vm.check_string(args, 2)?;
    let bytes = vm.heap.str_bytes(name);
    if let [c] = bytes {
        // Case-insensitive `x`/`y`/`z`; `w` maps to the fourth component,
        // which a 3-wide vector does not have.
        let lower = c | 0x20;
        let component = match lower {
            b'x' => v.first(),
            b'y' => v.get(1),
            b'z' => v.get(2),
            _ => None,
        };
        if let Some(&x) = component {
            return vm.ret(&[Value::Number(f64::from(x))]);
        }
    }
    let name = String::from_utf8_lossy(bytes).into_owned();
    Err(vm.lib_error(format!("attempt to index vector with '{name}'")))
}

pub(crate) fn open(vm: &mut Core) -> VmResult<()> {
    let functions: &[(&str, NativeFn)] = &[
        ("create", plain(create)),
        ("magnitude", plain(magnitude)),
        ("normalize", plain(normalize)),
        ("cross", plain(cross)),
        ("dot", plain(dot)),
        ("angle", plain(angle)),
        ("floor", plain(floor)),
        ("ceil", plain(ceil)),
        ("abs", plain(abs)),
        ("sign", plain(sign)),
        ("clamp", plain(clamp)),
        ("max", plain(max)),
        ("min", plain(min)),
        ("lerp", plain(lerp)),
    ];
    let t = vm.open_lib("vector", functions)?;
    vm.raw_set_str(t, "zero", Value::Vector([0.0; 3]))?;
    vm.raw_set_str(t, "one", Value::Vector([1.0; 3]))?;
    let meta = vm.new_table(0, 1)?;
    let f = vm.new_builtin("", plain(index), &[])?;
    vm.raw_set_str(meta, "__index", f)?;
    vm.heap.table_mut(meta).readonly = true;
    vm.vector_meta = Some(meta);
    Ok(())
}
