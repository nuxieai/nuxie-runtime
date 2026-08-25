//! Direct ports of the two upstream `LuauDirectFieldGet` test files.
//!
//! The GC regression executes through luaur's pinned process-global FFlag.
//! The benchmark remains expected-red until the high-level userdata adapter
//! exposes Rive's native direct-field registrations for Paint and Mat2D.
#![cfg(feature = "luau")]

use std::sync::Mutex;
use std::time::{Duration, Instant};

use nuxie_scripting::vm::ScriptVm;

mod support;
use support::ScriptVmSourceTestExt as _;

fn rive_vm() -> ScriptVm {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    vm
}

static FLAG_LOCK: Mutex<()> = Mutex::new(());

struct ScopedDirectFieldFlag(bool);

impl ScopedDirectFieldFlag {
    fn new(value: bool) -> Self {
        let saved = luaur_rt::luau_direct_field_get();
        luaur_rt::set_luau_direct_field_get(value);
        Self(saved)
    }
}

impl Drop for ScopedDirectFieldFlag {
    fn drop(&mut self) {
        luaur_rt::set_luau_direct_field_get(self.0);
    }
}

fn best_run(source: &str) -> Duration {
    const WARMUP: usize = 3;
    const RUNS: usize = 5;
    let mut best = Duration::MAX;
    for run in 0..WARMUP + RUNS {
        let start = Instant::now();
        let _: f64 = rive_vm().eval(source).unwrap();
        let elapsed = start.elapsed();
        if run >= WARMUP {
            best = best.min(elapsed);
        }
    }
    best
}

fn benchmark_with_flag(source: &str, enabled: bool) -> Duration {
    let _flag = ScopedDirectFieldFlag::new(enabled);
    best_run(source)
}

#[test]
#[ignore = "expected red: luaur does not expose the pinned LuauDirectFieldGet FFlag"]
fn userdata_direct_field_get_benchmark() {
    let _lock = FLAG_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    const N: usize = 1_000_000;
    let operations = [
        (
            "Paint.thickness",
            format!(
                "local p = Paint.new()\n\
                 p.thickness = 2\n\
                 local s = 0\n\
                 for i = 1, {N} do s = s + p.thickness end\n\
                 return s\n"
            ),
        ),
        (
            "Paint.color",
            format!(
                "local p = Paint.new()\n\
                 p.color = 0xFFFFFFFF\n\
                 local s = 0\n\
                 for i = 1, {N} do s = s + p.color end\n\
                 return s\n"
            ),
        ),
        (
            "Mat2D.xx + .ty",
            format!(
                "local m = Mat2D.values(1, 2, 3, 4, 5, 6)\n\
                 local s = 0\n\
                 for i = 1, {N} do s = s + m.xx + m.ty end\n\
                 return s\n"
            ),
        ),
        (
            "Mat4.m11..m44",
            format!(
                "local m = Mat4.identity()\n\
                 local s = 0\n\
                 for i = 1, {N} do s = s + m.m11 + m.m22 + m.m33 + m.m44 end\n\
                 return s\n"
            ),
        ),
    ];

    for (name, source) in operations {
        let off = benchmark_with_flag(&source, false);
        let on = benchmark_with_flag(&source, true);
        eprintln!("field={name} off={off:?} on={on:?}");
        assert!(on < off);
    }
}

#[test]
fn scripting_vm_survives_direct_field_flag_flip_after_newstate() {
    let _lock = FLAG_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let _flag = ScopedDirectFieldFlag::new(false);
    let vm = rive_vm();
    let value: f64 = vm.eval("return 1").unwrap();
    assert_eq!(value, 1.0);

    luaur_rt::set_luau_direct_field_get(true);
    vm.eval::<()>(
        "for i = 0, 63 do\n\
             local table = {}\n\
         end",
    )
    .unwrap();
    vm.lua().gc_collect().unwrap();
}
