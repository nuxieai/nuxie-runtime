#![cfg(feature = "luau")]

use std::collections::BTreeMap;

use nuxie_runtime::{
    NoopScriptHost, ScriptDataConverterMethod, ScriptInstance, ScriptMethod, ScriptValue,
};
use nuxie_scripting::host_commands::{HostCommand, HostCommandHost, HostCommandLimits, HostValue};
use nuxie_scripting::vm::ScriptVm;

#[test]
fn caller_named_module_queues_generic_commands_in_fifo_order() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().expect("Rive globals install");
    let host = HostCommandHost::install(&vm, "bridge", HostCommandLimits::default())
        .expect("generic host module installs");

    vm.begin_script_cycle();
    let checkpoint = host.begin_cycle();
    vm.lua()
        .load(
            r#"
            local bridge = require("bridge")
            bridge.command("opened", nil)
            bridge.command("selected", {
                sku = "sku-1",
                flags = { true, false },
            })
        "#,
        )
        .eval::<()>()
        .expect("commands execute");

    assert_eq!(
        host.drain(checkpoint),
        vec![
            HostCommand {
                name: "opened".to_owned(),
                payload: HostValue::Null,
            },
            HostCommand {
                name: "selected".to_owned(),
                payload: HostValue::Object(BTreeMap::from([
                    (
                        "flags".to_owned(),
                        HostValue::List(vec![HostValue::Bool(true), HostValue::Bool(false)]),
                    ),
                    ("sku".to_owned(), HostValue::String("sku-1".to_owned())),
                ])),
            },
        ]
    );
    vm.end_script_cycle();
}

#[test]
fn failed_cycle_discards_every_command_before_the_next_cycle() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().expect("Rive globals install");
    let host = HostCommandHost::install(&vm, "bridge", HostCommandLimits::default())
        .expect("generic host module installs");

    vm.begin_script_cycle();
    let failed = host.begin_cycle();
    let error = vm
        .lua()
        .load(
            r#"
                local bridge = require("bridge")
                bridge.command("must_not_escape", { value = 1 })
                error("fail after enqueue")
            "#,
        )
        .eval::<()>()
        .expect_err("authored failure aborts the cycle");
    assert!(error.to_string().contains("fail after enqueue"));
    host.rollback_cycle(failed);
    vm.end_script_cycle();

    vm.begin_script_cycle();
    let succeeding = host.begin_cycle();
    vm.lua()
        .load(
            r#"
                local bridge = require("bridge")
                bridge.command("next", true)
            "#,
        )
        .eval::<()>()
        .expect("next cycle succeeds");
    assert_eq!(
        host.drain(succeeding),
        vec![HostCommand {
            name: "next".to_owned(),
            payload: HostValue::Bool(true),
        }]
    );
    vm.end_script_cycle();
}

#[test]
fn malformed_payload_never_enqueues_a_partial_command() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().expect("Rive globals install");
    let host = HostCommandHost::install(&vm, "bridge", HostCommandLimits::default())
        .expect("generic host module installs");

    vm.begin_script_cycle();
    let checkpoint = host.begin_cycle();
    let error = vm
        .lua()
        .load(
            r#"
                local bridge = require("bridge")
                local cyclic = {}
                cyclic.self = cyclic
                bridge.command("invalid", cyclic)
            "#,
        )
        .eval::<()>()
        .expect_err("cyclic values are rejected");
    assert!(error.to_string().contains("cyclic host command value"));
    assert!(host.drain(checkpoint).is_empty());
    vm.end_script_cycle();
}

#[test]
fn empty_table_has_the_deterministic_object_representation() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().expect("Rive globals install");
    let host = HostCommandHost::install(&vm, "bridge", HostCommandLimits::default())
        .expect("generic host module installs");

    vm.begin_script_cycle();
    let checkpoint = host.begin_cycle();
    vm.lua()
        .load(
            r#"
                local bridge = require("bridge")
                bridge.command("empty", {})
            "#,
        )
        .eval::<()>()
        .expect("empty object command succeeds");
    assert_eq!(
        host.drain(checkpoint),
        vec![HostCommand {
            name: "empty".to_owned(),
            payload: HostValue::Object(BTreeMap::new()),
        }]
    );
    vm.end_script_cycle();
}

#[test]
fn module_never_runs_host_work_synchronously_or_outside_a_cycle() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().expect("Rive globals install");
    let host = HostCommandHost::install(&vm, "bridge", HostCommandLimits::default())
        .expect("generic host module installs");

    let error = vm
        .lua()
        .load(
            r#"
                local bridge = require("bridge")
                bridge.command("outside", true)
            "#,
        )
        .eval::<()>()
        .expect_err("the module cannot publish outside transaction ownership");
    assert!(error.to_string().contains("active runtime transaction"));

    vm.begin_script_cycle();
    let checkpoint = host.begin_cycle();
    assert!(host.drain(checkpoint).is_empty());
    vm.end_script_cycle();
}

#[test]
fn converter_callback_failure_is_recorded_for_transaction_veto_and_rollback() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().expect("Rive globals install");
    let host = HostCommandHost::install(&vm, "bridge", HostCommandLimits::default())
        .expect("generic host module installs");
    let table = vm
        .lua()
        .load(
            r#"
                local bridge = require("bridge")
                return {
                    convert = function(_self, _value)
                        bridge.command("must_rollback", { source = "converter" })
                        error("converter failed after command")
                    end,
                }
            "#,
        )
        .eval()
        .expect("converter table loads");
    let mut instance = vm.script_instance_from_table(table);

    vm.begin_script_cycle();
    let failed = host.begin_cycle();
    let error = instance
        .call_data_converter(ScriptDataConverterMethod::Convert, ScriptValue::Number(1.0))
        .expect_err("converter callback fails");
    assert!(error.to_string().contains("converter failed after command"));
    assert!(
        host.callback_failure()
            .is_some_and(|failure| failure.contains("converter failed after command")),
        "the transaction-level side channel must survive a runtime path swallowing the error"
    );
    host.rollback_cycle(failed);
    vm.end_script_cycle();

    vm.begin_script_cycle();
    let next = host.begin_cycle();
    assert!(host.callback_failure().is_none());
    assert!(host.drain(next).is_empty());
    vm.end_script_cycle();
}

#[test]
fn transition_callback_failure_is_recorded_for_transaction_veto_and_rollback() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().expect("Rive globals install");
    let host = HostCommandHost::install(&vm, "bridge", HostCommandLimits::default())
        .expect("generic host module installs");
    let table = vm
        .lua()
        .load(
            r#"
                local bridge = require("bridge")
                return {
                    evaluate = function(_self)
                        bridge.command("must_rollback", { source = "transition" })
                        error("transition failed after command")
                    end,
                }
            "#,
        )
        .eval()
        .expect("transition table loads");
    let mut instance = vm.script_instance_from_table(table);

    vm.begin_script_cycle();
    let failed = host.begin_cycle();
    let error = instance
        .call_method(ScriptMethod::Evaluate, &[], &mut NoopScriptHost)
        .expect_err("transition callback fails");
    assert!(
        error
            .to_string()
            .contains("transition failed after command")
    );
    assert!(
        host.callback_failure()
            .is_some_and(|failure| failure.contains("transition failed after command")),
        "transition evaluation currently consumes ordinary errors as false, so commit must veto"
    );
    host.rollback_cycle(failed);
    vm.end_script_cycle();

    vm.begin_script_cycle();
    let next = host.begin_cycle();
    assert!(host.callback_failure().is_none());
    assert!(host.drain(next).is_empty());
    vm.end_script_cycle();
}
