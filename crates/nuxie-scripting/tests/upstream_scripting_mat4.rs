//! One-for-one ports of `tests/unit_tests/runtime/scripting/scripting_mat4_test.cpp`.
//!
//! Keep these cases separate from the broader binding tests: this file is the
//! source-correspondence ledger for the pinned upstream test file.
#![cfg(feature = "luau")]

use std::time::{Duration, Instant};

use luaur_rt::Value;
use nuxie_scripting::vm::ScriptVm;

mod support;
use support::ScriptVmSourceTestExt as _;

fn rive_vm() -> ScriptVm {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    vm
}

fn eval_number(source: &str) -> f64 {
    rive_vm().eval(source).unwrap()
}

fn assert_approx(actual: f64, expected: f64, margin: f64) {
    assert!(
        (actual - expected).abs() <= margin,
        "expected {expected} +/- {margin}, got {actual}"
    );
}

#[test]
fn mat4_identity_has_expected_values() {
    assert_eq!(eval_number("return Mat4.identity().m11"), 1.0);
    assert_eq!(eval_number("return Mat4.identity().m22"), 1.0);
    assert_eq!(eval_number("return Mat4.identity().m33"), 1.0);
    assert_eq!(eval_number("return Mat4.identity().m44"), 1.0);
    assert_eq!(eval_number("return Mat4.identity().m12"), 0.0);
    assert_eq!(eval_number("return Mat4.identity()[1]"), 1.0);
    assert_eq!(eval_number("return Mat4.identity()[6]"), 1.0);
}

#[test]
fn mat4_values_stores_column_major() {
    let values: (f64, f64, f64, f64, f64, f64) = rive_vm()
        .eval(
            "local m = Mat4.values(\n\
               1, 2, 3, 4,\n\
               5, 6, 7, 8,\n\
               9,10,11,12,\n\
              13,14,15,16)\n\
             return m.m11, m.m21, m.m31, m.m41, m.m14, m.m44\n",
        )
        .unwrap();
    assert_eq!(values.0, 1.0);
    assert_eq!(values.1, 2.0);
    assert_eq!(values.2, 3.0);
    assert_eq!(values.3, 4.0);
    assert_eq!(values.4, 13.0);
    assert_eq!(values.5, 16.0);
}

#[test]
fn mat4_translation_transforms_a_point() {
    let values: (f64, f64, f64) = rive_vm()
        .eval(
            "local m = Mat4.fromTranslation(10, 20, 30)\n\
             local v = m:transformPoint(1, 2, 3)\n\
             return v.x, v.y, v.z\n",
        )
        .unwrap();
    assert_eq!(values.0, 11.0);
    assert_eq!(values.1, 22.0);
    assert_eq!(values.2, 33.0);
}

#[test]
fn mat4_look_at_builds_a_view_matrix() {
    let values: (f64, f64, f64, f64, f64, f64) = rive_vm()
        .eval(
            "local view = Mat4.lookAt(Vector.xyz(0, 0, 5), Vector.origin(),\n\
                 Vector.xyz(0, 1, 0))\n\
             local v = view:transformPoint(0, 0, 0)\n\
             return v.x, v.y, v.z, view.m11, view.m22, view.m33\n",
        )
        .unwrap();
    assert_eq!(values.0, 0.0);
    assert_eq!(values.1, 0.0);
    assert_eq!(values.2, -5.0);
    assert_eq!(values.3, 1.0);
    assert_eq!(values.4, 1.0);
    assert_eq!(values.5, 1.0);

    let side: (f64, f64, f64) = rive_vm()
        .eval(
            "local view = Mat4.lookAt(Vector.xyz(5, 0, 0), Vector.origin(),\n\
                 Vector.xyz(0, 1, 0))\n\
             local v = view:transformPoint(1, 0, 0)\n\
             return v.x, v.y, v.z\n",
        )
        .unwrap();
    assert_approx(side.0, 0.0, 1e-6);
    assert_approx(side.1, 0.0, 1e-6);
    assert_approx(side.2, -4.0, f64::EPSILON);
}

#[test]
fn mat4_ortho_maps_depth_to_zero_through_one() {
    let values: (f64, f64, f64, f64, f64, f64) = rive_vm()
        .eval(
            "local proj = Mat4.ortho(-2, 2, -1, 1, 0, 10)\n\
             local near = proj:transformPoint(2, 1, 0)\n\
             local far = proj:transformPoint(-2, -1, -10)\n\
             return near.x, near.y, near.z, far.x, far.y, far.z\n",
        )
        .unwrap();
    assert_eq!(values.0, 1.0);
    assert_eq!(values.1, 1.0);
    assert_eq!(values.2, 0.0);
    assert_eq!(values.3, -1.0);
    assert_eq!(values.4, -1.0);
    assert_eq!(values.5, 1.0);
}

#[test]
fn mat4_ortho_times_look_at_round_trips_a_point() {
    let values: (f64, f64, f64) = rive_vm()
        .eval(
            "local view = Mat4.lookAt(Vector.xyz(0, 0, 5), Vector.origin(),\n\
                 Vector.xyz(0, 1, 0))\n\
             local proj = Mat4.ortho(-4, 4, -4, 4, 5, 15)\n\
             local vp = Mat4.multiply(Mat4.identity(), proj, view)\n\
             local v = vp:transformPoint(2, -2, -5)\n\
             return v.x, v.y, v.z\n",
        )
        .unwrap();
    assert_approx(values.0, 0.5, f64::EPSILON);
    assert_approx(values.1, -0.5, f64::EPSILON);
    assert_approx(values.2, 0.5, f64::EPSILON);
}

#[test]
fn mat4_transform_vec4_returns_homogeneous_components() {
    let values: (f64, f64, f64, f64) = rive_vm()
        .eval(
            "local m = Mat4.fromTranslation(10, 20, 30)\n\
             return m:transformVec4(1, 2, 3, 1)\n",
        )
        .unwrap();
    assert_eq!(values, (11.0, 22.0, 33.0, 1.0));
}

#[test]
fn transform_point_result_supports_z_and_index_three() {
    let values: (f64, f64, f64) = rive_vm()
        .eval(
            "local m = Mat4.fromTranslation(10, 20, 30)\n\
             local v = m:transformPoint(1, 2, 3)\n\
             return v[1], v[2], v[3]\n",
        )
        .unwrap();
    assert_eq!(values, (11.0, 22.0, 33.0));
}

#[test]
fn mat4_multiply_composes_transforms() {
    let values: (f64, f64, f64) = rive_vm()
        .eval(
            "local t = Mat4.fromTranslation(10, 0, 0)\n\
             local s = Mat4.fromScale(2, 2, 2)\n\
             local m = t * s\n\
             local v = m:transformPoint(1, 1, 1)\n\
             return v.x, v.y, v.z\n",
        )
        .unwrap();
    assert_eq!(values, (12.0, 2.0, 2.0));
}

#[test]
fn mat4_invert_round_trips() {
    let error = eval_number(
        "local m = Mat4.fromTranslation(3, -4, 5) * Mat4.fromScale(2, 2, 2)\n\
         local inv = m:invert()\n\
         local r = m * inv\n\
         local id = Mat4.identity()\n\
         return math.abs(r.m11 - 1) + math.abs(r.m22 - 1) + math.abs(r.m33 - 1) + math.abs(r.m44 - 1)\n",
    );
    assert!(error < 1e-5);
}

#[test]
fn mat4_multiply_writes_in_place() {
    let values: (f64, f64, f64, f64) = rive_vm()
        .eval(
            "local out = Mat4.identity()\n\
             local a = Mat4.fromTranslation(1, 2, 3)\n\
             local b = Mat4.fromScale(4, 4, 4)\n\
             Mat4.multiply(out, a, b)\n\
             return out.m14, out.m24, out.m34, out.m11\n",
        )
        .unwrap();
    assert_eq!(values.0, 1.0);
    assert_eq!(values.1, 2.0);
    assert_eq!(values.2, 3.0);
    assert_eq!(values.3, 4.0);
}

#[test]
fn mat4_multiply_affine_matches_multiply_for_affine_inputs() {
    let values: (f64, f64, f64, f64, f64) = rive_vm()
        .eval(
            "local a = Mat4.fromTranslation(3, -1, 5) * Mat4.fromRotationY(0.7)\n\
             local b = Mat4.fromScale(2, 0.5, 1) * Mat4.fromRotationZ(-0.3)\n\
             local slow = Mat4.identity()\n\
             local fast = Mat4.identity()\n\
             Mat4.multiply(slow, a, b)\n\
             Mat4.multiplyAffine(fast, a, b)\n\
             local diff = 0\n\
             for i = 1, 16 do diff = diff + math.abs(slow[i] - fast[i]) end\n\
             return diff, fast.m41, fast.m42, fast.m43, fast.m44\n",
        )
        .unwrap();
    assert_eq!(values.0, 0.0);
    assert_eq!(values.1, 0.0);
    assert_eq!(values.2, 0.0);
    assert_eq!(values.3, 0.0);
    assert_eq!(values.4, 1.0);
}

#[test]
fn mat4_invert_affine_round_trips() {
    let error = eval_number(
        "local m = Mat4.fromTranslation(3, -4, 5) * Mat4.fromRotationY(0.4) * Mat4.fromScale(2, 2, 2)\n\
         local inv = m:invertAffine()\n\
         assert(inv ~= nil)\n\
         local r = m * inv\n\
         return math.abs(r.m11 - 1) + math.abs(r.m22 - 1) + math.abs(r.m33 - 1) + math.abs(r.m44 - 1) + math.abs(r.m14) + math.abs(r.m24) + math.abs(r.m34)\n",
    );
    assert!(error < 1e-5);
}

#[test]
fn mat4_invert_affine_writes_in_place() {
    let values: (bool, f64, f64, f64) = rive_vm()
        .eval(
            "local m = Mat4.fromTranslation(10, 0, 0)\n\
             local out = Mat4.identity()\n\
             local ok = Mat4.invertAffine(out, m)\n\
             return ok, out.m14, out.m24, out.m34\n",
        )
        .unwrap();
    assert!(values.0);
    assert_eq!(values.1, -10.0);
    assert_eq!(values.2, 0.0);
    assert_eq!(values.3, 0.0);
}

#[test]
fn mat4_invert_affine_returns_nil_on_singular_linear_part() {
    let value: Value = rive_vm()
        .eval(
            "local m = Mat4.fromScale(2, 0, 1)\n\
             return m:invertAffine()\n",
        )
        .unwrap();
    assert!(matches!(value, Value::Nil), "got {value:?}");
}

#[test]
fn mat4_perspective_reverse_z_has_expected_layout() {
    let values: (f64, f64, f64, f64, f64) = rive_vm()
        .eval(
            "local p = Mat4.perspectiveReverseZ(math.rad(90), 1, 5)\n\
             return p.m11, p.m22, p.m33, p.m43, p.m34\n",
        )
        .unwrap();
    assert_approx(values.0, 1.0, 1e-6);
    assert_approx(values.1, 1.0, 1e-6);
    assert_eq!(values.2, 0.0);
    assert_eq!(values.3, -1.0);
    assert_eq!(values.4, 5.0);
}

#[test]
fn mat4_write_to_buffer_stores_64_bytes_column_major() {
    let values: (f64, f64, f64) = rive_vm()
        .eval(
            "local m = Mat4.values(\n\
               1, 2, 3, 4,  5, 6, 7, 8,  9,10,11,12, 13,14,15,16)\n\
             local buf = buffer.create(80)\n\
             m:writeToBuffer(buf, 16)\n\
             return buffer.readf32(buf, 16), buffer.readf32(buf, 16+4*4), buffer.readf32(buf, 16+15*4)\n",
        )
        .unwrap();
    assert_eq!(values.0, 1.0);
    assert_eq!(values.1, 5.0);
    assert_eq!(values.2, 16.0);
}

const LUAU_BUFFER_MATMUL_PRELUDE: &str = r#"
local function m4get(buf: buffer, i: number): number
    return buffer.readf32(buf, i * 4)
end
local function m4set(buf: buffer, i: number, v: number)
    buffer.writef32(buf, i * 4, v)
end
local function m4identity(): buffer
    local b = buffer.create(64)
    m4set(b, 0, 1)
    m4set(b, 5, 1)
    m4set(b, 10, 1)
    m4set(b, 15, 1)
    return b
end
local function m4mul(out: buffer, a: buffer, b: buffer)
    for col = 0, 3 do
        for row = 0, 3 do
            local sum: number = 0
            for k = 0, 3 do
                sum += m4get(a, k * 4 + row) * m4get(b, col * 4 + k)
            end
            m4set(out, col * 4 + row, sum)
        end
    end
end
"#;

fn best_run(source: &str, warmup: usize, runs: usize) -> Duration {
    let mut best = Duration::MAX;
    for run in 0..warmup + runs {
        let start = Instant::now();
        rive_vm().eval::<()>(source).unwrap();
        let elapsed = start.elapsed();
        if run >= warmup {
            best = best.min(elapsed);
        }
    }
    best
}

#[test]
#[ignore = "upstream performance benchmark; run explicitly in a stable benchmark environment"]
fn mat4_perf_native_vs_luau_buffer_matmul() {
    const N: usize = 20_000;
    const WARMUP: usize = 1;
    const RUNS: usize = 3;

    let native_mul = best_run(
        &format!(
            "local a = Mat4.fromTranslation(1, 2, 3)\n\
             local b = Mat4.fromRotationZ(0.1)\n\
             local m = Mat4.identity()\n\
             for i = 1, {N} do m = a * b end\n"
        ),
        WARMUP,
        RUNS,
    );
    let native_in_place = best_run(
        &format!(
            "local a = Mat4.fromTranslation(1, 2, 3)\n\
             local b = Mat4.fromRotationZ(0.1)\n\
             local out = Mat4.identity()\n\
             for i = 1, {N} do Mat4.multiply(out, a, b) end\n"
        ),
        WARMUP,
        RUNS,
    );
    let luau_buffer = best_run(
        &format!(
            "{LUAU_BUFFER_MATMUL_PRELUDE}\n\
             local a = m4identity()\n\
             local b = m4identity()\n\
             m4set(a, 12, 1); m4set(a, 13, 2); m4set(a, 14, 3)\n\
             m4set(b, 0, 0.99); m4set(b, 1, 0.099); m4set(b, 4, -0.099); m4set(b, 5, 0.99)\n\
             local out = m4identity()\n\
             for i = 1, {N} do m4mul(out, a, b) end\n"
        ),
        WARMUP,
        RUNS,
    );
    let luau_table = best_run(
        &format!(
            "local function tnew()\n\
               return {{1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1}}\n\
             end\n\
             local function tmul(out, a, b)\n\
               for col = 0, 3 do\n\
                 for row = 0, 3 do\n\
                   local s = 0\n\
                   for k = 0, 3 do\n\
                     s += a[k*4 + row + 1] * b[col*4 + k + 1]\n\
                   end\n\
                   out[col*4 + row + 1] = s\n\
                 end\n\
               end\n\
             end\n\
             local a = tnew(); a[13] = 1; a[14] = 2; a[15] = 3\n\
             local b = tnew(); b[1] = 0.99; b[2] = 0.099; b[5] = -0.099; b[6] = 0.99\n\
             local out = tnew()\n\
             for i = 1, {N} do tmul(out, a, b) end\n"
        ),
        WARMUP,
        RUNS,
    );

    eprintln!(
        "Mat4 matmul perf ({N} iterations, best of {RUNS}, includes VM setup):\n\
         native a*b: {native_mul:?}\n\
         native multiply(out): {native_in_place:?}\n\
         Luau buffer mul: {luau_buffer:?}\n\
         Luau table mul: {luau_table:?}"
    );
    assert!(native_in_place <= luau_buffer);
    assert!(native_in_place <= luau_table);
}
