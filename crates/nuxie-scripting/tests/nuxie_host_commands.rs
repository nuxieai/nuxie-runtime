#![cfg(feature = "luau")]

use std::collections::BTreeMap;

use nuxie_scripting::vm::{HostCommand, HostValue, ScriptResourceLimit, ScriptVm};

#[test]
fn nuxie_trigger_normalizes_a_scalar_payload_and_queues_it() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result: luaur_rt::Value = vm
        .eval(
            r#"
                local nuxie = require("nuxie")
                return nuxie.trigger("checkout", 42)
            "#,
        )
        .unwrap();

    assert!(matches!(result, luaur_rt::Value::Nil));
    assert_eq!(
        vm.drain_host_commands(),
        vec![HostCommand::Trigger {
            name: "checkout".to_owned(),
            properties: BTreeMap::from([("value".to_owned(), HostValue::Number(42.0))]),
        }]
    );
}

#[test]
fn nuxie_trigger_normalizes_an_array_payload_under_value() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    vm.eval::<()>(r#"require("nuxie").trigger("checkout", { "a", "b" })"#)
        .unwrap();

    assert_eq!(
        vm.drain_host_commands(),
        vec![HostCommand::Trigger {
            name: "checkout".to_owned(),
            properties: BTreeMap::from([(
                "value".to_owned(),
                HostValue::Array(vec![
                    HostValue::String("a".to_owned()),
                    HostValue::String("b".to_owned()),
                ]),
            )]),
        }]
    );
}

#[test]
fn nuxie_host_functions_queue_commands_in_call_order_and_return_nil() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let results: (luaur_rt::Value, luaur_rt::Value) = vm
        .eval(
            r#"
                local nuxie = require("nuxie")
                return nuxie.trigger("opened"), nuxie.response.set("plan", "pro")
            "#,
        )
        .unwrap();

    assert!(matches!(results.0, luaur_rt::Value::Nil));
    assert!(matches!(results.1, luaur_rt::Value::Nil));
    assert_eq!(
        vm.drain_host_commands(),
        vec![
            HostCommand::Trigger {
                name: "opened".to_owned(),
                properties: BTreeMap::new(),
            },
            HostCommand::ResponseSet {
                field: "plan".to_owned(),
                value: HostValue::String("pro".to_owned()),
            },
        ]
    );
}

#[test]
fn host_values_preserve_nested_objects_and_arrays_with_sorted_object_keys() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            nuxie.trigger("catalog", {
                zeta = 3,
                alpha = "first",
                nested = { true, { id = "sku-1", enabled = false } },
            })
            nuxie.response.set("selection", { "sku-1", "sku-2" })
        "#,
    )
    .unwrap();

    assert_eq!(
        vm.drain_host_commands(),
        vec![
            HostCommand::Trigger {
                name: "catalog".to_owned(),
                properties: BTreeMap::from([
                    ("alpha".to_owned(), HostValue::String("first".to_owned())),
                    (
                        "nested".to_owned(),
                        HostValue::Array(vec![
                            HostValue::Bool(true),
                            HostValue::Object(BTreeMap::from([
                                ("enabled".to_owned(), HostValue::Bool(false)),
                                ("id".to_owned(), HostValue::String("sku-1".to_owned()),),
                            ])),
                        ]),
                    ),
                    ("zeta".to_owned(), HostValue::Number(3.0)),
                ]),
            },
            HostCommand::ResponseSet {
                field: "selection".to_owned(),
                value: HostValue::Array(vec![
                    HostValue::String("sku-1".to_owned()),
                    HostValue::String("sku-2".to_owned()),
                ]),
            },
        ]
    );
}

#[test]
fn response_set_with_nil_emits_no_command() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result: luaur_rt::Value = vm
        .eval(
            r#"
                local nuxie = require("nuxie")
                return nuxie.response.set("plan", nil)
            "#,
        )
        .unwrap();

    assert!(matches!(result, luaur_rt::Value::Nil));
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn nuxie_module_is_require_only_and_readonly_after_sandboxing() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let (global_is_private, same_module, module_write_failed, nested_write_failed): (
        bool,
        bool,
        bool,
        bool,
    ) = vm
        .eval(
            r#"
                local first = require("nuxie")
                local second = require("nuxie")
                local moduleWrite = pcall(function()
                    first.trigger = nil
                end)
                local nestedWrite = pcall(function()
                    first.response.set = nil
                end)
                return Nuxie == nil and nuxie == nil,
                    first == second,
                    not moduleWrite,
                    not nestedWrite
            "#,
        )
        .unwrap();

    assert!(global_is_private);
    assert!(same_module);
    assert!(module_write_failed);
    assert!(nested_write_failed);
}

#[test]
fn non_finite_numbers_are_rejected_without_queuing_a_partial_command() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let accepted: bool = vm
        .eval(
            r#"
                local nuxie = require("nuxie")
                return pcall(function()
                    nuxie.trigger("invalid", 0 / 0)
                end)
            "#,
        )
        .unwrap();

    assert!(!accepted);
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn invalid_table_shapes_and_unsupported_values_are_transactionally_rejected() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let accepted: (bool, bool, bool, bool, bool, bool) = vm
        .eval(
            r#"
                local nuxie = require("nuxie")
                local mixed = pcall(function()
                    nuxie.response.set("invalid", { [1] = "a", named = "b" })
                end)
                local sparse = pcall(function()
                    nuxie.response.set("invalid", { [1] = "a", [3] = "c" })
                end)
                local unsupported = pcall(function()
                    nuxie.response.set("invalid", function() end)
                end)
                local emptyTrigger = pcall(function()
                    nuxie.trigger("")
                end)
                local emptyField = pcall(function()
                    nuxie.response.set("", "value")
                end)
                local emptyObjectKey = pcall(function()
                    nuxie.response.set("invalid", { [""] = "value" })
                end)
                return mixed, sparse, unsupported,
                    emptyTrigger, emptyField, emptyObjectKey
            "#,
        )
        .unwrap();

    assert_eq!(accepted, (false, false, false, false, false, false));
    assert_eq!(vm.terminal_resource_limit(), None);
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn host_value_depth_is_limited_to_thirty_two_levels() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
                local nuxie = require("nuxie")
                local root = {}
                local cursor = root
                for _ = 1, 32 do
                    cursor.next = {}
                    cursor = cursor.next
                end
                pcall(function()
                    nuxie.response.set("too_deep", root)
                end)
                nuxie.trigger("must_not_escape")
            "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::HostDepth)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn cyclic_tables_are_rejected_as_cycles_without_queuing() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let (accepted, error): (bool, String) = vm
        .eval(
            r#"
                local nuxie = require("nuxie")
                local cyclic = {}
                cyclic.self = cyclic
                local ok, err = pcall(function()
                    nuxie.response.set("cyclic", cyclic)
                end)
                return ok, tostring(err)
            "#,
        )
        .unwrap();

    assert!(!accepted);
    assert!(
        error.contains("cyclic host value"),
        "unexpected error: {error}"
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn host_value_nodes_include_scalars_and_are_limited_to_four_thousand_ninety_six() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
                local nuxie = require("nuxie")
                local nodes = {}
                for index = 1, 4096 do
                    nodes[index] = index
                end
                pcall(function()
                    nuxie.response.set("too_many_nodes", nodes)
                end)
                nuxie.trigger("must_not_escape")
            "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::HostNodes)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn host_value_edges_are_limited_to_sixteen_thousand_three_hundred_eighty_four() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
                local nuxie = require("nuxie")
                local edges = {}
                for index = 1, 16385 do
                    edges[index] = index
                end
                pcall(function()
                    nuxie.response.set("too_many_edges", edges)
                end)
                nuxie.trigger("must_not_escape")
            "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::HostEdges)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn host_value_edge_preflight_accepts_the_exact_edge_bound() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let (accepted, error): (bool, String) = vm
        .eval(
            r#"
                local nuxie = require("nuxie")
                local edges = {}
                local unsupported = function() end
                for index = 1, 16384 do
                    edges[index] = unsupported
                end
                local ok, err = pcall(function()
                    nuxie.response.set("exact_edges", edges)
                end)
                return ok, tostring(err)
            "#,
        )
        .unwrap();

    assert!(!accepted);
    assert!(
        error.contains("unsupported host value type function"),
        "unexpected error: {error}"
    );
    assert_eq!(vm.terminal_resource_limit(), None);
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn aggregate_host_nodes_reject_many_individually_bounded_empty_containers() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            local function valueWithNodes(childCount)
                local value = {}
                for index = 1, childCount do
                    value[index] = {}
                end
                return value
            end

            for index = 1, 255 do
                nuxie.response.set("accepted_" .. tostring(index), valueWithNodes(15))
            end
            pcall(function()
                nuxie.response.set("aggregate_overflow", valueWithNodes(16))
            end)
            nuxie.trigger("must_not_escape")
        "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::HostNodes)
    );
    assert_eq!(vm.drain_host_commands().len(), 255);
}

#[test]
fn aggregate_host_nodes_accept_the_exact_bound_and_count_the_trigger_property_root() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            local function valueWithNodes(childCount)
                local value = {}
                for index = 1, childCount do
                    value[index] = {}
                end
                return value
            end

            for index = 1, 255 do
                nuxie.response.set("accepted_" .. tostring(index), valueWithNodes(15))
            end

            local properties = {}
            for index = 1, 15 do
                properties["key_" .. tostring(index)] = {}
            end
            nuxie.trigger("exact_bound", properties)
        "#,
    )
    .unwrap();

    assert_eq!(vm.terminal_resource_limit(), None);
    assert_eq!(vm.drain_host_commands().len(), 256);
}

#[test]
fn individual_host_strings_are_limited_to_one_mebibyte() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
                local nuxie = require("nuxie")
                local oversized = string.rep("x", 1024 * 1024 + 1)
                pcall(function()
                    nuxie.response.set("oversized", oversized)
                end)
                nuxie.trigger("must_not_escape")
            "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::HostString)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn host_command_names_fields_and_object_keys_use_the_identifier_bound() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
                local nuxie = require("nuxie")
                local oversized = string.rep("x", 4097)
                pcall(function()
                    nuxie.trigger(oversized)
                end)
                nuxie.trigger("must_not_escape")
            "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::HostIdentifier)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn aggregate_host_value_content_is_limited_to_four_mebibytes() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
                local nuxie = require("nuxie")
                local chunk = string.rep("x", 900000)
                pcall(function()
                    nuxie.response.set("oversized", {
                        chunk, chunk, chunk, chunk, chunk,
                    })
                end)
                nuxie.trigger("must_not_escape")
            "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::HostValueBytes)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn aggregate_host_command_content_is_limited_across_the_whole_cycle() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
                local nuxie = require("nuxie")
                local oneMiB = string.rep("x", 1024 * 1024)
                pcall(function()
                    for index = 1, 5 do
                        nuxie.response.set("field_" .. tostring(index), oneMiB)
                    end
                end)
                nuxie.trigger("must_not_escape")
            "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::CommandContent)
    );
    assert_eq!(vm.drain_host_commands().len(), 3);
}

#[test]
fn aggregate_host_command_content_accepts_the_exact_cycle_limit() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            local oneMiB = string.rep("x", 1024 * 1024)
            for index = 1, 3 do
                nuxie.response.set("field_" .. tostring(index), oneMiB)
            end
            nuxie.response.set("field_4", string.rep("y", 1024 * 1024 - 120))
        "#,
    )
    .unwrap();

    assert_eq!(vm.drain_host_commands().len(), 4);
}

#[test]
fn rolling_back_a_failed_cycle_preserves_only_pre_cycle_commands() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();
    vm.eval::<()>(r#"require("nuxie").trigger("during_import")"#)
        .unwrap();

    let checkpoint = vm.begin_host_cycle();
    vm.eval::<()>(r#"require("nuxie").trigger("failed_frame")"#)
        .unwrap();
    vm.rollback_host_cycle(checkpoint);

    assert_eq!(
        vm.drain_host_commands(),
        vec![HostCommand::Trigger {
            name: "during_import".to_owned(),
            properties: BTreeMap::new(),
        }]
    );
}

#[test]
fn cycle_begin_and_full_rollback_reset_aggregate_host_structure_budget() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    vm.eval::<()>(
        r#"
            local value = {}
            for index = 1, 2999 do
                value[index] = {}
            end
            require("nuxie").response.set("before_cycle", value)
        "#,
    )
    .unwrap();

    let checkpoint = vm.begin_host_cycle();
    vm.eval::<()>(
        r#"
            local value = {}
            for index = 1, 2999 do
                value[index] = {}
            end
            require("nuxie").response.set("rolled_back", value)
        "#,
    )
    .unwrap();
    vm.rollback_host_cycle(checkpoint);

    vm.eval::<()>(
        r#"
            local value = {}
            for index = 1, 2999 do
                value[index] = {}
            end
            require("nuxie").response.set("after_rollback", value)
        "#,
    )
    .unwrap();

    assert_eq!(vm.terminal_resource_limit(), None);
    assert_eq!(vm.drain_host_commands().len(), 2);
}

#[test]
fn each_cycle_accepts_at_most_two_hundred_fifty_six_commands() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let error = vm
        .eval::<()>(
            r#"
                local nuxie = require("nuxie")
                for index = 1, 257 do
                    nuxie.trigger("event_" .. tostring(index))
                end
            "#,
        )
        .unwrap_err();

    assert!(error.to_string().contains("256 host commands"));
    assert_eq!(vm.drain_host_commands().len(), 256);

    vm.begin_host_cycle();
    vm.eval::<()>(r#"require("nuxie").trigger("next_cycle")"#)
        .unwrap();
    assert_eq!(vm.drain_host_commands().len(), 1);
}

#[test]
fn command_exhaustion_cannot_be_swallowed_by_pcall() {
    let vm = ScriptVm::new();
    let checkpoint = vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            pcall(function()
                for index = 1, 257 do
                    nuxie.trigger("event_" .. tostring(index))
                end
            end)
        "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::Commands)
    );
    vm.rollback_host_cycle(checkpoint);
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::Commands)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn each_cycle_is_limited_to_one_hundred_thousand_interrupt_safepoints() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();

    let error = vm
        .eval::<()>(
            r#"
                local total = 0
                for index = 1, 200001 do
                    total += index
                end
            "#,
        )
        .unwrap_err();

    assert!(error.to_string().contains("100000 script safepoints"));
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::Safepoints)
    );

    vm.begin_host_cycle();
    assert_eq!(vm.eval::<i64>("return 42").unwrap(), 42);
}

#[test]
fn instruction_exhaustion_cannot_be_swallowed_to_emit_host_effects() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            pcall(function()
                local total = 0
                for index = 1, 200001 do
                    total += index
                end
            end)
            nuxie.trigger("must_not_escape")
        "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::Safepoints)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn each_luau_vm_has_a_sixteen_mebibyte_memory_ceiling() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();

    let error = vm
        .eval::<luaur_rt::Value>(r#"return string.rep("x", 32 * 1024 * 1024)"#)
        .unwrap_err();

    assert!(
        matches!(error, luaur_rt::Error::MemoryError(_)),
        "unexpected error: {error}"
    );
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::Memory)
    );
}

#[test]
fn memory_exhaustion_cannot_be_swallowed_to_emit_host_effects() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            pcall(function()
                string.rep("x", 32 * 1024 * 1024)
            end)
            nuxie.trigger("must_not_escape")
        "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::Memory)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn xpcall_error_handlers_cannot_hide_memory_exhaustion() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            xpcall(
                function() string.rep("x", 32 * 1024 * 1024) end,
                function() return "hidden" end
            )
            nuxie.trigger("must_not_escape")
        "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::Memory)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn coroutine_resume_cannot_hide_memory_exhaustion() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            local worker = coroutine.create(function()
                string.rep("x", 32 * 1024 * 1024)
            end)
            coroutine.resume(worker)
            nuxie.trigger("must_not_escape")
        "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::Memory)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn xpcall_handler_cannot_observe_or_hide_a_typed_host_limit() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            local oversized = string.rep("x", 4097)
            xpcall(
                function() nuxie.trigger(oversized) end,
                function()
                    nuxie.trigger("handler_must_not_escape")
                    return "hidden"
                end
            )
            nuxie.trigger("must_not_escape")
        "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::HostIdentifier)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn coroutine_resume_cannot_hide_a_typed_host_limit() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let result = vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            local oversized = string.rep("x", 1024 * 1024 + 1)
            local worker = coroutine.create(function()
                nuxie.response.set("too_large", oversized)
            end)
            coroutine.resume(worker)
            nuxie.trigger("must_not_escape")
        "#,
    );

    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::HostString)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn terminal_limit_is_cycle_local_machine_readable_and_isolated_per_vm() {
    let exhausted = ScriptVm::new();
    exhausted.begin_host_cycle();
    exhausted.install_rive_globals().unwrap();
    let sibling = ScriptVm::new();
    sibling.begin_host_cycle();
    sibling.install_rive_globals().unwrap();

    assert!(
        exhausted
            .eval::<()>(
                r#"
                    local oversized = string.rep("x", 4097)
                    pcall(function() require("nuxie").trigger(oversized) end)
                "#,
            )
            .is_err()
    );
    let limit = exhausted
        .terminal_resource_limit()
        .expect("the exhausted VM retains a typed limit");
    assert_eq!(limit, ScriptResourceLimit::HostIdentifier);
    assert_eq!(limit.code(), "script.resource.host_identifier");

    sibling
        .eval::<()>(r#"require("nuxie").trigger("sibling_ok")"#)
        .unwrap();
    assert_eq!(sibling.terminal_resource_limit(), None);
    assert_eq!(sibling.drain_host_commands().len(), 1);

    exhausted.begin_host_cycle();
    assert_eq!(exhausted.terminal_resource_limit(), None);
    exhausted
        .eval::<()>(r#"require("nuxie").trigger("next_cycle_ok")"#)
        .unwrap();
    assert_eq!(exhausted.drain_host_commands().len(), 1);
}

#[test]
fn effect_rollback_truncates_commands_without_refunding_cycle_budget() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();
    vm.eval::<()>(r#"require("nuxie").trigger("before")"#)
        .unwrap();

    let effects = vm.checkpoint_host_effects();
    vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            for index = 1, 254 do
                nuxie.trigger("rolled_back_" .. tostring(index))
            end
        "#,
    )
    .unwrap();
    vm.rollback_host_effects(effects);

    vm.eval::<()>(r#"require("nuxie").trigger("last_allowed")"#)
        .unwrap();
    let result = vm.eval::<()>(
        r#"
            pcall(function()
                require("nuxie").trigger("budget_was_not_refunded")
            end)
        "#,
    );
    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::Commands)
    );
    assert_eq!(
        vm.drain_host_commands(),
        vec![
            HostCommand::Trigger {
                name: "before".to_owned(),
                properties: BTreeMap::new(),
            },
            HostCommand::Trigger {
                name: "last_allowed".to_owned(),
                properties: BTreeMap::new(),
            },
        ]
    );
}

#[test]
fn effect_rollback_does_not_refund_host_command_content_budget() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let effects = vm.checkpoint_host_effects();
    vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            local oneMiB = string.rep("x", 1024 * 1024)
            for index = 1, 3 do
                nuxie.response.set("rolled_" .. tostring(index), oneMiB)
            end
        "#,
    )
    .unwrap();
    vm.rollback_host_effects(effects);
    assert!(vm.drain_host_commands().is_empty());

    let result = vm.eval::<()>(
        r#"
            local oneMiB = string.rep("x", 1024 * 1024)
            pcall(function()
                require("nuxie").response.set("not_refunded", oneMiB)
            end)
        "#,
    );
    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::CommandContent)
    );
    assert!(vm.drain_host_commands().is_empty());
}

#[test]
fn effect_rollback_does_not_refund_aggregate_host_structure_budget() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let effects = vm.checkpoint_host_effects();
    vm.eval::<()>(
        r#"
            local value = {}
            for index = 1, 2047 do
                value[index] = {}
            end
            require("nuxie").response.set("rolled_back", value)
        "#,
    )
    .unwrap();
    vm.rollback_host_effects(effects);
    assert!(vm.drain_host_commands().is_empty());

    vm.eval::<()>(
        r#"
            local value = {}
            for index = 1, 2047 do
                value[index] = {}
            end
            require("nuxie").response.set("survives", value)
        "#,
    )
    .unwrap();

    let result = vm.eval::<()>(
        r#"
            pcall(function()
                require("nuxie").trigger("not_refunded")
            end)
        "#,
    );
    assert!(result.is_err());
    assert_eq!(
        vm.terminal_resource_limit(),
        Some(ScriptResourceLimit::HostNodes)
    );
    assert_eq!(vm.drain_host_commands().len(), 1);
}

#[test]
fn protected_calls_still_report_ordinary_script_errors_as_values() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    let (pcall_ok, pcall_error, xpcall_ok, xpcall_error, resume_ok, resume_error): (
        bool,
        String,
        bool,
        String,
        bool,
        String,
    ) = vm
        .eval(
            r#"
                local pcallOk, pcallError = pcall(function() error("boom") end)
                local xpcallOk, xpcallError = xpcall(
                    function() error("boom") end,
                    function(err) return "handled:" .. tostring(err) end
                )
                local worker = coroutine.create(function() error("boom") end)
                local resumeOk, resumeError = coroutine.resume(worker)
                return pcallOk, tostring(pcallError),
                    xpcallOk, tostring(xpcallError),
                    resumeOk, tostring(resumeError)
            "#,
        )
        .unwrap();

    assert!(!pcall_ok);
    assert!(pcall_error.contains("boom"));
    assert!(!xpcall_ok);
    assert!(xpcall_error.contains("handled:") && xpcall_error.contains("boom"));
    assert!(!resume_ok);
    assert!(resume_error.contains("boom"));
}

#[test]
fn host_values_at_the_exact_depth_node_and_string_limits_are_accepted() {
    let vm = ScriptVm::new();
    vm.begin_host_cycle();
    vm.install_rive_globals().unwrap();

    vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")

            local depth = {}
            local cursor = depth
            for _ = 1, 31 do
                cursor.next = {}
                cursor = cursor.next
            end
            nuxie.response.set("depth", depth)
        "#,
    )
    .unwrap();
    assert_eq!(vm.drain_host_commands().len(), 1);
    vm.begin_host_cycle();

    vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            local nodes = {}
            for index = 1, 4095 do
                nodes[index] = index
            end
            nuxie.response.set("nodes", nodes)
        "#,
    )
    .unwrap();
    assert_eq!(vm.drain_host_commands().len(), 1);
    vm.begin_host_cycle();

    vm.eval::<()>(
        r#"
            local nuxie = require("nuxie")
            local oneMiB = string.rep("x", 1024 * 1024)
            nuxie.response.set("string", oneMiB)
        "#,
    )
    .unwrap();
    assert_eq!(vm.drain_host_commands().len(), 1);
}
