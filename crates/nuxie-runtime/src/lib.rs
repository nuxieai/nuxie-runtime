mod animation;
mod artboard;
mod artboard_component_list_order;
mod artboard_data_bind;
mod components;
mod constraints;
mod data_bind_container;
mod data_bind_graph;
mod data_converter;
mod data_converter_trigger;
mod draw;
mod focus;
mod math;
mod nested_bool;
mod nested_number;
mod nested_trigger;
mod objects;
mod parent_traversal;
mod project_data_converter;
mod properties;
mod rectangles_to_contour;
mod script_asset;
mod script_input_artboard;
mod script_input_boolean;
mod script_input_color;
mod script_input_number;
mod script_input_string;
mod script_input_trigger;
mod script_input_viewmodel_property;
mod scripted_data_converter;
mod scripted_object;
mod scripting;
mod state_machine;
mod text;
mod text_input;
mod view_model;
// #RB-1: retained-identity view-model core (map Phase RB). Additive while
// consumers migrate; the compensation family deletes when migration ends.
pub mod retained_data_bind;
pub mod view_model_cell;

pub use animation::{
    LinearAnimationInstance, RuntimeKeyFrame, RuntimeKeyFrameBool, RuntimeKeyFrameCallback,
    RuntimeKeyFrameColor, RuntimeKeyFrameDouble, RuntimeKeyFrameString, RuntimeKeyFrameUint,
    RuntimeKeyedObject, RuntimeKeyedProperty, RuntimeKeyedPropertyTarget, RuntimeLinearAnimation,
    RuntimeLinearAnimationHandle,
};
#[cfg(feature = "tools")]
pub use artboard::RuntimeNestedRemapAnimationReport;
pub use artboard::{
    ArtboardInstance, ExternalFontAssetError, RuntimeArtboardOccurrenceSegment, RuntimeComponents,
    RuntimeEventProperty, RuntimeEventPropertyValue,
};
pub use components::{
    ComponentDirt, Mat2D, RuntimeComponent, RuntimeComponentCapabilities, TransformProperty,
    TransformRuntimeState, UpdateComponentsReport,
};
pub(crate) use data_bind_graph::{
    RuntimeDataBindGraph, RuntimeDataBindGraphApplyPhase, RuntimeDataBindGraphConverter,
    RuntimeDataBindGraphTargetsMut, RuntimeDataBindGraphValue,
    data_bind_flags_apply_source_to_target, data_bind_flags_apply_target_to_source,
};
pub use draw::{
    RuntimeContourMeasure, RuntimeDrawableDispatch, RuntimeDrawableDispatchKind,
    RuntimeDrawableDispatchObjectKind, RuntimeFeatherState, RuntimeGeometryHit,
    RuntimeGeometryHitOccurrence, RuntimeGeometryHitPathSegment, RuntimeGradientStop,
    RuntimeImageAssetOwners, RuntimeImageDimensionConflict, RuntimeLayoutBoundsReport,
    RuntimePathCommand, RuntimePathMeasure, RuntimePathSample, RuntimeRenderPaints,
    RuntimeSemanticTextHit, RuntimeShapePaintCommand, RuntimeShapePaintKind,
    RuntimeShapePaintPathKind, RuntimeShapePaintState, preallocate_source_render_paints,
    runtime_path_commands_from_raw_path,
};
pub use focus::{
    FocusBounds, FocusDirection, FocusEdgeBehavior, FocusEvent, FocusEventKind, FocusManager,
    FocusNode, FocusNodeId, FocusPoint,
};
pub use math::random::{
    RuntimeRandomTestValuesGuard, runtime_random_call_count, set_runtime_deterministic_mode,
    set_runtime_random_test_values,
};
pub use objects::InstanceSlot;
pub use project_data_converter::{
    ProjectDataConverterCatalog, ProjectDataConverterCompileError, ProjectDataConverterContext,
    ProjectDataConverterDefinition, ProjectDataConverterEasing, ProjectDataConverterFormat,
    ProjectDataConverterKind, ProjectDataConverterMathOperation, ProjectDataConverterOutputType,
    ProjectDataConverterProgram, ProjectDataConverterProgramError, ProjectDataConverterRangeClamp,
    ProjectDataConverterResolver, ProjectDataConverterReverseResult,
    ProjectDataConverterRuntimeError, ProjectDataConverterSpec, ProjectDataConverterState,
    ProjectDataConverterStringPadSide, ProjectDataConverterStringTrimMode,
    ProjectDataConverterValidationRule, ProjectDataValue, ProjectDataValuePath,
    ProjectDataViewModelReference,
};
#[doc(hidden)]
pub use script_asset::scripted_object_inits;
pub use script_input_viewmodel_property::{
    ScriptInputViewModelPropertyPath, bound_script_view_model_from_owned_context,
    bound_script_view_model_from_owned_path, bound_script_view_model_snapshot,
    bound_script_view_model_snapshot_from_path,
};
#[doc(hidden)]
pub use scripted_data_converter::{
    RuntimeScriptedDataConverterDataBindSnapshot, RuntimeScriptedDataConverterOccurrenceSnapshot,
};
pub use scripting::{
    NoopScriptHost, ScriptAnimation, ScriptAnimationTime, ScriptArtboard,
    ScriptArtboardDataContext, ScriptArtboardParentContext, ScriptArtboardResolver,
    ScriptCoreString, ScriptDataConverterMethod, ScriptDataConverterOptionalCall, ScriptError,
    ScriptHost, ScriptImage, ScriptInstance, ScriptListenerActionDefinition,
    ScriptListenerActionHydration, ScriptListenerActionMethod, ScriptListenerInputDefinition,
    ScriptListenerInputHydration, ScriptListenerInputKind, ScriptListenerInputSnapshot,
    ScriptListenerInputSnapshotValue, ScriptMethod, ScriptModule, ScriptModuleFailure, ScriptNode,
    ScriptPaint, ScriptValue, ScriptViewModel, ScriptViewModelInputResolver,
    ScriptViewModelProperty, ScriptedDrawableInputResult, ScriptedStateMachineObjectKind,
    ScriptingVm, bound_script_artboard_input, bound_script_input_value, bound_script_trigger_input,
    script_node_for_artboard, script_view_model_from_owned, script_view_model_from_owned_context,
    script_view_model_from_owned_snapshot, script_view_models,
};
#[doc(hidden)]
pub use state_machine::RuntimeFileStateMachineActionCatalog;
pub use state_machine::{
    FocusState, RuntimeLayerState, RuntimeNestedStateMachineReport,
    RuntimeScriptedListenerDataConverterBindStep, RuntimeStateMachine,
    RuntimeStateMachineDataConverterBindStep, RuntimeStateMachineInput, RuntimeStateMachineLayer,
    ScriptGamepadInputChange, ScriptGamepadMappingKind, ScriptGamepadSnapshot,
    ScriptListenerInvocation, ScriptPointerEventKind, StateMachineEventContext,
    StateMachineEventStringProperty, StateMachineInputInstance, StateMachineInputKind,
    StateMachineInstance, StateMachineReportedEvent,
};
#[cfg(feature = "tools")]
#[doc(hidden)]
pub use state_machine::{
    RuntimeNestedEventChainPhase, RuntimeNestedEventChainStep, RuntimeNestedEventChainTrace,
    RuntimeNestedNotifyBatchEntry, RuntimeNestedNotifyBatchTrace,
};
pub(crate) use state_machine::{
    RuntimeTransitionInterpolator, StateMachineBindableArtboardInstance,
    StateMachineBindableAssetInstance, StateMachineBindableBooleanInstance,
    StateMachineBindableColorInstance, StateMachineBindableEnumInstance,
    StateMachineBindableIntegerInstance, StateMachineBindableListInstance,
    StateMachineBindableNumberInstance, StateMachineBindableStringInstance,
    StateMachineBindableTriggerInstance, StateMachineBindableViewModelInstance,
    StateMachineTransitionDurationInstance,
};
pub use text::{
    embedded_font_is_parseable, embedded_fonts_are_parseable, static_text_support_error,
};
pub use view_model::{
    RuntimeBindableArtboard, RuntimeDataContext, RuntimeDataContextInstanceRef,
    RuntimeDataContextLookupKind, RuntimeDataContextLookupReport, RuntimeDataContextValueRef,
    RuntimeDefaultViewModelArtboardSourceHandle, RuntimeDefaultViewModelAssetSourceHandle,
    RuntimeDefaultViewModelBooleanSourceHandle, RuntimeDefaultViewModelColorSourceHandle,
    RuntimeDefaultViewModelEnumSourceHandle, RuntimeDefaultViewModelListSourceHandle,
    RuntimeDefaultViewModelNumberSourceHandle, RuntimeDefaultViewModelStringSourceHandle,
    RuntimeDefaultViewModelSymbolListIndexSourceHandle, RuntimeDefaultViewModelTriggerSourceHandle,
    RuntimeDefaultViewModelViewModelSourceHandle, RuntimeFontAssetValue,
    RuntimeImportedViewModelArtboardSourceHandle, RuntimeImportedViewModelAssetSourceHandle,
    RuntimeImportedViewModelBooleanSourceHandle, RuntimeImportedViewModelColorSourceHandle,
    RuntimeImportedViewModelEnumSourceHandle, RuntimeImportedViewModelInstanceContext,
    RuntimeImportedViewModelListSourceHandle, RuntimeImportedViewModelNumberSourceHandle,
    RuntimeImportedViewModelStringSourceHandle,
    RuntimeImportedViewModelSymbolListIndexSourceHandle,
    RuntimeImportedViewModelTriggerSourceHandle, RuntimeImportedViewModelViewModelSourceHandle,
    RuntimeOwnedViewModelArtboardSourceHandle, RuntimeOwnedViewModelAssetSourceHandle,
    RuntimeOwnedViewModelBooleanSourceHandle, RuntimeOwnedViewModelColorSourceHandle,
    RuntimeOwnedViewModelContext, RuntimeOwnedViewModelContextHandle,
    RuntimeOwnedViewModelEnumSourceHandle, RuntimeOwnedViewModelFontAssetSourceHandle,
    RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance,
    RuntimeOwnedViewModelListSourceHandle, RuntimeOwnedViewModelListStringMatchBooleanHandle,
    RuntimeOwnedViewModelNumberSourceHandle, RuntimeOwnedViewModelStringSourceHandle,
    RuntimeOwnedViewModelSymbolListIndexSourceHandle, RuntimeOwnedViewModelTriggerSourceHandle,
    RuntimeOwnedViewModelViewModelSourceHandle, RuntimeViewModelImage, RuntimeViewModelLinkError,
    ViewModelInstanceArtboardRuntime, ViewModelInstanceAssetFontRuntime,
    ViewModelInstanceAssetImageRuntime, ViewModelInstanceBooleanRuntime,
    ViewModelInstanceColorRuntime, ViewModelInstanceEnumRuntime, ViewModelInstanceListIndexRuntime,
    ViewModelInstanceListRuntime, ViewModelInstanceNumberRuntime, ViewModelInstanceRuntime,
    ViewModelInstanceRuntimeProperty, ViewModelInstanceStringRuntime,
    ViewModelInstanceTriggerRuntime, ViewModelInstanceValueRuntime, ViewModelRuntime,
    ViewModelRuntimeDataType, ViewModelRuntimeProperty, runtime_data_context_lookup_reports,
    runtime_global_view_model_indices, runtime_global_view_model_names,
};
pub(crate) use view_model::{
    RuntimeViewModelPointer, runtime_default_view_model_artboard_property_path_for_name,
    runtime_default_view_model_artboard_property_path_for_name_path,
    runtime_default_view_model_asset_property_path_for_name,
    runtime_default_view_model_asset_property_path_for_name_path,
    runtime_default_view_model_boolean_property_path_for_name,
    runtime_default_view_model_boolean_property_path_for_name_path,
    runtime_default_view_model_color_property_path_for_name,
    runtime_default_view_model_color_property_path_for_name_path,
    runtime_default_view_model_enum_property_path_for_name,
    runtime_default_view_model_enum_property_path_for_name_path,
    runtime_default_view_model_list_property_path_for_name,
    runtime_default_view_model_list_property_path_for_name_path,
    runtime_default_view_model_number_property_path_for_name,
    runtime_default_view_model_number_property_path_for_name_path,
    runtime_default_view_model_string_property_path_for_name,
    runtime_default_view_model_string_property_path_for_name_path,
    runtime_default_view_model_symbol_list_index_property_path_for_name,
    runtime_default_view_model_symbol_list_index_property_path_for_name_path,
    runtime_default_view_model_trigger_property_path_for_name,
    runtime_default_view_model_trigger_property_path_for_name_path,
    runtime_default_view_model_view_model_property_path_for_name,
    runtime_default_view_model_view_model_property_path_for_name_path,
    runtime_view_model_view_model_property_path_for_name_path,
};
#[doc(hidden)]
pub use view_model_cell::RuntimeFileViewModelInstanceCatalog;

#[cfg(test)]
#[test]
fn fl_c5_public_reexports_survive_file_split() {
    use crate::{
        FocusState, RuntimeStateMachine, ScriptError, ScriptListenerActionHydration,
        StateMachineEventContext, StateMachineEventStringProperty, StateMachineInstance,
        StateMachineReportedEvent,
    };

    macro_rules! methods_are_reachable {
        ($owner:ty; $($method:ident),+ $(,)?) => {
            $(let _ = <$owner>::$method;)+
        };
    }

    fn definition_fields_are_reachable(machine: &RuntimeStateMachine) {
        let _ = machine.global_id;
        let _ = &machine.name;
        let _ = &machine.inputs;
        let _ = &machine.layers;
    }
    let _ = definition_fields_are_reachable as fn(&RuntimeStateMachine);
    fn is_clone<T: Clone>() {}
    is_clone::<StateMachineInstance>();
    let focus_state = FocusState::default();
    let _ = (focus_state.has_focus, focus_state.expects_keyboard_input);

    methods_are_reachable!(RuntimeStateMachine;
        scripted_objects,
        scripted_listener_actions,
    );
    methods_are_reachable!(StateMachineInstance;
        state_machine_index,
        input_index_named,
        set_bool,
        set_number,
        fire_trigger,
        focus_up,
        focus_down,
        focus_left,
        focus_right,
        set_focus,
        focus_state,
        internal_focus_manager,
        has_external_focus_manager,
        clear_external_focus_manager,
        semantic_manager,
        enable_semantics,
        set_external_semantic_manager,
        fire_semantic_action,
        key_input,
        text_input,
        gamepad_dispatch,
        pointer_down,
        pointer_down_with_event_context,
        pointer_down_with_owned_view_model_context,
        pointer_down_with_owned_view_model_and_event_context,
        try_pointer_down_with_script_host,
        try_pointer_down_with_timestamp_and_script_host,
        try_pointer_down_with_owned_view_model_context_and_script_host,
        pointer_move,
        pointer_move_with_owned_view_model_context,
        try_pointer_move_with_script_host,
        try_pointer_move_with_timestamp_and_script_host,
        try_pointer_move_with_owned_view_model_context_and_script_host,
        pointer_up,
        pointer_up_with_event_context,
        pointer_up_with_owned_view_model_context,
        pointer_up_with_owned_view_model_and_event_context,
        try_pointer_up_with_script_host,
        try_pointer_up_with_timestamp_and_script_host,
        try_pointer_up_with_owned_view_model_context_and_script_host,
        pointer_exit,
        pointer_exit_with_owned_view_model_context,
        try_pointer_exit_with_script_host,
        try_pointer_exit_with_timestamp_and_script_host,
        try_pointer_exit_with_owned_view_model_context_and_script_host,
        take_reported_events,
        reported_event_snapshot,
        has_pending_listener_view_model_reports,
        script_error,
        retain_scripted_object_data_context_error,
        scripted_objects,
        scripted_listener_actions,
        set_script_instance_for_global,
        set_script_input_for_global,
        set_scripted_listener_action_instance,
        set_scripted_object_instance,
        install_scripted_object_data_context,
        synchronize_scripted_input_groups,
        scripted_listener_action_input_snapshots,
        bind_scripted_listener_action_sources,
        bind_scripted_listener_input_source,
        bind_scripted_listener_converter_own_sources,
        finalize_scripted_listener_input_sources,
        scripted_listener_data_context_view_models,
        scripted_listener_artboard_parent_context,
        scripted_listener_bound_view_model,
        resolve_scripted_listener_scalar_binding,
        resolve_scripted_listener_artboard_binding,
        resolve_scripted_listener_trigger_binding,
        apply_scripted_listener_action_source_updates,
        set_scripted_listener_artboard_resolver,
        scripted_listener_data_converter_targets,
        scripted_listener_data_converter_occurrences,
        scripted_listener_data_converter_bind_steps,
        scripted_listener_data_converter_input_snapshots,
        set_scripted_listener_data_converter_instance,
        has_scripted_listener_data_converter_instance,
        rebind_scripted_listener_data_converter_final_input,
        scripted_data_converter_occurrence_snapshots,
        scripted_data_converter_input_snapshots,
        set_scripted_data_converter_instance,
        has_scripted_data_converter_instance,
        bind_scripted_data_converter_sources,
        rebind_scripted_data_converter_final_inputs,
        begin_scripted_object_data_context_bind,
        begin_retained_scripted_object_data_context_rebind,
        finish_scripted_object_data_context_bind,
        adopt_scripted_listener_action_state_from,
        rehome_owned_data_context_for_transaction,
        bind_empty_data_context,
        bind_default_view_model_context,
        bind_view_model_instance_context,
        bind_imported_view_model_context,
        bind_owned_view_model_context,
        bind_owned_view_model_handle,
        bind_owned_view_model_context_handle,
        bind_owned_view_model_context_mut,
        bind_owned_view_model_contexts,
        bind_script_artboard_data_context,
        set_bindable_number_for_data_bind,
        set_bindable_boolean_for_data_bind,
        set_bindable_integer_for_data_bind,
        set_bindable_color_for_data_bind,
        set_bindable_string_for_data_bind,
        set_bindable_enum_for_data_bind,
        set_bindable_asset_for_data_bind,
        set_bindable_artboard_for_data_bind,
        set_bindable_list_for_data_bind,
        set_bindable_trigger_for_data_bind,
        set_bindable_view_model_for_data_bind,
        bindable_number_value_for_data_bind,
        bindable_boolean_value_for_data_bind,
        bindable_integer_value_for_data_bind,
        bindable_color_value_for_data_bind,
        bindable_string_value_for_data_bind,
        bindable_enum_value_for_data_bind,
        bindable_asset_value_for_data_bind,
        bindable_artboard_value_for_data_bind,
        bindable_list_property_value_for_data_bind,
        bindable_trigger_value_for_data_bind,
        bindable_view_model_instance_index_for_data_bind,
        default_view_model_number_source_value_for_data_bind,
        default_view_model_boolean_source_value_for_data_bind,
        default_view_model_string_source_value_for_data_bind,
        default_view_model_color_source_value_for_data_bind,
        default_view_model_enum_source_value_for_data_bind,
        default_view_model_symbol_list_index_source_value_for_data_bind,
        default_view_model_asset_source_value_for_data_bind,
        default_view_model_artboard_source_value_for_data_bind,
        default_view_model_list_source_item_count_for_data_bind,
        default_view_model_trigger_source_value_for_data_bind,
        default_view_model_view_model_source_instance_index_for_data_bind,
        set_default_view_model_number_source_for_data_bind,
        set_default_view_model_number_source_by_property_name,
        default_view_model_number_source_handle_by_property_name,
        default_view_model_number_source_handle_by_property_name_path,
        set_default_view_model_number_source_by_source_handle,
        set_default_view_model_boolean_source_for_data_bind,
        set_default_view_model_boolean_source_by_property_name,
        default_view_model_boolean_source_handle_by_property_name,
        default_view_model_boolean_source_handle_by_property_name_path,
        set_default_view_model_boolean_source_by_source_handle,
        set_default_view_model_string_source_for_data_bind,
        set_default_view_model_string_source_by_property_name,
        default_view_model_string_source_handle_by_property_name,
        default_view_model_string_source_handle_by_property_name_path,
        set_default_view_model_string_source_by_source_handle,
        set_default_view_model_color_source_for_data_bind,
        set_default_view_model_color_source_by_property_name,
        default_view_model_color_source_handle_by_property_name,
        default_view_model_color_source_handle_by_property_name_path,
        set_default_view_model_color_source_by_source_handle,
        set_default_view_model_enum_source_for_data_bind,
        set_default_view_model_enum_source_by_property_name,
        default_view_model_enum_source_handle_by_property_name,
        default_view_model_enum_source_handle_by_property_name_path,
        set_default_view_model_enum_source_by_source_handle,
        set_default_view_model_symbol_list_index_source_for_data_bind,
        set_default_view_model_symbol_list_index_source_by_property_name,
        default_view_model_symbol_list_index_source_handle_by_property_name,
        default_view_model_symbol_list_index_source_handle_by_property_name_path,
        set_default_view_model_symbol_list_index_source_by_source_handle,
        set_default_view_model_asset_source_for_data_bind,
        set_default_view_model_asset_source_by_property_name,
        default_view_model_asset_source_handle_by_property_name,
        default_view_model_asset_source_handle_by_property_name_path,
        set_default_view_model_asset_source_by_source_handle,
        set_default_view_model_artboard_source_for_data_bind,
        set_default_view_model_artboard_source_by_property_name,
        default_view_model_artboard_source_handle_by_property_name,
        default_view_model_artboard_source_handle_by_property_name_path,
        set_default_view_model_artboard_source_by_source_handle,
        set_default_view_model_trigger_source_for_data_bind,
        set_default_view_model_trigger_source_by_property_name,
        default_view_model_trigger_source_handle_by_property_name,
        default_view_model_trigger_source_handle_by_property_name_path,
        set_default_view_model_trigger_source_by_source_handle,
        set_default_view_model_list_source_item_count_for_data_bind,
        set_default_view_model_list_source_item_count_by_property_name,
        default_view_model_list_source_handle_by_property_name,
        default_view_model_list_source_handle_by_property_name_path,
        set_default_view_model_list_source_item_count_by_source_handle,
        set_default_view_model_view_model_source_for_data_bind,
        relink_default_view_model_view_model_source_for_data_bind,
        relink_default_view_model_view_model_source_by_property_name,
        default_view_model_view_model_source_handle_by_property_name,
        default_view_model_view_model_source_handle_by_property_name_path,
        relink_default_view_model_view_model_source_by_source_handle,
        relink_view_model_instance_view_model_source_for_data_bind,
        relink_imported_view_model_context_view_model_source_for_data_bind,
        set_imported_view_model_context_number_source_for_data_bind,
        set_imported_view_model_context_boolean_source_for_data_bind,
        set_imported_view_model_context_string_source_for_data_bind,
        set_imported_view_model_context_color_source_for_data_bind,
        set_imported_view_model_context_enum_source_for_data_bind,
        set_imported_view_model_context_symbol_list_index_source_for_data_bind,
        set_imported_view_model_context_asset_source_for_data_bind,
        set_imported_view_model_context_artboard_source_for_data_bind,
        set_imported_view_model_context_trigger_source_for_data_bind,
        set_imported_view_model_context_list_source_item_count_for_data_bind,
        set_owned_view_model_context_number_source_for_data_bind,
        set_owned_view_model_context_boolean_source_for_data_bind,
        set_owned_view_model_context_string_source_for_data_bind,
        set_owned_view_model_context_color_source_for_data_bind,
        set_owned_view_model_context_enum_source_for_data_bind,
        set_owned_view_model_context_symbol_list_index_source_for_data_bind,
        set_owned_view_model_context_asset_source_for_data_bind,
        set_owned_view_model_context_artboard_source_for_data_bind,
        set_owned_view_model_context_trigger_source_for_data_bind,
        set_owned_view_model_context_list_source_item_count_for_data_bind,
        set_owned_view_model_context_view_model_source_for_data_bind,
        bind_state_machine_data_bind_source,
        bind_state_machine_data_converter_own_sources,
        finalize_state_machine_data_bind_source,
        rebind_state_machine_data_converter_final_input,
        update_data_binds_apply_target_to_source,
        set_data_bind_formula_random_values,
        data_bind_formula_random_call_count,
        transition_duration_binding_count,
        transition_duration_binding_value,
        view_model_trigger_count,
        view_model_trigger_value_count,
        view_model_trigger_property_id,
    );

    type HydrationFactory =
        fn(&StateMachineInstance) -> Result<ScriptListenerActionHydration, ScriptError>;
    let _ = StateMachineInstance::hydrate_and_initialize_scripted_data_converter_instance::<
        HydrationFactory,
    >;
    let _ = StateMachineInstance::hydrate_and_initialize_scripted_listener_data_converter_instance::<
        HydrationFactory,
    >;
    let _ = StateMachineInstance::hydrate_and_initialize_scripted_listener_action_instance::<
        HydrationFactory,
    >;
    let _ =
        StateMachineInstance::hydrate_and_initialize_scripted_object_instance::<HydrationFactory>;
    let _ =
        StateMachineInstance::hydrate_and_initialize_scripted_object_instance_after_context_install::<
            HydrationFactory,
        >;

    methods_are_reachable!(StateMachineEventContext;
        from_geometry_hit,
        path,
        occurrence,
    );
    methods_are_reachable!(StateMachineEventStringProperty;
        name,
        value,
    );
    methods_are_reachable!(StateMachineReportedEvent;
        event_local_index,
        event_core_type,
        name,
        url,
        target,
        string_properties,
        properties,
        seconds_delay,
        context,
    );
}
