#![cfg(feature = "scripting")]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    reason = "C-ABI fixture tests use bounded counters and explicit teardown"
)]

use luaur_compiler::functions::luau_compile::luau_compile;
use nux_capi::*;
use nuxie_schema::definition_by_name;

fn compile_luau(source: &[u8]) -> Vec<u8> {
    luaur_common::set_all_flags(true);
    let mut output_size = 0;
    let output = luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut output_size,
    );
    assert!(!output.is_null());
    unsafe { std::slice::from_raw_parts(output.cast(), output_size) }.to_vec()
}

fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = definition_by_name(type_name).unwrap();
    definition
        .properties
        .iter()
        .chain(
            definition
                .ancestors
                .iter()
                .flat_map(|ancestor| definition_by_name(ancestor).unwrap().properties.iter()),
        )
        .find(|property| property.name == property_name)
        .unwrap()
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(definition_by_name(type_name).unwrap().type_key.int),
    );
    properties(bytes);
    push_var_uint(bytes, 0);
}

fn push_uint(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u64) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value);
}

fn push_f32(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: f32) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_blob(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &[u8]) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

fn push_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
    push_blob(bytes, type_name, name, value.as_bytes());
}

fn successful_source() -> &'static [u8] {
    br#"
        local bridge = require("bridge")
        return function(_context)
            return {
                init = function(_self) return true end,
                performAction = function(_self, _invocation)
                    bridge.command("opened", nil)
                    bridge.command("selected", {
                        sku = "sku-1",
                        quantity = 2.5,
                        flags = { true, false },
                    })
                end,
            }
        end
    "#
}

fn scripted_fixture(source: &[u8]) -> Vec<u8> {
    let mut payload = vec![0];
    payload.extend(compile_luau(source));

    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 18_250);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "GenericHostCommands");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 100.0);
        push_f32(bytes, "ParametricPath", "height", 100.0);
    });
    push_object(&mut bytes, "StateMachine", |bytes| {
        push_string(bytes, "StateMachine", "name", "HostCommands");
    });
    push_object(&mut bytes, "StateMachineListenerSingle", |bytes| {
        push_uint(bytes, "StateMachineListener", "targetId", 1);
        push_uint(bytes, "StateMachineListenerSingle", "listenerTypeValue", 2);
    });
    push_object(&mut bytes, "ScriptedListenerAction", |bytes| {
        push_uint(bytes, "ScriptedListenerAction", "scriptAssetId", 0);
    });
    bytes
}

fn scripted_drawable_fixture(source: &[u8]) -> Vec<u8> {
    let mut payload = vec![0];
    payload.extend(compile_luau(source));

    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 18_251);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "GenericHostDrawable");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
    });
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 0);
    });
    push_object(&mut bytes, "StateMachine", |bytes| {
        push_string(bytes, "StateMachine", "name", "DrawableAdvance");
    });
    bytes
}

fn scripted_transition_fixture(source: &[u8]) -> Vec<u8> {
    let mut payload = vec![0];
    payload.extend(compile_luau(source));

    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 18_252);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "GenericHostTransition");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
    });
    push_object(&mut bytes, "LinearAnimation", |bytes| {
        push_string(bytes, "LinearAnimation", "name", "Idle");
    });
    push_object(&mut bytes, "StateMachine", |bytes| {
        push_string(bytes, "StateMachine", "name", "TransitionEvaluate");
    });
    push_object(&mut bytes, "StateMachineLayer", |_| {});
    push_object(&mut bytes, "AnyState", |_| {});
    push_object(&mut bytes, "StateTransition", |bytes| {
        push_uint(bytes, "StateTransition", "stateToId", 2);
    });
    push_object(&mut bytes, "ScriptedTransitionCondition", |bytes| {
        push_uint(bytes, "ScriptedTransitionCondition", "scriptAssetId", 0);
    });
    push_object(&mut bytes, "EntryState", |_| {});
    push_object(&mut bytes, "StateTransition", |bytes| {
        push_uint(bytes, "StateTransition", "stateToId", 2);
    });
    push_object(&mut bytes, "AnimationState", |bytes| {
        push_uint(bytes, "AnimationState", "animationId", 0);
    });
    push_object(&mut bytes, "ExitState", |_| {});
    bytes
}

fn view(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}

fn copy(value: NuxStringView) -> String {
    if value.len == 0 {
        return String::new();
    }
    String::from_utf8(
        unsafe { std::slice::from_raw_parts(value.data.cast::<u8>(), value.len) }.to_vec(),
    )
    .unwrap()
}

fn raw_step(
    player: *mut NuxPlayer,
    pointers: &[NuxPlayerPointerEvent],
) -> (NuxStatus, *mut NuxPlayerStepResult) {
    let operation = NuxPlayerStep {
        struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
        pointers: pointers.as_ptr(),
        pointer_count: pointers.len(),
        elapsed_seconds: 0.016,
        ..NuxPlayerStep::default()
    };
    let mut result = std::ptr::null_mut();
    let status = unsafe { nux_player_step(player, &operation, &mut result) };
    (status, result)
}

fn step(player: *mut NuxPlayer, pointers: &[NuxPlayerPointerEvent]) -> *mut NuxPlayerStepResult {
    let (status, result) = raw_step(player, pointers);
    if status != NuxStatus::Ok {
        let mut diagnostic = NuxCapiDiagnosticView::default();
        assert_eq!(
            unsafe { nux_player_step_result_diagnostic(result, &mut diagnostic) },
            NuxStatus::Ok
        );
        panic!(
            "player step failed: {status:?}: {}: {}",
            copy(diagnostic.code),
            copy(diagnostic.message)
        );
    }
    result
}

fn pointer_click() -> [NuxPlayerPointerEvent; 2] {
    [
        NuxPlayerPointerEvent {
            kind: NUX_PLAYER_POINTER_KIND_DOWN,
            x: 50.0,
            y: 50.0,
            pointer_id: 0,
            timestamp_seconds: 0.0,
        },
        NuxPlayerPointerEvent {
            kind: NUX_PLAYER_POINTER_KIND_UP,
            x: 50.0,
            y: 50.0,
            pointer_id: 0,
            timestamp_seconds: 0.0,
        },
    ]
}

fn trusted_import(bytes: &[u8], config: &NuxHostCommandImportConfig) -> *mut NuxFile {
    let mut file = std::ptr::null_mut();
    let mut result = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_file_import_trusted_with_host_commands(
                bytes.as_ptr(),
                bytes.len(),
                config,
                &mut file,
                &mut result,
            )
        },
        NuxStatus::Ok
    );
    assert_eq!(unsafe { nux_capi_result_free(result) }, NuxStatus::Ok);
    file
}

fn listener_player(
    file: *mut NuxFile,
    callbacks: &NuxRenderCallbacks,
) -> (*mut NuxArtboardInstance, *mut NuxPlayer) {
    let mut artboard = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &mut artboard) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_draw(artboard, callbacks) },
        NuxStatus::Ok
    );
    let mut player = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_state_machine_named(artboard, view("HostCommands"), &mut player) },
        NuxStatus::Ok
    );
    let initialization = step(player, &[]);
    assert_eq!(
        unsafe { nux_player_step_result_free(initialization) },
        NuxStatus::Ok
    );
    (artboard, player)
}

#[test]
fn trusted_generic_commands_are_owned_by_the_successful_step_result() {
    let bytes = scripted_fixture(successful_source());
    let mut config = NuxHostCommandImportConfig {
        module_name: view("bridge"),
        ..NuxHostCommandImportConfig::default()
    };
    config.struct_size = std::mem::size_of::<NuxHostCommandImportConfig>() as u32;
    let file = trusted_import(&bytes, &config);
    let callbacks = NuxRenderCallbacks::default();
    let (artboard, player) = listener_player(file, &callbacks);
    let pointers = pointer_click();
    let result = step(player, &pointers);
    let mut info = NuxPlayerStepInfo::default();
    assert_eq!(
        unsafe { nux_player_step_result_info(result, &mut info) },
        NuxStatus::Ok
    );
    assert_eq!(info.host_command_count, 2);

    let mut undersized = NuxHostCommandView::default();
    undersized.struct_size = (NUX_HOST_COMMAND_VIEW_V3_MIN_SIZE - 1) as u32;
    assert_eq!(
        unsafe { nux_player_step_result_host_command(result, 0, &mut undersized) },
        NuxStatus::InvalidStructSize
    );

    let mut opened = NuxHostCommandView::default();
    assert_eq!(
        unsafe { nux_player_step_result_host_command(result, 0, &mut opened) },
        NuxStatus::Ok
    );
    assert_eq!(copy(opened.name), "opened");
    let mut opened_value = NuxHostValueView::default();
    assert_eq!(
        unsafe {
            nux_player_step_result_host_value(result, opened.root_value_index, &mut opened_value)
        },
        NuxStatus::Ok
    );
    assert_eq!(opened_value.kind, NUX_HOST_VALUE_KIND_NULL);

    let mut selected = NuxHostCommandView::default();
    assert_eq!(
        unsafe { nux_player_step_result_host_command(result, 1, &mut selected) },
        NuxStatus::Ok
    );
    assert_eq!(copy(selected.name), "selected");
    let mut root = NuxHostValueView::default();
    assert_eq!(
        unsafe { nux_player_step_result_host_value(result, selected.root_value_index, &mut root) },
        NuxStatus::Ok
    );
    assert_eq!(root.kind, NUX_HOST_VALUE_KIND_OBJECT);
    assert_eq!(root.child_count, 3);

    let mut flags = NuxHostValueChildView::default();
    let mut quantity = NuxHostValueChildView::default();
    let mut sku = NuxHostValueChildView::default();
    assert_eq!(
        unsafe {
            nux_player_step_result_host_value_child(
                result,
                selected.root_value_index,
                0,
                &mut flags,
            )
        },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe {
            nux_player_step_result_host_value_child(
                result,
                selected.root_value_index,
                1,
                &mut quantity,
            )
        },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe {
            nux_player_step_result_host_value_child(result, selected.root_value_index, 2, &mut sku)
        },
        NuxStatus::Ok
    );
    assert_eq!(copy(flags.key), "flags");
    assert_eq!(copy(quantity.key), "quantity");
    assert_eq!(copy(sku.key), "sku");
    let mut flags_value = NuxHostValueView::default();
    let mut quantity_value = NuxHostValueView::default();
    let mut sku_value = NuxHostValueView::default();
    assert_eq!(
        unsafe { nux_player_step_result_host_value(result, flags.value_index, &mut flags_value) },
        NuxStatus::Ok
    );
    assert_eq!(flags_value.kind, NUX_HOST_VALUE_KIND_LIST);
    assert_eq!(flags_value.child_count, 2);
    assert_eq!(
        unsafe {
            nux_player_step_result_host_value(result, quantity.value_index, &mut quantity_value)
        },
        NuxStatus::Ok
    );
    assert_eq!(quantity_value.kind, NUX_HOST_VALUE_KIND_NUMBER);
    assert_eq!(quantity_value.number_value, 2.5);
    for (index, expected) in [true, false].into_iter().enumerate() {
        let mut child = NuxHostValueChildView::default();
        assert_eq!(
            unsafe {
                nux_player_step_result_host_value_child(
                    result,
                    flags.value_index,
                    index,
                    &mut child,
                )
            },
            NuxStatus::Ok
        );
        assert!(child.key.data.is_null());
        assert_eq!(child.key.len, 0);
        let mut value = NuxHostValueView::default();
        assert_eq!(
            unsafe { nux_player_step_result_host_value(result, child.value_index, &mut value) },
            NuxStatus::Ok
        );
        assert_eq!(value.kind, NUX_HOST_VALUE_KIND_BOOL);
        assert_eq!(value.bool_value, expected);
    }
    assert_eq!(
        unsafe { nux_player_step_result_host_value(result, sku.value_index, &mut sku_value) },
        NuxStatus::Ok
    );
    let copied_sku = copy(sku_value.string_value);
    assert_eq!(sku_value.kind, NUX_HOST_VALUE_KIND_STRING);
    assert_eq!(copied_sku, "sku-1");

    assert_eq!(
        unsafe { nux_player_step_result_free(result) },
        NuxStatus::Ok
    );
    assert_eq!(copied_sku, "sku-1", "caller copy survives result release");
    unsafe {
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}

#[test]
fn ordinary_import_keeps_the_same_authored_module_inert() {
    let bytes = scripted_fixture(successful_source());
    let mut file = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_file_import(bytes.as_ptr(), bytes.len(), &mut file) },
        NuxStatus::Ok
    );
    let callbacks = NuxRenderCallbacks::default();
    let (artboard, player) = listener_player(file, &callbacks);
    let result = step(player, &pointer_click());
    let mut info = NuxPlayerStepInfo::default();
    assert_eq!(
        unsafe { nux_player_step_result_info(result, &mut info) },
        NuxStatus::Ok
    );
    assert_eq!(info.host_command_count, 0);
    unsafe {
        nux_player_step_result_free(result);
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}

#[test]
fn authored_failure_rolls_back_commands_and_poisons_the_occurrence() {
    let bytes = scripted_fixture(
        br#"
            local bridge = require("bridge")
            return function(_context)
                return {
                    init = function(_self) return true end,
                    performAction = function(_self, _invocation)
                        bridge.command("partial", { escaped = false })
                        error("fail after command")
                    end,
                }
            end
        "#,
    );
    let config = NuxHostCommandImportConfig {
        module_name: view("bridge"),
        ..NuxHostCommandImportConfig::default()
    };
    let file = trusted_import(&bytes, &config);
    let callbacks = NuxRenderCallbacks::default();
    let (artboard, player) = listener_player(file, &callbacks);

    let (status, failed) = raw_step(player, &pointer_click());
    assert_eq!(status, NuxStatus::RuntimeError);
    let mut command = NuxHostCommandView::default();
    assert_eq!(
        unsafe { nux_player_step_result_host_command(failed, 0, &mut command) },
        NuxStatus::RuntimeError,
        "failed steps never expose pre-failure commands"
    );
    let mut diagnostic = NuxCapiDiagnosticView::default();
    assert_eq!(
        unsafe { nux_player_step_result_diagnostic(failed, &mut diagnostic) },
        NuxStatus::Ok
    );
    assert!(copy(diagnostic.message).contains("scripted pointer dispatch failed"));
    assert_eq!(
        unsafe { nux_player_step_result_free(failed) },
        NuxStatus::Ok
    );

    let (status, poisoned) = raw_step(player, &[]);
    assert_eq!(status, NuxStatus::RuntimeError);
    assert_eq!(
        unsafe { nux_player_step_result_free(poisoned) },
        NuxStatus::Ok
    );
    unsafe {
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}

#[test]
fn scripted_drawable_failure_rolls_back_commands_and_poisons_the_occurrence() {
    let bytes = scripted_drawable_fixture(
        br#"
            local bridge = require("bridge")
            return function(_context)
                return {
                    init = function(_self) return true end,
                    advance = function(_self, _seconds)
                        bridge.command("partial_drawable", { escaped = false })
                        error("fail after drawable command")
                    end,
                    draw = function(_self, _renderer) end,
                }
            end
        "#,
    );
    let config = NuxHostCommandImportConfig {
        module_name: view("bridge"),
        ..NuxHostCommandImportConfig::default()
    };
    let file = trusted_import(&bytes, &config);
    let callbacks = NuxRenderCallbacks::default();
    let mut artboard = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &mut artboard) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_draw(artboard, &callbacks) },
        NuxStatus::Ok
    );
    let mut player = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_player_new_state_machine_named(artboard, view("DrawableAdvance"), &mut player)
        },
        NuxStatus::Ok
    );

    let (status, failed) = raw_step(player, &[]);
    assert_eq!(status, NuxStatus::RuntimeError);
    let mut command = NuxHostCommandView::default();
    assert_eq!(
        unsafe { nux_player_step_result_host_command(failed, 0, &mut command) },
        NuxStatus::RuntimeError,
        "failed drawable steps never expose pre-failure commands"
    );
    assert_eq!(
        unsafe { nux_player_step_result_free(failed) },
        NuxStatus::Ok
    );

    let (status, poisoned) = raw_step(player, &[]);
    assert_eq!(status, NuxStatus::RuntimeError);
    assert_eq!(
        unsafe { nux_player_step_result_free(poisoned) },
        NuxStatus::Ok
    );
    unsafe {
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}

#[test]
fn swallowed_transition_failure_rejects_commit_and_poisons_the_occurrence() {
    let bytes = scripted_transition_fixture(
        br#"
            local bridge = require("bridge")
            return function(_context)
                return {
                    init = function(_self) return true end,
                    evaluate = function(_self)
                        bridge.command("partial_transition", { escaped = false })
                        error("fail after transition command")
                    end,
                }
            end
        "#,
    );
    let config = NuxHostCommandImportConfig {
        module_name: view("bridge"),
        ..NuxHostCommandImportConfig::default()
    };
    let file = trusted_import(&bytes, &config);
    let mut artboard = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &mut artboard) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_draw(artboard, &NuxRenderCallbacks::default()) },
        NuxStatus::Ok
    );
    let mut player = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_player_new_state_machine_named(artboard, view("TransitionEvaluate"), &mut player)
        },
        NuxStatus::Ok
    );

    let (status, failed) = raw_step(player, &[]);
    assert_eq!(status, NuxStatus::RuntimeError);
    let mut command = NuxHostCommandView::default();
    assert_eq!(
        unsafe { nux_player_step_result_host_command(failed, 0, &mut command) },
        NuxStatus::RuntimeError,
        "a swallowed transition error must veto the transaction before drain"
    );
    let mut diagnostic = NuxCapiDiagnosticView::default();
    assert_eq!(
        unsafe { nux_player_step_result_diagnostic(failed, &mut diagnostic) },
        NuxStatus::Ok
    );
    assert!(copy(diagnostic.message).contains("fail after transition command"));
    assert_eq!(
        unsafe { nux_player_step_result_free(failed) },
        NuxStatus::Ok
    );

    let (status, poisoned) = raw_step(player, &[]);
    assert_eq!(status, NuxStatus::RuntimeError);
    assert_eq!(
        unsafe { nux_player_step_result_free(poisoned) },
        NuxStatus::Ok
    );
    unsafe {
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}

#[test]
fn trusted_import_config_rejects_unbounded_or_incomplete_policies() {
    let bytes = scripted_fixture(successful_source());
    let cases = [
        (
            NuxHostCommandImportConfig {
                struct_size: (NUX_HOST_COMMAND_IMPORT_CONFIG_V3_MIN_SIZE - 1) as u32,
                module_name: view("bridge"),
                ..NuxHostCommandImportConfig::default()
            },
            NuxStatus::InvalidStructSize,
        ),
        (
            NuxHostCommandImportConfig {
                module_name: NuxStringView::default(),
                ..NuxHostCommandImportConfig::default()
            },
            NuxStatus::InvalidArgument,
        ),
        (
            NuxHostCommandImportConfig {
                module_name: view("bridge"),
                max_commands_per_step: 0,
                ..NuxHostCommandImportConfig::default()
            },
            NuxStatus::InvalidArgument,
        ),
        (
            NuxHostCommandImportConfig {
                module_name: view("bridge"),
                max_script_memory_bytes: NUX_SCRIPT_VM_MEMORY_BYTES_HARD_MAX + 1,
                ..NuxHostCommandImportConfig::default()
            },
            NuxStatus::LimitExceeded,
        ),
    ];

    for (config, expected) in cases {
        let mut file = std::ptr::null_mut();
        let mut result = std::ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_file_import_trusted_with_host_commands(
                    bytes.as_ptr(),
                    bytes.len(),
                    &config,
                    &mut file,
                    &mut result,
                )
            },
            expected
        );
        assert!(file.is_null());
        assert!(!result.is_null());
        assert_eq!(unsafe { nux_capi_result_free(result) }, NuxStatus::Ok);
    }
}
