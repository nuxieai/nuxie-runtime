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

mod data_bind_context;
mod data_converter_group;
mod listener_viewmodel_change;
mod state_machine_instance;
mod text_input_listener_group;
mod viewmodel_instance_trigger;

#[cfg(test)]
use state_machine_instance::RuntimeDeferredCallbackProbe;
use state_machine_instance::{
    AudioEventOccurrence, AudioEventSeam, HitComponent, HitDrawable, RuntimeConstructorPhase,
    RuntimeNestedEventRegistration, RuntimeQueuedFocusEvent, RuntimeQueuedSemanticEvent,
    RuntimeStateMachineDataBindOccurrence, RuntimeViewModelListenerCellBinding,
    RuntimeViewModelListenerInstance,
};

#[derive(Debug)]
pub struct StateMachineInstance {
    state_machine_index: usize,
    profile_name: Arc<str>,
    /// Retained authored definition owner. C++ stores `const StateMachine*
    /// m_machine` on the instance and reuses it for every advance
    /// (`state_machine_instance.hpp:123,386`;
    /// `state_machine_instance.cpp:1707-1711`).
    ///
    /// Rust retains the immutable definition arena as well as the index so
    /// the matching definition address remains stable for this occurrence.
    state_machine_definitions: Option<Arc<Vec<RuntimeStateMachine>>>,
    /// Exact listener definitions retained by this occurrence's dispatch
    /// groups. C++ groups store listener pointers once at construction rather
    /// than rediscovering them from the Artboard during every callback.
    pub(super) listener_definitions: Arc<Vec<RuntimeStateMachineListener>>,
    default_view_model_index: Option<usize>,
    /// Shared authored instances for bare default/serialized binds. The C++
    /// probe retains `ViewModel::instance(index)` directly (`main.cpp:4683-4721`).
    file_view_model_instances: Option<RuntimeFileViewModelInstanceCatalog>,
    default_view_model_trigger_instance: Option<RuntimeViewModelInstanceCells>,
    active_file_view_model_binding: Option<(usize, usize)>,
    active_owned_view_model_advance_context: Option<RuntimeOwnedViewModelAdvanceContext>,
    /// The internal focus domain exists before layer entry. `Drop` explicitly
    /// releases this value before bind/layer/script state; an external
    /// projection releases only its `Rc` reference and leaves the shared
    /// owner's domain intact.
    focus: RuntimeFocusTree,
    /// Retained C++ `m_focusManager` adaptation while an external manager is
    /// selected. This stores the owner-safe internal projection, not manager
    /// internals from the RECORDED `src/input/focus_manager.cpp` seam owned by
    /// manifest row B6-0238 (`focus.rs`, DIVERGENT).
    internal_focus: Option<RuntimeFocusTree>,
    /// Selection flag paired with RuntimeFocusTree's owner-safe shared-domain
    /// identity check for the C++ same-pointer no-op.
    external_focus_manager_selected: bool,
    owns_focus_domain: bool,
    #[cfg(test)]
    focus_manager_phase_trace: Vec<&'static str>,
    /// Whether the instance-owned retained semantic manager is enabled.
    internal_semantic_manager_enabled: bool,
    /// Retained semantic domain. It is created by `enable_semantics` and
    /// populated from live Artboard occurrences at the first semantic
    /// operation, mirroring C++'s lazy opt-in without retaining an Artboard
    /// borrow across calls.
    semantic_tree: Option<RuntimeSemanticTree>,
    external_semantic_manager_identity: Option<u64>,
    /// Compatibility resolver for the recorded external-manager seam.
    /// Production internal routing uses `semantic_tree`; this defaults to
    /// absent and is injected only by focused boundary tests.
    semantic_node_resolver: Option<Rc<dyn SemanticNodeResolver>>,
    #[cfg(test)]
    semantic_manager_phase_trace: Vec<&'static str>,
    inputs: Vec<StateMachineInputInstance>,
    bindable_numbers: Vec<StateMachineBindableNumberInstance>,
    bindable_integers: Vec<StateMachineBindableIntegerInstance>,
    bindable_colors: Vec<StateMachineBindableColorInstance>,
    bindable_strings: Vec<StateMachineBindableStringInstance>,
    bindable_enums: Vec<StateMachineBindableEnumInstance>,
    bindable_assets: Vec<StateMachineBindableAssetInstance>,
    bindable_artboards: Vec<StateMachineBindableArtboardInstance>,
    bindable_lists: Vec<StateMachineBindableListInstance>,
    bindable_triggers: Vec<StateMachineBindableTriggerInstance>,
    bindable_view_models: Vec<StateMachineBindableViewModelInstance>,
    bindable_booleans: Vec<StateMachineBindableBooleanInstance>,
    default_view_model_triggers: Arc<Vec<RuntimeViewModelTrigger>>,
    transition_durations: Vec<StateMachineTransitionDurationInstance>,
    /// Rust initialization for C++ `m_layerCount`, which has no header
    /// initializer. It is derived from the supplied machine before access.
    layer_count: usize,
    /// Bind owners are declared before layers so their retained cells and
    /// converter tables drop first, matching C++ teardown.
    pub(super) data_bind_graph: RuntimeDataBindGraph,
    /// One C++ `DataBindContainer` queue shared by ordinary state-machine
    /// binds and every DataBind cloned with a scripted listener action.
    data_bind_container: RuntimeDataBindContainerQueue,
    data_bind_occurrences: Vec<RuntimeStateMachineDataBindOccurrence>,
    key_frame_data_bind_graphs: Vec<Option<RuntimeDataBindGraph>>,
    next_key_frame_data_bind_occurrence_id: u64,
    layers: Vec<StateMachineLayerInstance>,
    reported_events: Vec<StateMachineReportedEvent>,
    /// Prefix of `reported_events` already consumed by C++ `applyEvents`.
    /// Public reports remain frame-visible after listener delivery, so this
    /// cursor keeps the two lifetimes distinct without replaying listeners.
    reported_event_listener_index: usize,
    /// Prefix of `reported_events` already returned through Rust's draining
    /// host seam. The core queue remains intact until `applyEvents`, like C++.
    host_reported_event_index: usize,
    /// Retained C++ `m_reportingEvents` analog used while notifications may
    /// enqueue the next batch into `reported_events`.
    reporting_events: Vec<StateMachineReportedEvent>,
    /// Events first reported inside the current `applyEvents` loop. They have
    /// already reached listeners, but remain visible to the host until the
    /// next loop begins.
    events_applied_during_loop: Vec<StateMachineReportedEvent>,
    /// Rust draining-host cursor for `events_applied_during_loop`.
    host_events_applied_during_loop_index: usize,
    /// Owner-safe output of the immediate nested bubbling phase. The artboard
    /// owner drains this FIFO and delivers it to the next ancestor; retaining
    /// values here avoids a raw child-to-parent pointer.
    bubbled_event_reports: Vec<StateMachineReportedEvent>,
    bubbled_event_report_index: usize,
    /// Audio occurrences whose reporting machine has an owner. C++ reaches
    /// these only after synchronous ancestor notification unwinds. Rust keeps
    /// the typed occurrences here until the owner-mediated frame path has
    /// completed that ancestor dispatch.
    deferred_owner_audio_occurrences: Vec<AudioEventOccurrence>,
    /// Whether this occurrence is owned by a `NestedStateMachine` notifier.
    /// Root occurrences have no upward event edge and must not accumulate an
    /// outgoing bubble batch.
    event_bubble_owner_attached: bool,
    notifying_event_listeners: bool,
    /// C++ `m_reportedListenerViewModels`: every retained listener-cell
    /// mutation appends its listener index, preserving duplicates and
    /// dependent-registration order until next-frame `applyEvents`.
    reported_listener_view_models: RuntimeCellNotificationQueue,
    /// Retained C++ `m_reportingListenerViewModels` batch buffer.
    reporting_listener_view_models: Vec<usize>,
    /// Nested-ViewModel source reports discovered through Rust's external
    /// context adaptation become pending only after the current frame's
    /// `applyEvents`, matching the later C++ DataBind occurrence update.
    post_apply_listener_view_models: Vec<usize>,
    pub(super) needs_advance: bool,
    /// C++ `m_DataContext` can be mutated by the staged main/global setters
    /// before any DataBind is rebound. The existing Rust graph APIs used the
    /// bound source cells as their only context record, which collapsed that
    /// distinction. Retain the public composite shape separately so
    /// `setViewModelInstance`/`setGlobalViewModelInstance` can update slot
    /// ownership without applying paths until `bind`.
    primary_data_context: Option<RuntimeStateMachineDataContext>,
    pub(super) owned_data_context: Option<RuntimeOwnedDataContext>,
    #[cfg(test)]
    owned_data_bind_context_bind_count: usize,
    #[cfg(test)]
    bind_phase_trace: Vec<&'static str>,
    #[cfg(test)]
    event_dispatch_phase_trace: Vec<&'static str>,
    #[cfg(test)]
    event_total_order_trace: Option<(&'static str, &'static str, Rc<RefCell<Vec<&'static str>>>)>,
    #[cfg(test)]
    event_settlement_total_order_trace: Option<(&'static str, Rc<RefCell<Vec<&'static str>>>)>,
    #[cfg(test)]
    nested_event_forward_test: Option<StateMachineReportedEvent>,
    audio_event_seam: Rc<dyn AudioEventSeam>,
    audio_event_selection_count: usize,
    audio_event_last_occurrence: Option<AudioEventOccurrence>,
    #[cfg(test)]
    advance_phase_trace: Vec<&'static str>,
    #[cfg(test)]
    raw_advance_call_count: usize,
    #[cfg(test)]
    transition_probe_count: usize,
    #[cfg(test)]
    data_context_advance_call_count: usize,
    #[cfg(test)]
    bind_advance_test_report: Option<StateMachineReportedEvent>,
    /// C++ `ViewModelInstance::m_dependents` push channel: structural
    /// ViewModel replacement dirties this sink so the retained DataContext is
    /// relinked without polling a root mutation generation every frame.
    pub(super) owned_view_model_rebind_sink: RuntimeCellDirtSink,
    /// Fresh component-provided listener/proxy owners constructed for this
    /// StateMachineInstance (`state_machine_instance.cpp:1969-2013`).
    draggable_proxies: Vec<RuntimeDraggableProxy>,
    /// Complete polymorphic hit-owner hierarchy in current C++ hit order.
    hit_components: Vec<Box<dyn HitComponent>>,
    /// One retained ListenerGroup seam per authored/provider occurrence,
    /// including unresolved authored pointer targets.
    listener_groups: Vec<ListenerGroup>,
    /// Explicitly detachable, value-owned adaptation of nested notifier
    /// registrations.
    nested_event_registrations: Vec<RuntimeNestedEventRegistration>,
    disposed: bool,
    /// Explicit zero initialization for C++ `m_drawOrderChangeCounter`.
    /// WP3 owns constructor sorting and change-triggered re-sorting.
    draw_order_change_counter: u64,
    #[cfg(test)]
    constructor_phases: Vec<RuntimeConstructorPhase>,
    #[cfg(test)]
    drop_phase_receipt: Option<Rc<RefCell<Vec<&'static str>>>>,
    pub(super) scripted_object_definitions: Vec<ScriptListenerActionDefinition>,
    scripted_listener_action_definitions: Vec<ScriptListenerActionDefinition>,
    /// C++ deletes the cloned DataBinds before deleting the cloned
    /// ScriptedObjects. Keep the binding owner before every table-handle map
    /// so Rust field destruction preserves that same owner lifetime
    /// (`state_machine_instance.cpp:2169-2198`).
    scripted_object_bindings:
        Vec<super::scripted_listener_action::RuntimeScriptedListenerActionBindingOccurrence>,
    scripted_instances_by_global: BTreeMap<u32, RuntimeScriptInstanceHandle>,
    scripted_listener_action_instances: BTreeMap<u32, RuntimeScriptInstanceHandle>,
    /// C++ completes `cloneScriptedObject` plus the later
    /// `initScriptedObjects` pass once per concrete StateMachineInstance.
    ///
    /// Rust defers that File/VM-backed work until the facade supplies its
    /// authenticated scripting context. A snapshot clone starts cold because
    /// it cannot alias mutable Lua tables from the source occurrence.
    pub(super) scripted_object_initialization_complete: bool,
    /// Whether the owning Artboard already retained a DataContext when this
    /// exact StateMachineInstance occurrence was constructed. C++ observes
    /// this only inside the constructor: it assigns the Artboard context to
    /// every cloned ScriptedObject and runs `initScriptedObjects` before
    /// `ArtboardInstance::stateMachineAt` later calls `inheritDataContext`
    /// (`state_machine_instance.cpp:2072-2082`; `artboard.cpp:2844-2856`).
    pub(super) scripted_constructor_context_was_prebound: bool,
    /// Whether this occurrence has completed the latest C++
    /// `internalDataContext` bind across both ordinary and cloned
    /// ScriptedObject DataBinds. This is independent of Lua table
    /// initialization: no-Factory entry points must still bind the fixed
    /// occurrences, while a later Factory-backed call owns the cold script
    /// lifecycle.
    pub(super) scripted_data_context_bind_complete: bool,
    /// AF-8 facade adaptation: the public convenience advance accepts a root
    /// ViewModel every frame, while C++ `internalDataContext` is an explicit
    /// lifecycle boundary. Retain the exact last hydrated root identity so an
    /// A->A frame is not invented into a rebind and A->B still rehydrates
    /// every persistent scripted occurrence before same-frame advance.
    pub(super) scripted_facade_root_view_model: Option<RuntimeOwnedViewModelHandle>,
    /// The owning Artboard's file is stable for the lifetime of the concrete
    /// C++ StateMachineInstance. Retain that same authority so the public
    /// no-argument `updateDataBinds(true)` boundary can reconcile cloned
    /// ScriptInput targets without borrowing the Artboard again.
    scripted_listener_runtime_file: Option<Arc<RuntimeFile>>,
    scripted_listener_artboard_resolver: Option<Rc<dyn ScriptArtboardResolver>>,
    pub(super) script_error: Option<ScriptError>,
    view_model_listeners: Vec<RuntimeViewModelListenerInstance>,
    focus_listener_groups: Vec<focus_listener_group::RuntimeFocusListenerGroup>,
    keyboard_listener_groups: Vec<keyboard_listener_group::RuntimeKeyboardListenerGroup>,
    gamepad_listener_groups: Vec<gamepad_listener_group::RuntimeGamepadListenerGroup>,
    gamepad_scripted_drawables: Vec<gamepad_listener_group::RuntimeGamepadScriptedDrawable>,
    pub(super) embedder_gamepads: BTreeMap<i32, ScriptGamepadSnapshot>,
    scripted_input_group_generation: u64,
    semantic_listener_groups: Vec<semantic_listener_group::RuntimeSemanticListenerGroup>,
    queued_focus_events: Vec<RuntimeQueuedFocusEvent>,
    queued_semantic_events: Vec<RuntimeQueuedSemanticEvent>,
    #[cfg(test)]
    deferred_callback_probe: Option<RuntimeDeferredCallbackProbe>,
}

pub(super) struct RuntimeStateMachineListenerActionExecutor<'a> {
    needs_advance: &'a mut bool,
    pub(super) data_bind_graph: &'a mut RuntimeDataBindGraph,
    data_bind_facilities_ready: bool,
    owned_view_model_context: Option<&'a mut RuntimeOwnedViewModelInstance>,
    owned_data_context: Option<RuntimeOwnedDataContext>,
    file_data_context_instance: Option<RuntimeViewModelInstanceCells>,
    scripted_listener_action_instances: &'a BTreeMap<u32, RuntimeScriptInstanceHandle>,
    scripted_instances_by_global: &'a BTreeMap<u32, RuntimeScriptInstanceHandle>,
    focus: &'a mut RuntimeFocusTree,
    host: &'a mut dyn ScriptHost,
}
pub use state_machine_instance::{FocusState, RuntimeHitResult};
pub(crate) use state_machine_instance::{
    RuntimeDataContextBindError, RuntimeSemanticOccurrenceKey, RuntimeSemanticRoute,
    SemanticNodeResolver, closest_semantic_node,
};
#[cfg(any(test, feature = "tools"))]
pub use state_machine_instance::{
    RuntimeNestedEventChainPhase, RuntimeNestedEventChainStep, RuntimeNestedEventChainTrace,
    RuntimeNestedNotifyBatchEntry, RuntimeNestedNotifyBatchTrace,
};
