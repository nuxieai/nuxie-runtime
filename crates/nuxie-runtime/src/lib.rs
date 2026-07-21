mod animation;
mod artboard;
mod artboard_data_bind;
mod components;
mod constraints;
mod data_bind_graph;
mod draw;
mod focus;
mod objects;
mod project_data_converter;
mod properties;
mod scripting;
mod state_machine;
mod text;
mod view_model;
// #RB-1: retained-identity view-model core (map Phase RB). Additive while
// consumers migrate; the compensation family deletes when migration ends.
pub mod view_model_cell;

pub use animation::{
    LinearAnimationInstance, RuntimeKeyFrameBool, RuntimeKeyFrameCallback, RuntimeKeyFrameColor,
    RuntimeKeyFrameDouble, RuntimeKeyFrameString, RuntimeKeyFrameUint, RuntimeKeyedObject,
    RuntimeKeyedProperty, RuntimeLinearAnimation,
};
pub use artboard::{
    ArtboardInstance, ExternalFontAssetError, RuntimeArtboardOccurrenceSegment,
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
    RuntimeContourMeasure, RuntimeDrawCommand, RuntimeDrawCommandKind,
    RuntimeDrawCommandObjectKind, RuntimeFeatherState, RuntimeGeometryCache, RuntimeGeometryHit,
    RuntimeGeometryHitOccurrence, RuntimeGeometryHitPathSegment, RuntimeGradientStop,
    RuntimeImageDimensionConflict, RuntimeLayoutBoundsReport, RuntimePathCommand,
    RuntimePathMeasure, RuntimePathSample, RuntimeRenderImages, RuntimeRenderPaintCache,
    RuntimeRenderPaints, RuntimeRenderPathCache, RuntimeSemanticTextHit, RuntimeShapePaintCommand,
    RuntimeShapePaintKind, RuntimeShapePaintPathKind, RuntimeShapePaintState,
    preallocate_render_paint_cache_for_artboard_instance,
    preallocate_render_paint_cache_for_artboard_tree,
    preallocate_render_paint_cache_for_artboard_tree_with_external_images,
    preallocate_render_paint_cache_for_scripted_artboard_tree,
    preallocate_render_paint_cache_for_scripted_artboard_tree_after_source_paints,
    preallocate_render_paint_cache_for_scripted_artboard_tree_with_file_registration,
    preallocate_render_paints, preallocate_render_paints_for_artboard_tree,
    preallocate_source_render_paints, runtime_path_commands_from_raw_path,
};
pub use focus::{
    FocusBounds, FocusDirection, FocusEdgeBehavior, FocusEvent, FocusEventKind, FocusManager,
    FocusNode, FocusNodeId, FocusPoint,
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
pub use scripting::{
    NoopScriptHost, ScriptAnimation, ScriptAnimationTime, ScriptArtboard,
    ScriptDataConverterMethod, ScriptError, ScriptHost, ScriptImage, ScriptInstance,
    ScriptListenerActionDefinition, ScriptListenerActionHydration, ScriptListenerActionMethod,
    ScriptListenerInputDefinition, ScriptListenerInputHydration, ScriptListenerInputKind,
    ScriptListenerInvocation, ScriptMethod, ScriptModule, ScriptModuleFailure, ScriptNode,
    ScriptPaint, ScriptPointerEventKind, ScriptValue, ScriptViewModel, ScriptViewModelProperty,
    ScriptingVm, bound_script_artboard_input, bound_script_input_value, bound_script_trigger_input,
    bound_script_view_model_from_owned_context, bound_script_view_model_snapshot,
    script_node_for_artboard, script_view_model_from_owned, script_view_model_from_owned_snapshot,
    script_view_models,
};
pub use state_machine::{
    RuntimeLayerState, RuntimeStateMachine, RuntimeStateMachineInput, RuntimeStateMachineLayer,
    StateMachineEventContext, StateMachineInputInstance, StateMachineInputKind,
    StateMachineInstance, StateMachineReportedEvent,
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
    RuntimeDataContext, RuntimeDataContextInstanceRef, RuntimeDataContextLookupKind,
    RuntimeDataContextLookupReport, RuntimeDataContextValueRef,
    RuntimeDefaultViewModelArtboardSourceHandle, RuntimeDefaultViewModelAssetSourceHandle,
    RuntimeDefaultViewModelBooleanSourceHandle, RuntimeDefaultViewModelColorSourceHandle,
    RuntimeDefaultViewModelEnumSourceHandle, RuntimeDefaultViewModelListSourceHandle,
    RuntimeDefaultViewModelNumberSourceHandle, RuntimeDefaultViewModelStringSourceHandle,
    RuntimeDefaultViewModelSymbolListIndexSourceHandle, RuntimeDefaultViewModelTriggerSourceHandle,
    RuntimeDefaultViewModelViewModelSourceHandle, RuntimeImportedViewModelArtboardSourceHandle,
    RuntimeImportedViewModelAssetSourceHandle, RuntimeImportedViewModelBooleanSourceHandle,
    RuntimeImportedViewModelColorSourceHandle, RuntimeImportedViewModelEnumSourceHandle,
    RuntimeImportedViewModelInstanceContext, RuntimeImportedViewModelListSourceHandle,
    RuntimeImportedViewModelNumberSourceHandle, RuntimeImportedViewModelStringSourceHandle,
    RuntimeImportedViewModelSymbolListIndexSourceHandle,
    RuntimeImportedViewModelTriggerSourceHandle, RuntimeImportedViewModelViewModelSourceHandle,
    RuntimeOwnedViewModelArtboardSourceHandle, RuntimeOwnedViewModelAssetSourceHandle,
    RuntimeOwnedViewModelBooleanSourceHandle, RuntimeOwnedViewModelColorSourceHandle,
    RuntimeOwnedViewModelContext, RuntimeOwnedViewModelContextHandle,
    RuntimeOwnedViewModelEnumSourceHandle, RuntimeOwnedViewModelHandle,
    RuntimeOwnedViewModelInstance, RuntimeOwnedViewModelListSourceHandle,
    RuntimeOwnedViewModelListStringMatchBooleanHandle, RuntimeOwnedViewModelNumberSourceHandle,
    RuntimeOwnedViewModelStringSourceHandle, RuntimeOwnedViewModelSymbolListIndexSourceHandle,
    RuntimeOwnedViewModelTriggerSourceHandle, RuntimeOwnedViewModelViewModelSourceHandle,
    RuntimeViewModelLinkError, runtime_data_context_lookup_reports,
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
