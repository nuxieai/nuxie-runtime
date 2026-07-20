#![cfg(feature = "luau")]

use luaur_rt::{Function, Table, Value};
use nuxie_binary::read_runtime_file_with_scripting;
use nuxie_scripting::vm::{ScopeKey, ScriptVm};

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
fn scoped_keys_preserve_root_and_chunknames_are_trace_readable() {
    let scope = ScopeKey::new(7, 3);
    assert_eq!(ScriptVm::scoped_module_key("mesh", ScopeKey::ROOT), "mesh");
    assert_eq!(ScriptVm::scoped_module_key("mesh", scope), "mesh\u{1f}7:3");
    assert_eq!(ScriptVm::readable_chunkname("mesh", scope), "7-3/mesh");
    assert!(!ScriptVm::readable_chunkname("mesh", scope).contains(':'));
}

#[test]
fn same_named_source_modules_coexist_and_bare_require_stays_in_scope() {
    let vm = ScriptVm::new();
    let v1 = ScopeKey::new(1, 1);
    let v2 = ScopeKey::new(1, 2);

    vm.register_source_module_scoped("utils/math", v1, "return { value = 1 }")
        .unwrap();
    vm.register_source_module_scoped("utils/math", v2, "return { value = 2 }")
        .unwrap();
    let caller1 = vm
        .register_source_module_scoped("caller", v1, "return require('utils/math')")
        .unwrap();
    let caller2 = vm
        .register_source_module_scoped("caller", v2, "return require('utils/math')")
        .unwrap();

    assert_eq!(table(caller1).get::<i64>("value").unwrap(), 1);
    assert_eq!(table(caller2).get::<i64>("value").unwrap(), 2);
    assert_eq!(
        table(vm.registered_module_scoped("utils/math", v1).unwrap())
            .get::<i64>("value")
            .unwrap(),
        1
    );
    assert_eq!(
        table(vm.registered_module_scoped("utils/math", v2).unwrap())
            .get::<i64>("value")
            .unwrap(),
        2
    );
    assert!(matches!(
        vm.registered_module("utils/math").unwrap(),
        Value::Nil
    ));
}

#[test]
fn nested_library_pin_uses_the_defining_chunk_for_lazy_require() {
    let vm = ScriptVm::new();
    let outer = ScopeKey::new(20, 6);
    let inner = ScopeKey::new(21, 4);
    let host_inner = ScopeKey::new(21, 5);

    vm.add_import(ScopeKey::ROOT, "OuterLib", outer);
    vm.add_import(ScopeKey::ROOT, "InnerLib", host_inner);
    vm.add_import(outer, "InnerLib", inner);
    vm.register_source_module_scoped("mesh", ScopeKey::ROOT, "return { value = 7 }")
        .unwrap();
    vm.register_source_module_scoped("mesh", inner, "return { value = 42 }")
        .unwrap();
    vm.register_source_module_scoped("mesh", host_inner, "return { value = 84 }")
        .unwrap();
    let uses = vm
        .register_source_module_scoped(
            "uses",
            outer,
            "return { load = function() return require('lib:InnerLib/mesh') end }",
        )
        .unwrap();

    // Invoke the returned closure from Rust after registration has completed.
    // Scope must come from the closure's defining chunk, not mutable VM state.
    let load: Function = table(uses).get("load").unwrap();
    let resolved: Table = load.call(()).unwrap();
    assert_eq!(resolved.get::<i64>("value").unwrap(), 42);

    let host_uses = vm
        .register_source_module_scoped(
            "host_uses",
            ScopeKey::ROOT,
            "return require('lib:InnerLib/mesh')",
        )
        .unwrap();
    assert_eq!(table(host_uses).get::<i64>("value").unwrap(), 84);

    assert_eq!(
        vm.resolve_require("20-6/uses", "lib:InnerLib/mesh"),
        ScriptVm::scoped_module_key("mesh", inner)
    );
    assert_eq!(
        vm.resolve_require("host_uses", "lib:InnerLib/mesh"),
        ScriptVm::scoped_module_key("mesh", host_inner),
        "the same label must resolve through each caller's own pin table"
    );
}

#[test]
fn unpinned_library_require_never_rebinds_to_callers_same_named_module() {
    let vm = ScriptVm::new();
    vm.register_source_module_scoped("mesh", ScopeKey::ROOT, "return { value = 7 }")
        .unwrap();

    let error = vm
        .register_source_module_scoped("app", ScopeKey::ROOT, "return require('lib:geo/mesh')")
        .unwrap_err();
    let display = error.to_string();
    assert!(display.contains("lib:geo/mesh"), "got: {display}");
    assert!(
        !display.contains('\u{1f}'),
        "internal cache key leaked: {display}"
    );
    assert!(matches!(vm.registered_module("app").unwrap(), Value::Nil));
}

#[test]
fn scoped_source_errors_use_colon_free_readable_chunknames() {
    let vm = ScriptVm::new();
    let error = vm
        .register_source_module_scoped("thing", ScopeKey::new(9, 4), "error('boom')")
        .unwrap_err();
    let display = error.to_string();
    assert!(display.contains("9-4/thing"), "got: {display}");
    assert!(
        !display.contains('\u{1f}'),
        "internal cache key leaked: {display}"
    );
}

#[test]
fn scoped_bytecode_registration_retries_dependency_chain_in_its_scope() {
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
    let scope = ScopeKey::new(41, 9);
    let failures = vm.perform_scoped_registration(
        scripts
            .iter()
            .map(|(name, payload)| (*name, scope, *payload)),
    );
    assert!(
        failures.is_empty(),
        "scoped bytecode dependency registration did not converge: {}",
        failures
            .iter()
            .map(|(name, _, error)| format!("{name}: {error}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    assert!(matches!(
        vm.registered_module_scoped("Transform1", scope).unwrap(),
        Value::Table(_)
    ));
    assert!(matches!(
        vm.registered_module_scoped("ChainedConverter", scope)
            .unwrap(),
        Value::Function(_)
    ));
    assert!(matches!(
        vm.registered_module("Transform1").unwrap(),
        Value::Nil
    ));
}
