//! Downstream signature checks for the approved upstream-shaped public API.
//! The superseded facade census contained only compile-time contracts, not
//! behavioral cases. No compatibility surface is required by these checks.

use nuxie_runtime::{
    CoreHandle, File, FileAssetLoaderRef, ImportResult, RuntimeArtboardInstanceHandle,
    RuntimeFactoryHandle, RuntimeFileHandle, RuntimeScriptingVmHandle,
    RuntimeStateMachineInstanceHandle,
    source::{
        animation::{
            state_machine_input_instance::{SMIBool, SMIInput, SMINumber, SMITrigger},
            state_machine_instance::StateMachineInstance,
        },
        hit_result::HitResult,
        math::vec2d::Vec2D,
        renderer::Renderer,
        viewmodel::runtime::{
            viewmodel_instance_boolean_runtime::ViewModelInstanceBooleanRuntime,
            viewmodel_instance_list_runtime::ViewModelInstanceListRuntime,
            viewmodel_instance_number_runtime::ViewModelInstanceNumberRuntime,
            viewmodel_instance_runtime::{
                PropertyData, RuntimeViewModelInstanceHandle, ViewModelInstanceRuntime,
            },
            viewmodel_instance_string_runtime::ViewModelInstanceStringRuntime,
            viewmodel_instance_value_runtime::ViewModelInstanceValueRuntime,
            viewmodel_runtime::RuntimeViewModelHandle,
        },
    },
};

#[test]
fn native_file_requires_factory_and_artboard_draw_takes_only_renderer() {
    let _: fn(
        &[u8],
        RuntimeFactoryHandle,
        Option<&mut ImportResult>,
        Option<FileAssetLoaderRef>,
        Option<RuntimeScriptingVmHandle>,
    ) -> Option<RuntimeFileHandle> = File::import;
    let _: fn(&File) -> usize = File::artboard_count;
    let _: fn(&File) -> Option<RuntimeArtboardInstanceHandle> = File::artboard_default;
    let _: fn(&File, usize) -> Option<RuntimeArtboardInstanceHandle> = File::artboard_at;
    let _: fn(&File, &str) -> Option<RuntimeArtboardInstanceHandle> = File::artboard_named;
    let _: fn(&RuntimeArtboardInstanceHandle, &mut Renderer<'_>) =
        RuntimeArtboardInstanceHandle::draw;
    let _: fn(&RuntimeArtboardInstanceHandle) -> Option<RuntimeStateMachineInstanceHandle> =
        RuntimeArtboardInstanceHandle::default_state_machine_handle;
}

#[test]
fn native_state_machine_inputs_and_pointer_signatures_are_public() {
    let _: fn(&StateMachineInstance) -> usize = StateMachineInstance::input_count;
    let _: fn(&StateMachineInstance, usize) -> Option<&SMIInput> = StateMachineInstance::input;
    let _: fn(&StateMachineInstance, u32) -> Option<&SMIBool> = StateMachineInstance::bool_input;
    let _: fn(&StateMachineInstance, u32) -> Option<&SMINumber> =
        StateMachineInstance::number_input;
    let _: fn(&StateMachineInstance, u32) -> Option<&SMITrigger> =
        StateMachineInstance::trigger_input;
    let _: fn(&RuntimeStateMachineInstanceHandle, &str, bool) =
        RuntimeStateMachineInstanceHandle::set_bool;
    let _: fn(&RuntimeStateMachineInstanceHandle, &str, f32) =
        RuntimeStateMachineInstanceHandle::set_number;
    let _: fn(&mut StateMachineInstance, Vec2D, i32) -> HitResult =
        StateMachineInstance::pointer_down;
    let _: fn(&mut StateMachineInstance, Vec2D, i32) -> HitResult =
        StateMachineInstance::pointer_up;
    let _: fn(&mut StateMachineInstance, Vec2D, f32, i32) -> HitResult =
        StateMachineInstance::pointer_move;
    let _: fn(&mut StateMachineInstance, Vec2D, i32) -> HitResult =
        StateMachineInstance::pointer_exit;
    let _: fn(&RuntimeStateMachineInstanceHandle, f32) -> bool =
        RuntimeStateMachineInstanceHandle::advance_and_apply;
    let _: fn(&mut StateMachineInstance, Option<CoreHandle>) =
        StateMachineInstance::bind_view_model_instance;
}

#[test]
fn native_view_model_instances_and_typed_properties_are_public() {
    let _: fn(&File, usize) -> Option<RuntimeViewModelHandle> = File::view_model_by_index;
    let _: fn(&File, &str) -> Option<RuntimeViewModelHandle> = File::view_model_by_name;
    let _: fn(&RuntimeViewModelHandle) -> RuntimeViewModelInstanceHandle =
        RuntimeViewModelHandle::create_default_instance;
    let _: fn(&ViewModelInstanceRuntime) -> CoreHandle = ViewModelInstanceRuntime::instance;
    let _: fn(&ViewModelInstanceRuntime, &str) -> Option<ViewModelInstanceNumberRuntime> =
        ViewModelInstanceRuntime::property_number;
    let _: fn(&ViewModelInstanceRuntime, &str) -> Option<ViewModelInstanceStringRuntime> =
        ViewModelInstanceRuntime::property_string;
    let _: fn(&ViewModelInstanceRuntime, &str) -> Option<ViewModelInstanceBooleanRuntime> =
        ViewModelInstanceRuntime::property_boolean;
    let _: fn(&ViewModelInstanceRuntime, &str) -> Option<ViewModelInstanceListRuntime> =
        ViewModelInstanceRuntime::property_list;
    let _: fn(&ViewModelInstanceRuntime, &str) -> Option<RuntimeViewModelInstanceHandle> =
        ViewModelInstanceRuntime::property_view_model;
    let _: fn(&ViewModelInstanceRuntime, &str) -> Option<ViewModelInstanceValueRuntime> =
        ViewModelInstanceRuntime::property;
    let _: fn(&ViewModelInstanceRuntime) -> Vec<PropertyData> =
        ViewModelInstanceRuntime::properties;
    let _: fn(&ViewModelInstanceNumberRuntime) -> f32 = ViewModelInstanceNumberRuntime::value;
    let _: fn(&ViewModelInstanceNumberRuntime, f32) = ViewModelInstanceNumberRuntime::set_value;
}
