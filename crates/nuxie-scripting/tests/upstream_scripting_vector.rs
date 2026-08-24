//! One-for-one ports of `tests/unit_tests/runtime/scripting/scripting_vector_test.cpp`.
#![cfg(feature = "luau")]

use std::time::{Duration, Instant};

use luaur_rt::Lua;
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

fn eval_bool(source: &str) -> bool {
    rive_vm().eval(source).unwrap()
}

fn assert_approx(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-6,
        "expected approximately {expected}, got {actual}"
    );
}

#[test]
fn vector_can_be_constructed() {
    assert_eq!(
        eval_number("local _vec = Vector.xy(1,2)\nreturn _vec.x"),
        1.0
    );
    assert_eq!(
        eval_number("local _vec = Vector.xy(1,2)\nreturn _vec.y"),
        2.0
    );
    assert_eq!(
        eval_number("local _vec = Vector.xy(33,33)\nreturn _vec.x"),
        33.0
    );
    assert_eq!(
        eval_number("local _vec = Vector.xy(33,33)\nreturn _vec.y"),
        33.0
    );
    assert_eq!(
        eval_number("local _vec = Vector.origin()\nreturn _vec.x"),
        0.0
    );
    assert_eq!(
        eval_number("local _vec = Vector.origin()\nreturn _vec.y"),
        0.0
    );
}

#[test]
fn vector_static_methods_work() {
    assert_eq!(
        eval_number("return Vector.distance(Vector.origin(),Vector.xy(10,0))"),
        10.0
    );
    assert_eq!(
        eval_number("return Vector.distanceSquared(Vector.origin(),Vector.xy(10,0))"),
        100.0
    );
    assert_eq!(
        eval_number("return Vector.dot(Vector.xy(1,0),Vector.xy(-1,0))"),
        -1.0
    );
    assert_eq!(
        eval_number("return Vector.lerp(Vector.origin(),Vector.xy(1,2), 0.5).x"),
        0.5
    );
    assert_eq!(
        eval_number("return Vector.lerp(Vector.origin(),Vector.xy(1,2), 0.5).y"),
        1.0
    );
}

#[test]
fn vector_static_cross_scale_and_add_scale_and_sub_work() {
    assert_eq!(
        eval_number("return Vector.cross(Vector.xy(1,0),Vector.xy(0,1))"),
        1.0
    );
    assert_eq!(
        eval_number("return Vector.cross(Vector.xy(0,1),Vector.xy(1,0))"),
        -1.0
    );
    assert_eq!(
        eval_number("return Vector.scaleAndAdd(Vector.xy(1,2),Vector.xy(3,4),2).x"),
        7.0
    );
    assert_eq!(
        eval_number("return Vector.scaleAndAdd(Vector.xy(1,2),Vector.xy(3,4),2).y"),
        10.0
    );
    assert_eq!(
        eval_number("return Vector.scaleAndSub(Vector.xy(7,10),Vector.xy(3,4),2).x"),
        1.0
    );
    assert_eq!(
        eval_number("return Vector.scaleAndSub(Vector.xy(7,10),Vector.xy(3,4),2).y"),
        2.0
    );
}

#[test]
fn vector_static_length_length_squared_normalized_work() {
    assert_eq!(eval_number("return Vector.length(Vector.xy(3,4))"), 5.0);
    assert_eq!(
        eval_number("return Vector.lengthSquared(Vector.xy(3,4))"),
        25.0
    );
    assert_eq!(
        eval_number("return Vector.normalized(Vector.xy(10,0)).x"),
        1.0
    );
    assert_eq!(
        eval_number("return Vector.normalized(Vector.xy(10,0)).y"),
        0.0
    );
}

#[test]
fn vector_3d_constructor_and_cross3_work() {
    let values: (f64, f64, f64) = rive_vm()
        .eval("local v = Vector.xyz(1, 2, 3)\nreturn v.x, v.y, v.z\n")
        .unwrap();
    assert_eq!(values.0, 1.0);
    assert_eq!(values.1, 2.0);
    assert_eq!(values.2, 3.0);

    let cross: (f64, f64, f64) = rive_vm()
        .eval(
            "local v = Vector.cross3(Vector.xyz(1,0,0), Vector.xyz(0,1,0))\n\
             return v.x, v.y, v.z\n",
        )
        .unwrap();
    assert_eq!(cross.0, 0.0);
    assert_eq!(cross.1, 0.0);
    assert_eq!(cross.2, 1.0);

    assert_eq!(
        eval_number("return Vector.cross3(Vector.xyz(0,1,0), Vector.xyz(1,0,0)).z"),
        -1.0
    );
}

#[test]
fn vector_magnitude_ops_use_all_3_components() {
    assert_eq!(eval_number("return Vector.length(Vector.xyz(1,2,2))"), 3.0);
    assert_eq!(
        eval_number("return Vector.lengthSquared(Vector.xyz(1,2,3))"),
        14.0
    );
    assert_eq!(
        eval_number("return Vector.dot(Vector.xyz(1,2,3), Vector.xyz(4,5,6))"),
        32.0
    );
    assert_eq!(
        eval_number("return Vector.distance(Vector.xyz(1,1,1), Vector.xyz(1,1,4))"),
        3.0
    );
    assert_eq!(
        eval_number("return Vector.distanceSquared(Vector.xyz(0,0,0), Vector.xyz(1,2,3))"),
        14.0
    );

    let normalized: (f64, f64) = rive_vm()
        .eval(
            "local v = Vector.normalized(Vector.xyz(0,3,4))\n\
             return v.y, v.z\n",
        )
        .unwrap();
    assert_approx(normalized.0, 0.6);
    assert_approx(normalized.1, 0.8);

    assert_eq!(
        eval_number("return Vector.normalized(Vector.origin()).x"),
        0.0
    );
    assert_eq!(
        eval_number("return Vector.lerp(Vector.xyz(0,0,10), Vector.xyz(0,0,20), 0.5).z"),
        15.0
    );
    assert_eq!(
        eval_number("return Vector.scaleAndAdd(Vector.xyz(0,0,1), Vector.xyz(0,0,2), 3).z"),
        7.0
    );
    assert_eq!(
        eval_number("return Vector.scaleAndSub(Vector.xyz(0,0,7), Vector.xyz(0,0,2), 3).z"),
        1.0
    );
}

#[test]
fn vector_write_to_buffer_and_write_vec4_work() {
    let values: (f64, f64, f64) = rive_vm()
        .eval(
            "local buf = buffer.create(16)\n\
             Vector.xyz(1, 2, 3):writeToBuffer(buf, 4)\n\
             return buffer.readf32(buf, 4),\n\
                 buffer.readf32(buf, 8),\n\
                 buffer.readf32(buf, 12)\n",
        )
        .unwrap();
    assert_eq!(values.0, 1.0);
    assert_eq!(values.1, 2.0);
    assert_eq!(values.2, 3.0);

    let vec4: (f64, f64) = rive_vm()
        .eval(
            "local buf = buffer.create(16)\n\
             Vector.xyz(1, 2, 3):writeVec4(buf, 0, 4)\n\
             return buffer.readf32(buf, 0),\n\
                 buffer.readf32(buf, 12)\n",
        )
        .unwrap();
    assert_eq!(vec4.0, 1.0);
    assert_eq!(vec4.1, 4.0);

    assert!(!eval_bool(
        "local buf = buffer.create(12)\n\
         return (pcall(function()\n\
             Vector.origin():writeToBuffer(buf, 4)\n\
         end))\n"
    ));
    assert!(!eval_bool(
        "local buf = buffer.create(16)\n\
         return (pcall(function()\n\
             Vector.origin():writeVec4(buf, 4, 1)\n\
         end))\n"
    ));
}

#[test]
fn vector_fastcall_and_c_binding_paths_agree() {
    let result = eval_bool(
        r#"local a = Vector.xyz(1.5, -2.25, 3.75)
local b = Vector.xyz(-4.5, 5.25, -6.5)
local function check(name, x, y)
    if x ~= y then
        error(`{name} diverged: {x} vs {y}`)
    end
end
-- namecall (C binding) vs static (fastcall)
check('length', a:length(), Vector.length(a))
check('lengthSquared', a:lengthSquared(), Vector.lengthSquared(a))
check('normalized', a:normalized(), Vector.normalized(a))
check('distance', a:distance(b), Vector.distance(a, b))
check('distanceSquared', a:distanceSquared(b),
    Vector.distanceSquared(a, b))
check('dot', a:dot(b), Vector.dot(a, b))
for _, t in {0, 0.375, 1} do
    check(`lerp t={t}`, a:lerp(b, t), Vector.lerp(a, b, t))
end
-- ops with no instance form: indirect call (C binding) vs direct
-- static (fastcall); a table load can't compile to FASTCALL
local ind = {
    Vector.scaleAndAdd,
    Vector.scaleAndSub,
    Vector.cross,
    Vector.normalized,
}
check('scaleAndAdd', ind[1](a, b, 2.5), Vector.scaleAndAdd(a, b, 2.5))
check('scaleAndSub', ind[2](a, b, 2.5), Vector.scaleAndSub(a, b, 2.5))
check('cross', ind[3](a, b), Vector.cross(a, b))
-- zero-length normalize stays zero (not NaN) on both paths
local z = Vector.origin()
check('normalizedZero', ind[4](z), Vector.normalized(z))
check('normalizedZeroValue', Vector.normalized(z), z)
return true
"#,
    );
    assert!(result);
}

#[test]
fn vector_indexing_work() {
    assert_eq!(eval_number("return Vector.xy(19, 27)[1]"), 19.0);
    assert_eq!(eval_number("return Vector.xy(19, 27)[2]"), 27.0);
}

#[test]
fn vector_methods_work() {
    assert_eq!(
        eval_number("return Vector.origin():distance(Vector.xy(10,0))"),
        10.0
    );
    assert_eq!(
        eval_number("return Vector.origin():distanceSquared(Vector.xy(10,0))"),
        100.0
    );
    assert_eq!(
        eval_number("return Vector.xy(1,0):dot(Vector.xy(-1,0))"),
        -1.0
    );
    assert_eq!(
        eval_number("return Vector.origin():lerp(Vector.xy(1,2), 0.5).x"),
        0.5
    );
    assert_eq!(
        eval_number("return Vector.origin():lerp(Vector.xy(1,2), 0.5).y"),
        1.0
    );
}

#[test]
fn vector_meta_methods_work() {
    assert_eq!(eval_number("return -Vector.xy(12,13).x"), -12.0);
    assert_eq!(eval_number("return -Vector.xy(12,13).y"), -13.0);
    assert_eq!(
        eval_number("return (Vector.xy(12,13)+Vector.xy(2,3)).y"),
        16.0
    );
    assert_eq!(
        eval_number("return (Vector.xy(12,13)+Vector.xy(2,3)).x"),
        14.0
    );
    assert_eq!(
        eval_number("return (Vector.xy(12,13)-Vector.xy(2,3)).y"),
        10.0
    );
    assert_eq!(
        eval_number("return (Vector.xy(12,13)-Vector.xy(2,3)).x"),
        10.0
    );
    assert_eq!(eval_number("return (Vector.xy(12,13)*3).x"), 36.0);
    assert_eq!(eval_number("return (Vector.xy(12,13)*3).y"), 39.0);
    assert_eq!(eval_number("return (Vector.xy(12,13)/3).x"), 4.0);
    assert_approx(eval_number("return (Vector.xy(12,13)/3).y"), 4.3333333);
    assert!(!eval_bool("return Vector.xy(1,2) == Vector.xy(2,1)"));
    assert!(eval_bool("return Vector.xy(1,2) == Vector.xy(1,2)"));
    assert!(eval_bool("return Vector.xy(1,2) ~= Vector.xy(2,1)"));
    assert!(!eval_bool("return Vector.xy(1,2) ~= Vector.xy(1,2)"));
}

#[test]
fn closure_test() {
    let lua = Lua::new();
    let callback = lua
        .create_function(|_, ()| {
            eprintln!("index from callback upvalue is: {}", 222_u32);
            Ok(())
        })
        .unwrap();
    lua.globals().set("callMyFunc", callback).unwrap();
    lua.sandbox(true).unwrap();
    lua.load("callMyFunc()")
        .set_name("test_source")
        .exec()
        .unwrap();
}

fn best_run(source: &str, warmup: usize, runs: usize) -> Duration {
    let mut best = Duration::MAX;
    for run in 0..warmup + runs {
        let start = Instant::now();
        let _: f64 = rive_vm().eval(source).unwrap();
        let elapsed = start.elapsed();
        if run >= warmup {
            best = best.min(elapsed);
        }
    }
    best
}

#[test]
#[ignore = "upstream performance benchmark; run explicitly in a stable benchmark environment"]
fn vector_fast_function_benchmark() {
    const N: usize = 1_000_000;
    const WARMUP: usize = 3;
    const RUNS: usize = 5;

    let operations = [
        (
            "dot",
            format!(
                "local a = Vector.xy(3, 4)\n\
                 local b = Vector.xy(1, -2)\n\
                 local r = 0\n\
                 for i = 1, {N} do r = r + Vector.dot(a, b) end\n\
                 return r\n"
            ),
            format!(
                "local a = Vector.xy(3, 4)\n\
                 local b = Vector.xy(1, -2)\n\
                 local r = 0\n\
                 for i = 1, {N} do r = r + a:dot(b) end\n\
                 return r\n"
            ),
        ),
        (
            "distance",
            format!(
                "local a = Vector.xy(3, 4)\n\
                 local b = Vector.xy(1, -2)\n\
                 local r = 0\n\
                 for i = 1, {N} do r = r + Vector.distance(a, b) end\n\
                 return r\n"
            ),
            format!(
                "local a = Vector.xy(3, 4)\n\
                 local b = Vector.xy(1, -2)\n\
                 local r = 0\n\
                 for i = 1, {N} do r = r + a:distance(b) end\n\
                 return r\n"
            ),
        ),
        (
            "length",
            format!(
                "local a = Vector.xy(3, 4)\n\
                 local r = 0\n\
                 for i = 1, {N} do r = r + Vector.length(a) end\n\
                 return r\n"
            ),
            format!(
                "local a = Vector.xy(3, 4)\n\
                 local r = 0\n\
                 for i = 1, {N} do r = r + a:length() end\n\
                 return r\n"
            ),
        ),
        (
            "lerp",
            format!(
                "local a = Vector.xy(0, 0)\n\
                 local b = Vector.xy(10, 20)\n\
                 local r\n\
                 for i = 1, {N} do r = Vector.lerp(a, b, 0.5) end\n\
                 return r.x\n"
            ),
            format!(
                "local a = Vector.xy(0, 0)\n\
                 local b = Vector.xy(10, 20)\n\
                 local r\n\
                 for i = 1, {N} do r = a:lerp(b, 0.5) end\n\
                 return r.x\n"
            ),
        ),
    ];

    let mut results = Vec::new();
    for (name, static_body, instance_body) in &operations {
        results.push((
            *name,
            best_run(static_body, WARMUP, RUNS),
            best_run(instance_body, WARMUP, RUNS),
        ));
    }

    eprintln!("Vector Fast Function Benchmark ({N} iters, best of {RUNS} runs)");
    for (name, static_time, instance_time) in results {
        eprintln!("{name}: static={static_time:?}, instance={instance_time:?}");
    }

    let dot_result: f64 = rive_vm().eval(&operations[0].1).unwrap();
    assert_approx(dot_result, -5.0 * N as f64);
}
