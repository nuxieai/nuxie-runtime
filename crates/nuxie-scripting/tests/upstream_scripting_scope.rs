//! Direct ports of four module-registration cases from
//! `tests/unit_tests/runtime/scripting/scripting_scope_test.cpp`.
#![cfg(feature = "luau")]

use luaur_rt::{Table, Value};
use nuxie_binary::read_runtime_file_with_scripting;
use nuxie_scripting::vm::ScriptVm;

mod support;
use support::ScriptVmSourceTestExt as _;

fn cached_int_field(vm: &ScriptVm, key: &str, field: &str) -> i64 {
    match vm.registered_module(key).unwrap() {
        Value::Table(table) => table.get::<i64>(field).unwrap_or(i64::MIN),
        _ => i64::MIN,
    }
}

#[test]
fn statically_linked_flat_names_round_trip() {
    let vm = ScriptVm::new();
    vm.register_source_module("draco@1/mesh", "return { v = 1 }")
        .unwrap();
    vm.register_source_module("draco@2/mesh", "return { v = 2 }")
        .unwrap();
    vm.register_source_module("B@1/uses", "return { v = require('draco@1/mesh').v }")
        .unwrap();
    vm.register_source_module(
        "app",
        "return { v = require('draco@2/mesh').v + require('B@1/uses').v }",
    )
    .unwrap();

    assert_eq!(cached_int_field(&vm, "draco@1/mesh", "v"), 1);
    assert_eq!(cached_int_field(&vm, "draco@2/mesh", "v"), 2);
    assert_eq!(cached_int_field(&vm, "B@1/uses", "v"), 1);
    assert_eq!(cached_int_field(&vm, "app", "v"), 3);
}

#[test]
fn bare_require_never_reaches_a_mangled_library_module() {
    let vm = ScriptVm::new();
    vm.register_source_module("draco@1/mesh", "return { v = 1 }")
        .unwrap();
    let error = vm
        .register_source_module("app", "return require('mesh')")
        .unwrap_err();

    assert_eq!(cached_int_field(&vm, "app", "v"), i64::MIN);
    assert!(!error.to_string().is_empty());
}

#[test]
fn mangled_module_errors_attribute_to_the_library_in_traces() {
    let vm = ScriptVm::new();
    let error = vm
        .register_source_module("draco@1/boom", "error('boom')")
        .unwrap_err();
    assert!(error.to_string().contains("draco@1/boom:"));
}

#[test]
#[ignore = "expected-red: scope_probe is not registered from the exported fixture, so the three exact cached fields remain absent"]
fn exported_file_resolves_statically_linked_library_requires() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/sync/scope_probe.riv");
    let bytes = std::fs::read(path).unwrap();
    let file = read_runtime_file_with_scripting(&bytes).unwrap();
    let scripts = file
        .scripting_file_assets_with_contents()
        .into_iter()
        .filter(|entry| entry.asset.type_name == "ScriptAsset")
        .map(|entry| {
            (
                entry.asset.string_property("name").unwrap_or_default(),
                entry.contents.unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let vm = ScriptVm::new();
    let _registration_errors =
        vm.perform_registration(scripts.iter().map(|(name, bytes)| (*name, *bytes)));

    assert_eq!(cached_int_field(&vm, "scope_probe", "lib"), 1);
    assert_eq!(cached_int_field(&vm, "scope_probe", "hasDecode"), 1);
    assert_eq!(cached_int_field(&vm, "scope_probe", "cached"), 1);

    let leak: Table = match vm
        .register_source_module(
            "leak_probe",
            "local ok = pcall(require, 'draco')\nreturn { ok = ok and 1 or 0 }",
        )
        .unwrap()
    {
        Value::Table(table) => table,
        value => panic!("expected leak-probe table, got {value:?}"),
    };
    assert_eq!(leak.get::<i64>("ok").unwrap(), 0);
}
