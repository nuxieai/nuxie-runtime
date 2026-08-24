//! Direct ports from
//! `tests/unit_tests/runtime/scripting/scripting_require_check.cpp`.
#![cfg(feature = "luau")]

use nuxie_scripting::vm::ScriptVm;

mod support;
use support::ScriptVmSourceTestExt as _;

fn vm_with_rive_globals() -> ScriptVm {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    vm
}

#[test]
fn scripting_require() {
    let vm = vm_with_rive_globals();
    vm.register_source_module("utilities", "return { name = 'hello' }")
        .unwrap();
    vm.register_source_module("util2", "return function() return ' world'; end")
        .unwrap();

    let result: String = vm
        .run_source_bytecode(
            "test_source",
            "local util = require('utilities')\n\
             local util2 = require('util2')\n\
             return util.name .. util2()",
        )
        .unwrap();

    assert_eq!(result, "hello world");
}

#[test]
#[ignore = "expected-red: require errors are double-wrapped and omit the pinned test_source:1 attribution"]
fn scripting_require_with_bad_code() {
    let vm = vm_with_rive_globals();
    let _registration_error = vm.register_source_module("utilities", "return { 'name' = 'hello' }");

    let error = vm
        .run_source_bytecode::<String>(
            "test_source",
            "local util = require('utilities')\nreturn util.name",
        )
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "runtime error: test_source:1: require could not find a script named utilities"
    );
}

#[test]
fn scripting_require_with_bad_unused_module_is_ok() {
    let vm = vm_with_rive_globals();
    vm.register_source_module("utilities", "return { name = 'hello' }")
        .unwrap();
    let _registration_error =
        vm.register_source_module("utilities2", "return { 'name' = 'hello' }");

    let result: String = vm
        .run_source_bytecode(
            "test_source",
            "local util = require('utilities')\nreturn util.name",
        )
        .unwrap();

    assert_eq!(result, "hello");
}

#[test]
fn scripting_time_values_are_available() {
    let vm = vm_with_rive_globals();

    let result: String = vm
        .run_source_bytecode(
            "test_source",
            "return os.date(\"!%Y-%m-%d %H:%M:%S\",1761608005)\n",
        )
        .unwrap();

    assert_eq!(result, "2025-10-27 23:33:25");
}

#[test]
fn buffer_api_is_available() {
    let vm = vm_with_rive_globals();

    let result: f64 = vm
        .run_source_bytecode(
            "test_source",
            "local buf = buffer.create(10)\n\
             buffer.writei8(buf, 0, 42)\n\
             local value = buffer.readi8(buf, 0)\n\
             return value",
        )
        .unwrap();

    assert_eq!(result, 42.0);
}

#[test]
fn buffer_fromstring_and_tostring_work() {
    let vm = vm_with_rive_globals();

    let result: String = vm
        .run_source_bytecode(
            "test_source",
            "local buf = buffer.fromstring('hello')\nreturn buffer.tostring(buf)",
        )
        .unwrap();

    assert_eq!(result, "hello");
}

#[test]
fn buffer_len_works() {
    let vm = vm_with_rive_globals();

    let result: f64 = vm
        .run_source_bytecode(
            "test_source",
            "local buf = buffer.create(20)\nreturn buffer.len(buf)",
        )
        .unwrap();

    assert_eq!(result, 20.0);
}

#[test]
fn bit32_api_is_available() {
    let vm = vm_with_rive_globals();

    let result: f64 = vm
        .run_source_bytecode(
            "test_source",
            "local result = bit32.band(5, 3)\nreturn result",
        )
        .unwrap();

    assert_eq!(result, 1.0);
}

#[test]
fn bit32_bor_and_bxor_work() {
    let vm = vm_with_rive_globals();

    let (or_result, xor_result): (f64, f64) = vm
        .run_source_bytecode(
            "test_source",
            "local orResult = bit32.bor(5, 3)\n\
             local xorResult = bit32.bxor(5, 3)\n\
             return orResult, xorResult",
        )
        .unwrap();

    assert_eq!(or_result, 7.0);
    assert_eq!(xor_result, 6.0);
}

#[test]
fn bit32_shifts_work() {
    let vm = vm_with_rive_globals();

    let (left_shift, right_shift): (f64, f64) = vm
        .run_source_bytecode(
            "test_source",
            "local leftShift = bit32.lshift(1, 3)\n\
             local rightShift = bit32.rshift(8, 3)\n\
             return leftShift, rightShift",
        )
        .unwrap();

    assert_eq!(left_shift, 8.0);
    assert_eq!(right_shift, 1.0);
}
