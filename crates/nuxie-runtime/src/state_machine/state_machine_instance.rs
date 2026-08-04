// Compatibility hub for the C++-counterpart state-machine instance modules.
use super::focused_input_dispatch::RuntimeInputDispatchOutcome;
use super::listener_types::RuntimeListenerViewModelPath;
use super::*;
use crate::artboard_component_list_order::runtime_component_list_order;
use crate::artboard_data_bind::RuntimeOwnedDataContext;
#[cfg(any(test, feature = "tools"))]
use crate::components::TransformProperty;
use crate::components::{ComponentHandle, Mat2D, RuntimeShapeState};
use crate::constraints::draggable_constraint::{RuntimeDraggableProxy, runtime_draggable_proxies};
use crate::constraints::{
    runtime_draggable_proxy_drag, runtime_draggable_proxy_end, runtime_draggable_proxy_start,
};
use crate::data_bind_container::RuntimeDataBindContainerQueue;
use crate::data_bind_graph::{
    RuntimeDataBindGraphContextKind, data_bind_flags_apply_source_to_target,
};
use crate::data_context::RuntimeStateMachineDataContext;
use crate::draw::{runtime_path_geometry_hit_test, runtime_text_value_run_hit_test};
use crate::focus::RuntimeFocusTree;
use crate::listener_group::{ListenerGroup, ListenerGroupKind, select_listener_action};
use crate::properties::property_key_for_name;
use crate::script_asset::RuntimeScriptImplementedMethods;
use crate::scripting::RuntimeScriptInstanceHandle;
use crate::semantic_data::{RuntimeSemanticData, SemanticActionType, SemanticNodeHandle};
use crate::semantic_manager::{SemanticDrainError, SemanticManager, SemanticsDiff};
use crate::semantic_runtime_tree::RuntimeSemanticTree;
use crate::view_model::{
    RuntimeBlobAssetValue, RuntimeFontAssetValue, RuntimeOwnedViewModelAdvanceContext,
};
use crate::view_model_cell::{
    RuntimeCellDirt, RuntimeCellDirtSink, RuntimeCellNotificationQueue,
    RuntimeFileViewModelInstanceCatalog, RuntimeViewModelCell, RuntimeViewModelCellValue,
    RuntimeViewModelInstanceCells,
};
use crate::{
    ArtboardInstance, ComponentDirt, NoopScriptHost, RuntimeDataBindGraph,
    RuntimeDataBindGraphApplyPhase, RuntimeDataBindGraphTargetsMut, RuntimeDataBindGraphValue,
    RuntimeDefaultViewModelArtboardSourceHandle, RuntimeDefaultViewModelAssetSourceHandle,
    RuntimeDefaultViewModelBooleanSourceHandle, RuntimeDefaultViewModelColorSourceHandle,
    RuntimeDefaultViewModelEnumSourceHandle, RuntimeDefaultViewModelListSourceHandle,
    RuntimeDefaultViewModelNumberSourceHandle, RuntimeDefaultViewModelStringSourceHandle,
    RuntimeDefaultViewModelSymbolListIndexSourceHandle, RuntimeDefaultViewModelTriggerSourceHandle,
    RuntimeDefaultViewModelViewModelSourceHandle, RuntimeImportedViewModelInstanceContext,
    RuntimeOwnedViewModelContext, RuntimeOwnedViewModelContextHandle, RuntimeOwnedViewModelHandle,
    RuntimeOwnedViewModelInstance, ScriptArtboardDataContext, ScriptArtboardParentContext,
    ScriptArtboardResolver, ScriptCoreString, ScriptError, ScriptHost, ScriptInstance,
    ScriptListenerActionDefinition, ScriptListenerInvocation, ScriptMethod, ScriptPointerEventKind,
    ScriptValue, ScriptViewModel, ScriptedDrawablePointerHit,
    runtime_default_view_model_artboard_property_path_for_name,
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
};
use nuxie_binary::RuntimeFile;
use nuxie_render_api::Factory as RenderFactory;
#[cfg(any(test, feature = "tools"))]
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(test)]
use crate::ScriptListenerActionMethod;

mod state_machine_instance;

pub(super) use state_machine_instance::RuntimeStateMachineListenerActionExecutor;
pub use state_machine_instance::{FocusState, RuntimeHitResult, StateMachineInstance};
pub(crate) use state_machine_instance::{
    RuntimeDataContextBindError, RuntimeSemanticOccurrenceKey, RuntimeSemanticRoute,
    SemanticNodeResolver, closest_semantic_node,
};
#[cfg(any(test, feature = "tools"))]
pub use state_machine_instance::{
    RuntimeNestedEventChainPhase, RuntimeNestedEventChainStep, RuntimeNestedEventChainTrace,
    RuntimeNestedNotifyBatchEntry, RuntimeNestedNotifyBatchTrace,
};
