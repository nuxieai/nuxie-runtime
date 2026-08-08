#![cfg(feature = "scripting")]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    reason = "C-ABI fixture tests use bounded counters and explicit teardown"
)]

use luaur_compiler::functions::luau_compile::luau_compile;
use nux_capi::*;
use nuxie_schema::definition_by_name;

#[path = "support/composed_import.rs"]
mod composed_import;
use composed_import::scripted_view_model_asset_fixture;

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

fn push_color(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u32) {
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
    push_object(&mut bytes, "Fill", |bytes| {
        push_uint(bytes, "Component", "parentId", 1);
    });
    push_object(&mut bytes, "SolidColor", |bytes| {
        push_uint(bytes, "Component", "parentId", 2);
        push_color(bytes, "SolidColor", "colorValue", 0xff33_66aa);
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

fn correlated_step(
    player: *mut NuxPlayer,
    pointers: &[NuxPlayerPointerEvent],
    correlation_id: u64,
) -> *mut NuxPlayerStepResult {
    correlated_step_with_delta(player, pointers, correlation_id, 0.016)
}

fn correlated_step_with_delta(
    player: *mut NuxPlayer,
    pointers: &[NuxPlayerPointerEvent],
    correlation_id: u64,
    elapsed_seconds: f32,
) -> *mut NuxPlayerStepResult {
    let operation = NuxPlayerStep {
        pointers: pointers.as_ptr(),
        pointer_count: pointers.len(),
        elapsed_seconds,
        correlation_id,
        ..NuxPlayerStep::default()
    };
    let mut result = std::ptr::null_mut();
    let status = unsafe { nux_player_step(player, &operation, &mut result) };
    if status != NuxStatus::Ok {
        let mut diagnostic = NuxCapiDiagnosticView::default();
        assert_eq!(
            unsafe { nux_player_step_result_diagnostic(result, &mut diagnostic) },
            NuxStatus::Ok
        );
        panic!(
            "correlated step failed: {status:?}: {}",
            copy(diagnostic.message)
        );
    }
    result
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

fn view_model_number(instance: *const NuxViewModelInstance, name: &str) -> f32 {
    let mut snapshot = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_snapshot(instance, &mut snapshot) },
        NuxStatus::Ok
    );
    let mut info = NuxViewModelSnapshotInfo::default();
    assert_eq!(
        unsafe { nux_view_model_snapshot_info(snapshot, &mut info) },
        NuxStatus::Ok
    );
    let mut found = None;
    for index in 0..info.value_count {
        let mut value = NuxViewModelSnapshotValueView::default();
        assert_eq!(
            unsafe { nux_view_model_snapshot_value(snapshot, index, &mut value) },
            NuxStatus::Ok
        );
        if copy(value.name) == name {
            assert_eq!(value.kind, NUX_VIEW_MODEL_VALUE_KIND_NUMBER);
            found = Some(value.number_value);
            break;
        }
    }
    assert_eq!(
        unsafe { nux_view_model_snapshot_free(snapshot) },
        NuxStatus::Ok
    );
    found.unwrap_or_else(|| panic!("missing view-model number {name}"))
}

fn view_model_link_identity(instance: *const NuxViewModelInstance, name: &str) -> u64 {
    let mut snapshot = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_snapshot(instance, &mut snapshot) },
        NuxStatus::Ok
    );
    let mut info = NuxViewModelSnapshotInfo::default();
    assert_eq!(
        unsafe { nux_view_model_snapshot_info(snapshot, &mut info) },
        NuxStatus::Ok
    );
    let mut found = None;
    for index in 0..info.value_count {
        let mut value = NuxViewModelSnapshotValueView::default();
        assert_eq!(
            unsafe { nux_view_model_snapshot_value(snapshot, index, &mut value) },
            NuxStatus::Ok
        );
        if copy(value.name) == name {
            assert_eq!(value.kind, NUX_VIEW_MODEL_VALUE_KIND_VIEW_MODEL);
            found = Some(value.referenced_instance_id);
            break;
        }
    }
    assert_eq!(
        unsafe { nux_view_model_snapshot_free(snapshot) },
        NuxStatus::Ok
    );
    found.unwrap_or_else(|| panic!("missing view-model link {name}"))
}

fn mutate_view_model_number(instance: *mut NuxViewModelInstance, path: &str, value: f32) {
    let mutation = NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER,
        instance,
        path: view(path),
        number_value: value,
        ..NuxViewModelMutation::default()
    };
    let batch = NuxViewModelMutationBatch {
        mutations: &mutation,
        mutation_count: 1,
        ..NuxViewModelMutationBatch::default()
    };
    let mut result = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_mutate(&batch, &mut result) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_mutation_result_free(result) },
        NuxStatus::Ok
    );
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
fn player_change_journals_are_ordered_owned_and_never_cross_drain_shared_view_models() {
    let bytes = scripted_view_model_asset_fixture(
        br#"
            local bridge = require("bridge")
            return function(context)
                return {
                    init = function(_self) return true end,
                    performAction = function(_self, _invocation)
                        local root = context:viewModel()
                        if root ~= nil and root.amount.value == 0 then
                            root.amount.value = 10
                            root.amount.value = 20
                        end
                        bridge.command("performed", nil)
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
    let mut asset_count = 0;
    assert_eq!(
        unsafe { nux_file_asset_count(file, &mut asset_count) },
        NuxStatus::Ok
    );
    assert_eq!(
        asset_count, 2,
        "the configured script and image remain first-class file assets"
    );

    let mut first_artboard = std::ptr::null_mut();
    let mut second_artboard = std::ptr::null_mut();
    let mut unbound_artboard = std::ptr::null_mut();
    for out in [
        &mut first_artboard,
        &mut second_artboard,
        &mut unbound_artboard,
    ] {
        assert_eq!(
            unsafe { nux_artboard_instance_new(file, 0, out) },
            NuxStatus::Ok
        );
    }
    let mut view_model = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 0, 0, &mut view_model) },
        NuxStatus::Ok
    );
    let mut instance_identity = 0;
    assert_eq!(
        unsafe { nux_view_model_instance_identity(view_model, &mut instance_identity) },
        NuxStatus::Ok
    );
    assert_ne!(instance_identity, 0);
    for artboard in [first_artboard, second_artboard] {
        assert_eq!(
            unsafe { nux_artboard_instance_bind_view_model(artboard, view_model) },
            NuxStatus::Ok
        );
    }
    let callbacks = NuxRenderCallbacks::default();
    for artboard in [first_artboard, second_artboard, unbound_artboard] {
        assert_eq!(
            unsafe { nux_artboard_instance_draw(artboard, &callbacks) },
            NuxStatus::Ok
        );
    }
    let mut first_player = std::ptr::null_mut();
    let mut second_player = std::ptr::null_mut();
    let mut unbound_player = std::ptr::null_mut();
    for (artboard, out) in [
        (first_artboard, &mut first_player),
        (second_artboard, &mut second_player),
        (unbound_artboard, &mut unbound_player),
    ] {
        assert_eq!(
            unsafe { nux_player_new_state_machine_named(artboard, view("HostCommands"), out) },
            NuxStatus::Ok
        );
        let initialized = correlated_step_with_delta(*out, &[], 0, 0.0);
        assert_eq!(
            unsafe { nux_player_step_result_free(initialized) },
            NuxStatus::Ok
        );
    }

    let correlation_id = 0xc0ff_eeu64;
    let first_result = correlated_step(first_player, &pointer_click(), correlation_id);
    let mut first_info = NuxPlayerStepInfo::default();
    assert_eq!(
        unsafe { nux_player_step_result_info(first_result, &mut first_info) },
        NuxStatus::Ok
    );
    assert_eq!(first_info.host_command_count, 1);
    assert_eq!(first_info.view_model_change_count, 2);
    for (index, expected) in [10.0, 20.0].into_iter().enumerate() {
        let mut change = NuxViewModelChangeView::default();
        assert_eq!(
            unsafe { nux_player_step_result_view_model_change(first_result, index, &mut change) },
            NuxStatus::Ok
        );
        assert_eq!(change.origin, NUX_VIEW_MODEL_CHANGE_ORIGIN_RUNTIME);
        assert_eq!(change.correlation_id, correlation_id);
        assert_eq!(change.owner_instance_id, instance_identity);
        assert_eq!(change.property_index, 0);
        assert_eq!(change.kind, NUX_VIEW_MODEL_VALUE_KIND_NUMBER);
        assert_eq!(change.number_value, expected);
    }

    let second_result = correlated_step_with_delta(second_player, &[], 22, 0.0);
    let mut second_info = NuxPlayerStepInfo::default();
    assert_eq!(
        unsafe { nux_player_step_result_info(second_result, &mut second_info) },
        NuxStatus::Ok
    );
    assert_eq!(
        second_info.view_model_change_count, 0,
        "a second player cannot drain another operation"
    );

    let unbound_result = correlated_step(unbound_player, &pointer_click(), 33);
    let mut unbound_info = NuxPlayerStepInfo::default();
    assert_eq!(
        unsafe { nux_player_step_result_info(unbound_result, &mut unbound_info) },
        NuxStatus::Ok
    );
    assert_eq!(unbound_info.host_command_count, 1);
    assert_eq!(
        unbound_info.view_model_change_count, 0,
        "an unbound occurrence never leaves a latent journal"
    );

    unsafe {
        nux_player_step_result_free(second_result);
        nux_player_step_result_free(unbound_result);
        nux_player_free(first_player);
        nux_player_free(second_player);
        nux_player_free(unbound_player);
        nux_artboard_instance_free(first_artboard);
        nux_artboard_instance_free(second_artboard);
        nux_artboard_instance_free(unbound_artboard);
        nux_view_model_instance_free(view_model);
        nux_file_free(file);
    }
    let mut retained = NuxViewModelChangeView::default();
    assert_eq!(
        unsafe { nux_player_step_result_view_model_change(first_result, 1, &mut retained) },
        NuxStatus::Ok
    );
    assert_eq!(retained.owner_instance_id, instance_identity);
    assert_eq!(retained.number_value, 20.0);
    assert_eq!(
        unsafe { nux_player_step_result_free(first_result) },
        NuxStatus::Ok
    );
}

#[test]
fn scripted_sibling_occurrences_share_only_one_foreign_resource_domain() {
    let bytes = scripted_view_model_asset_fixture(
        br#"
            return function(_context)
                return {
                    init = function(_self) return true end,
                    performAction = function(_self, _invocation) end,
                }
            end
        "#,
    );
    let host = NuxHostCommandImportConfig {
        module_name: view("bridge"),
        ..NuxHostCommandImportConfig::default()
    };
    let file = trusted_import(&bytes, &host);
    let mut view_model = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 0, 0, &mut view_model) },
        NuxStatus::Ok
    );
    let mut occurrences = [std::ptr::null_mut(); 3];
    for occurrence in &mut occurrences {
        assert_eq!(
            unsafe { nux_artboard_instance_new(file, 0, occurrence) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_artboard_instance_bind_view_model(*occurrence, view_model) },
            NuxStatus::Ok
        );
    }

    let callbacks = NuxRenderCallbacks::default();
    assert_eq!(
        unsafe { nux_artboard_instance_draw(occurrences[0], &callbacks) },
        NuxStatus::Ok
    );
    let same_domain_callbacks = callbacks;
    assert_eq!(
        unsafe { nux_artboard_instance_draw(occurrences[1], &same_domain_callbacks) },
        NuxStatus::Ok,
        "a distinct descriptor for the same resource domain reuses the stable File VM factory"
    );

    let mut distinct_context = ();
    let distinct_callbacks = NuxRenderCallbacks {
        user_data: std::ptr::from_mut(&mut distinct_context).cast(),
        ..callbacks
    };
    assert_eq!(
        unsafe { nux_artboard_instance_draw(occurrences[2], &distinct_callbacks) },
        NuxStatus::RuntimeError,
        "a different scripted factory domain fails precisely during scripted hydration, not as a blanket occurrence binding mismatch"
    );

    for occurrence in occurrences {
        assert_eq!(
            unsafe { nux_artboard_instance_free(occurrence) },
            NuxStatus::Ok
        );
    }
    assert_eq!(
        unsafe { nux_view_model_instance_free(view_model) },
        NuxStatus::Ok
    );
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
}

#[test]
fn scripted_failure_rolls_back_bound_view_model_and_poisons_the_occurrence() {
    let bytes = scripted_view_model_asset_fixture(
        br#"
            local bridge = require("bridge")
            return function(context)
                return {
                    init = function(_self) return true end,
                    performAction = function(_self, _invocation)
                        local root = context:viewModel()
                        root.amount.value = 10
                        root.amount.value = 20
                        root.child.value = Data.Child.new("temporary-child")
                        root.child.value.score.value = 99
                        bridge.command("not-committed", nil)
                        error("rollback requested")
                    end,
                }
            end
        "#,
    );
    let host = NuxHostCommandImportConfig {
        module_name: view("bridge"),
        ..NuxHostCommandImportConfig::default()
    };
    let file = trusted_import(&bytes, &host);
    let mut artboard = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &mut artboard) },
        NuxStatus::Ok
    );
    let mut view_model = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 0, 0, &mut view_model) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_bind_view_model(artboard, view_model) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_draw(artboard, &NuxRenderCallbacks::default()) },
        NuxStatus::Ok
    );
    let mut player = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_state_machine_named(artboard, view("HostCommands"), &mut player) },
        NuxStatus::Ok
    );
    let initialized = correlated_step_with_delta(player, &[], 0, 0.0);
    assert_eq!(
        unsafe { nux_player_step_result_free(initialized) },
        NuxStatus::Ok
    );
    let original_child = view_model_link_identity(view_model, "child");
    assert_ne!(original_child, 0);
    assert_eq!(view_model_number(view_model, "score"), 4.0);

    let (status, failed) = raw_step(player, &pointer_click());
    assert_eq!(status, NuxStatus::RuntimeError);
    let mut failed_status = NuxStatus::Ok;
    assert_eq!(
        unsafe { nux_player_step_result_status(failed, &mut failed_status) },
        NuxStatus::Ok
    );
    assert_eq!(failed_status, NuxStatus::RuntimeError);
    let mut failed_change = NuxViewModelChangeView::default();
    assert_eq!(
        unsafe { nux_player_step_result_view_model_change(failed, 0, &mut failed_change) },
        NuxStatus::NotFound
    );
    assert_eq!(
        unsafe { nux_player_step_result_free(failed) },
        NuxStatus::Ok
    );
    assert_eq!(view_model_number(view_model, "amount"), 0.0);
    assert_eq!(
        view_model_link_identity(view_model, "child"),
        original_child
    );
    assert_eq!(view_model_number(view_model, "score"), 4.0);

    mutate_view_model_number(view_model, "amount", 7.0);
    assert_eq!(view_model_number(view_model, "amount"), 7.0);

    let (status, poisoned) = raw_step(player, &[]);
    assert_eq!(status, NuxStatus::RuntimeError);
    assert_eq!(
        unsafe { nux_player_step_result_free(poisoned) },
        NuxStatus::Ok
    );
    assert_eq!(view_model_number(view_model, "amount"), 7.0);

    assert_eq!(unsafe { nux_player_free(player) }, NuxStatus::Ok);
    assert_eq!(
        unsafe { nux_artboard_instance_free(artboard) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_free(view_model) },
        NuxStatus::Ok
    );
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
}

#[test]
fn journal_limit_failure_rolls_back_shared_view_model_and_leaves_handle_usable() {
    let bytes = scripted_view_model_asset_fixture(
        br#"
            return function(context)
                return {
                    init = function(_self) return true end,
                    performAction = function(_self, _invocation)
                        local root = context:viewModel()
                        for index = 1, 4097 do
                            root.amount.value = index
                        end
                    end,
                }
            end
        "#,
    );
    let host = NuxHostCommandImportConfig {
        module_name: view("bridge"),
        ..NuxHostCommandImportConfig::default()
    };
    let file = trusted_import(&bytes, &host);
    let mut artboard = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &mut artboard) },
        NuxStatus::Ok
    );
    let mut view_model = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 0, 0, &mut view_model) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_bind_view_model(artboard, view_model) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_draw(artboard, &NuxRenderCallbacks::default()) },
        NuxStatus::Ok
    );
    let mut player = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_state_machine_named(artboard, view("HostCommands"), &mut player) },
        NuxStatus::Ok
    );
    let initialized = correlated_step_with_delta(player, &[], 0, 0.0);
    assert_eq!(
        unsafe { nux_player_step_result_free(initialized) },
        NuxStatus::Ok
    );

    let (status, failed) = raw_step(player, &pointer_click());
    assert_eq!(status, NuxStatus::LimitExceeded);
    let mut change = NuxViewModelChangeView::default();
    assert_eq!(
        unsafe { nux_player_step_result_view_model_change(failed, 0, &mut change) },
        NuxStatus::NotFound
    );
    assert_eq!(
        unsafe { nux_player_step_result_free(failed) },
        NuxStatus::Ok
    );
    assert_eq!(view_model_number(view_model, "amount"), 0.0);

    mutate_view_model_number(view_model, "amount", 8.0);
    assert_eq!(view_model_number(view_model, "amount"), 8.0);

    let (status, poisoned) = raw_step(player, &[]);
    assert_eq!(status, NuxStatus::RuntimeError);
    assert_eq!(
        unsafe { nux_player_step_result_free(poisoned) },
        NuxStatus::Ok
    );
    assert_eq!(view_model_number(view_model, "amount"), 8.0);

    assert_eq!(unsafe { nux_player_free(player) }, NuxStatus::Ok);
    assert_eq!(
        unsafe { nux_artboard_instance_free(artboard) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_free(view_model) },
        NuxStatus::Ok
    );
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
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
