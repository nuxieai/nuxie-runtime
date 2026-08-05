//! Direct parity coverage for pinned `logging_scripting_context.cpp` and
//! `lua_rive_base.cpp`. Upstream has no asserted host-sink test, so this pins
//! the observable callback contract at the Rust host boundary.
#![cfg(feature = "luau")]

use std::cell::RefCell;
use std::rc::Rc;

use luaur_rt::{Error, Table};
use nuxie_runtime::{NoopScriptHost, ScriptInstance, ScriptMethod};
use nuxie_scripting::vm::{ScriptVm, ScriptingLogLevel};

#[test]
fn host_sink_receives_one_complete_info_line_per_nonempty_print_call() {
    let lines = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&lines);
    let vm = ScriptVm::new_with_log_sink(move |level, line| {
        captured.borrow_mut().push((level, line.to_vec()));
    });
    vm.install_rive_globals().expect("Rive globals install");

    vm.eval::<()>(
        r#"
        local object = setmetatable({}, {
            __tostring = function()
                return "custom"
            end,
        })
        print("alpha", 7, true, object)
        print()
        "#,
    )
    .expect("print script runs");

    assert_eq!(
        lines.borrow().as_slice(),
        [(ScriptingLogLevel::Info, b"alpha7truecustom".to_vec())]
    );
}

#[test]
fn reentrant_print_from_tostring_uses_the_context_owned_line_buffer() {
    let lines = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&lines);
    let vm = ScriptVm::new_with_log_sink(move |level, line| {
        captured.borrow_mut().push((level, line.to_vec()));
    });
    vm.install_rive_globals().expect("Rive globals install");

    vm.eval::<()>(
        r#"
        local object = setmetatable({}, {
            __tostring = function()
                print("inner")
                return "object"
            end,
        })
        print("discarded prefix", object, "suffix")
        "#,
    )
    .expect("reentrant print script runs");

    assert_eq!(
        lines.borrow().as_slice(),
        [
            (ScriptingLogLevel::Info, b"inner".to_vec()),
            (ScriptingLogLevel::Info, b"objectsuffix".to_vec()),
        ]
    );
}

#[test]
fn surfaced_lua_errors_are_routed_to_the_host_as_error_lines() {
    let lines = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&lines);
    let vm = ScriptVm::new_with_log_sink(move |level, line| {
        captured.borrow_mut().push((level, line.to_vec()));
    });
    vm.install_rive_globals().expect("Rive globals install");

    let error = vm
        .eval::<()>("error('host logging failure')")
        .expect_err("Lua failure surfaces");
    let Error::RuntimeError(raw_lua_message) = error else {
        panic!("expected runtime error, got {error:?}");
    };

    assert_eq!(
        lines.borrow().as_slice(),
        [(ScriptingLogLevel::Error, raw_lua_message.into_bytes())],
        "the sink receives Lua error text without Error::Display prefixes"
    );
}

#[test]
fn bytecode_load_and_invalid_module_results_are_error_lines() {
    let lines = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&lines);
    let vm = ScriptVm::new_with_log_sink(move |level, line| {
        captured.borrow_mut().push((level, line.to_vec()));
    });
    vm.install_rive_globals().expect("Rive globals install");

    let load_error = vm
        .load_bytecode("malformed", &[0, 1, 2, 3])
        .expect_err("validator rejects malformed bytecode");
    let Error::RuntimeError(raw_load_message) = load_error else {
        panic!("expected runtime error, got {load_error:?}");
    };

    let module_error = vm
        .register_source_module("invalid-result", "return 42")
        .expect_err("module result must be a table or function");
    let Error::RuntimeError(raw_module_message) = module_error else {
        panic!("expected runtime error, got {module_error:?}");
    };

    assert_eq!(
        lines.borrow().as_slice(),
        [
            (ScriptingLogLevel::Error, raw_load_message.into_bytes()),
            (ScriptingLogLevel::Error, raw_module_message.into_bytes()),
        ]
    );
}

#[test]
fn protocol_callback_errors_use_the_same_vm_owned_host_sink() {
    let lines = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&lines);
    let vm = ScriptVm::new_with_log_sink(move |level, line| {
        captured.borrow_mut().push((level, line.to_vec()));
    });
    vm.install_rive_globals().expect("Rive globals install");
    let table: Table = vm
        .eval(
            r#"
            return {
                update = function()
                    error("callback logging failure")
                end,
            }
            "#,
        )
        .expect("protocol table evaluates");
    let mut instance = vm.script_instance_from_table(table);

    let error = instance
        .call_method(ScriptMethod::Update, &[], &mut NoopScriptHost)
        .expect_err("protocol callback fails");

    let raw_lua_message = error
        .message()
        .strip_prefix("runtime error: ")
        .expect("ScriptError retains the luaur display prefix");

    assert_eq!(
        lines.borrow().as_slice(),
        [(
            ScriptingLogLevel::Error,
            raw_lua_message.as_bytes().to_vec()
        )]
    );
}

#[test]
fn a_sink_can_be_replaced_after_globals_are_sandboxed() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().expect("Rive globals install");

    let lines = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&lines);
    vm.set_log_sink(move |level, line| {
        captured.borrow_mut().push((level, line.to_vec()));
    });
    vm.eval::<()>("print('late sink')")
        .expect("late sink receives print");

    assert_eq!(
        lines.borrow().as_slice(),
        [(ScriptingLogLevel::Info, b"late sink".to_vec())]
    );
}
mod support;
use support::ScriptVmSourceTestExt as _;
