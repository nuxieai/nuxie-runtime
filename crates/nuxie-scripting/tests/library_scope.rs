#![cfg(feature = "luau")]

use luaur_rt::{Table, Value};
use nuxie_binary::read_runtime_file_with_scripting;
use nuxie_scripting::vm::ScriptVm;

fn table(value: Value) -> Table {
    match value {
        Value::Table(value) => value,
        other => panic!("expected table, got {other:?}"),
    }
}

fn asset_path(name: &str) -> std::path::PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
        .join("tests/unit_tests/assets")
        .join(name)
}

#[test]
fn prelinked_source_modules_use_their_exported_names_verbatim() {
    let vm = ScriptVm::new();
    vm.register_source_module("Inner#1@1/utils/math", "return { value = 1 }")
        .unwrap();
    vm.register_source_module("Inner#1@2/utils/math", "return { value = 2 }")
        .unwrap();

    let caller1 = vm
        .register_source_module("Outer#2@1/caller", "return require('Inner#1@1/utils/math')")
        .unwrap();
    let caller2 = vm
        .register_source_module("Outer#2@2/caller", "return require('Inner#1@2/utils/math')")
        .unwrap();

    assert_eq!(table(caller1).get::<i64>("value").unwrap(), 1);
    assert_eq!(table(caller2).get::<i64>("value").unwrap(), 2);
}

#[test]
fn runtime_require_does_not_reinterpret_legacy_library_labels() {
    let vm = ScriptVm::new();
    vm.register_source_module("mesh", "return { value = 7 }")
        .unwrap();

    let error = vm
        .register_source_module("app", "return require('lib:geo/mesh')")
        .unwrap_err();
    let display = error.to_string();
    assert!(display.contains("lib:geo/mesh"), "got: {display}");
    assert!(matches!(vm.registered_module("app").unwrap(), Value::Nil));
}

#[test]
fn prelinked_source_errors_keep_the_mangled_chunkname() {
    let vm = ScriptVm::new();
    let error = vm
        .register_source_module("Thing#9@4/thing", "error('boom')")
        .unwrap_err();
    assert!(error.to_string().contains("Thing#9@4/thing"));
}

#[test]
fn prelinked_bytecode_registration_retries_dependency_chain() {
    let bytes = std::fs::read(asset_path("script_dependency_test.riv"))
        .expect("script dependency corpus fixture");
    let file = read_runtime_file_with_scripting(&bytes).expect("fixture imports");
    let scripts = file
        .scripting_file_assets_with_contents()
        .into_iter()
        .filter(|entry| entry.asset.type_name == "ScriptAsset")
        .map(|entry| {
            (
                entry.asset.string_property("name").unwrap_or_default(),
                entry.contents.expect("in-band bytecode"),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 6);

    let vm = ScriptVm::new();
    let failures = vm.perform_registration(scripts.iter().map(|(name, payload)| (*name, *payload)));
    assert!(
        failures.is_empty(),
        "prelinked bytecode registration did not converge: {}",
        failures
            .iter()
            .map(|(name, error)| format!("{name}: {error}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    assert!(matches!(
        vm.registered_module("Transform1").unwrap(),
        Value::Table(_)
    ));
    assert!(matches!(
        vm.registered_module("ChainedConverter").unwrap(),
        Value::Function(_)
    ));
}

#[test]
fn failed_candidate_modules_do_not_poison_the_parent_vm_stack() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/sync/scope_probe.riv");
    let bytes = std::fs::read(&path).expect("vendored scope probe fixture");
    let file = read_runtime_file_with_scripting(&bytes).expect("fixture imports");
    let scripts = file
        .scripting_file_assets_with_contents()
        .into_iter()
        .filter(|entry| entry.asset.type_name == "ScriptAsset")
        .map(|entry| {
            (
                entry.asset.string_property("name").unwrap_or_default(),
                entry.contents.expect("in-band bytecode"),
            )
        })
        .collect::<Vec<_>>();

    let vm = ScriptVm::new();
    let failures = vm.perform_registration(scripts.iter().map(|(name, payload)| (*name, *payload)));
    assert!(
        !failures.is_empty(),
        "the prelinked compatibility fixture intentionally has unresolved bare imports"
    );

    let result = vm
        .register_source_module("after_failed_candidate_graph", "return { value = 42 }")
        .expect("failed utility registration must preserve checked parent-stack headroom");
    assert_eq!(table(result).get::<i64>("value").unwrap(), 42);
}
mod support;
use support::ScriptVmSourceTestExt as _;
