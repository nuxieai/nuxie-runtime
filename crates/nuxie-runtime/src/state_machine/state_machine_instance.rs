// Runtime instance orchestration for the C++ state machine path.
// Mirrors /Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp.
use super::focused_input_dispatch::RuntimeInputDispatchOutcome;
use super::listener_types::RuntimeListenerViewModelPath;
use super::*;
use crate::artboard_data_bind::RuntimeOwnedDataContext;
use crate::constraints::{
    RuntimeDraggableProxy, runtime_draggable_proxies, runtime_draggable_proxy_drag,
    runtime_draggable_proxy_end, runtime_draggable_proxy_hit_test, runtime_draggable_proxy_start,
};
use crate::data_bind_container::RuntimeDataBindContainerQueue;
use crate::data_bind_graph::data_bind_flags_apply_source_to_target;
use crate::focus::RuntimeFocusTree;
use crate::scripting::RuntimeScriptInstanceHandle;
use crate::view_model::{RuntimeFontAssetValue, RuntimeOwnedViewModelAdvanceContext};
use crate::view_model_cell::{
    RuntimeCellDirt, RuntimeCellDirtSink, RuntimeCellNotificationQueue,
    RuntimeFileViewModelInstanceCatalog, RuntimeViewModelCell, RuntimeViewModelCellValue,
    RuntimeViewModelInstanceCells,
};
use crate::{
    ArtboardInstance, NoopScriptHost, RuntimeDataBindGraph, RuntimeDataBindGraphApplyPhase,
    RuntimeDataBindGraphTargetsMut, RuntimeDataBindGraphValue,
    RuntimeDefaultViewModelArtboardSourceHandle, RuntimeDefaultViewModelAssetSourceHandle,
    RuntimeDefaultViewModelBooleanSourceHandle, RuntimeDefaultViewModelColorSourceHandle,
    RuntimeDefaultViewModelEnumSourceHandle, RuntimeDefaultViewModelListSourceHandle,
    RuntimeDefaultViewModelNumberSourceHandle, RuntimeDefaultViewModelStringSourceHandle,
    RuntimeDefaultViewModelSymbolListIndexSourceHandle, RuntimeDefaultViewModelTriggerSourceHandle,
    RuntimeDefaultViewModelViewModelSourceHandle, RuntimeImportedViewModelInstanceContext,
    RuntimeOwnedViewModelContext, RuntimeOwnedViewModelContextHandle, RuntimeOwnedViewModelHandle,
    RuntimeOwnedViewModelInstance, ScriptArtboardDataContext, ScriptArtboardParentContext,
    ScriptArtboardResolver, ScriptCoreString, ScriptError, ScriptHost, ScriptInstance,
    ScriptListenerActionDefinition, ScriptListenerInvocation, ScriptPointerEventKind, ScriptValue,
    ScriptViewModel, runtime_default_view_model_artboard_property_path_for_name,
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
use std::rc::Rc;

#[cfg(test)]
use crate::{ScriptListenerActionMethod, ScriptMethod};

#[derive(Debug)]
pub struct StateMachineInstance {
    state_machine_index: usize,
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
    requires_post_update_state_probe: bool,
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
    /// C++ `m_reportedListenerViewModels`: every retained listener-cell
    /// mutation appends its listener index, preserving duplicates and
    /// dependent-registration order until next-frame `applyEvents`.
    reported_listener_view_models: RuntimeCellNotificationQueue,
    /// Retained C++ `m_reportingListenerViewModels` batch buffer.
    reporting_listener_view_models: Vec<usize>,
    changed_state_count: usize,
    pub(super) needs_advance: bool,
    has_advanced_once: bool,
    // A mounted NestedStateMachine is initialized before its parent binding
    // settles. C++ gives that occurrence one outer-update probe after the
    // mounted artboard updates, even when its authored conditions are stable.
    post_update_probe_pending: bool,
    pub(super) data_bind_graph: RuntimeDataBindGraph,
    /// One C++ `DataBindContainer` queue shared by ordinary state-machine
    /// binds and every DataBind cloned with a scripted listener action.
    data_bind_container: RuntimeDataBindContainerQueue,
    data_bind_occurrences: Vec<RuntimeStateMachineDataBindOccurrence>,
    key_frame_data_bind_graphs: Vec<Option<RuntimeDataBindGraph>>,
    pub(super) owned_data_context: Option<RuntimeOwnedDataContext>,
    #[cfg(test)]
    owned_data_bind_context_bind_count: usize,
    /// C++ `ViewModelInstance::m_dependents` push channel: structural
    /// ViewModel replacement dirties this sink so the retained DataContext is
    /// relinked without polling a root mutation generation every frame.
    pub(super) owned_view_model_rebind_sink: RuntimeCellDirtSink,
    pointer_down_listener_hits: Vec<RuntimePointerDownListenerHit>,
    pointer_listener_states: Vec<RuntimePointerListenerState>,
    pointer_positions: Vec<RuntimePointerPosition>,
    /// Fresh component-provided listener/proxy owners constructed for this
    /// StateMachineInstance (`state_machine_instance.cpp:1969-2013`).
    draggable_proxies: Vec<RuntimeDraggableProxy>,
    focus: RuntimeFocusTree,
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
    scripted_input_group_generation: u64,
    semantic_listener_groups: Vec<semantic_listener_group::RuntimeSemanticListenerGroup>,
    queued_focus_events: Vec<ScriptListenerInvocation>,
    queued_semantic_events: Vec<ScriptListenerInvocation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeStateMachineDataBindOccurrence {
    Ordinary {
        data_bind_index: usize,
    },
    ScriptedObject {
        action_binding_index: usize,
        input_index: usize,
    },
}

#[derive(Debug, Clone)]
struct RuntimePointerDownListenerHit {
    pointer_id: i32,
    listener_index: usize,
    drag_phase: Option<RuntimePointerDragPhase>,
    event_context: Option<StateMachineEventContext>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimePointerDragPhase {
    Armed,
    Dragging,
}

#[derive(Debug, Clone)]
struct RuntimePointerListenerState {
    pointer_id: i32,
    listener_index: usize,
    is_hovered: bool,
    previous_x: f32,
    previous_y: f32,
}

#[derive(Debug, Clone, Copy)]
struct RuntimePointerInput {
    x: f32,
    y: f32,
    previous_x: f32,
    previous_y: f32,
    timestamp_seconds: f32,
    id: i32,
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

impl RuntimeStateMachineListenerActionExecutor<'_> {
    pub(super) fn perform_scheduled_view_model_change(
        &mut self,
        artboard: &mut ArtboardInstance,
        bindable_global_id: u32,
        value: &RuntimeListenerViewModelChangeValue,
        mut targets: RuntimeScheduledListenerActionTargetsMut<'_>,
    ) -> bool {
        if !self.data_bind_facilities_ready {
            return false;
        }
        let data_bind_index = self
            .data_bind_graph
            .bindable_data_bind_to_source_index(bindable_global_id);
        let artboard_value = match value {
            RuntimeListenerViewModelChangeValue::Number(value) => {
                RuntimeDataBindGraphValue::Number(*value)
            }
            RuntimeListenerViewModelChangeValue::Integer(value) => {
                RuntimeDataBindGraphValue::SymbolListIndex(*value)
            }
            RuntimeListenerViewModelChangeValue::Color(value) => {
                RuntimeDataBindGraphValue::Color(*value)
            }
            RuntimeListenerViewModelChangeValue::String(value) => {
                RuntimeDataBindGraphValue::String(value.clone())
            }
            RuntimeListenerViewModelChangeValue::Enum(value) => {
                RuntimeDataBindGraphValue::Enum(*value)
            }
            RuntimeListenerViewModelChangeValue::Asset(value) => {
                RuntimeDataBindGraphValue::Asset(value.data_bind_asset_index())
            }
            RuntimeListenerViewModelChangeValue::Artboard(value) => {
                RuntimeDataBindGraphValue::Artboard(*value)
            }
            RuntimeListenerViewModelChangeValue::Trigger(value) => {
                RuntimeDataBindGraphValue::Trigger(*value)
            }
            RuntimeListenerViewModelChangeValue::Boolean(value) => {
                RuntimeDataBindGraphValue::Boolean(*value)
            }
            RuntimeListenerViewModelChangeValue::List(value) => RuntimeDataBindGraphValue::List {
                item_count: usize::try_from(*value).unwrap_or(usize::MAX),
            },
            RuntimeListenerViewModelChangeValue::ViewModel(value) => {
                RuntimeDataBindGraphValue::ViewModel(*value)
            }
        };
        let path = data_bind_index.and_then(|data_bind_index| {
            self.data_bind_graph
                .source_path_for_data_bind(data_bind_index)
        });
        let source_changed = if let Some(data_bind_index) = data_bind_index
            && let Some(context) = self.owned_view_model_context.take()
        {
            let changed = self.perform_owned_view_model_change(
                &mut *context,
                data_bind_index,
                value,
                &mut targets,
            );
            self.owned_view_model_context = Some(context);
            changed
        } else if let Some(data_bind_index) = data_bind_index
            && self.owned_data_context.is_some()
        {
            self.perform_owned_data_context_change(data_bind_index, value, &mut targets)
        } else if let Some(data_bind_index) = data_bind_index {
            self.data_bind_graph
                .set_active_view_model_source_for_data_bind(data_bind_index, artboard_value.clone())
        } else {
            false
        };
        let target_dirtied = self
            .data_bind_graph
            .dirty_bindable_data_bind_to_target(bindable_global_id);
        if !source_changed && !target_dirtied {
            return false;
        }
        if source_changed && let Some(path) = path {
            artboard.set_artboard_data_bind_value_for_path(&path, artboard_value);
        }
        // Pinned `ListenerViewModelChange::perform` updates only the
        // target-to-source bind, then calls `addDirt(Bindings, true)` on the
        // paired source-to-target bind. It does not run `updateDataBinds`
        // inside the action FIFO (`listener_viewmodel_change.cpp:42-80`).
        // Keeping the target dirty until the normal data-bind boundary means
        // a later action in this same FIFO still observes its pre-batch
        // target value.
        true
    }

    fn perform_owned_data_context_change(
        &mut self,
        data_bind_index: usize,
        value: &RuntimeListenerViewModelChangeValue,
        targets: &mut RuntimeScheduledListenerActionTargetsMut<'_>,
    ) -> bool {
        let Some(source_path) = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index)
        else {
            return false;
        };
        let Some((context_handle, property_path)) = self
            .owned_data_context
            .as_ref()
            .and_then(|context| context.resolved_property_path(&source_path))
        else {
            return false;
        };

        if let RuntimeListenerViewModelChangeValue::Trigger(value) = value {
            let Some(bindable_trigger) = targets
                .bindable_triggers
                .iter_mut()
                .find(|trigger| trigger.has_data_bind_index(data_bind_index))
            else {
                return false;
            };
            bindable_trigger.set_value(*value);
            let mut context = context_handle.borrow_mut();
            if !self
                .data_bind_graph
                .fire_owned_view_model_context_trigger_source_for_data_bind_at_property_path(
                    &mut context,
                    data_bind_index,
                    *value,
                    &property_path,
                )
            {
                return false;
            }
            return true;
        }

        let asset_value = match value {
            RuntimeListenerViewModelChangeValue::Asset(fallback) => Some(
                targets
                    .bindable_assets
                    .iter()
                    .find(|asset| asset.has_data_bind_index(data_bind_index))
                    .map(|asset| asset.value.clone())
                    .unwrap_or_else(|| fallback.clone()),
            ),
            _ => None,
        };
        let graph_value = match value {
            RuntimeListenerViewModelChangeValue::Number(value) => {
                RuntimeDataBindGraphValue::Number(*value)
            }
            RuntimeListenerViewModelChangeValue::Integer(value) => {
                RuntimeDataBindGraphValue::SymbolListIndex(*value)
            }
            RuntimeListenerViewModelChangeValue::Color(value) => {
                RuntimeDataBindGraphValue::Color(*value)
            }
            RuntimeListenerViewModelChangeValue::String(value) => {
                RuntimeDataBindGraphValue::String(value.clone())
            }
            RuntimeListenerViewModelChangeValue::Enum(value) => {
                RuntimeDataBindGraphValue::Enum(*value)
            }
            RuntimeListenerViewModelChangeValue::Asset(_) => RuntimeDataBindGraphValue::Asset(
                asset_value
                    .as_ref()
                    .map(RuntimeBindableAssetValue::data_bind_asset_index)
                    .unwrap_or_default(),
            ),
            RuntimeListenerViewModelChangeValue::Artboard(value) => {
                RuntimeDataBindGraphValue::Artboard(*value)
            }
            RuntimeListenerViewModelChangeValue::Boolean(value) => {
                RuntimeDataBindGraphValue::Boolean(*value)
            }
            RuntimeListenerViewModelChangeValue::List(value) => RuntimeDataBindGraphValue::List {
                item_count: usize::try_from(*value).unwrap_or(usize::MAX),
            },
            RuntimeListenerViewModelChangeValue::ViewModel(value) => {
                RuntimeDataBindGraphValue::ViewModel(*value)
            }
            RuntimeListenerViewModelChangeValue::Trigger(_) => unreachable!(),
        };
        let mut context = context_handle.borrow_mut();
        let Some(context_changed) =
            StateMachineInstance::apply_listener_view_model_change_at_property_path(
                &mut context,
                &property_path,
                value,
                asset_value.as_ref(),
            )
        else {
            return false;
        };
        let graph_changed = self
            .data_bind_graph
            .set_active_view_model_source_for_data_bind(data_bind_index, graph_value);
        if matches!(value, RuntimeListenerViewModelChangeValue::Number(_)) {
            // The listener wrote the retained cell above. Its owning binds,
            // including converter-operand dependencies, are already dirty;
            // fold that pushed dirt before this frame's data-bind pass.
            self.data_bind_graph.collect_retained_source_dirt();
        }
        context_changed || graph_changed
    }

    fn perform_owned_view_model_change(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: &RuntimeListenerViewModelChangeValue,
        targets: &mut RuntimeScheduledListenerActionTargetsMut<'_>,
    ) -> bool {
        match value {
            RuntimeListenerViewModelChangeValue::Number(value) => self
                .data_bind_graph
                .set_owned_view_model_context_number_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
            RuntimeListenerViewModelChangeValue::Integer(value) => self
                .data_bind_graph
                .set_owned_view_model_context_symbol_list_index_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
            RuntimeListenerViewModelChangeValue::Color(value) => self
                .data_bind_graph
                .set_owned_view_model_context_color_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
            RuntimeListenerViewModelChangeValue::String(value) => self
                .data_bind_graph
                .set_owned_view_model_context_string_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                ),
            RuntimeListenerViewModelChangeValue::Enum(value) => self
                .data_bind_graph
                .set_owned_view_model_context_enum_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
            RuntimeListenerViewModelChangeValue::Asset(value) => self
                .data_bind_graph
                .set_owned_view_model_context_asset_source_for_data_bind(
                    context,
                    data_bind_index,
                    value.data_bind_asset_index(),
                ),
            RuntimeListenerViewModelChangeValue::Artboard(value) => self
                .data_bind_graph
                .set_owned_view_model_context_artboard_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
            RuntimeListenerViewModelChangeValue::Boolean(value) => self
                .data_bind_graph
                .set_owned_view_model_context_boolean_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
            RuntimeListenerViewModelChangeValue::Trigger(value) => {
                let Some(bindable_trigger) = targets
                    .bindable_triggers
                    .iter_mut()
                    .find(|trigger| trigger.has_data_bind_index(data_bind_index))
                else {
                    return false;
                };
                bindable_trigger.set_value(*value);
                if !self
                    .data_bind_graph
                    .fire_owned_view_model_context_trigger_source_for_data_bind(
                        context,
                        data_bind_index,
                        *value,
                    )
                {
                    return false;
                }
                true
            }
            RuntimeListenerViewModelChangeValue::List(value) => self
                .data_bind_graph
                .set_active_view_model_source_for_data_bind(
                    data_bind_index,
                    RuntimeDataBindGraphValue::List {
                        item_count: usize::try_from(*value).unwrap_or(usize::MAX),
                    },
                ),
            RuntimeListenerViewModelChangeValue::ViewModel(value) => self
                .data_bind_graph
                .set_active_view_model_source_for_data_bind(
                    data_bind_index,
                    RuntimeDataBindGraphValue::ViewModel(*value),
                ),
        }
    }
}

impl RuntimeScheduledListenerActionExecutor for RuntimeStateMachineListenerActionExecutor<'_> {
    fn mark_direct_input_changed(&mut self) {
        *self.needs_advance = true;
    }

    fn target_has_focus(&self, target_local_id: usize) -> bool {
        self.focus.target_has_focus(target_local_id)
    }

    fn evaluate_scripted_condition(&self, global_id: u32) -> bool {
        super::scripted_transition_condition::evaluate_scripted_condition(
            global_id,
            self.scripted_instances_by_global,
        )
    }

    fn retained_view_model_source(&self, bindable_global_id: u32) -> Option<RuntimeViewModelCell> {
        if !self.data_bind_facilities_ready {
            return None;
        }
        self.data_bind_graph
            .retained_source_for_bindable_target(bindable_global_id)
    }

    fn fire_view_model_trigger(&mut self, path: &RuntimeStateMachineFireTriggerPath) -> bool {
        path.perform(
            self.data_bind_facilities_ready,
            self.owned_data_context.as_ref(),
            self.file_data_context_instance.as_ref(),
            self.data_bind_graph,
        )
    }

    fn perform_instance_action(
        &mut self,
        artboard: &mut ArtboardInstance,
        action: &RuntimeScheduledListenerAction,
        targets: RuntimeScheduledListenerActionTargetsMut<'_>,
    ) -> Result<bool, ScriptError> {
        match action {
            RuntimeScheduledListenerAction::ViewModelChange(action) => {
                Ok(action.perform(self, artboard, targets))
            }
            RuntimeScheduledListenerAction::Scripted {
                definition: Some(definition),
                ..
            } => super::scripted_listener_action::perform_scripted_listener_action(
                self.scripted_listener_action_instances,
                definition,
                &ScriptListenerInvocation::None,
                self.host,
            ),
            RuntimeScheduledListenerAction::Scripted {
                definition: None, ..
            }
            | RuntimeScheduledListenerAction::Noop { .. } => Ok(false),
            RuntimeScheduledListenerAction::AlignTarget(action) => {
                Ok(action.perform(artboard, &ScriptListenerInvocation::None))
            }
            RuntimeScheduledListenerAction::FocusTarget(action) => {
                Ok(action.perform(artboard, self.focus))
            }
            RuntimeScheduledListenerAction::FocusClear(action) => Ok(action.perform(self.focus)),
            RuntimeScheduledListenerAction::FocusTraversal(action) => {
                Ok(action.perform(artboard, self.focus))
            }
            RuntimeScheduledListenerAction::FireEvent(_)
            | RuntimeScheduledListenerAction::BoolChange(_)
            | RuntimeScheduledListenerAction::NumberChange(_)
            | RuntimeScheduledListenerAction::TriggerChange(_) => Err(ScriptError::new(
                "ordinary listener action reached the instance-owned executor",
            )),
        }
    }
}

impl Clone for StateMachineInstance {
    fn clone(&self) -> Self {
        let active_file_view_model_binding = self.active_file_view_model_binding;
        let file_view_model_instances = self
            .file_view_model_instances
            .as_ref()
            .map(RuntimeFileViewModelInstanceCatalog::detached_clone);
        let default_view_model_trigger_instance = self
            .default_view_model_index
            .and_then(|index| file_view_model_instances.as_ref()?.instance(index, 0));
        let reported_listener_view_models = self.reported_listener_view_models.detached_clone();
        let view_model_listeners = self
            .view_model_listeners
            .iter()
            .enumerate()
            .map(|(listener_index, listener)| {
                listener.clone_for_queue(&reported_listener_view_models, listener_index)
            })
            .collect();
        let mut cloned = Self {
            state_machine_index: self.state_machine_index,
            state_machine_definitions: self.state_machine_definitions.as_ref().map(Arc::clone),
            listener_definitions: Arc::clone(&self.listener_definitions),
            default_view_model_index: self.default_view_model_index,
            file_view_model_instances,
            default_view_model_trigger_instance,
            active_file_view_model_binding,
            active_owned_view_model_advance_context: self
                .active_owned_view_model_advance_context
                .clone(),
            requires_post_update_state_probe: self.requires_post_update_state_probe,
            inputs: self.inputs.clone(),
            bindable_numbers: self.bindable_numbers.clone(),
            bindable_integers: self.bindable_integers.clone(),
            bindable_colors: self.bindable_colors.clone(),
            bindable_strings: self.bindable_strings.clone(),
            bindable_enums: self.bindable_enums.clone(),
            bindable_assets: self.bindable_assets.clone(),
            bindable_artboards: self.bindable_artboards.clone(),
            bindable_lists: self.bindable_lists.clone(),
            bindable_triggers: self.bindable_triggers.clone(),
            bindable_view_models: self.bindable_view_models.clone(),
            bindable_booleans: self.bindable_booleans.clone(),
            default_view_model_triggers: Arc::clone(&self.default_view_model_triggers),
            transition_durations: self.transition_durations.clone(),
            layers: self.layers.clone(),
            // Public Clone is Rust's explicit state-snapshot adaptation (C++
            // has no StateMachineInstance copy constructor). Copy report
            // values and cursors without aliasing their Vec storage; a fresh
            // C++-shaped occurrence is created through `new`, not `clone`.
            reported_events: self.reported_events.clone(),
            reported_event_listener_index: self.reported_event_listener_index,
            host_reported_event_index: self.host_reported_event_index,
            reporting_events: self.reporting_events.clone(),
            reported_listener_view_models,
            reporting_listener_view_models: Vec::new(),
            changed_state_count: self.changed_state_count,
            needs_advance: self.needs_advance,
            has_advanced_once: self.has_advanced_once,
            post_update_probe_pending: self.post_update_probe_pending,
            data_bind_graph: self.data_bind_graph.clone_for_state_machine_snapshot(),
            data_bind_container: RuntimeDataBindContainerQueue::default(),
            data_bind_occurrences: Vec::new(),
            key_frame_data_bind_graphs: self.key_frame_data_bind_graphs.clone(),
            owned_data_context: self.owned_data_context.clone(),
            #[cfg(test)]
            owned_data_bind_context_bind_count: self.owned_data_bind_context_bind_count,
            owned_view_model_rebind_sink: RuntimeCellDirtSink::new(),
            pointer_down_listener_hits: self.pointer_down_listener_hits.clone(),
            pointer_listener_states: self.pointer_listener_states.clone(),
            pointer_positions: self.pointer_positions.clone(),
            draggable_proxies: self
                .draggable_proxies
                .iter()
                .map(RuntimeDraggableProxy::clone_cold)
                .collect(),
            scripted_instances_by_global: BTreeMap::new(),
            focus: self.focus.clone(),
            scripted_object_definitions: self.scripted_object_definitions.clone(),
            scripted_listener_action_definitions: self.scripted_listener_action_definitions.clone(),
            scripted_object_bindings: self
                .scripted_object_bindings
                .iter()
                .map(
                    super::scripted_listener_action::RuntimeScriptedListenerActionBindingOccurrence::fresh_clone,
                )
                .collect(),
            // The Rust snapshot does not alias stateful script tables. VM
            // table handles carry mutable per-occurrence state and must be
            // regenerated by the facade before script execution.
            scripted_listener_action_instances: BTreeMap::new(),
            scripted_object_initialization_complete: false,
            // Public Clone is a Rust snapshot adaptation. Its script tables
            // are deliberately regenerated, so a retained cloned DataContext
            // is already present at that new cold occurrence boundary.
            scripted_constructor_context_was_prebound: self.owned_data_context.is_some(),
            scripted_data_context_bind_complete: false,
            scripted_facade_root_view_model: None,
            scripted_listener_runtime_file: self.scripted_listener_runtime_file.clone(),
            scripted_listener_artboard_resolver: self.scripted_listener_artboard_resolver.clone(),
            script_error: None,
            view_model_listeners,
            focus_listener_groups: self.focus_listener_groups.clone(),
            keyboard_listener_groups: self.keyboard_listener_groups.clone(),
            gamepad_listener_groups: self.gamepad_listener_groups.clone(),
            gamepad_scripted_drawables: self.gamepad_scripted_drawables.clone(),
            scripted_input_group_generation: self.scripted_input_group_generation,
            semantic_listener_groups: self.semantic_listener_groups.clone(),
            // Snapshot pending callback values without aliasing their queues.
            // `StateMachineInstance::new` remains the cold-remount boundary.
            queued_focus_events: self.queued_focus_events.clone(),
            queued_semantic_events: self.queued_semantic_events.clone(),
        };
        for layer in &mut cloned.layers {
            layer.refresh_view_model_trigger_layer_id();
        }
        cloned.rebind_file_trigger_cells_after_clone(active_file_view_model_binding);
        cloned.register_owned_view_model_rebind_dependents();
        cloned.initialize_data_bind_container();
        cloned
    }
}

fn script_pointer_invocation(
    pointer: RuntimePointerInput,
    listener_type: RuntimeListenerType,
) -> ScriptListenerInvocation {
    let event = match listener_type {
        RuntimeListenerType::Enter => ScriptPointerEventKind::Enter,
        RuntimeListenerType::Exit => ScriptPointerEventKind::Exit,
        RuntimeListenerType::Down => ScriptPointerEventKind::Down,
        RuntimeListenerType::Up => ScriptPointerEventKind::Up,
        RuntimeListenerType::Move => ScriptPointerEventKind::Move,
        RuntimeListenerType::Click => ScriptPointerEventKind::Click,
        RuntimeListenerType::DragStart => ScriptPointerEventKind::DragStart,
        RuntimeListenerType::DragEnd => ScriptPointerEventKind::DragEnd,
        RuntimeListenerType::Drag => ScriptPointerEventKind::Drag,
        RuntimeListenerType::Event
        | RuntimeListenerType::ComponentProvided
        | RuntimeListenerType::TextInput
        | RuntimeListenerType::ViewModel
        | RuntimeListenerType::Focus
        | RuntimeListenerType::Blur
        | RuntimeListenerType::Keyboard
        | RuntimeListenerType::SemanticAction
        | RuntimeListenerType::Gamepad => return ScriptListenerInvocation::None,
    };
    ScriptListenerInvocation::Pointer {
        x: pointer.x,
        y: pointer.y,
        previous_x: pointer.previous_x,
        previous_y: pointer.previous_y,
        pointer_id: pointer.id,
        event,
        timestamp_seconds: pointer.timestamp_seconds,
    }
}

fn listener_target_direct_child(
    artboard: &ArtboardInstance,
    target_local_id: usize,
    child_type: &str,
) -> Option<usize> {
    let target = artboard.component_handle(target_local_id)?;
    if !nuxie_schema::definition_by_name(artboard.component_at(target).type_name)
        .is_some_and(|definition| definition.is_a("Node"))
    {
        return None;
    }
    (0..artboard.component_child_len(target)).find_map(|index| {
        let child = artboard.component_child_at(target, index)?;
        let child = artboard.component_at(child);
        (child.type_name == child_type).then_some(child.local_id)
    })
}

fn listener_uses_report_queue(listener: &RuntimeStateMachineListener) -> bool {
    listener_types_use_report_queue(&listener.listener_types)
}

fn listener_types_use_report_queue(listener_types: &[RuntimeListenerType]) -> bool {
    listener_types.iter().any(|kind| {
        matches!(
            kind,
            RuntimeListenerType::Event | RuntimeListenerType::ViewModel
        )
    })
}

fn validate_pointer_timestamp(timestamp_seconds: f32) -> Result<(), ScriptError> {
    if timestamp_seconds.is_finite() && timestamp_seconds >= 0.0 {
        Ok(())
    } else {
        Err(ScriptError::new(
            "pointer timestamp must be finite and nonnegative",
        ))
    }
}

#[derive(Debug, Clone, Copy)]
struct RuntimePointerPosition {
    pointer_id: i32,
    x: f32,
    y: f32,
}

#[derive(Debug)]
struct RuntimeViewModelListenerInstance {
    /// Stable authored listener-definition arena plus index, matching C++'s
    /// retained `const StateMachineListener*`.
    listener_definitions: Arc<Vec<RuntimeStateMachineListener>>,
    listener_index: usize,
    property_bindings: Vec<RuntimeViewModelListenerPropertyBinding>,
}

#[derive(Debug, Clone, Copy)]
enum RuntimeViewModelListenerSource {
    Single,
    Input(usize),
}

#[derive(Debug)]
struct RuntimeViewModelListenerPropertyBinding {
    source: RuntimeViewModelListenerSource,
    /// The retained scalar cell this listener's condition currently reads,
    /// with this listener's dirt sink
    /// registered as a dependent (C++ `ListenerViewModelPropertyBinding`,
    /// src/animation/state_machine_instance.cpp:1331-1407 at pin d788e8ec).
    /// `None` for list/view-model conditions and unresolved paths.
    cell_binding: Option<RuntimeViewModelListenerCellBinding>,
}

impl RuntimeViewModelListenerInstance {
    fn new(
        listener_definitions: Arc<Vec<RuntimeStateMachineListener>>,
        listener_index: usize,
    ) -> Option<Self> {
        let listener = listener_definitions.get(listener_index)?;
        if !listener.has_listener(RuntimeListenerType::ViewModel) {
            return None;
        }
        let property_bindings = if listener.view_model_path.is_some() {
            vec![RuntimeViewModelListenerPropertyBinding {
                source: RuntimeViewModelListenerSource::Single,
                cell_binding: None,
            }]
        } else {
            (0..listener.view_model_input_types.len())
                .map(|input_index| RuntimeViewModelListenerPropertyBinding {
                    source: RuntimeViewModelListenerSource::Input(input_index),
                    cell_binding: None,
                })
                .collect()
        };
        Some(Self {
            listener_definitions,
            listener_index,
            property_bindings,
        })
    }

    fn listener(&self) -> &RuntimeStateMachineListener {
        &self.listener_definitions[self.listener_index]
    }

    fn actions(&self) -> &[RuntimeScheduledListenerAction] {
        &self.listener().listener_actions
    }

    /// A cloned machine re-registers a FRESH reporting sink on the same
    /// retained cell and the clone's own queue. Pending reports stay with the
    /// original machine, matching C++ instance-local listener vectors.
    fn clone_for_queue(&self, queue: &RuntimeCellNotificationQueue, listener_index: usize) -> Self {
        Self {
            listener_definitions: Arc::clone(&self.listener_definitions),
            listener_index: self.listener_index,
            property_bindings: self
                .property_bindings
                .iter()
                .map(|binding| RuntimeViewModelListenerPropertyBinding {
                    source: binding.source,
                    cell_binding: binding.cell_binding.as_ref().map(|cell_binding| {
                        RuntimeViewModelListenerCellBinding::new(
                            cell_binding.cell.clone(),
                            queue,
                            listener_index,
                        )
                    }),
                })
                .collect(),
        }
    }
}

/// One listener condition's dependent registration on its retained cell
/// (C++ `ListenerViewModelPropertyBindingListener`). Dropping the binding
/// unregisters implicitly: the cell only holds the sink weakly.
struct RuntimeViewModelListenerCellBinding {
    cell: RuntimeViewModelCell,
    _sink: RuntimeCellDirtSink,
}

impl RuntimeViewModelListenerCellBinding {
    fn new(
        cell: RuntimeViewModelCell,
        queue: &RuntimeCellNotificationQueue,
        listener_index: usize,
    ) -> Self {
        let sink = RuntimeCellDirtSink::reporting_listener(queue, listener_index);
        cell.add_dependent(&sink);
        Self { cell, _sink: sink }
    }
}

impl std::fmt::Debug for RuntimeViewModelListenerCellBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeViewModelListenerCellBinding")
            .field("cell", &self.cell)
            .finish_non_exhaustive()
    }
}

fn relink_view_model_listener_cell(
    binding: &mut RuntimeViewModelListenerPropertyBinding,
    cell: Option<RuntimeViewModelCell>,
    queue: &RuntimeCellNotificationQueue,
    listener_index: usize,
) {
    let same_cell = binding
        .cell_binding
        .as_ref()
        .zip(cell.as_ref())
        .is_some_and(|(binding, cell)| binding.cell.ptr_eq(cell));
    if same_cell {
        return;
    }
    binding.cell_binding =
        cell.map(|cell| RuntimeViewModelListenerCellBinding::new(cell, queue, listener_index));
}

fn apply_scripted_input_update(
    instance: &RuntimeScriptInstanceHandle,
    input_name: &ScriptCoreString,
    value: super::scripted_listener_action::RuntimeScriptedListenerBoundValue,
    artboard_resolver: Option<&dyn ScriptArtboardResolver>,
    artboard_parent_context: Option<&ScriptArtboardParentContext>,
    host: &mut dyn ScriptHost,
) -> Result<bool, ScriptError> {
    // A ScriptedObject whose `init` returned false/errored has already
    // released both `m_vm` and `m_self`. Every C++ ScriptInput setter is then
    // inert because `state() == nullptr`. A table that is merely waiting for
    // DataContext hydration is still live and must continue accepting input
    // updates (`scripted_object.cpp:43-175,277-303`).
    if !instance.borrow_mut().script_lifetime_valid() {
        return Ok(false);
    }
    let result = match value {
        super::scripted_listener_action::RuntimeScriptedListenerBoundValue::Value(value) => {
            instance.borrow_mut().set_input_core(input_name, value)
        }
        super::scripted_listener_action::RuntimeScriptedListenerBoundValue::Trigger(value) => {
            if value == 0 {
                return Ok(false);
            }
            instance
                .borrow_mut()
                .call_input_trigger_core(input_name, host)
        }
        super::scripted_listener_action::RuntimeScriptedListenerBoundValue::Artboard(
            artboard_id,
        ) => {
            let Some(artboard_resolver) = artboard_resolver else {
                return Ok(false);
            };
            let artboard = match artboard_resolver
                .resolve_script_artboard(artboard_id, artboard_parent_context)
            {
                Ok(artboard) => artboard,
                Err(error) if error.resource_code().is_some() => return Err(error),
                Err(_) => return Ok(false),
            };
            instance
                .borrow_mut()
                .set_artboard_input_core(input_name, artboard)
        }
    };
    match result {
        Ok(()) => Ok(true),
        Err(error) if error.resource_code().is_some() => Err(error),
        // Pinned C++ treats an ordinary ScriptInput projection failure as an
        // inert occurrence and continues updating later authored inputs.
        Err(_) => Ok(false),
    }
}

impl StateMachineInstance {
    #[cfg(test)]
    pub(crate) fn reset_layer_construction_number_snapshots() {
        super::state_machine_layer_instance::reset_layer_construction_number_snapshots();
    }

    #[cfg(test)]
    pub(crate) fn layer_construction_number_snapshots() -> Vec<Vec<Option<f32>>> {
        super::state_machine_layer_instance::layer_construction_number_snapshots()
    }

    pub(crate) fn install_external_focus(
        &mut self,
        parent_focus: &RuntimeFocusTree,
        owner_identity: u64,
    ) {
        self.focus = parent_focus.external_for_owner(owner_identity);
    }

    #[cfg(test)]
    pub(crate) fn set_focus_target_for_test(&mut self, target_local: usize) -> bool {
        self.focus.set_focus_target(target_local)
    }

    #[cfg(test)]
    pub(crate) fn sync_focus_for_test(&mut self, artboard: &ArtboardInstance) {
        self.focus.sync(artboard);
    }

    fn rebind_file_trigger_cells_after_clone(
        &mut self,
        active_file_view_model_binding: Option<(usize, usize)>,
    ) {
        let Some((view_model_index, instance_index)) = active_file_view_model_binding else {
            return;
        };
        let Some(instance) = self
            .file_view_model_instances
            .as_ref()
            .and_then(|catalog| catalog.instance(view_model_index, instance_index))
        else {
            return;
        };
        self.data_bind_graph
            .bind_file_view_model_trigger_sources(&instance);
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.bind_file_view_model_trigger_sources(&instance);
        }
    }

    pub(crate) fn new(
        state_machine_index: usize,
        state_machine: &RuntimeStateMachine,
        artboard: &mut ArtboardInstance,
    ) -> Self {
        let state_machine_definitions = artboard
            .state_machines
            .get(state_machine_index)
            .filter(|definition| std::ptr::eq(*definition, state_machine))
            .map(|_| Arc::clone(&artboard.state_machines));
        let inputs = (0..state_machine.inputs.len())
            .map(|index| StateMachineInputInstance::new(index, Arc::clone(&state_machine.inputs)))
            .collect::<Vec<_>>();
        let bindable_numbers = state_machine
            .bindable_numbers
            .iter()
            .map(StateMachineBindableNumberInstance::new)
            .collect::<Vec<_>>();
        let bindable_integers = state_machine
            .bindable_integers
            .iter()
            .map(StateMachineBindableIntegerInstance::new)
            .collect::<Vec<_>>();
        let bindable_colors = state_machine
            .bindable_colors
            .iter()
            .map(StateMachineBindableColorInstance::new)
            .collect::<Vec<_>>();
        let bindable_strings = state_machine
            .bindable_strings
            .iter()
            .map(StateMachineBindableStringInstance::new)
            .collect::<Vec<_>>();
        let bindable_enums = state_machine
            .bindable_enums
            .iter()
            .map(StateMachineBindableEnumInstance::new)
            .collect::<Vec<_>>();
        let bindable_assets = state_machine
            .bindable_assets
            .iter()
            .map(StateMachineBindableAssetInstance::new)
            .collect::<Vec<_>>();
        let bindable_artboards = state_machine
            .bindable_artboards
            .iter()
            .map(StateMachineBindableArtboardInstance::new)
            .collect::<Vec<_>>();
        let bindable_lists = state_machine
            .bindable_lists
            .iter()
            .map(StateMachineBindableListInstance::new)
            .collect::<Vec<_>>();
        let bindable_triggers = state_machine
            .bindable_triggers
            .iter()
            .map(StateMachineBindableTriggerInstance::new)
            .collect::<Vec<_>>();
        let bindable_view_models = state_machine
            .bindable_view_models
            .iter()
            .map(StateMachineBindableViewModelInstance::new)
            .collect::<Vec<_>>();
        let bindable_booleans = state_machine
            .bindable_booleans
            .iter()
            .map(StateMachineBindableBooleanInstance::new)
            .collect::<Vec<_>>();
        let file_view_model_instances = artboard.runtime_file_view_model_instances();
        let default_view_model_trigger_instance =
            state_machine
                .default_view_model_index
                .and_then(|view_model_index| {
                    file_view_model_instances
                        .as_ref()?
                        .instance(view_model_index, 0)
                });
        let default_view_model_triggers = Arc::clone(&state_machine.view_model_triggers);
        let transition_durations = state_machine
            .transition_duration_bindings
            .iter()
            .map(StateMachineTransitionDurationInstance::new)
            .collect::<Vec<_>>();
        // Pinned C++ clones every StateMachine DataBind converter occurrence
        // and lets that occurrence create/own its script table when
        // `bindFromContext` runs. An Artboard-global table is definition
        // identity, not occurrence identity, and would alias repeated uses of
        // one ScriptedDataConverter
        // (`state_machine_instance.cpp:1754-1772`;
        // `scripted_data_converter.cpp:170-188,235-273`).
        let data_bind_graph = RuntimeDataBindGraph::new(state_machine);
        let mut key_frame_data_bind_graphs = artboard
            .linear_animations
            .iter()
            .map(|animation| {
                RuntimeDataBindGraph::new_key_frame_bindings(
                    &animation.key_frame_data_bind_templates,
                )
                .map(|mut graph| {
                    graph.attach_scripted_instances(
                        &artboard.scripted_data_converter_instances_by_global,
                    );
                    graph
                })
            })
            .collect::<Vec<_>>();
        if key_frame_data_bind_graphs.iter().all(Option::is_none) {
            key_frame_data_bind_graphs.clear();
        }
        let layer_capacity = state_machine.layers.len();
        let listener_definitions = Arc::clone(&state_machine.listeners);
        let view_model_listeners = (0..listener_definitions.len())
            .filter_map(|listener_index| {
                let listener = listener_definitions.get(listener_index)?;
                if listener.has_listener(RuntimeListenerType::Event) {
                    return None;
                }
                RuntimeViewModelListenerInstance::new(
                    Arc::clone(&listener_definitions),
                    listener_index,
                )
            })
            .collect();
        // Pinned C++ retains the FocusManager identity during layer entry
        // callbacks but does not build the complete artboard focus topology
        // until every layer has initialized
        // (`state_machine_instance.cpp:1747-1752,2123-2127`).
        let focus = RuntimeFocusTree::new_unsynchronized(artboard);
        let scripted_object_bindings = state_machine
            .scripted_object_bindings
            .iter()
            .map(|binding| binding.instantiate())
            .collect::<Vec<_>>();
        let mut instance = Self {
            state_machine_index,
            state_machine_definitions,
            listener_definitions,
            default_view_model_index: state_machine.default_view_model_index,
            file_view_model_instances,
            default_view_model_trigger_instance,
            active_file_view_model_binding: None,
            active_owned_view_model_advance_context: None,
            requires_post_update_state_probe: state_machine.requires_post_update_state_probe(),
            inputs,
            bindable_numbers,
            bindable_integers,
            bindable_colors,
            bindable_strings,
            bindable_enums,
            bindable_assets,
            bindable_artboards,
            bindable_lists,
            bindable_triggers,
            bindable_view_models,
            bindable_booleans,
            default_view_model_triggers,
            transition_durations,
            layers: Vec::with_capacity(layer_capacity),
            reported_events: Vec::new(),
            reported_event_listener_index: 0,
            host_reported_event_index: 0,
            reporting_events: Vec::new(),
            reported_listener_view_models: RuntimeCellNotificationQueue::default(),
            reporting_listener_view_models: Vec::new(),
            changed_state_count: 0,
            needs_advance: false,
            has_advanced_once: false,
            post_update_probe_pending: false,
            data_bind_graph,
            data_bind_container: RuntimeDataBindContainerQueue::default(),
            data_bind_occurrences: Vec::new(),
            key_frame_data_bind_graphs,
            owned_data_context: None,
            #[cfg(test)]
            owned_data_bind_context_bind_count: 0,
            owned_view_model_rebind_sink: RuntimeCellDirtSink::new(),
            pointer_down_listener_hits: Vec::new(),
            pointer_listener_states: Vec::new(),
            pointer_positions: Vec::new(),
            draggable_proxies: runtime_draggable_proxies(artboard),
            scripted_instances_by_global: BTreeMap::new(),
            focus,
            scripted_object_definitions: state_machine.scripted_objects.clone(),
            scripted_listener_action_definitions: state_machine.scripted_listener_actions.clone(),
            scripted_object_bindings,
            scripted_listener_action_instances: BTreeMap::new(),
            scripted_object_initialization_complete: false,
            scripted_constructor_context_was_prebound: false,
            scripted_data_context_bind_complete: false,
            scripted_facade_root_view_model: None,
            scripted_listener_runtime_file: artboard.runtime_file_arc(),
            scripted_listener_artboard_resolver: None,
            script_error: None,
            view_model_listeners,
            focus_listener_groups: Vec::new(),
            keyboard_listener_groups: Vec::new(),
            gamepad_listener_groups: Vec::new(),
            gamepad_scripted_drawables: Vec::new(),
            scripted_input_group_generation: 0,
            semantic_listener_groups: Vec::new(),
            queued_focus_events: Vec::new(),
            queued_semantic_events: Vec::new(),
        };
        instance.initialize_layers_in_authored_order(artboard, state_machine);
        // Ordinary StateMachine binds are added immediately after layers.
        // Scripted-object binds join only once the listener/hit/TextInput
        // facilities exist, matching cloneScriptedObject's later C++ point.
        instance.initialize_ordinary_data_bind_container();
        // Entry focus actions ran before C++ constructs listener groups. Do
        // not replay their manager callbacks into groups registered below.
        instance.focus.discard_unregistered_events();
        instance.initialize_listener_groups(artboard);
        instance.append_scripted_data_binds_to_container();
        instance.scripted_input_group_generation = artboard.script_attachment_generation();
        instance
            .focus
            .synchronize_after_layer_initialization(artboard);
        // `Artboard::buildFocusTree` installs the parent's manager while it
        // visits nested artboards. Rust's retained projection is built above,
        // then the nested state-machine occurrences are pointed at that same
        // domain without copying manager state.
        artboard.install_external_focus_domain(&instance.focus);
        instance
    }

    fn initialize_ordinary_data_bind_container(&mut self) {
        self.data_bind_container = RuntimeDataBindContainerQueue::default();
        self.data_bind_occurrences.clear();

        for (occurrence, data_bind_index) in self
            .data_bind_graph
            .add_data_binds_to_container(&mut self.data_bind_container)
        {
            debug_assert_eq!(occurrence, self.data_bind_occurrences.len());
            self.data_bind_occurrences
                .push(RuntimeStateMachineDataBindOccurrence::Ordinary { data_bind_index });
        }
    }

    fn initialize_data_bind_container(&mut self) {
        self.initialize_ordinary_data_bind_container();
        self.append_scripted_data_binds_to_container();
    }

    /// cloneScriptedObject appends its binds after the ordinary StateMachine
    /// binds; it does not rebuild/re-home the ordinary container prefix.
    fn append_scripted_data_binds_to_container(&mut self) {
        for (action_binding_index, binding) in self.scripted_object_bindings.iter_mut().enumerate()
        {
            for (occurrence, input_index) in
                binding.add_data_binds_to_container(&mut self.data_bind_container)
            {
                debug_assert_eq!(occurrence, self.data_bind_occurrences.len());
                self.data_bind_occurrences.push(
                    RuntimeStateMachineDataBindOccurrence::ScriptedObject {
                        action_binding_index,
                        input_index,
                    },
                );
            }
        }
    }

    fn initialize_listener_groups(&mut self, artboard: &ArtboardInstance) {
        for (listener_index, listener) in self.listener_definitions.iter().enumerate() {
            // Pinned C++ gives reported-event and ViewModel listeners their
            // own constructor paths and immediately continues the listener
            // loop. Even a malformed mixed listener therefore does not also
            // register focus, keyboard, gamepad, semantic, or pointer groups
            // (`state_machine_instance.cpp:1829-1842`).
            if listener_uses_report_queue(listener) {
                continue;
            }
            let Some(focus_data_local_id) =
                listener_target_direct_child(artboard, listener.target_local_id, "FocusData")
            else {
                continue;
            };
            if let Some(group) = focus_listener_group::RuntimeFocusListenerGroup::new(
                listener_index,
                focus_data_local_id,
                listener,
            ) {
                self.focus_listener_groups.push(group);
            }
            if let Some(group) = keyboard_listener_group::RuntimeKeyboardListenerGroup::new(
                listener_index,
                focus_data_local_id,
                listener,
            ) {
                self.keyboard_listener_groups.push(group);
            }
            if let Some(group) = gamepad_listener_group::RuntimeGamepadListenerGroup::new(
                listener_index,
                focus_data_local_id,
                listener,
            ) {
                self.gamepad_listener_groups.push(group);
            }
        }
        for (listener_index, listener) in self.listener_definitions.iter().enumerate() {
            if listener_uses_report_queue(listener) {
                continue;
            }
            let Some(semantic_data_local_id) =
                listener_target_direct_child(artboard, listener.target_local_id, "SemanticData")
            else {
                continue;
            };
            if let Some(group) = semantic_listener_group::RuntimeSemanticListenerGroup::new(
                listener_index,
                semantic_data_local_id,
                listener,
            ) {
                self.semantic_listener_groups.push(group);
            }
        }

        self.initialize_scripted_input_groups(artboard);
    }

    fn initialize_scripted_input_groups(&mut self, artboard: &ArtboardInstance) {
        // Pinned C++ initializes scripted objects before this scan, then
        // registers listener-less keyboard/text focus groups and a separate
        // authored-order gamepad broadcast list
        // (`state_machine_instance.cpp:2077-2120`). The Artboard owns the
        // concrete script occurrence; this state-machine occurrence retains
        // only the exact component identity and fresh group registrations.
        for component in artboard.components().iter() {
            if !nuxie_schema::definition_by_name(component.type_name)
                .is_some_and(|definition| definition.is_a("ScriptedDrawable"))
            {
                continue;
            }
            if !artboard.has_script_instance_for_global(component.global_id) {
                continue;
            };
            let Some(implemented) =
                artboard.script_implemented_methods_for_global(component.global_id)
            else {
                continue;
            };
            // Pinned C++ copies the serialized OptionalScriptedMethods bitfield
            // onto the occurrence during ScriptAsset initialization. Listener
            // membership uses those authored bits; only actual callback
            // dispatch probes the Lua field (`script_asset.cpp:145-159`,
            // `state_machine_instance.cpp:2083-2119`).
            let wants_keyboard = implemented.wants_keyboard();
            let wants_text = implemented.wants_text();
            let wants_gamepad_connected = implemented.wants_gamepad_connect();
            let wants_gamepad_event = implemented.wants_gamepad_event();
            let wants_gamepad_disconnected = implemented.wants_gamepad_disconnect();
            if (wants_keyboard || wants_text)
                && let Some(focus_data_local_id) =
                    listener_target_direct_child(artboard, component.local_id, "FocusData")
                && let Some(group) = keyboard_listener_group::RuntimeKeyboardListenerGroup::scripted(
                    component.local_id,
                    focus_data_local_id,
                    component.global_id,
                    wants_keyboard,
                    wants_text,
                )
            {
                self.keyboard_listener_groups.push(group);
            }
            if let Some(scripted) = gamepad_listener_group::RuntimeGamepadScriptedDrawable::new(
                component.global_id,
                wants_gamepad_connected,
                wants_gamepad_event,
                wants_gamepad_disconnected,
            ) {
                self.gamepad_scripted_drawables.push(scripted);
            }
        }
    }

    /// Complete the scripted-input portion of C++ state-machine construction
    /// after the Rust facade has mounted the concrete script occurrences.
    ///
    /// Pinned C++ mounts scripts before `StateMachineInstance` performs its
    /// authored-order scan. Rust's authenticated facade intentionally creates
    /// the machine first and mounts scripts with a renderer factory afterward,
    /// so this is the source-corresponding completion seam. Rebuilding only
    /// scripted groups preserves every ordinary listener registration and
    /// prevents duplicate occurrences when facade preparation is idempotent.
    #[doc(hidden)]
    pub fn synchronize_scripted_input_groups(&mut self, artboard: &ArtboardInstance) {
        self.keyboard_listener_groups
            .retain(|group| group.scripted_global_id.is_none());
        self.gamepad_scripted_drawables.clear();
        self.initialize_scripted_input_groups(artboard);
        self.scripted_input_group_generation = artboard.script_attachment_generation();
    }

    fn ensure_scripted_input_groups_current(&mut self, artboard: &ArtboardInstance) {
        if self.scripted_input_group_generation != artboard.script_attachment_generation() {
            self.synchronize_scripted_input_groups(artboard);
        }
    }

    fn initialize_layers_in_authored_order(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
    ) {
        let file_data_context_instance =
            self.active_file_view_model_binding
                .and_then(|(view_model_index, instance_index)| {
                    self.file_view_model_instances
                        .as_ref()?
                        .instance(view_model_index, instance_index)
                });
        let mut host = NoopScriptHost;
        for (layer_index, layer) in state_machine.layers.iter().enumerate() {
            // Pinned C++ constructs one layer occurrence and immediately runs
            // `init`/`changeState(entry)` before allocating the next layer
            // (`state_machine_instance.cpp:1747-1752,150-175,378-409`).
            // Entry actions may mutate shared state before the next authored
            // layer is initialized. The concrete pinned constructors used by
            // this family do not read state-machine inputs during
            // `makeInstance`; the test-only construction snapshots below are
            // therefore an ordering observer, not a claim that C++ has an
            // input-consuming constructor.
            self.layers.push(StateMachineLayerInstance::new(
                layer,
                artboard,
                &self.inputs,
                &self.bindable_numbers,
                &self.key_frame_data_bind_graphs,
            ));
            let result = {
                let mut executor = RuntimeStateMachineListenerActionExecutor {
                    needs_advance: &mut self.needs_advance,
                    data_bind_graph: &mut self.data_bind_graph,
                    // C++ runs each layer's entry callbacks before cloning
                    // state-machine DataBinds and building the bindable lookup
                    // maps. Keep those later facilities unavailable even
                    // though Rust must allocate the graph field before
                    // constructing `Self`
                    // (`state_machine_instance.cpp:1747-1754`;
                    // `listener_viewmodel_change.cpp:42-80`).
                    data_bind_facilities_ready: false,
                    owned_view_model_context: None,
                    owned_data_context: self.owned_data_context.clone(),
                    file_data_context_instance: file_data_context_instance.clone(),
                    scripted_listener_action_instances: &self.scripted_listener_action_instances,
                    scripted_instances_by_global: &self.scripted_instances_by_global,
                    focus: &mut self.focus,
                    host: &mut host,
                };
                self.layers[layer_index].perform_initial_entry_actions(
                    artboard,
                    layer,
                    RuntimeScheduledListenerActionTargetsMut {
                        inputs: &mut self.inputs,
                        reported_events: &mut self.reported_events,
                        bindable_numbers: &mut self.bindable_numbers,
                        bindable_integers: &mut self.bindable_integers,
                        bindable_colors: &mut self.bindable_colors,
                        bindable_strings: &mut self.bindable_strings,
                        bindable_enums: &mut self.bindable_enums,
                        bindable_assets: &mut self.bindable_assets,
                        bindable_artboards: &mut self.bindable_artboards,
                        bindable_lists: &mut self.bindable_lists,
                        bindable_triggers: &mut self.bindable_triggers,
                        bindable_view_models: &mut self.bindable_view_models,
                        bindable_booleans: &mut self.bindable_booleans,
                        transition_durations: &mut self.transition_durations,
                    },
                    &mut executor,
                )
            };
            if let Err(error) = result {
                // Rust exposes the first script failure instead of swallowing
                // it like C++, but the diagnostic cannot reorder or suppress
                // the serial `StateMachineLayerInstance::init` loop. Retain
                // the first error while still initializing every later layer
                // (`state_machine_instance.cpp:1747-1752`).
                self.script_error.get_or_insert(error);
            }
        }
    }

    /// Attach one VM-owned scripted object table to this concrete state
    /// machine instance. Shared definitions only retain the authored global
    /// id; callers must instantiate a fresh table for every instance.
    pub fn set_script_instance_for_global(
        &mut self,
        global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) {
        self.scripted_instances_by_global
            .insert(global_id, RuntimeScriptInstanceHandle::new(instance));
    }

    pub fn set_script_input_for_global(
        &mut self,
        global_id: u32,
        name: &str,
        value: ScriptValue,
    ) -> Result<(), ScriptError> {
        let instance = self
            .scripted_instances_by_global
            .get(&global_id)
            .ok_or_else(|| ScriptError::new(format!("missing state-machine script {global_id}")))?;
        instance.borrow_mut().set_input(name, value)
    }

    /// Protocol tables required by this concrete state-machine occurrence.
    pub fn scripted_listener_actions(&self) -> &[ScriptListenerActionDefinition] {
        &self.scripted_listener_action_definitions
    }

    /// Complete imported `StateMachine::scriptedObjects()` collection.
    #[doc(hidden)]
    pub fn scripted_objects(&self) -> &[ScriptListenerActionDefinition] {
        &self.scripted_object_definitions
    }

    #[doc(hidden)]
    pub fn scripted_listener_action_input_snapshots(
        &self,
        action_global_id: u32,
    ) -> Option<Vec<crate::ScriptListenerInputSnapshot>> {
        self.scripted_object_bindings
            .iter()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
            .map(|occurrence| occurrence.input_snapshots())
    }

    /// Attach every cloned ScriptInput DataBind to the current occurrence's
    /// live data context without applying its source value.
    ///
    /// Pinned C++ calls `bindDataBindsFromContext` before scripted-object
    /// hydration/init, then waits until `updateDataBinds(false)` to project
    /// source changes (`state_machine_instance.cpp:2901-2913`).
    #[doc(hidden)]
    pub fn bind_scripted_listener_action_sources(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        explicit_rebind: bool,
    ) {
        let data_context = self.owned_data_context.clone();
        for occurrence in &mut self.scripted_object_bindings {
            if let Some(data_context) = data_context.as_ref() {
                occurrence.bind_sources_from_data_context(file, data_context, explicit_rebind);
            } else if let Some(root_context) = root_context {
                occurrence.bind_sources(file, root_context, explicit_rebind);
            }
        }
    }

    /// Bind one outer ScriptInput occurrence at its exact authored position in
    /// the C++ DataBind walk.
    #[doc(hidden)]
    pub fn bind_scripted_listener_input_source(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        action_global_id: u32,
        input_global_id: u32,
        explicit_rebind: bool,
    ) -> bool {
        let data_context = self.owned_data_context.clone();
        self.scripted_object_bindings
            .iter_mut()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
            .is_some_and(|occurrence| {
                if let Some(data_context) = data_context.as_ref() {
                    occurrence.bind_listener_input_source_from_data_context(
                        file,
                        data_context,
                        input_global_id,
                        explicit_rebind,
                    )
                } else if let Some(root_context) = root_context {
                    occurrence.bind_listener_input_source(
                        file,
                        root_context,
                        input_global_id,
                        explicit_rebind,
                    )
                } else {
                    false
                }
            })
    }

    /// Bind only one concrete converter occurrence, never its Group children.
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn bind_scripted_listener_converter_own_sources(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        action_global_id: u32,
        input_global_id: u32,
        converter_path: &[usize],
        explicit_rebind: bool,
    ) -> bool {
        let data_context = self.owned_data_context.clone();
        self.scripted_object_bindings
            .iter_mut()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
            .is_some_and(|occurrence| {
                if let Some(data_context) = data_context.as_ref() {
                    occurrence.bind_converter_own_sources_from_data_context_at_path(
                        file,
                        data_context,
                        input_global_id,
                        converter_path,
                        explicit_rebind,
                    )
                } else if let Some(root_context) = root_context {
                    occurrence.bind_converter_own_sources_at_path(
                        file,
                        root_context,
                        input_global_id,
                        converter_path,
                        explicit_rebind,
                    )
                } else {
                    false
                }
            })
    }

    /// Complete the outer bind's retained dependency list after every
    /// converter occurrence has bound and reinitialized in C++ order.
    #[doc(hidden)]
    pub fn finalize_scripted_listener_input_sources(
        &mut self,
        action_global_id: u32,
        input_global_id: u32,
    ) -> bool {
        self.scripted_object_bindings
            .iter_mut()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
            .is_some_and(|occurrence| occurrence.finalize_listener_input_sources(input_global_id))
    }

    #[doc(hidden)]
    pub fn scripted_listener_data_context_view_models(
        &self,
        file: &RuntimeFile,
        fallback_root: Option<&RuntimeOwnedViewModelHandle>,
    ) -> (Option<ScriptViewModel>, Vec<ScriptViewModel>) {
        if let Some(data_context) = self.owned_data_context.as_ref() {
            let mut contexts = data_context.main_context_chain(file).into_iter();
            if let Some(main) = contexts.next() {
                let main = crate::script_view_model_from_owned_context(file, &main);
                let parents = contexts
                    .filter_map(|context| {
                        crate::script_view_model_from_owned_context(file, &context)
                    })
                    .collect();
                return (main, parents);
            }
            // An occurrence-owned but empty DataContext is still
            // authoritative. Pinned C++ asks only this ScriptedObject's
            // DataContext; it does not substitute an unrelated facade root
            // when `mainViewModelInstance()` is null
            // (`lua_scripted_context.cpp:129-146`).
            return (None, Vec::new());
        }
        (
            fallback_root.and_then(|root| crate::script_view_model_from_owned(file, root)),
            Vec::new(),
        )
    }

    /// Retain the exact local/global/parent DataContext chain that C++
    /// supplies to a freshly projected ScriptInputArtboard.
    #[doc(hidden)]
    pub fn scripted_listener_artboard_parent_context(
        &self,
        fallback_root: Option<&RuntimeOwnedViewModelContextHandle>,
    ) -> Option<ScriptArtboardParentContext> {
        self.owned_data_context
            .clone()
            .map(ScriptArtboardParentContext::from_runtime)
            .or_else(|| {
                fallback_root.map(|context| {
                    ScriptArtboardParentContext::from_runtime(
                        RuntimeOwnedDataContext::from_context_handle(context),
                    )
                })
            })
    }

    #[doc(hidden)]
    pub fn has_scripted_listener_data_context(&self) -> bool {
        self.owned_data_context.is_some()
    }

    /// Record the C++ constructor-only fact that this occurrence inherited
    /// its owning Artboard's already-retained DataContext.
    #[doc(hidden)]
    pub fn mark_scripted_constructor_context_prebound(&mut self) {
        self.scripted_constructor_context_was_prebound = true;
    }

    /// Whether fixed ScriptedObjects must receive the constructor's live
    /// hydration pass before the later `inheritDataContext` converter bind.
    #[doc(hidden)]
    pub fn scripted_constructor_context_was_prebound(&self) -> bool {
        self.scripted_constructor_context_was_prebound
    }

    #[doc(hidden)]
    pub fn scripted_listener_bound_view_model(
        &self,
        file: &RuntimeFile,
        path: &crate::ScriptInputViewModelPropertyPath,
        fallback_root: Option<&RuntimeOwnedViewModelContextHandle>,
    ) -> Option<Option<ScriptViewModel>> {
        match self.owned_data_context.as_ref() {
            Some(data_context) => data_context.bound_script_view_model(file, path),
            None => fallback_root.and_then(|context| {
                crate::script_input_viewmodel_property::
                    bound_script_view_model_property_from_owned_path(file, context, path)
            }),
        }
    }

    /// Resolve one ScriptInput DataBind through this concrete occurrence's
    /// cloned converter state.
    ///
    /// This is intentionally keyed by both action and input identity. A
    /// shared file-level converter helper would alias state across
    /// StateMachineInstances, unlike C++ `ScriptedObject::cloneProperties`.
    fn resolve_scripted_listener_input_binding(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        action_global_id: u32,
        input_global_id: u32,
        emit_unchanged: bool,
    ) -> Result<
        Option<super::scripted_listener_action::RuntimeScriptedListenerBoundValue>,
        ScriptError,
    > {
        let occurrence = self
            .scripted_object_bindings
            .iter_mut()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "state machine has no scripted listener binding occurrence global {action_global_id}",
                ))
            })?;
        occurrence.resolve(file, context, input_global_id, emit_unchanged)
    }

    #[doc(hidden)]
    pub fn resolve_scripted_listener_scalar_binding(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        action_global_id: u32,
        input_global_id: u32,
        emit_unchanged: bool,
    ) -> Result<Option<ScriptValue>, ScriptError> {
        match self.resolve_scripted_listener_input_binding(
            file,
            context,
            action_global_id,
            input_global_id,
            emit_unchanged,
        )? {
            Some(super::scripted_listener_action::RuntimeScriptedListenerBoundValue::Value(
                value,
            )) => Ok(Some(value)),
            Some(value) => Err(ScriptError::new(format!(
                "scripted listener scalar input global {input_global_id} received {value:?}",
            ))),
            None => Ok(None),
        }
    }

    #[doc(hidden)]
    pub fn resolve_scripted_listener_artboard_binding(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        action_global_id: u32,
        input_global_id: u32,
        emit_unchanged: bool,
    ) -> Result<Option<u64>, ScriptError> {
        match self.resolve_scripted_listener_input_binding(
            file,
            context,
            action_global_id,
            input_global_id,
            emit_unchanged,
        )? {
            Some(super::scripted_listener_action::RuntimeScriptedListenerBoundValue::Artboard(
                value,
            )) => Ok(Some(value)),
            Some(value) => Err(ScriptError::new(format!(
                "scripted listener artboard input global {input_global_id} received {value:?}",
            ))),
            None => Ok(None),
        }
    }

    #[doc(hidden)]
    pub fn resolve_scripted_listener_trigger_binding(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        action_global_id: u32,
        input_global_id: u32,
        emit_unchanged: bool,
    ) -> Result<Option<u64>, ScriptError> {
        match self.resolve_scripted_listener_input_binding(
            file,
            context,
            action_global_id,
            input_global_id,
            emit_unchanged,
        )? {
            Some(super::scripted_listener_action::RuntimeScriptedListenerBoundValue::Trigger(
                value,
            )) => Ok(Some(value)),
            Some(value) => Err(ScriptError::new(format!(
                "scripted listener trigger input global {input_global_id} received {value:?}",
            ))),
            None => Ok(None),
        }
    }

    fn apply_scripted_object_bindings(
        &mut self,
        artboard: &ArtboardInstance,
        owned_context: Option<&RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if self.scripted_object_bindings.is_empty() {
            return Ok(false);
        }
        let Some(file) = artboard.runtime_file_arc() else {
            return Ok(false);
        };
        let data_context = self.owned_data_context.clone();
        let mut updates = Vec::new();
        let resolver = self.scripted_listener_artboard_resolver.clone();
        let artboard_parent_context = self.scripted_listener_artboard_parent_context(None);
        let mut changed = false;
        {
            let mut apply_converter_input =
                |instance: &RuntimeScriptInstanceHandle,
                 input_name: &ScriptCoreString,
                 value: super::scripted_listener_action::RuntimeScriptedListenerBoundValue|
                 -> Result<(), ScriptError> {
                    changed |= apply_scripted_input_update(
                        instance,
                        input_name,
                        value,
                        resolver.as_deref(),
                        artboard_parent_context.as_ref(),
                        host,
                    )?;
                    Ok(())
                };
            for binding in &mut self.scripted_object_bindings {
                let mut binding_updates = match data_context.as_ref() {
                    Some(data_context) => {
                        binding.update_scripted_converter_inputs_from_data_context(
                            &file,
                            data_context,
                            &mut apply_converter_input,
                        )?;
                        binding
                            .resolve_runtime_table_updates_from_data_context(&file, data_context)?
                    }
                    None => match owned_context {
                        Some(context) => {
                            binding.update_scripted_converter_inputs(
                                &file,
                                context,
                                &mut apply_converter_input,
                            )?;
                            binding.resolve_runtime_table_updates(&file, context)?
                        }
                        None => Vec::new(),
                    },
                };
                updates.append(&mut binding_updates);
            }
        }

        for update in updates {
            let Some(instance) = self
                .scripted_listener_action_instances
                .get(&update.action_global_id)
                .cloned()
            else {
                // C++ still updates the cloned ScriptInput while its script
                // table is absent. The occurrence cache above retains that
                // value so a later complete hydration can install it.
                continue;
            };
            changed |= apply_scripted_input_update(
                &instance,
                &update.input_name,
                update.value,
                resolver.as_deref(),
                artboard_parent_context.as_ref(),
                host,
            )?;
        }
        Ok(changed)
    }

    /// Apply the already-bound `ScriptInput` DataBinds without advancing
    /// layers or the rest of the state machine.
    ///
    /// This is the facade transaction seam for C++
    /// `DataBindContainer::updateDataBinds(false)`: after
    /// `internalDataContext` binds the cloned listener inputs, a host-side
    /// atomic state commit must project those sources in authored order
    /// before it exposes listener callbacks. The ordinary runtime advance
    /// calls the same private implementation.
    #[doc(hidden)]
    pub fn apply_scripted_listener_action_source_updates(
        &mut self,
        artboard: &ArtboardInstance,
        owned_context: Option<&RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if self.scripted_data_context_prepare_pending() {
            return Ok(false);
        }
        // This compatibility entry point is not a separate scripted-object
        // traversal. It is the same outer DataBindContainer false pass used
        // by StateMachineInstance::advance, so ordinary and ScriptInput
        // occurrences retain one authored queue/order.
        let had_work = self.data_bind_container.has_pending_work();
        self.update_data_binds_false(artboard, owned_context, host)?;
        Ok(had_work)
    }

    /// Attach one freshly generated protocol table to this occurrence.
    ///
    /// A table cannot be shared implicitly across state-machine instances:
    /// every instance owns its own attachment map, keyed by the authored
    /// scripted action's global id.
    pub fn set_scripted_listener_action_instance(
        &mut self,
        action_global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) -> Result<(), ScriptError> {
        let definition = self
            .scripted_listener_action_definitions
            .iter()
            .find(|definition| definition.action_global_id() == action_global_id)
            .cloned()
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "state machine has no scripted listener action global {action_global_id}",
                ))
            })?;
        if !self
            .scripted_object_definitions
            .iter()
            .any(|object| object.scripted_object_global_id() == action_global_id)
        {
            // Focused low-level tests may install an action definition
            // directly. Production construction already receives the
            // complete source `scriptedObjects()` vector.
            self.scripted_object_definitions.push(definition);
        }
        self.set_scripted_object_instance(action_global_id, instance)
    }

    /// Attach the fresh stateful table for any imported state-machine
    /// `ScriptedObject`. Listener actions and scripted transition conditions
    /// share one C++ source-definition to occurrence map.
    #[doc(hidden)]
    pub fn set_scripted_object_instance(
        &mut self,
        global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) -> Result<(), ScriptError> {
        let definition = self
            .scripted_object_definitions
            .iter()
            .find(|definition| definition.scripted_object_global_id() == global_id)
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "state machine has no scripted object global {global_id}",
                ))
            })?;
        if self.scripted_instances_by_global.contains_key(&global_id) {
            return Err(ScriptError::new(format!(
                "scripted object global {global_id} is already attached",
            )));
        }
        let handle = RuntimeScriptInstanceHandle::new(instance);
        self.scripted_instances_by_global
            .insert(global_id, handle.clone());
        if definition.scripted_object_kind()
            == crate::ScriptedStateMachineObjectKind::ListenerAction
        {
            self.scripted_listener_action_instances
                .insert(global_id, handle);
        }
        Ok(())
    }

    #[doc(hidden)]
    pub fn set_scripted_listener_artboard_resolver(
        &mut self,
        resolver: Box<dyn ScriptArtboardResolver>,
    ) {
        self.scripted_listener_artboard_resolver = Some(Rc::from(resolver));
    }

    #[doc(hidden)]
    pub fn scripted_listener_data_converter_targets(
        &self,
    ) -> Vec<(u32, u32, Vec<usize>, u32, bool)> {
        let mut targets = Vec::new();
        for occurrence in &self.scripted_object_bindings {
            for (input_global_id, converter_path, converter_global_id, inits) in
                occurrence.scripted_converter_targets()
            {
                targets.push((
                    occurrence.action_global_id(),
                    input_global_id,
                    converter_path,
                    converter_global_id,
                    inits,
                ));
            }
        }
        targets
    }

    /// Enumerate every cloned ScriptedDataConverter occurrence, including
    /// those that already own a live script table.
    ///
    /// C++ calls `ScriptedDataConverter::reinit` on every DataContext bind,
    /// not only when the prior generator failed. The `attached` bit lets the
    /// facade distinguish generator work from persistent-table rehydration
    /// without aliasing occurrences by converter global id.
    #[doc(hidden)]
    pub fn scripted_listener_data_converter_occurrences(
        &self,
    ) -> Vec<(u32, u32, Vec<usize>, u32, bool, bool)> {
        let mut occurrences = Vec::new();
        for occurrence in &self.scripted_object_bindings {
            for (input_global_id, converter_path, converter_global_id, inits, attached) in
                occurrence.scripted_converter_occurrences()
            {
                occurrences.push((
                    occurrence.action_global_id(),
                    input_global_id,
                    converter_path,
                    converter_global_id,
                    inits,
                    attached,
                ));
            }
        }
        occurrences
    }

    /// Immutable occurrence-keyed view of scripted converters cloned by the
    /// state machine's own authored DataBinds.
    ///
    /// This is a parity-evidence delegate, not a mutation or attachment API.
    /// A converter definition id may appear more than once; the parent
    /// DataBind index plus Group path is the concrete occurrence identity.
    #[doc(hidden)]
    pub fn scripted_data_converter_occurrence_snapshots(
        &self,
    ) -> Vec<crate::RuntimeScriptedDataConverterOccurrenceSnapshot> {
        self.data_bind_graph
            .scripted_converter_occurrence_snapshots()
    }

    #[doc(hidden)]
    pub fn state_machine_data_converter_bind_steps(
        &self,
    ) -> Vec<RuntimeStateMachineDataConverterBindStep> {
        runtime_state_machine_data_converter_bind_steps(&self.data_bind_graph)
    }

    #[doc(hidden)]
    pub fn scripted_data_converter_input_snapshots(
        &self,
        parent_data_bind_index: usize,
        converter_path: &[usize],
    ) -> Option<Vec<crate::ScriptListenerInputSnapshot>> {
        self.data_bind_graph
            .scripted_converter_input_snapshots_at_occurrence(
                parent_data_bind_index,
                converter_path,
            )
    }

    #[doc(hidden)]
    pub fn has_scripted_data_converter_instance(
        &self,
        parent_data_bind_index: usize,
        converter_path: &[usize],
    ) -> bool {
        self.data_bind_graph
            .scripted_converter_instance_at_occurrence(parent_data_bind_index, converter_path)
            .is_some()
    }

    #[doc(hidden)]
    pub fn bind_state_machine_data_bind_source(&mut self, data_bind_index: usize) -> bool {
        let Some(data_context) = self.owned_data_context.clone() else {
            return false;
        };
        self.data_bind_graph
            .bind_owned_view_model_data_context_for_data_bind(data_bind_index, &data_context)
    }

    #[doc(hidden)]
    pub fn bind_state_machine_data_converter_own_sources(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        parent_data_bind_index: usize,
        converter_path: &[usize],
        explicit_rebind: bool,
    ) -> bool {
        let data_context = self.owned_data_context.clone();
        if let Some(data_context) = data_context.as_ref() {
            self.data_bind_graph
                .bind_converter_own_sources_from_data_context_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    file,
                    data_context,
                    explicit_rebind,
                )
        } else if let Some(root_context) = root_context {
            self.data_bind_graph
                .bind_converter_own_sources_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    file,
                    root_context,
                    explicit_rebind,
                )
        } else {
            false
        }
    }

    #[doc(hidden)]
    pub fn finalize_state_machine_data_bind_source(&mut self, data_bind_index: usize) -> bool {
        let Some(data_context) = self.owned_data_context.clone() else {
            return false;
        };
        self.data_bind_graph
            .finalize_owned_view_model_data_context_for_data_bind(data_bind_index, &data_context)
    }

    #[doc(hidden)]
    pub fn rebind_state_machine_data_converter_final_input(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        parent_data_bind_index: usize,
        converter_path: &[usize],
        input_index: usize,
        data_bind_index: usize,
    ) -> bool {
        let data_context = self.owned_data_context.clone();
        if let Some(data_context) = data_context.as_ref() {
            self.data_bind_graph
                .rebind_scripted_converter_final_input_from_data_context_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    input_index,
                    data_bind_index,
                    file,
                    data_context,
                )
        } else if let Some(root_context) = root_context {
            self.data_bind_graph
                .rebind_scripted_converter_final_input_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    input_index,
                    data_bind_index,
                    file,
                    root_context,
                )
        } else {
            false
        }
    }

    /// Bind only the custom-input DataBinds owned by one concrete cloned
    /// `ScriptedDataConverter`.
    ///
    /// C++ performs this before `reinit`/hydration and repeats the direct
    /// ScriptInput bind after hydration. The parent DataBind index plus Group
    /// path identifies the clone; a converter definition id does not
    /// (`scripted_data_converter.cpp:170-188`;
    /// `data_converter_group.cpp:63-74`).
    #[doc(hidden)]
    pub fn bind_scripted_data_converter_sources(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        parent_data_bind_index: usize,
        converter_path: &[usize],
        explicit_rebind: bool,
    ) -> bool {
        let data_context = self.owned_data_context.clone();
        if let Some(data_context) = data_context.as_ref() {
            self.data_bind_graph
                .bind_scripted_converter_sources_from_data_context_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    file,
                    data_context,
                    explicit_rebind,
                )
        } else if let Some(root_context) = root_context {
            self.data_bind_graph
                .bind_scripted_converter_sources_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    file,
                    root_context,
                    explicit_rebind,
                )
        } else {
            false
        }
    }

    /// Repeat only each custom ScriptInput's retained final DataBind after a
    /// successful hydrate/init. Pinned C++ walks custom properties here, not
    /// the converter's complete DataBind collection
    /// (`scripted_data_converter.cpp:176-187`).
    #[doc(hidden)]
    pub fn rebind_scripted_data_converter_final_inputs(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        parent_data_bind_index: usize,
        converter_path: &[usize],
    ) -> bool {
        let data_context = self.owned_data_context.clone();
        if let Some(data_context) = data_context.as_ref() {
            self.data_bind_graph
                .rebind_scripted_converter_final_inputs_from_data_context_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    file,
                    data_context,
                )
        } else if let Some(root_context) = root_context {
            self.data_bind_graph
                .rebind_scripted_converter_final_inputs_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                    file,
                    root_context,
                )
        } else {
            false
        }
    }

    #[doc(hidden)]
    pub fn set_scripted_data_converter_instance(
        &mut self,
        parent_data_bind_index: usize,
        converter_path: &[usize],
        converter_global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) -> Result<(), ScriptError> {
        let handle = RuntimeScriptInstanceHandle::new(instance);
        if !self.data_bind_graph.attach_scripted_instance_at_occurrence(
            parent_data_bind_index,
            converter_path,
            converter_global_id,
            &handle,
        ) {
            return Err(ScriptError::new(format!(
                "state-machine DataBind {parent_data_bind_index} has no ScriptedDataConverter occurrence {converter_path:?} (global {converter_global_id})",
            )));
        }
        Ok(())
    }

    /// Complete one C++ `ScriptedDataConverter::reinit` attempt for one
    /// state-machine DataBind occurrence.
    #[doc(hidden)]
    pub fn hydrate_and_initialize_scripted_data_converter_instance<F>(
        &mut self,
        parent_data_bind_index: usize,
        converter_path: &[usize],
        context: crate::ScriptListenerActionHydration,
        inits: bool,
        factory: Option<&mut dyn nuxie_render_api::Factory>,
        prepare_hydration: F,
    ) -> Result<bool, ScriptError>
    where
        F: FnOnce(&Self) -> Result<crate::ScriptListenerActionHydration, ScriptError>,
    {
        let handle = self
            .data_bind_graph
            .scripted_converter_instance_at_occurrence(
                parent_data_bind_index,
                converter_path,
            )
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "state-machine DataBind {parent_data_bind_index} has no attached ScriptedDataConverter occurrence {converter_path:?}",
                ))
            })?;
        let mut factory = factory;
        {
            let mut instance = handle.borrow_mut();
            context.install_context(&mut **instance)?;
            if let Some(factory) = factory.as_deref_mut() {
                instance.prepare_init_retry_with_factory(factory)?;
            } else {
                instance.prepare_init_retry()?;
            }
        }

        let hydration = prepare_hydration(self)?;
        let mut instance = handle.borrow_mut();
        hydration.apply_inputs(&mut **instance, &mut NoopScriptHost)?;
        let hydrated = if !inits || !instance.user_init_pending()? {
            true
        } else if let Some(factory) = factory {
            instance.call_init_with_factory(&mut NoopScriptHost, factory)?
        } else {
            instance.call_init(&mut NoopScriptHost)?
        };
        drop(instance);
        if hydrated {
            let marked = self
                .data_bind_graph
                .mark_scripted_converter_hydrated_at_occurrence(
                    parent_data_bind_index,
                    converter_path,
                );
            debug_assert!(
                marked,
                "the hydrated ScriptedDataConverter must retain its exact parent DataBind"
            );
        }
        Ok(hydrated)
    }

    #[doc(hidden)]
    pub fn scripted_listener_data_converter_bind_steps(
        &self,
    ) -> Vec<super::RuntimeScriptedListenerDataConverterBindStep> {
        self.scripted_object_bindings
            .iter()
            .flat_map(|occurrence| occurrence.scripted_converter_bind_steps())
            .collect()
    }

    #[doc(hidden)]
    pub fn has_scripted_listener_data_converter_instance(
        &self,
        action_global_id: u32,
        input_global_id: u32,
        converter_path: &[usize],
    ) -> bool {
        self.scripted_object_bindings
            .iter()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
            .is_some_and(|occurrence| {
                occurrence.has_scripted_converter_instance_at_path(input_global_id, converter_path)
            })
    }

    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn rebind_scripted_listener_data_converter_final_input(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        action_global_id: u32,
        listener_input_global_id: u32,
        converter_path: &[usize],
        converter_input_index: usize,
        data_bind_index: usize,
    ) -> bool {
        let data_context = self.owned_data_context.clone();
        self.scripted_object_bindings
            .iter_mut()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
            .is_some_and(|occurrence| {
                occurrence.rebind_scripted_converter_final_input(
                    file,
                    root_context,
                    data_context.as_ref(),
                    listener_input_global_id,
                    converter_path,
                    converter_input_index,
                    data_bind_index,
                )
            })
    }

    #[doc(hidden)]
    pub fn scripted_listener_data_converter_input_snapshots(
        &self,
        action_global_id: u32,
        input_global_id: u32,
        converter_path: &[usize],
    ) -> Option<Vec<crate::ScriptListenerInputSnapshot>> {
        self.scripted_object_bindings
            .iter()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)?
            .scripted_converter_input_snapshots(input_global_id, converter_path)
    }

    #[doc(hidden)]
    pub fn set_scripted_listener_data_converter_instance(
        &mut self,
        action_global_id: u32,
        input_global_id: u32,
        converter_path: &[usize],
        converter_global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) -> Result<(), ScriptError> {
        let handle = RuntimeScriptInstanceHandle::new(instance);
        let Some(occurrence) = self
            .scripted_object_bindings
            .iter_mut()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
        else {
            return Err(ScriptError::new(format!(
                "state machine has no scripted listener binding occurrence global {action_global_id}",
            )));
        };
        if !occurrence.attach_scripted_converter_instance_at_path(
            input_global_id,
            converter_path,
            &handle,
        ) {
            return Err(ScriptError::new(format!(
                "ScriptedListenerAction global {action_global_id} input global {input_global_id} has no ScriptedDataConverter occurrence {converter_path:?} (global {converter_global_id})",
            )));
        }
        Ok(())
    }

    /// Run one complete C++-ordered hydration attempt for one retained
    /// `ScriptedDataConverter` occurrence.
    ///
    /// `StateMachineInstance::internalDataContext` assigns the live context
    /// before `initScriptedObjects`; `ScriptedObject::hydrateScriptInputs`
    /// then validates the whole occurrence before applying any input, and
    /// only afterward calls user `init` and `didHydrateScriptInputs`
    /// (`state_machine_instance.cpp:2886-2913`;
    /// `scripted_object.cpp:313-437`). Keep those phases inside one API so a
    /// facade cannot repeat the context/generator preamble or accidentally
    /// validate multiple scripted-object occurrences as one transaction.
    #[doc(hidden)]
    pub fn hydrate_and_initialize_scripted_listener_data_converter_instance<F>(
        &mut self,
        action_global_id: u32,
        input_global_id: u32,
        converter_path: &[usize],
        context: crate::ScriptListenerActionHydration,
        inits: bool,
        factory: Option<&mut dyn nuxie_render_api::Factory>,
        prepare_hydration: F,
    ) -> Result<bool, ScriptError>
    where
        F: FnOnce(&Self) -> Result<crate::ScriptListenerActionHydration, ScriptError>,
    {
        let handle = self
            .scripted_object_bindings
            .iter()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
            .and_then(|occurrence| {
                occurrence
                    .scripted_converter_instance_at_path(input_global_id, converter_path)
            })
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "ScriptedListenerAction global {action_global_id} input global {input_global_id} has no attached ScriptedDataConverter occurrence {converter_path:?}",
                ))
            })?;
        let mut factory = factory;

        {
            let mut instance = handle.borrow_mut();
            context.install_context(&mut **instance)?;
            if let Some(factory) = factory.as_deref_mut() {
                instance.prepare_init_retry_with_factory(factory)?;
            } else {
                instance.prepare_init_retry()?;
            }
        }

        let hydration = prepare_hydration(self)?;
        let mut instance = handle.borrow_mut();
        hydration.apply_inputs(&mut **instance, &mut NoopScriptHost)?;
        let hydrated = if !inits || !instance.user_init_pending()? {
            true
        } else if let Some(factory) = factory {
            instance.call_init_with_factory(&mut NoopScriptHost, factory)?
        } else {
            instance.call_init(&mut NoopScriptHost)?
        };
        drop(instance);
        if hydrated {
            let marked = self
                .scripted_object_bindings
                .iter_mut()
                .find(|occurrence| occurrence.action_global_id() == action_global_id)
                .is_some_and(|occurrence| {
                    occurrence.mark_scripted_converter_hydrated(input_global_id, converter_path)
                });
            debug_assert!(
                marked,
                "the hydrated ScriptedDataConverter occurrence must retain its outer DataBind"
            );
        }
        Ok(hydrated)
    }

    #[doc(hidden)]
    pub fn has_scripted_listener_action_instance(&self, action_global_id: u32) -> bool {
        self.scripted_listener_action_instances
            .contains_key(&action_global_id)
    }

    #[doc(hidden)]
    pub fn has_scripted_object_instance(&self, global_id: u32) -> bool {
        self.scripted_instances_by_global.contains_key(&global_id)
    }

    #[doc(hidden)]
    pub fn scripted_listener_action_user_init_pending(
        &self,
        action_global_id: u32,
    ) -> Result<bool, ScriptError> {
        let handle = self
            .scripted_listener_action_instances
            .get(&action_global_id)
            .cloned()
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "scripted listener action global {action_global_id} is not attached",
                ))
            })?;
        handle.borrow_mut().user_init_pending()
    }

    /// Run one complete C++-ordered hydration attempt for one retained
    /// `ScriptedListenerAction` occurrence.
    ///
    /// A hydration-prerequisite failure leaves the generated table attached
    /// and pending. A failed user `init` retains the backend retry recipe, and
    /// the next live-context bind recreates it before validating inputs. C++
    /// performs this sequence once per `ScriptedObject`, never once for a
    /// facade batch (`scripted_object.cpp:313-437`;
    /// `state_machine_instance.cpp:2886-2913`).
    #[doc(hidden)]
    pub fn hydrate_and_initialize_scripted_listener_action_instance<F>(
        &mut self,
        action_global_id: u32,
        context: crate::ScriptListenerActionHydration,
        inits: bool,
        factory: Option<&mut dyn nuxie_render_api::Factory>,
        prepare_hydration: F,
    ) -> Result<bool, ScriptError>
    where
        F: FnOnce(&Self) -> Result<crate::ScriptListenerActionHydration, ScriptError>,
    {
        self.hydrate_and_initialize_scripted_object_instance(
            action_global_id,
            context,
            inits,
            factory,
            prepare_hydration,
        )
    }

    #[doc(hidden)]
    pub fn hydrate_and_initialize_scripted_object_instance<F>(
        &mut self,
        global_id: u32,
        context: crate::ScriptListenerActionHydration,
        inits: bool,
        factory: Option<&mut dyn nuxie_render_api::Factory>,
        prepare_hydration: F,
    ) -> Result<bool, ScriptError>
    where
        F: FnOnce(&Self) -> Result<crate::ScriptListenerActionHydration, ScriptError>,
    {
        self.install_scripted_object_data_context(global_id, &context)?;
        self.hydrate_and_initialize_scripted_object_instance_after_context_install(
            global_id,
            inits,
            factory,
            prepare_hydration,
        )
    }

    /// Install one cloned `ScriptedObject`'s live DataContext without
    /// hydrating or initializing its script table.
    ///
    /// `StateMachineInstance::internalDataContext` assigns the new context to
    /// every retained ScriptedObject before `initScriptedObjects` enters the
    /// first occurrence (`state_machine_instance.cpp:2901-2913`). The facade
    /// uses this split operation to preserve that collection-wide barrier.
    #[doc(hidden)]
    pub fn install_scripted_object_data_context(
        &mut self,
        global_id: u32,
        context: &crate::ScriptListenerActionHydration,
    ) -> Result<(), ScriptError> {
        let handle = self
            .scripted_instances_by_global
            .get(&global_id)
            .cloned()
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "scripted object global {global_id} is not attached"
                ))
            })?;
        context.install_context(&mut **handle.borrow_mut())
    }

    /// Complete one cloned `ScriptedObject`'s generator retry, hydration, and
    /// optional user init after the collection-wide DataContext-install
    /// barrier has completed.
    #[doc(hidden)]
    pub fn hydrate_and_initialize_scripted_object_instance_after_context_install<F>(
        &mut self,
        global_id: u32,
        inits: bool,
        factory: Option<&mut dyn nuxie_render_api::Factory>,
        prepare_hydration: F,
    ) -> Result<bool, ScriptError>
    where
        F: FnOnce(&Self) -> Result<crate::ScriptListenerActionHydration, ScriptError>,
    {
        let handle = self
            .scripted_instances_by_global
            .get(&global_id)
            .cloned()
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "scripted object global {global_id} is not attached"
                ))
            })?;
        let mut factory = factory;
        {
            let mut instance = handle.borrow_mut();
            if let Some(factory) = factory.as_deref_mut() {
                instance.prepare_init_retry_with_factory(factory)?;
            } else {
                instance.prepare_init_retry()?;
            }
        }

        let hydration = prepare_hydration(self)?;
        let mut instance = handle.borrow_mut();
        hydration.apply_inputs(&mut **instance, &mut NoopScriptHost)?;
        if !inits {
            return Ok(true);
        }
        if !instance.user_init_pending()? {
            return Ok(true);
        }
        match factory {
            Some(factory) => instance.call_init_with_factory(&mut NoopScriptHost, factory),
            None => instance.call_init(&mut NoopScriptHost),
        }
    }

    /// Adopt listener-script state when a validated transactional candidate
    /// replaces this same state-machine occurrence.
    ///
    /// Ordinary clones stay cold so a new occurrence cannot accidentally
    /// share mutable script tables. A transaction candidate is different: it
    /// is a speculative copy of the same occurrence, and prevalidation does
    /// not execute listener callbacks. The facade calls this only immediately
    /// before committing that candidate.
    pub fn adopt_scripted_listener_action_state_from(
        &mut self,
        source: &Self,
    ) -> Result<(), ScriptError> {
        if self.state_machine_index != source.state_machine_index
            || self.scripted_object_definitions != source.scripted_object_definitions
            || self.scripted_listener_action_definitions
                != source.scripted_listener_action_definitions
        {
            return Err(ScriptError::new(
                "cannot adopt scripted object state from a different state-machine occurrence shape",
            ));
        }
        if !self.scripted_instances_by_global.is_empty()
            || !self.scripted_listener_action_instances.is_empty()
            || self.script_error.is_some()
        {
            return Err(ScriptError::new(
                "transaction candidate already has scripted object state",
            ));
        }
        if source.scripted_instances_by_global.keys().any(|global_id| {
            !source
                .scripted_object_definitions
                .iter()
                .any(|definition| definition.scripted_object_global_id() == *global_id)
        }) {
            return Err(ScriptError::new(
                "source occurrence has an unknown scripted object attachment",
            ));
        }
        if source
            .scripted_listener_action_instances
            .iter()
            .any(|(global_id, handle)| {
                source
                    .scripted_instances_by_global
                    .get(global_id)
                    .is_none_or(|canonical| canonical != handle)
            })
        {
            return Err(ScriptError::new(
                "source occurrence aliases a scripted listener through a noncanonical table",
            ));
        }

        self.scripted_instances_by_global = source.scripted_instances_by_global.clone();
        self.scripted_listener_action_instances = source.scripted_listener_action_instances.clone();
        self.data_bind_graph = source.data_bind_graph.clone_for_state_machine_transaction();
        self.scripted_object_bindings = source
            .scripted_object_bindings
            .iter()
            .map(
                super::scripted_listener_action::RuntimeScriptedListenerActionBindingOccurrence::rehomed_clone,
            )
            .collect();
        self.scripted_listener_artboard_resolver =
            source.scripted_listener_artboard_resolver.clone();
        self.script_error = source.script_error.clone();
        // `StateBatch` is a transactional snapshot of this same concrete
        // occurrence, not a fresh C++ construction. Rehoming its already-live
        // script tables must retain the completed clone/reinit lifecycle so
        // the next factory advance cannot replay the cold pass.
        self.scripted_object_initialization_complete =
            source.scripted_object_initialization_complete;
        self.scripted_constructor_context_was_prebound =
            source.scripted_constructor_context_was_prebound;
        self.scripted_data_context_bind_complete = source.scripted_data_context_bind_complete;
        // The outer state-machine queue is occurrence-owned too. Rebuild it
        // after installing the rehomed scripted bindings so retained source
        // dirt reaches this candidate, never the source transaction.
        self.initialize_data_bind_container();
        Ok(())
    }

    /// Re-home the complete occurrence-owned DataContext onto a transaction's
    /// detached ViewModel roots without flattening its local/global/scoped/
    /// parent topology.
    #[doc(hidden)]
    pub fn rehome_owned_data_context_for_transaction(
        &mut self,
        roots: &[(RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelHandle)],
    ) {
        let Some(data_context) = self.owned_data_context.as_ref() else {
            return;
        };
        let data_context = data_context.rehomed_clone_with_roots(roots);
        self.owned_data_context = Some(data_context.clone());
        self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        self.retain_owned_view_model_advance_context(&data_context);
        self.register_owned_view_model_rebind_dependents();
        self.scripted_data_context_bind_complete = false;
    }

    /// The first listener-script failure retained by compatibility entry
    /// points that cannot return a `Result`.
    pub fn script_error(&self) -> Option<&ScriptError> {
        self.script_error.as_ref()
    }

    /// Retain a facade-level DataContext bind failure on the concrete
    /// occurrence. C++ has no typed resource error; Rust's binding safety
    /// fence is terminal, so a partially executed callback sequence must not
    /// be retried as if the same explicit bind had never happened.
    #[doc(hidden)]
    pub fn retain_scripted_object_data_context_error(&mut self, error: ScriptError) {
        self.script_error.get_or_insert(error);
    }

    pub(super) fn retain_script_result<T: Default>(&mut self, result: Result<T, ScriptError>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => {
                self.script_error.get_or_insert(error);
                T::default()
            }
        }
    }

    /// Consume an ordinary protected-call failure like pinned C++ while
    /// retaining Rust's explicit terminal resource fence.
    ///
    /// The concrete Luau adapter already performs this translation, but the
    /// runtime-neutral `ScriptInstance` interface also admits test and host
    /// adapters. Keeping the rule at the state-machine owner prevents an
    /// ordinary adapter error from truncating the remaining focus/keyboard/
    /// text/gamepad callback FIFO.
    pub(super) fn retain_protected_script_result<T>(
        &mut self,
        result: Result<T, ScriptError>,
        ordinary_error_result: T,
    ) -> T {
        match result {
            Ok(value) => value,
            Err(error) if error.resource_code().is_some() => {
                self.script_error.get_or_insert(error);
                ordinary_error_result
            }
            Err(_) => ordinary_error_result,
        }
    }

    pub fn state_machine_index(&self) -> usize {
        self.state_machine_index
    }

    pub(crate) fn retained_state_machine_definitions(
        &self,
    ) -> Option<Arc<Vec<RuntimeStateMachine>>> {
        self.state_machine_definitions
            .as_ref()?
            .get(self.state_machine_index)?;
        self.state_machine_definitions.as_ref().map(Arc::clone)
    }

    pub(crate) fn requires_post_update_state_probe(&self) -> bool {
        self.requires_post_update_state_probe
    }

    pub fn changed_state_count(&self) -> usize {
        self.changed_state_count
    }

    /// Re-enter every layer's retained EntryState occurrence.
    ///
    /// This mirrors pinned C++ `StateMachineInstance::resetState` and is
    /// deliberately distinct from resetting Artboard component dirt.
    pub fn reset_state(&mut self, artboard: &mut ArtboardInstance) {
        let Some(definitions) = self.retained_state_machine_definitions() else {
            return;
        };
        let Some(state_machine) = definitions.get(self.state_machine_index) else {
            return;
        };
        let file_data_context_instance =
            self.active_file_view_model_binding
                .and_then(|(view_model_index, instance_index)| {
                    self.file_view_model_instances
                        .as_ref()?
                        .instance(view_model_index, instance_index)
                });
        let mut host = NoopScriptHost;
        for (layer_index, layer) in state_machine
            .layers
            .iter()
            .enumerate()
            .take(self.layers.len())
        {
            let mut executor = RuntimeStateMachineListenerActionExecutor {
                needs_advance: &mut self.needs_advance,
                data_bind_graph: &mut self.data_bind_graph,
                data_bind_facilities_ready: true,
                owned_view_model_context: None,
                owned_data_context: self.owned_data_context.clone(),
                file_data_context_instance: file_data_context_instance.clone(),
                scripted_listener_action_instances: &self.scripted_listener_action_instances,
                scripted_instances_by_global: &self.scripted_instances_by_global,
                focus: &mut self.focus,
                host: &mut host,
            };
            let result = self.layers[layer_index].reset_state(
                artboard,
                layer,
                &self.key_frame_data_bind_graphs,
                RuntimeScheduledListenerActionTargetsMut {
                    inputs: &mut self.inputs,
                    reported_events: &mut self.reported_events,
                    bindable_numbers: &mut self.bindable_numbers,
                    bindable_integers: &mut self.bindable_integers,
                    bindable_colors: &mut self.bindable_colors,
                    bindable_strings: &mut self.bindable_strings,
                    bindable_enums: &mut self.bindable_enums,
                    bindable_assets: &mut self.bindable_assets,
                    bindable_artboards: &mut self.bindable_artboards,
                    bindable_lists: &mut self.bindable_lists,
                    bindable_triggers: &mut self.bindable_triggers,
                    bindable_view_models: &mut self.bindable_view_models,
                    bindable_booleans: &mut self.bindable_booleans,
                    transition_durations: &mut self.transition_durations,
                },
                &mut executor,
            );
            if let Err(error) = result {
                self.script_error = Some(error);
                break;
            }
        }
        // C++ focus actions synchronously notify registered
        // FocusListenerGroups while resetState re-enters each layer. Preserve
        // that callback batch even though the listener actions themselves run
        // at the next ordinary advance boundary.
        self.capture_focus_callbacks();
        // Pinned C++ `StateMachineInstance::resetState` only re-enters each
        // layer (`state_machine_instance.cpp:2670-2676`). Genuine input,
        // focus, or event actions mark the owning instance through their
        // normal callbacks; reset itself does not invent work.
    }

    pub(crate) fn reset_changed_state_count_for_outer_settlement(&mut self) {
        self.changed_state_count = 0;
    }

    pub fn needs_advance(&self) -> bool {
        self.needs_advance
    }

    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    pub fn input(&self, index: usize) -> Option<&StateMachineInputInstance> {
        self.inputs.get(index).filter(|input| !input.is_null())
    }

    pub fn input_named(&self, name: &str) -> Option<&StateMachineInputInstance> {
        self.inputs
            .iter()
            .find(|input| !input.is_null() && input.name() == Some(name))
    }

    pub fn input_index_named(&self, name: &str) -> Option<usize> {
        self.inputs
            .iter()
            .position(|input| !input.is_null() && input.name() == Some(name))
    }

    pub fn set_bool(&mut self, index: usize, value: bool) -> bool {
        let Some(input) = self.inputs.get_mut(index) else {
            return false;
        };
        if !input.set_bool(value) {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_number(&mut self, index: usize, value: f32) -> bool {
        let Some(input) = self.inputs.get_mut(index) else {
            return false;
        };
        if !input.set_number(value) {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn fire_trigger(&mut self, index: usize) -> bool {
        let Some(input) = self.inputs.get_mut(index) else {
            return false;
        };
        if !input.fire_trigger() {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn has_focus_nodes(&self) -> bool {
        self.focus.has_focusable_content()
    }

    pub fn focus_next(&mut self) -> bool {
        self.change_focus(|focus| focus.traverse(0))
    }

    pub fn focus_previous(&mut self) -> bool {
        self.change_focus(|focus| focus.traverse(1))
    }

    pub fn focus_up(&mut self) -> bool {
        self.change_focus(|focus| focus.traverse(2))
    }

    pub fn focus_down(&mut self) -> bool {
        self.change_focus(|focus| focus.traverse(3))
    }

    pub fn focus_left(&mut self) -> bool {
        self.change_focus(|focus| focus.traverse(4))
    }

    pub fn focus_right(&mut self) -> bool {
        self.change_focus(|focus| focus.traverse(5))
    }

    pub fn clear_focus(&mut self) -> bool {
        self.change_focus(RuntimeFocusTree::clear_focus)
    }

    fn change_focus(&mut self, change: impl FnOnce(&mut RuntimeFocusTree) -> bool) -> bool {
        let changed = change(&mut self.focus);
        if changed {
            self.capture_focus_callbacks();
        }
        changed
    }

    fn capture_focus_callbacks(&mut self) {
        for (target_local_id, focus_data_local_id, kind) in self.focus.take_owner_events() {
            for group in &self.focus_listener_groups {
                if let Some(invocation) =
                    group.invocation_for(target_local_id, focus_data_local_id, kind)
                {
                    self.queued_focus_events.push(invocation);
                }
            }
        }
        if !self.queued_focus_events.is_empty() {
            self.needs_advance = true;
        }
    }

    /// Queue one callback from an already-resolved SemanticData occurrence.
    ///
    /// C++ `SemanticListenerGroup` receives this callback from SemanticData.
    /// The separate public `fireSemanticAction(semanticNodeId, ...)` manager
    /// lookup belongs to the still-pending whole StateMachineInstance owner;
    /// do not expose this local-id callback as that API.
    pub(crate) fn semantic_action_for_target(
        &mut self,
        target_local_id: usize,
        action_type: u32,
    ) -> bool {
        let mut queued = false;
        for group in &self.semantic_listener_groups {
            if group.target_local_id != target_local_id {
                continue;
            }
            let Some(listener) = self.listener_definitions.get(group.listener_index) else {
                continue;
            };
            if let Some(invocation) = group.invocation(listener, action_type) {
                self.queued_semantic_events.push(invocation);
                queued = true;
            }
        }
        self.needs_advance |= queued;
        queued
    }

    /// Dispatch a keyboard event to the currently focused listener groups.
    ///
    /// Listener actions deliberately return `false`: pinned C++ reserves the
    /// handled result for TextInput and listener-less scripted drawables.
    pub fn key_input(
        &mut self,
        artboard: &mut ArtboardInstance,
        key: u32,
        modifiers: u32,
        is_pressed: bool,
        is_repeat: bool,
    ) -> bool {
        if self.script_error.is_some() {
            return false;
        }
        if self.scripted_data_context_prepare_pending() {
            return false;
        }
        self.ensure_scripted_input_groups_current(artboard);
        if self.script_error.is_some() {
            return false;
        }
        if !self.focus.is_inert() {
            self.focus.sync(artboard);
        }
        let owner_identity = self.focus.owner_identity();
        for (owner, _, focus_data_local_id) in self.focus.focused_listener_chain() {
            let handled = if owner == owner_identity {
                self.key_input_at_focus_data(
                    artboard,
                    focus_data_local_id,
                    key,
                    modifiers,
                    is_pressed,
                    is_repeat,
                )
            } else {
                artboard.dispatch_nested_key_input_at_focus(
                    owner,
                    focus_data_local_id,
                    key,
                    modifiers,
                    is_pressed,
                    is_repeat,
                )
            };
            if handled.terminal_resource_failure {
                return false;
            }
            if handled.handled {
                return true;
            }
        }
        false
    }

    pub(crate) fn key_input_at_focus_data(
        &mut self,
        artboard: &mut ArtboardInstance,
        focus_data_local_id: usize,
        key: u32,
        modifiers: u32,
        is_pressed: bool,
        is_repeat: bool,
    ) -> RuntimeInputDispatchOutcome {
        if self.script_error.is_some() {
            return RuntimeInputDispatchOutcome::terminal();
        }
        if self.scripted_data_context_prepare_pending() {
            return RuntimeInputDispatchOutcome::default();
        }
        self.ensure_scripted_input_groups_current(artboard);
        if self.script_error.is_some() {
            return RuntimeInputDispatchOutcome::terminal();
        }
        let groups = self
            .keyboard_listener_groups
            .iter()
            .filter(|group| group.focus_data_local_id == focus_data_local_id)
            .cloned()
            .collect::<Vec<_>>();
        for group in groups {
            let outcome = group.key_input(self, artboard, key, modifiers, is_pressed, is_repeat);
            if outcome.terminal_resource_failure || outcome.handled {
                return outcome;
            }
        }
        RuntimeInputDispatchOutcome::default()
    }

    /// Dispatch owned committed text to the currently focused listener groups.
    pub fn text_input(&mut self, artboard: &mut ArtboardInstance, text: &str) -> bool {
        if self.script_error.is_some() {
            return false;
        }
        if self.scripted_data_context_prepare_pending() {
            return false;
        }
        self.ensure_scripted_input_groups_current(artboard);
        if self.script_error.is_some() {
            return false;
        }
        if !self.focus.is_inert() {
            self.focus.sync(artboard);
        }
        let owner_identity = self.focus.owner_identity();
        for (owner, _, focus_data_local_id) in self.focus.focused_listener_chain() {
            let handled = if owner == owner_identity {
                self.text_input_at_focus_data(artboard, focus_data_local_id, text)
            } else {
                artboard.dispatch_nested_text_input_at_focus(owner, focus_data_local_id, text)
            };
            if handled.terminal_resource_failure {
                return false;
            }
            if handled.handled {
                return true;
            }
        }
        false
    }

    pub(crate) fn text_input_at_focus_data(
        &mut self,
        artboard: &mut ArtboardInstance,
        focus_data_local_id: usize,
        text: &str,
    ) -> RuntimeInputDispatchOutcome {
        if self.script_error.is_some() {
            return RuntimeInputDispatchOutcome::terminal();
        }
        if self.scripted_data_context_prepare_pending() {
            return RuntimeInputDispatchOutcome::default();
        }
        self.ensure_scripted_input_groups_current(artboard);
        if self.script_error.is_some() {
            return RuntimeInputDispatchOutcome::terminal();
        }
        let groups = self
            .keyboard_listener_groups
            .iter()
            .filter(|group| group.focus_data_local_id == focus_data_local_id)
            .cloned()
            .collect::<Vec<_>>();
        for group in groups {
            let outcome = group.text_input(self, artboard, text);
            if outcome.terminal_resource_failure || outcome.handled {
                return outcome;
            }
        }
        RuntimeInputDispatchOutcome::default()
    }

    /// Dispatch one owned gamepad invocation through focused listener groups.
    ///
    /// The listener branch always returns false, matching C++; scripted
    /// drawable handling is retained by the scripting owner and is wired
    /// ahead of this branch by the public facade.
    pub fn gamepad_dispatch(
        &mut self,
        artboard: &mut ArtboardInstance,
        invocation: ScriptListenerInvocation,
    ) -> bool {
        if self.script_error.is_some() {
            return false;
        }
        if self.scripted_data_context_prepare_pending() {
            return false;
        }
        self.ensure_scripted_input_groups_current(artboard);
        if self.script_error.is_some() {
            return false;
        }
        if !self.focus.is_inert() {
            self.focus.sync(artboard);
        }
        let mut handled = false;
        let mut already_dispatched = None;
        let owner_identity = self.focus.owner_identity();
        for (owner, _, focus_data_local_id) in self.focus.focused_listener_chain() {
            let (node_outcome, dispatched) = if owner == owner_identity {
                self.gamepad_dispatch_at_focus_data(artboard, focus_data_local_id, &invocation)
            } else {
                artboard.dispatch_nested_gamepad_at_focus(owner, focus_data_local_id, &invocation)
            };
            if dispatched.is_some() {
                already_dispatched = dispatched;
            }
            if node_outcome.terminal_resource_failure {
                return false;
            }
            handled |= node_outcome.handled;
            if node_outcome.handled {
                break;
            }
        }

        let broadcast =
            self.broadcast_gamepad_to_scripted_drawables(artboard, &invocation, already_dispatched);
        if broadcast.terminal_resource_failure {
            return false;
        }
        handled |= broadcast.handled;
        handled
    }

    pub(crate) fn broadcast_gamepad_to_scripted_drawables(
        &mut self,
        artboard: &mut ArtboardInstance,
        invocation: &ScriptListenerInvocation,
        already_dispatched: Option<(u64, u32)>,
    ) -> RuntimeInputDispatchOutcome {
        if self.script_error.is_some() {
            return RuntimeInputDispatchOutcome::terminal();
        }
        if self.scripted_data_context_prepare_pending() {
            return RuntimeInputDispatchOutcome::default();
        }
        self.ensure_scripted_input_groups_current(artboard);
        if self.script_error.is_some() {
            return RuntimeInputDispatchOutcome::terminal();
        }
        let nested =
            artboard.broadcast_nested_gamepad_to_scripted_drawables(invocation, already_dispatched);
        if nested.terminal_resource_failure {
            return RuntimeInputDispatchOutcome::terminal();
        }
        let mut handled = nested.handled;
        let owner_identity = artboard.instance_identity();
        // Pinned C++ broadcasts after the focus-tree pass to every scripted
        // drawable that declares the selected gamepad method, skipping the
        // focused drawable reported above (`gamepad_batch.cpp:298-362`).
        for scripted in self.gamepad_scripted_drawables.clone() {
            if Some((owner_identity, scripted.global_id)) == already_dispatched
                || !scripted.accepts(invocation)
            {
                continue;
            }
            let Some(script) = artboard.script_instance_for_global(scripted.global_id) else {
                continue;
            };
            // C++ `ScriptedDrawable::gamepadDispatch` returns true whenever
            // the selected method exists, even when the protected callback
            // raises. The script method has no handled return value.
            let result = script
                .borrow_mut()
                .call_scripted_drawable_input(invocation, &mut NoopScriptHost);
            let outcome = self.retain_protected_script_result(
                result,
                crate::ScriptedDrawableInputResult {
                    invoked: true,
                    handled: true,
                },
            );
            if self.script_error.is_some() {
                return RuntimeInputDispatchOutcome::terminal();
            }
            if outcome.invoked {
                artboard.wake_script_advance_for_global(scripted.global_id);
            }
            handled |= outcome.handled;
        }
        RuntimeInputDispatchOutcome::handled(handled)
    }

    pub(crate) fn gamepad_dispatch_at_focus_data(
        &mut self,
        artboard: &mut ArtboardInstance,
        focus_data_local_id: usize,
        invocation: &ScriptListenerInvocation,
    ) -> (RuntimeInputDispatchOutcome, Option<(u64, u32)>) {
        if self.script_error.is_some() {
            return (RuntimeInputDispatchOutcome::terminal(), None);
        }
        if self.scripted_data_context_prepare_pending() {
            return (RuntimeInputDispatchOutcome::default(), None);
        }
        self.ensure_scripted_input_groups_current(artboard);
        if self.script_error.is_some() {
            return (RuntimeInputDispatchOutcome::terminal(), None);
        }
        let groups = self
            .gamepad_listener_groups
            .iter()
            .filter(|group| group.focus_data_local_id == focus_data_local_id)
            .cloned()
            .collect::<Vec<_>>();
        let mut already_dispatched = None;
        for group in groups {
            let (outcome, dispatched) = group.gamepad_dispatch(self, artboard, invocation);
            already_dispatched = dispatched.or(already_dispatched);
            if outcome.terminal_resource_failure || outcome.handled {
                return (outcome, already_dispatched);
            }
        }
        (RuntimeInputDispatchOutcome::default(), already_dispatched)
    }

    fn pointer_input_for_listener(
        &mut self,
        listener_index: usize,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        is_hovered: bool,
    ) -> (RuntimePointerInput, bool) {
        let state_index = self.pointer_listener_states.iter().position(|state| {
            state.pointer_id == pointer_id && state.listener_index == listener_index
        });
        let (previous_x, previous_y, was_hovered) = match state_index {
            Some(index) => {
                let state = &mut self.pointer_listener_states[index];
                let was_hovered = state.is_hovered;
                if !was_hovered && is_hovered {
                    // Rive resets a listener group's prior position when the
                    // pointer enters it so the first callback cannot jump from
                    // an outside point.
                    state.previous_x = x;
                    state.previous_y = y;
                }
                state.is_hovered = is_hovered;
                (state.previous_x, state.previous_y, was_hovered)
            }
            None => {
                self.pointer_listener_states
                    .push(RuntimePointerListenerState {
                        pointer_id,
                        listener_index,
                        is_hovered,
                        previous_x: x,
                        previous_y: y,
                    });
                (x, y, false)
            }
        };
        (
            RuntimePointerInput {
                x,
                y,
                previous_x,
                previous_y,
                timestamp_seconds,
                id: pointer_id,
            },
            was_hovered,
        )
    }

    fn record_pointer_input_for_listener(
        &mut self,
        listener_index: usize,
        pointer: RuntimePointerInput,
    ) {
        if let Some(state) = self
            .pointer_listener_states
            .iter_mut()
            .find(|state| state.pointer_id == pointer.id && state.listener_index == listener_index)
        {
            state.previous_x = pointer.x;
            state.previous_y = pointer.y;
        }
    }

    fn release_pointer_input(&mut self, pointer_id: i32) {
        self.pointer_listener_states
            .retain(|state| state.pointer_id != pointer_id);
    }

    fn draggable_pointer_down(
        &mut self,
        artboard: &mut ArtboardInstance,
        position: (f32, f32),
        pointer_id: i32,
        timestamp_seconds: f32,
    ) -> bool {
        let mut hit = false;
        let mut hit_opaque = false;
        for proxy in &mut self.draggable_proxies {
            if hit_opaque || !runtime_draggable_proxy_hit_test(artboard, proxy, position) {
                continue;
            }
            if !proxy.active_pointers.contains(&pointer_id) {
                proxy.active_pointers.push(pointer_id);
            }
            runtime_draggable_proxy_start(artboard, proxy, position, timestamp_seconds);
            hit = true;
            hit_opaque |= proxy.opaque;
        }
        hit
    }

    fn draggable_pointer_move(
        &mut self,
        artboard: &mut ArtboardInstance,
        position: (f32, f32),
        pointer_id: i32,
        timestamp_seconds: f32,
    ) -> (bool, bool) {
        let mut hit = false;
        let mut started_scroll = false;
        for proxy in &mut self.draggable_proxies {
            if !proxy.active_pointers.contains(&pointer_id) {
                continue;
            }
            if runtime_draggable_proxy_drag(artboard, proxy, position, timestamp_seconds) {
                started_scroll |= !proxy.has_scrolled;
                proxy.has_scrolled = true;
                hit = true;
            }
        }
        (hit, started_scroll)
    }

    fn draggable_pointer_end(
        &mut self,
        artboard: &mut ArtboardInstance,
        pointer_id: i32,
    ) -> (bool, bool) {
        let mut hit = false;
        let mut ended_scroll = false;
        for proxy in &mut self.draggable_proxies {
            let Some(index) = proxy
                .active_pointers
                .iter()
                .position(|active| *active == pointer_id)
            else {
                continue;
            };
            proxy.active_pointers.remove(index);
            runtime_draggable_proxy_end(artboard, proxy);
            hit = true;
            ended_scroll |= proxy.has_scrolled;
            proxy.has_scrolled = false;
        }
        (hit, ended_scroll)
    }

    fn release_draggable_pointer(&mut self, pointer_id: i32) {
        // `updateListeners(exit)` calls `ListenerGroup::releaseEvent` after
        // processing. The base phase does not transition through clicked/out
        // for Exit, so `DraggableConstraintListenerGroup` does not call
        // `endDrag` (`listener_group.cpp:43-65,115-154`;
        // `draggable_constraint.cpp:32-85`).
        for proxy in &mut self.draggable_proxies {
            proxy.active_pointers.retain(|active| *active != pointer_id);
            proxy.has_scrolled = false;
        }
    }

    pub fn pointer_down(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        let result =
            self.try_pointer_down_with_script_host(artboard, x, y, pointer_id, &mut NoopScriptHost);
        self.retain_script_result(result)
    }

    pub(crate) fn hit_test(&self, artboard: &ArtboardInstance, x: f32, y: f32) -> bool {
        if !x.is_finite() || !y.is_finite() {
            return false;
        }
        if self
            .draggable_proxies
            .iter()
            .any(|proxy| runtime_draggable_proxy_hit_test(artboard, proxy, (x, y)))
        {
            return true;
        }
        for listener in self.listener_definitions.iter() {
            if listener_uses_report_queue(listener) {
                continue;
            }
            if [
                RuntimeListenerType::Down,
                RuntimeListenerType::Up,
                RuntimeListenerType::Move,
                RuntimeListenerType::Enter,
                RuntimeListenerType::Exit,
                RuntimeListenerType::Click,
                RuntimeListenerType::Drag,
                RuntimeListenerType::DragStart,
                RuntimeListenerType::DragEnd,
            ]
            .into_iter()
            .any(|kind| listener.has_listener(kind))
                && listener.hit_test(artboard, x, y)
            {
                return true;
            }
        }
        false
    }

    pub fn pointer_down_with_event_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        event_context: &StateMachineEventContext,
    ) -> bool {
        let result = self.pointer_down_with_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            0.0,
            None,
            Some(event_context),
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    pub fn pointer_down_with_owned_view_model_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        let result = self.try_pointer_down_with_owned_view_model_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            context,
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    pub fn pointer_down_with_owned_view_model_and_event_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
        event_context: &StateMachineEventContext,
    ) -> bool {
        let result = self.pointer_down_with_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            0.0,
            Some(context),
            Some(event_context),
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    pub fn try_pointer_down_with_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.try_pointer_down_with_timestamp_and_script_host(artboard, x, y, pointer_id, 0.0, host)
    }

    pub fn try_pointer_down_with_timestamp_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        validate_pointer_timestamp(timestamp_seconds)?;
        self.pointer_down_with_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            timestamp_seconds,
            None,
            None,
            host,
        )
    }

    pub fn try_pointer_down_with_owned_view_model_context_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.pointer_down_with_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            0.0,
            Some(context),
            None,
            host,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn pointer_down_with_context_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if self.scripted_data_context_prepare_pending() {
            return Ok(false);
        }
        if !x.is_finite() || !y.is_finite() {
            return Ok(false);
        }
        if !self.focus.is_inert() {
            self.focus.sync(artboard);
        }
        self.pointer_down_listener_hits
            .retain(|hit| hit.pointer_id != pointer_id);
        let mut hit = self.draggable_pointer_down(artboard, (x, y), pointer_id, timestamp_seconds);
        let listener_definitions = Arc::clone(&self.listener_definitions);
        for (listener_index, listener) in listener_definitions.iter().enumerate() {
            if listener_uses_report_queue(listener) {
                continue;
            }
            let listener_hit = listener.hit_test(artboard, x, y);
            let (hover_action, pointer) = self.update_pointer_listener_hover(
                listener_index,
                listener,
                listener_hit,
                pointer_id,
                x,
                y,
                timestamp_seconds,
            );
            let click_action = listener_hit && listener.has_listener(RuntimeListenerType::Click);
            let drag_action = listener_hit && listener.has_listener(RuntimeListenerType::Drag);
            if click_action || drag_action {
                self.pointer_down_listener_hits
                    .push(RuntimePointerDownListenerHit {
                        pointer_id,
                        listener_index,
                        drag_phase: drag_action.then_some(RuntimePointerDragPhase::Armed),
                        event_context: event_context.cloned(),
                    });
            }
            let direct_action = listener_hit && listener.has_listener(RuntimeListenerType::Down);
            let action_type =
                hover_action.or_else(|| direct_action.then_some(RuntimeListenerType::Down));
            if listener_hit
                && (click_action || drag_action || direct_action || hover_action.is_some())
            {
                hit = true;
            }
            if let Some(action_type) = action_type {
                let _ = self.perform_listener_actions_with_event_context(
                    artboard,
                    &listener.listener_actions,
                    owned_context.as_deref_mut(),
                    &script_pointer_invocation(pointer, action_type),
                    host,
                    event_context,
                )?;
                // C++ marks the machine after every matched pointer listener,
                // independent of whether an action changed a value
                // (`listener_group.cpp:218-225`).
                self.needs_advance = true;
            }
            self.record_pointer_input_for_listener(listener_index, pointer);
        }
        self.remember_pointer_position(pointer_id, x, y);
        Ok(hit)
    }

    pub fn pointer_move(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        seconds: f32,
        pointer_id: i32,
    ) -> bool {
        let result = self.try_pointer_move_with_timestamp_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            seconds,
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    pub fn pointer_move_with_owned_view_model_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        seconds: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        let result = validate_pointer_timestamp(seconds).and_then(|()| {
            self.update_pointer_listeners_with_script_host(
                artboard,
                RuntimeListenerType::Move,
                x,
                y,
                pointer_id,
                seconds,
                Some(context),
                &mut NoopScriptHost,
            )
        });
        self.retain_script_result(result)
    }

    pub fn try_pointer_move_with_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.try_pointer_move_with_timestamp_and_script_host(artboard, x, y, pointer_id, 0.0, host)
    }

    pub fn try_pointer_move_with_timestamp_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        validate_pointer_timestamp(timestamp_seconds)?;
        self.update_pointer_listeners_with_script_host(
            artboard,
            RuntimeListenerType::Move,
            x,
            y,
            pointer_id,
            timestamp_seconds,
            None,
            host,
        )
    }

    pub fn try_pointer_move_with_owned_view_model_context_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.update_pointer_listeners_with_script_host(
            artboard,
            RuntimeListenerType::Move,
            x,
            y,
            pointer_id,
            0.0,
            Some(context),
            host,
        )
    }

    pub fn pointer_up(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        let result =
            self.try_pointer_up_with_script_host(artboard, x, y, pointer_id, &mut NoopScriptHost);
        self.retain_script_result(result)
    }

    pub fn pointer_up_with_event_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        event_context: &StateMachineEventContext,
    ) -> bool {
        let result = self.pointer_up_with_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            0.0,
            None,
            Some(event_context),
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    pub fn pointer_up_with_owned_view_model_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        let result = self.try_pointer_up_with_owned_view_model_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            context,
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    pub fn pointer_up_with_owned_view_model_and_event_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
        event_context: &StateMachineEventContext,
    ) -> bool {
        let result = self.pointer_up_with_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            0.0,
            Some(context),
            Some(event_context),
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    pub fn try_pointer_up_with_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.try_pointer_up_with_timestamp_and_script_host(artboard, x, y, pointer_id, 0.0, host)
    }

    pub fn try_pointer_up_with_timestamp_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        validate_pointer_timestamp(timestamp_seconds)?;
        self.pointer_up_with_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            timestamp_seconds,
            None,
            None,
            host,
        )
    }

    pub fn try_pointer_up_with_owned_view_model_context_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.pointer_up_with_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            0.0,
            Some(context),
            None,
            host,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn pointer_up_with_context_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if self.scripted_data_context_prepare_pending() {
            return Ok(false);
        }
        if !x.is_finite() || !y.is_finite() {
            return Ok(false);
        }
        if !self.focus.is_inert() {
            self.focus.sync(artboard);
        }
        let (component_hit, component_ended_drag) =
            self.draggable_pointer_end(artboard, pointer_id);
        let mut hit = component_hit;
        if component_ended_drag {
            hit |= self.dispatch_direct_pointer_listener_type(
                artboard,
                pointer_id,
                RuntimeListenerType::DragEnd,
                x,
                y,
                timestamp_seconds,
                owned_context.as_deref_mut(),
                host,
            )?;
            // C++ `StateMachineInstance::dragEnd` immediately follows the
            // DragEnd listener pass with `pointerMove(position, timeStamp,
            // pointerId)`. The component-provided group has already left its
            // down phase, so this tail only needs the ordinary listener pass
            // (`state_machine_instance.cpp:1585-1607`).
            hit |= self.update_pointer_listeners_with_script_host_internal(
                artboard,
                RuntimeListenerType::Move,
                x,
                y,
                pointer_id,
                timestamp_seconds,
                owned_context.as_deref_mut(),
                host,
                false,
            )?;
        }
        let listener_definitions = Arc::clone(&self.listener_definitions);
        if self.pointer_down_listener_hits.iter().any(|capture| {
            capture.pointer_id == pointer_id
                && capture.drag_phase == Some(RuntimePointerDragPhase::Dragging)
        }) {
            hit |= self.dispatch_captured_pointer_listener_type(
                artboard,
                pointer_id,
                RuntimeListenerType::DragEnd,
                x,
                y,
                timestamp_seconds,
                owned_context.as_deref_mut(),
                host,
            )?;
        }
        for (listener_index, listener) in listener_definitions.iter().enumerate() {
            if listener_uses_report_queue(listener) {
                continue;
            }
            let listener_hit = listener.hit_test(artboard, x, y);
            let (hover_action, pointer) = self.update_pointer_listener_hover(
                listener_index,
                listener,
                listener_hit,
                pointer_id,
                x,
                y,
                timestamp_seconds,
            );
            let click_matched = listener.has_listener(RuntimeListenerType::Click)
                && self.pointer_down_listener_hits.iter().any(|hit| {
                    hit.pointer_id == pointer_id && hit.listener_index == listener_index
                });
            let direct_action = listener_hit && listener.has_listener(RuntimeListenerType::Up);
            let action_type = hover_action
                .or_else(|| (listener_hit && click_matched).then_some(RuntimeListenerType::Click))
                .or_else(|| direct_action.then_some(RuntimeListenerType::Up));
            if listener_hit && (click_matched || direct_action || hover_action.is_some()) {
                hit = true;
            }
            let captured_event_context = (action_type == Some(RuntimeListenerType::Click))
                .then(|| {
                    self.pointer_down_listener_hits
                        .iter()
                        .find(|capture| {
                            capture.pointer_id == pointer_id
                                && capture.listener_index == listener_index
                        })
                        .and_then(|capture| capture.event_context.clone())
                })
                .flatten();
            let action_event_context = captured_event_context.as_ref().or(event_context);
            if let Some(action_type) = action_type {
                let _ = self.perform_listener_actions_with_event_context(
                    artboard,
                    &listener.listener_actions,
                    owned_context.as_deref_mut(),
                    &script_pointer_invocation(pointer, action_type),
                    host,
                    action_event_context,
                )?;
                self.needs_advance = true;
            }
            self.record_pointer_input_for_listener(listener_index, pointer);
        }
        self.remember_pointer_position(pointer_id, x, y);
        self.pointer_down_listener_hits
            .retain(|hit| hit.pointer_id != pointer_id);
        Ok(hit)
    }

    pub fn pointer_exit(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        let result =
            self.try_pointer_exit_with_script_host(artboard, x, y, pointer_id, &mut NoopScriptHost);
        self.retain_script_result(result)
    }

    pub(crate) fn drag_start(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        timestamp_seconds: f32,
        pointer_id: i32,
    ) -> bool {
        let result = validate_pointer_timestamp(timestamp_seconds).and_then(|()| {
            self.dispatch_direct_pointer_listener_type(
                artboard,
                pointer_id,
                RuntimeListenerType::DragStart,
                x,
                y,
                timestamp_seconds,
                None,
                &mut NoopScriptHost,
            )
        });
        self.retain_script_result(result)
    }

    pub(crate) fn drag_end(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        timestamp_seconds: f32,
        pointer_id: i32,
    ) -> bool {
        let result = validate_pointer_timestamp(timestamp_seconds).and_then(|()| {
            let mut hit = self.dispatch_direct_pointer_listener_type(
                artboard,
                pointer_id,
                RuntimeListenerType::DragEnd,
                x,
                y,
                timestamp_seconds,
                None,
                &mut NoopScriptHost,
            )?;
            hit |= self.update_pointer_listeners_with_script_host(
                artboard,
                RuntimeListenerType::Move,
                x,
                y,
                pointer_id,
                timestamp_seconds,
                None,
                &mut NoopScriptHost,
            )?;
            Ok(hit)
        });
        self.retain_script_result(result)
    }

    pub fn pointer_exit_with_owned_view_model_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        let result = self.try_pointer_exit_with_owned_view_model_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            context,
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    pub fn try_pointer_exit_with_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.try_pointer_exit_with_timestamp_and_script_host(artboard, x, y, pointer_id, 0.0, host)
    }

    pub fn try_pointer_exit_with_timestamp_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        validate_pointer_timestamp(timestamp_seconds)?;
        self.update_pointer_listeners_with_script_host(
            artboard,
            RuntimeListenerType::Exit,
            x,
            y,
            pointer_id,
            timestamp_seconds,
            None,
            host,
        )
    }

    fn dispatch_captured_pointer_listener_type(
        &mut self,
        artboard: &mut ArtboardInstance,
        pointer_id: i32,
        listener_type: RuntimeListenerType,
        x: f32,
        y: f32,
        timestamp_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        let listener_definitions = Arc::clone(&self.listener_definitions);
        let mut captured_targets = Vec::new();
        for capture in self.pointer_down_listener_hits.iter().filter(|capture| {
            capture.pointer_id == pointer_id
                && capture.drag_phase == Some(RuntimePointerDragPhase::Dragging)
        }) {
            let Some(target_local_id) = listener_definitions
                .get(capture.listener_index)
                .map(|listener| listener.target_local_id)
            else {
                continue;
            };
            if !captured_targets
                .iter()
                .any(|(captured, _)| *captured == target_local_id)
            {
                captured_targets.push((target_local_id, capture.event_context.clone()));
            }
        }

        let (previous_x, previous_y) = self
            .pointer_positions
            .iter()
            .find(|position| position.pointer_id == pointer_id)
            .map_or((x, y), |position| (position.x, position.y));
        let pointer = RuntimePointerInput {
            x,
            y,
            previous_x,
            previous_y,
            timestamp_seconds,
            id: pointer_id,
        };
        let mut hit = false;
        for listener in listener_definitions.iter() {
            if listener_uses_report_queue(listener) {
                continue;
            }
            let Some((_, event_context)) = captured_targets
                .iter()
                .find(|(target, _)| *target == listener.target_local_id)
            else {
                continue;
            };
            if !listener.has_listener(listener_type) {
                continue;
            }
            hit = true;
            let _ = self.perform_listener_actions_with_event_context(
                artboard,
                &listener.listener_actions,
                owned_context.as_deref_mut(),
                &script_pointer_invocation(pointer, listener_type),
                host,
                event_context.as_ref(),
            )?;
            self.needs_advance = true;
        }
        Ok(hit)
    }

    #[allow(clippy::too_many_arguments)]
    fn dispatch_direct_pointer_listener_type(
        &mut self,
        artboard: &mut ArtboardInstance,
        pointer_id: i32,
        listener_type: RuntimeListenerType,
        x: f32,
        y: f32,
        timestamp_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if self.scripted_data_context_prepare_pending() {
            return Ok(false);
        }
        let listener_definitions = Arc::clone(&self.listener_definitions);
        let mut hit = false;
        for (listener_index, listener) in listener_definitions.iter().enumerate() {
            if listener_uses_report_queue(listener) {
                continue;
            }
            if !listener.has_listener(listener_type) || !listener.hit_test(artboard, x, y) {
                continue;
            }
            let (pointer, _) = self.pointer_input_for_listener(
                listener_index,
                x,
                y,
                pointer_id,
                timestamp_seconds,
                true,
            );
            let _ = self.perform_listener_actions_with_event_context(
                artboard,
                &listener.listener_actions,
                owned_context.as_deref_mut(),
                &script_pointer_invocation(pointer, listener_type),
                host,
                None,
            )?;
            self.needs_advance = true;
            self.record_pointer_input_for_listener(listener_index, pointer);
            hit = true;
        }
        Ok(hit)
    }

    pub fn try_pointer_exit_with_owned_view_model_context_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.update_pointer_listeners_with_script_host(
            artboard,
            RuntimeListenerType::Exit,
            x,
            y,
            pointer_id,
            0.0,
            Some(context),
            host,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn update_pointer_listeners_with_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        listener_type: RuntimeListenerType,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.update_pointer_listeners_with_script_host_internal(
            artboard,
            listener_type,
            x,
            y,
            pointer_id,
            timestamp_seconds,
            owned_context.as_deref_mut(),
            host,
            true,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn update_pointer_listeners_with_script_host_internal(
        &mut self,
        artboard: &mut ArtboardInstance,
        listener_type: RuntimeListenerType,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
        process_component_proxies: bool,
    ) -> Result<bool, ScriptError> {
        if self.scripted_data_context_prepare_pending() {
            return Ok(false);
        }
        if !x.is_finite() || !y.is_finite() {
            return Ok(false);
        }
        if !self.focus.is_inert() {
            self.focus.sync(artboard);
        }
        let (component_hit, component_starts_drag, component_ends_drag) =
            if !process_component_proxies {
                (false, false, false)
            } else {
                match listener_type {
                    RuntimeListenerType::Move => {
                        let (hit, starts) = self.draggable_pointer_move(
                            artboard,
                            (x, y),
                            pointer_id,
                            timestamp_seconds,
                        );
                        (hit, starts, false)
                    }
                    RuntimeListenerType::Exit => {
                        self.release_draggable_pointer(pointer_id);
                        (false, false, false)
                    }
                    _ => (false, false, false),
                }
            };
        let mut hit = component_hit;
        if component_starts_drag {
            hit |= self.dispatch_direct_pointer_listener_type(
                artboard,
                pointer_id,
                RuntimeListenerType::DragStart,
                x,
                y,
                timestamp_seconds,
                owned_context.as_deref_mut(),
                host,
            )?;
        }
        if component_ends_drag {
            hit |= self.dispatch_direct_pointer_listener_type(
                artboard,
                pointer_id,
                RuntimeListenerType::DragEnd,
                x,
                y,
                timestamp_seconds,
                owned_context.as_deref_mut(),
                host,
            )?;
        }
        let listener_definitions = Arc::clone(&self.listener_definitions);

        let mut starts_drag = false;
        if listener_type == RuntimeListenerType::Move {
            for capture in self
                .pointer_down_listener_hits
                .iter_mut()
                .filter(|capture| capture.pointer_id == pointer_id)
            {
                if capture.drag_phase == Some(RuntimePointerDragPhase::Armed) {
                    capture.drag_phase = Some(RuntimePointerDragPhase::Dragging);
                    starts_drag = true;
                }
            }
        }

        if starts_drag {
            hit |= self.dispatch_captured_pointer_listener_type(
                artboard,
                pointer_id,
                RuntimeListenerType::DragStart,
                x,
                y,
                timestamp_seconds,
                owned_context.as_deref_mut(),
                host,
            )?;
        }
        for (listener_index, listener) in listener_definitions.iter().enumerate() {
            if listener_uses_report_queue(listener) {
                continue;
            }
            let listener_hit =
                listener_type != RuntimeListenerType::Exit && listener.hit_test(artboard, x, y);
            let (hover_action, pointer) = self.update_pointer_listener_hover(
                listener_index,
                listener,
                listener_hit,
                pointer_id,
                x,
                y,
                timestamp_seconds,
            );
            let captured_drag = listener_type == RuntimeListenerType::Move
                && listener.has_listener(RuntimeListenerType::Drag)
                && self.pointer_down_listener_hits.iter().any(|capture| {
                    capture.pointer_id == pointer_id
                        && capture.listener_index == listener_index
                        && capture.drag_phase == Some(RuntimePointerDragPhase::Dragging)
                });
            let direct_action = listener_hit && listener.has_listener(listener_type);
            let action_type = hover_action
                .or_else(|| captured_drag.then_some(RuntimeListenerType::Drag))
                .or_else(|| direct_action.then_some(listener_type));
            if captured_drag || (listener_hit && (direct_action || hover_action.is_some())) {
                hit = true;
            }
            let captured_event_context = captured_drag
                .then(|| {
                    self.pointer_down_listener_hits
                        .iter()
                        .find(|capture| {
                            capture.pointer_id == pointer_id
                                && capture.listener_index == listener_index
                        })
                        .and_then(|capture| capture.event_context.clone())
                })
                .flatten();
            if let Some(action_type) = action_type {
                let _ = self.perform_listener_actions_with_event_context(
                    artboard,
                    &listener.listener_actions,
                    owned_context.as_deref_mut(),
                    &script_pointer_invocation(pointer, action_type),
                    host,
                    captured_event_context.as_ref(),
                )?;
                self.needs_advance = true;
            }
            self.record_pointer_input_for_listener(listener_index, pointer);
        }
        self.remember_pointer_position(pointer_id, x, y);
        if listener_type == RuntimeListenerType::Exit {
            self.release_pointer_input(pointer_id);
            self.pointer_down_listener_hits
                .retain(|capture| capture.pointer_id != pointer_id);
        }
        Ok(hit)
    }

    pub(crate) fn notify_events(
        &mut self,
        artboard: &mut ArtboardInstance,
        source_local_id: Option<usize>,
        events: &[StateMachineReportedEvent],
    ) -> bool {
        let result = self.try_notify_events_with_script_host(
            artboard,
            source_local_id,
            events,
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    pub(crate) fn try_notify_events_with_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        source_local_id: Option<usize>,
        events: &[StateMachineReportedEvent],
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.notify_events_with_context_and_script_host(
            artboard,
            source_local_id,
            events,
            None,
            host,
        )
    }

    pub(crate) fn notify_events_with_owned_view_model_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        source_local_id: Option<usize>,
        events: &[StateMachineReportedEvent],
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.notify_events_with_context(artboard, source_local_id, events, Some(context))
    }

    fn notify_events_with_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        source_local_id: Option<usize>,
        events: &[StateMachineReportedEvent],
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        let result = self.notify_events_with_context_and_script_host(
            artboard,
            source_local_id,
            events,
            owned_context.as_deref_mut(),
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    fn notify_events_with_context_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        source_local_id: Option<usize>,
        events: &[StateMachineReportedEvent],
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if self.script_error.is_some() || events.is_empty() {
            return Ok(false);
        }
        let listener_definitions = Arc::clone(&self.listener_definitions);
        let mut changed = false;
        for listener in listener_definitions.iter() {
            if !listener.has_listener(RuntimeListenerType::Event) {
                continue;
            }
            if source_local_id
                .is_some_and(|source_local_id| listener.target_local_id != source_local_id)
            {
                continue;
            }
            if source_local_id.is_none()
                && artboard
                    .runtime_object_type_name(listener.target_local_id)
                    .is_some_and(|type_name| {
                        type_name != "Artboard"
                            && !nuxie_schema::definition_by_name(type_name)
                                .is_some_and(|definition| definition.is_a("Event"))
                    })
            {
                // Pinned C++ keeps this legacy-file disambiguation even
                // though current editors no longer author the malformed
                // shape: a local event listener may target the Artboard or an
                // Event, but a resolved ordinary component must not receive
                // the report (`state_machine_instance.cpp:3078-3099`).
                continue;
            }
            for event in events {
                if !listener
                    .event_local_indices
                    .contains(&event.event_local_index())
                {
                    continue;
                }
                changed |= self.perform_listener_actions_with_event_context(
                    artboard,
                    &listener.listener_actions,
                    owned_context.as_deref_mut(),
                    &ScriptListenerInvocation::ReportedEvent {
                        event_local_index: event.event_local_index(),
                        seconds_delay: event.seconds_delay(),
                    },
                    host,
                    event.context.as_ref(),
                )?;
                if listener.is_single {
                    break;
                }
            }
        }
        Ok(changed)
    }

    pub(crate) fn apply_local_event_listeners(
        &mut self,
        artboard: &mut ArtboardInstance,
        mut next_event_index: usize,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        const MAX_EVENT_ITERATIONS: usize = 100;

        // A typed Rust resource failure is the terminal safety fence around
        // the otherwise C++-matching protected-call behavior. Once retained,
        // no deferred focus/semantic callback, event report, or ViewModel
        // listener occurrence may execute in a later batch.
        if self.script_error.is_some() {
            return false;
        }
        let mut changed =
            self.process_deferred_listener_group_events(artboard, owned_context.as_deref_mut());
        if self.script_error.is_some() {
            return changed;
        }
        if next_event_index >= self.reported_events.len()
            && self.reported_listener_view_models.is_empty()
        {
            return changed;
        }

        for _ in 0..MAX_EVENT_ITERATIONS {
            if next_event_index >= self.reported_events.len()
                && self.reported_listener_view_models.is_empty()
            {
                break;
            }

            // Mirrors C++ `StateMachineInstance::applyEvents()` updating
            // data binds before each queued notification batch
            // (`state_machine_instance.cpp:2320-2335`). Layer advancement is
            // deliberately left to the caller's single ordinary advance.
            let mut binding_host = NoopScriptHost;
            if let Err(error) =
                self.update_data_binds_false(artboard, owned_context.as_deref(), &mut binding_host)
            {
                self.script_error = Some(error);
                break;
            }
            let mut events = std::mem::take(&mut self.reporting_events);
            events.clear();
            events.extend_from_slice(&self.reported_events[next_event_index..]);
            for event in &mut events {
                event.refresh_from_live_artboard(artboard);
            }
            next_event_index = self.reported_events.len();
            // C++ swaps BOTH queues before notifying either one. Event
            // actions that mutate a listener cell therefore enqueue the next
            // batch rather than joining this reporting batch
            // (`state_machine_instance.cpp:2328-2335`).
            let mut listener_indices = std::mem::take(&mut self.reporting_listener_view_models);
            self.reported_listener_view_models
                .swap_into(&mut listener_indices);
            let event_changed = self.notify_events_with_context(
                artboard,
                None,
                &events,
                owned_context.as_deref_mut(),
            );
            self.reporting_events = events;
            changed |= event_changed;
            if self.script_error.is_some() {
                self.reporting_listener_view_models = listener_indices;
                break;
            }
            if event_changed && let Some(context) = owned_context.as_deref_mut() {
                changed |= self.bind_owned_view_model_context_mut(context);
            }

            // C++ reports the listener pointer once per genuine mutation and
            // preserves duplicates/FIFO order (`reportListenerViewModel`,
            // state_machine_instance.cpp:3021-3025,3048-3058). Temporarily
            // take both retained tables so actions can mutate this machine
            // and enqueue chained reports without cloning action vectors.
            let data_context = self.owned_data_context.take();
            let listeners = std::mem::take(&mut self.view_model_listeners);
            for &listener_index in &listener_indices {
                let Some(listener) = listeners.get(listener_index) else {
                    continue;
                };
                let invocation = ScriptListenerInvocation::ViewModelChange { listener_index };
                let action_result = if let Some(context) = owned_context.as_deref_mut() {
                    self.perform_listener_actions(
                        artboard,
                        listener.actions(),
                        Some(context),
                        &invocation,
                        &mut NoopScriptHost,
                    )
                } else if let Some(data_context) = data_context.as_ref() {
                    self.perform_listener_actions_for_data_context(
                        artboard,
                        data_context,
                        listener.actions(),
                        &invocation,
                    )
                } else {
                    self.perform_listener_actions(
                        artboard,
                        listener.actions(),
                        None,
                        &invocation,
                        &mut NoopScriptHost,
                    )
                };
                let action_changed = self.retain_script_result(action_result);
                changed |= action_changed;
                if self.script_error.is_some() {
                    break;
                }
            }
            self.view_model_listeners = listeners;
            self.owned_data_context = data_context;
            listener_indices.clear();
            self.reporting_listener_view_models = listener_indices;
            if self.script_error.is_some() {
                break;
            }
        }
        self.reported_event_listener_index = next_event_index.min(self.reported_events.len());
        changed
    }

    fn process_deferred_listener_group_events(
        &mut self,
        artboard: &mut ArtboardInstance,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        // FocusManager callbacks are immediate in C++; the retained Rust
        // manager records them until this occurrence can translate node
        // identity back to its registered FocusListenerGroups. Move the
        // resulting batch before dispatch so callbacks caused by these
        // actions remain queued for the next frame.
        self.capture_focus_callbacks();
        let focus_events = std::mem::take(&mut self.queued_focus_events);
        let mut changed = false;
        for invocation in focus_events {
            let listener_index = match invocation {
                ScriptListenerInvocation::Focus { listener_index, .. } => listener_index,
                _ => continue,
            };
            let Some(listener) = self.listener_definitions.get(listener_index) else {
                continue;
            };
            let actions = listener.listener_actions.clone();
            let result = self.perform_listener_actions(
                artboard,
                &actions,
                owned_context.as_deref_mut(),
                &invocation,
                &mut NoopScriptHost,
            );
            changed |= self.retain_script_result(result);
            if self.script_error.is_some() {
                break;
            }
        }
        if self.script_error.is_some() {
            return changed;
        }

        // Pinned C++ snapshots semantic events only after the focus batch has
        // finished (`state_machine_instance.cpp:2557-2558,2449-2490`).
        // Therefore a focus action that queues a semantic callback reaches
        // this same frame, while another semantic callback queued during this
        // loop waits for the next frame.
        let semantic_events = std::mem::take(&mut self.queued_semantic_events);
        for invocation in semantic_events {
            let listener_index = match invocation {
                ScriptListenerInvocation::Semantic { listener_index, .. } => listener_index,
                _ => continue,
            };
            let Some(listener) = self.listener_definitions.get(listener_index) else {
                continue;
            };
            let actions = listener.listener_actions.clone();
            let result = self.perform_listener_actions(
                artboard,
                &actions,
                owned_context.as_deref_mut(),
                &invocation,
                &mut NoopScriptHost,
            );
            changed |= self.retain_script_result(result);
            if self.script_error.is_some() {
                break;
            }
        }
        changed
    }

    fn update_pointer_listener_hover(
        &mut self,
        listener_index: usize,
        listener: &RuntimeStateMachineListener,
        is_hovered: bool,
        pointer_id: i32,
        x: f32,
        y: f32,
        timestamp_seconds: f32,
    ) -> (Option<RuntimeListenerType>, RuntimePointerInput) {
        let (pointer, was_hovered) = self.pointer_input_for_listener(
            listener_index,
            x,
            y,
            pointer_id,
            timestamp_seconds,
            is_hovered,
        );
        let action = match (was_hovered, is_hovered) {
            (false, true) if listener.has_listener(RuntimeListenerType::Enter) => {
                Some(RuntimeListenerType::Enter)
            }
            (true, false) if listener.has_listener(RuntimeListenerType::Exit) => {
                Some(RuntimeListenerType::Exit)
            }
            _ => None,
        };
        (action, pointer)
    }

    pub(super) fn perform_listener_actions(
        &mut self,
        artboard: &mut ArtboardInstance,
        listener_actions: &[RuntimeScheduledListenerAction],
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        invocation: &ScriptListenerInvocation,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.perform_listener_actions_with_event_context(
            artboard,
            listener_actions,
            owned_context,
            invocation,
            host,
            None,
        )
    }

    pub(super) fn perform_listener_actions_with_event_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        listener_actions: &[RuntimeScheduledListenerAction],
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        invocation: &ScriptListenerInvocation,
        host: &mut dyn ScriptHost,
        event_context: Option<&StateMachineEventContext>,
    ) -> Result<bool, ScriptError> {
        if self.scripted_data_context_prepare_pending() {
            return Ok(false);
        }
        let mut changed = false;
        for action in listener_actions {
            match action {
                RuntimeScheduledListenerAction::FireEvent(action) => {
                    if let Some(mut reported) = action.perform(artboard) {
                        // ListenerFireEvent ignores the C++ ListenerInvocation,
                        // but the Rust Scene facade separately retains the
                        // rendered hit occurrence that selected this listener.
                        // Keep that public occurrence metadata attached to the
                        // report while the Event payload itself remains live.
                        reported.context = event_context.cloned();
                        self.reported_events.push(reported);
                        changed = true;
                    }
                }
                RuntimeScheduledListenerAction::BoolChange(action) => {
                    let action_changed = action.perform(artboard, &mut self.inputs);
                    if action_changed && action.targets_direct_input(artboard) {
                        // C++ direct SMI setters call `valueChanged`, which
                        // marks this machine. A NestedInput marks its child
                        // occurrence instead, so do not smear that dirt onto
                        // the listener's owning machine.
                        self.needs_advance = true;
                    }
                    changed |= action_changed;
                }
                RuntimeScheduledListenerAction::NumberChange(action) => {
                    let action_changed = action.perform(artboard, &mut self.inputs);
                    if action_changed && action.targets_direct_input(artboard) {
                        self.needs_advance = true;
                    }
                    changed |= action_changed;
                }
                RuntimeScheduledListenerAction::TriggerChange(action) => {
                    let action_changed = action.perform(artboard, &mut self.inputs);
                    if action_changed && action.targets_direct_input(artboard) {
                        self.needs_advance = true;
                    }
                    changed |= action_changed;
                }
                RuntimeScheduledListenerAction::AlignTarget(action) => {
                    changed |= action.perform(artboard, invocation);
                }
                RuntimeScheduledListenerAction::ViewModelChange(action) => {
                    let Some(bindable_global_id) = action.bindable_global_id else {
                        continue;
                    };
                    let value = {
                        let targets = RuntimeScheduledListenerActionTargetsMut {
                            inputs: &mut self.inputs,
                            reported_events: &mut self.reported_events,
                            bindable_numbers: &mut self.bindable_numbers,
                            bindable_integers: &mut self.bindable_integers,
                            bindable_colors: &mut self.bindable_colors,
                            bindable_strings: &mut self.bindable_strings,
                            bindable_enums: &mut self.bindable_enums,
                            bindable_assets: &mut self.bindable_assets,
                            bindable_artboards: &mut self.bindable_artboards,
                            bindable_lists: &mut self.bindable_lists,
                            bindable_triggers: &mut self.bindable_triggers,
                            bindable_view_models: &mut self.bindable_view_models,
                            bindable_booleans: &mut self.bindable_booleans,
                            transition_durations: &mut self.transition_durations,
                        };
                        action
                            .occurrence_value(&targets, self.data_bind_graph.data_context_present())
                    };
                    let Some(value) = value else {
                        continue;
                    };
                    let data_bind_index = self
                        .data_bind_graph
                        .bindable_data_bind_to_source_index(bindable_global_id);
                    let source_changed = data_bind_index.is_some_and(|data_bind_index| {
                        if let Some(context) = owned_context.as_deref_mut() {
                            self.perform_listener_view_model_change(
                                data_bind_index,
                                &value,
                                Some(context),
                            )
                        } else if let Some(data_context) = self.owned_data_context.clone() {
                            self.perform_listener_view_model_change_for_data_context(
                                &data_context,
                                data_bind_index,
                                &value,
                            )
                        } else {
                            self.perform_listener_view_model_change(data_bind_index, &value, None)
                        }
                    });
                    let target_dirtied = self
                        .data_bind_graph
                        .dirty_bindable_data_bind_to_target(bindable_global_id);
                    changed |= source_changed || target_dirtied;
                }
                RuntimeScheduledListenerAction::Scripted {
                    definition: Some(definition),
                    ..
                } => {
                    let result = super::scripted_listener_action::perform_scripted_listener_action(
                        &self.scripted_listener_action_instances,
                        definition,
                        invocation,
                        host,
                    );
                    match result {
                        Ok(action_changed) => changed |= action_changed,
                        Err(error) => {
                            // A prior focus action has already completed. C++
                            // synchronously queued its callbacks before the
                            // later action failed; the Rust safety fence stops
                            // subsequent actions but must not erase that
                            // completed effect.
                            self.capture_focus_callbacks();
                            return Err(error);
                        }
                    }
                }
                RuntimeScheduledListenerAction::Scripted {
                    definition: None, ..
                }
                | RuntimeScheduledListenerAction::Noop { .. } => {}
                RuntimeScheduledListenerAction::FocusTarget(action) => {
                    changed |= action.perform(artboard, &mut self.focus);
                }
                RuntimeScheduledListenerAction::FocusClear(action) => {
                    changed |= action.perform(&mut self.focus);
                }
                RuntimeScheduledListenerAction::FocusTraversal(action) => {
                    changed |= action.perform(artboard, &mut self.focus);
                }
            }
        }
        // C++ FocusManager invokes FocusListenerGroup callbacks synchronously
        // during a focus action. Translate those retained callbacks now so
        // they mark this machine exactly when a matching group exists; the
        // queued listener actions still run at the next-frame batch boundary.
        self.capture_focus_callbacks();
        Ok(changed)
    }

    fn remember_pointer_position(&mut self, pointer_id: i32, x: f32, y: f32) {
        if let Some(position) = self
            .pointer_positions
            .iter_mut()
            .find(|position| position.pointer_id == pointer_id)
        {
            position.x = x;
            position.y = y;
        } else {
            self.pointer_positions
                .push(RuntimePointerPosition { pointer_id, x, y });
        }
    }

    fn perform_listener_view_model_change(
        &mut self,
        data_bind_index: usize,
        value: &RuntimeListenerViewModelChangeValue,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        match value {
            RuntimeListenerViewModelChangeValue::Number(value) => match owned_context {
                Some(context) => self.set_owned_view_model_context_number_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
                None => {
                    self.set_default_view_model_number_source_for_data_bind(data_bind_index, *value)
                }
            },
            RuntimeListenerViewModelChangeValue::Integer(value) => match owned_context {
                Some(context) => self
                    .set_owned_view_model_context_symbol_list_index_source_for_data_bind(
                        context,
                        data_bind_index,
                        *value,
                    ),
                None => self.set_default_view_model_symbol_list_index_source_for_data_bind(
                    data_bind_index,
                    *value,
                ),
            },
            RuntimeListenerViewModelChangeValue::Color(value) => match owned_context {
                Some(context) => self.set_owned_view_model_context_color_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
                None => {
                    self.set_default_view_model_color_source_for_data_bind(data_bind_index, *value)
                }
            },
            RuntimeListenerViewModelChangeValue::String(value) => match owned_context {
                Some(context) => self.set_owned_view_model_context_string_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                ),
                None => {
                    self.set_default_view_model_string_source_for_data_bind(data_bind_index, value)
                }
            },
            RuntimeListenerViewModelChangeValue::Enum(value) => match owned_context {
                Some(context) => self.set_owned_view_model_context_enum_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
                None => {
                    self.set_default_view_model_enum_source_for_data_bind(data_bind_index, *value)
                }
            },
            RuntimeListenerViewModelChangeValue::Asset(value) => {
                let value = self
                    .listener_asset_value_for_data_bind(data_bind_index, value)
                    .clone();
                let font_value = value.font_data_bind_value();
                match (owned_context, font_value.as_ref()) {
                    (Some(context), Some(font_value)) => self
                        .set_owned_view_model_context_font_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            font_value,
                        ),
                    (Some(context), None) => self
                        .set_owned_view_model_context_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            value.asset_index(),
                        ),
                    (None, _) => self.set_default_view_model_asset_source_for_data_bind(
                        data_bind_index,
                        value.data_bind_asset_index(),
                    ),
                }
            }
            RuntimeListenerViewModelChangeValue::Artboard(value) => match owned_context {
                Some(context) => self.set_owned_view_model_context_artboard_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
                None => self
                    .set_default_view_model_artboard_source_for_data_bind(data_bind_index, *value),
            },
            RuntimeListenerViewModelChangeValue::Trigger(value) => match owned_context {
                Some(context) => self.fire_owned_view_model_context_trigger_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
                None => self.perform_listener_trigger_view_model_change(data_bind_index, *value),
            },
            RuntimeListenerViewModelChangeValue::Boolean(value) => match owned_context {
                Some(context) => self.set_owned_view_model_context_boolean_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
                None => self
                    .set_default_view_model_boolean_source_for_data_bind(data_bind_index, *value),
            },
            RuntimeListenerViewModelChangeValue::List(value) => {
                let changed = self
                    .data_bind_graph
                    .set_active_view_model_source_for_data_bind(
                        data_bind_index,
                        RuntimeDataBindGraphValue::List {
                            item_count: usize::try_from(*value).unwrap_or(usize::MAX),
                        },
                    );
                self.needs_advance |= changed;
                changed
            }
            RuntimeListenerViewModelChangeValue::ViewModel(value) => {
                let changed = self
                    .data_bind_graph
                    .set_active_view_model_source_for_data_bind(
                        data_bind_index,
                        RuntimeDataBindGraphValue::ViewModel(*value),
                    );
                self.needs_advance |= changed;
                changed
            }
        }
    }

    fn perform_listener_trigger_view_model_change(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(bindable_trigger) = self
            .bindable_triggers
            .iter_mut()
            .find(|bindable_trigger| bindable_trigger.has_data_bind_index(data_bind_index))
        else {
            return false;
        };

        // Mirrors src/animation/listener_viewmodel_change.cpp: listener
        // actions invalidate the target-to-source binding even when the
        // trigger target value itself did not change.
        bindable_trigger.set_value(value);
        if !self
            .data_bind_graph
            .mark_trigger_target_dirty_for_data_bind(data_bind_index)
        {
            return false;
        }
        let applied = self
            .data_bind_graph
            .apply_default_view_model_target_to_source_for_data_bind(
                data_bind_index,
                &RuntimeDataBindGraphTargetsMut {
                    numbers: &mut self.bindable_numbers,
                    integers: &mut self.bindable_integers,
                    booleans: &mut self.bindable_booleans,
                    strings: &mut self.bindable_strings,
                    colors: &mut self.bindable_colors,
                    enums: &mut self.bindable_enums,
                    assets: &mut self.bindable_assets,
                    artboards: &mut self.bindable_artboards,
                    lists: &mut self.bindable_lists,
                    triggers: &mut self.bindable_triggers,
                    view_models: &mut self.bindable_view_models,
                    transition_durations: &mut self.transition_durations,
                    include_view_models: true,
                },
            );
        match applied {
            Ok(true) => {}
            Ok(false) => return false,
            Err(error) => {
                self.script_error.get_or_insert(error);
                return true;
            }
        }
        self.needs_advance = true;
        true
    }

    fn listener_asset_value_for_data_bind<'a>(
        &'a self,
        data_bind_index: usize,
        fallback: &'a RuntimeBindableAssetValue,
    ) -> &'a RuntimeBindableAssetValue {
        self.bindable_assets
            .iter()
            .find(|bindable_asset| bindable_asset.has_data_bind_index(data_bind_index))
            .map(|bindable_asset| &bindable_asset.value)
            .unwrap_or(fallback)
    }

    fn set_owned_view_model_context_font_asset_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: &RuntimeFontAssetValue,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_font_asset_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        // A listener can feed the updated source into another bindable in the
        // same frame. Refresh the full Font payload now; the scalar graph only
        // carries the generated propertyValue index.
        self.sync_bindable_font_assets_from_owned_context(context);
        true
    }

    pub fn set_bindable_number_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        let Some(bindable_number) = self
            .bindable_numbers
            .iter_mut()
            .find(|bindable_number| bindable_number.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_number.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_number_target_dirty_for_data_bind(data_bind_index);
        self.apply_direct_bindable_target_change(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_boolean_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        let Some(bindable_boolean) = self
            .bindable_booleans
            .iter_mut()
            .find(|bindable_boolean| bindable_boolean.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_boolean.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_boolean_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_integer_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(bindable_integer) = self
            .bindable_integers
            .iter_mut()
            .find(|bindable_integer| bindable_integer.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_integer.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_integer_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_color_for_data_bind(&mut self, data_bind_index: usize, value: u32) -> bool {
        let Some(bindable_color) = self
            .bindable_colors
            .iter_mut()
            .find(|bindable_color| bindable_color.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_color.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_color_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_string_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        let Some(bindable_string) = self
            .bindable_strings
            .iter_mut()
            .find(|bindable_string| bindable_string.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_string.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_string_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_enum_for_data_bind(&mut self, data_bind_index: usize, value: u64) -> bool {
        let Some(bindable_enum) = self
            .bindable_enums
            .iter_mut()
            .find(|bindable_enum| bindable_enum.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_enum.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_enum_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_asset_for_data_bind(&mut self, data_bind_index: usize, value: u64) -> bool {
        let Some(bindable_asset) = self
            .bindable_assets
            .iter_mut()
            .find(|bindable_asset| bindable_asset.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_asset.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_asset_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_artboard_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(bindable_artboard) = self
            .bindable_artboards
            .iter_mut()
            .find(|bindable_artboard| bindable_artboard.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_artboard.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_artboard_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_list_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: usize,
    ) -> bool {
        let Some(bindable_list) = self
            .bindable_lists
            .iter_mut()
            .find(|bindable_list| bindable_list.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_list.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_list_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_trigger_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(bindable_trigger) = self
            .bindable_triggers
            .iter_mut()
            .find(|bindable_trigger| bindable_trigger.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_trigger.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_trigger_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }

    pub fn set_bindable_view_model_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        let Some(value) = self
            .data_bind_graph
            .imported_view_model_target_value_for_data_bind(data_bind_index, instance_index)
        else {
            return false;
        };
        let Some(bindable_view_model) = self
            .bindable_view_models
            .iter_mut()
            .find(|bindable_view_model| bindable_view_model.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_view_model.set_imported_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_view_model_target_dirty_for_data_bind(data_bind_index);
        self.apply_direct_bindable_target_change(data_bind_index);
        self.needs_advance = true;
        true
    }

    fn apply_direct_bindable_target_change(&mut self, data_bind_index: usize) {
        let Some(data_bind_index) = self
            .data_bind_graph
            .bindable_data_bind_to_source_index_for_data_bind(data_bind_index)
        else {
            // C++ stores only explicitly Direction=ToSource occurrences in
            // `m_bindableDataBindsToSource`. A main-to-target TwoWay bind is
            // capable of reverse flow during `updateDataBinds(true)`, but a
            // direct BindableProperty host edit does not find it through this
            // map and therefore performs no immediate source write
            // (`state_machine_instance.cpp:1788-1805,3201-3210`).
            return;
        };
        // The public number/ViewModel setters mirror the C++ host mutation
        // seam used by the runtime probe: mutate the cloned BindableProperty,
        // immediately call that occurrence's `updateSourceBinding(true)`,
        // then drain source-to-target dirt with `updateDataBinds(false)`.
        // `advancedDataContext()` itself only advances retained ViewModel
        // values and must not own either operation
        // (`state_machine_instance.cpp:2587-2593`;
        // `data_bind.cpp:550-588`).
        let targets = RuntimeDataBindGraphTargetsMut {
            numbers: &mut self.bindable_numbers,
            integers: &mut self.bindable_integers,
            booleans: &mut self.bindable_booleans,
            strings: &mut self.bindable_strings,
            colors: &mut self.bindable_colors,
            enums: &mut self.bindable_enums,
            assets: &mut self.bindable_assets,
            artboards: &mut self.bindable_artboards,
            lists: &mut self.bindable_lists,
            triggers: &mut self.bindable_triggers,
            view_models: &mut self.bindable_view_models,
            transition_durations: &mut self.transition_durations,
            include_view_models: true,
        };
        if let Err(error) = self
            .data_bind_graph
            .apply_direct_bindable_target_to_source_for_data_bind(data_bind_index, &targets)
        {
            self.script_error.get_or_insert(error);
            return;
        }
        if let Err(error) = self
            .data_bind_graph
            .update_all_default_view_model_bindings_false(targets)
        {
            self.script_error.get_or_insert(error);
        }
    }

    pub fn default_view_model_view_model_source_instance_index_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        self.data_bind_graph
            .default_view_model_view_model_source_instance_index_for_data_bind(data_bind_index)
    }

    pub fn bindable_view_model_instance_index_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        let global_id = self
            .data_bind_graph
            .view_model_target_global_id_for_data_bind(data_bind_index)?;
        let value = self
            .bindable_view_models
            .iter()
            .find(|bindable_view_model| bindable_view_model.global_id == global_id)
            .map(|bindable_view_model| bindable_view_model.value)?;
        self.data_bind_graph
            .view_model_instance_index_for_data_bind_value(data_bind_index, value)
    }

    pub fn default_view_model_number_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<f32> {
        self.data_bind_graph
            .default_view_model_number_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_number_value_for_data_bind(&self, data_bind_index: usize) -> Option<f32> {
        if let Some(value) = self
            .data_bind_graph
            .number_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| {
                self.bindable_numbers
                    .iter()
                    .find(|bindable_number| bindable_number.global_id == global_id)
                    .map(|bindable_number| bindable_number.value)
            })
        {
            return Some(value);
        }
        self.bindable_numbers
            .iter()
            .find(|bindable_number| bindable_number.has_data_bind_index(data_bind_index))
            .map(|bindable_number| bindable_number.value)
    }

    pub fn default_view_model_boolean_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<bool> {
        self.data_bind_graph
            .default_view_model_boolean_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_boolean_value_for_data_bind(&self, data_bind_index: usize) -> Option<bool> {
        if let Some(value) = self
            .data_bind_graph
            .boolean_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_boolean_value(&self.bindable_booleans, global_id))
        {
            return Some(value);
        }
        self.bindable_booleans
            .iter()
            .find(|bindable_boolean| bindable_boolean.has_data_bind_index(data_bind_index))
            .map(|bindable_boolean| bindable_boolean.value)
    }

    pub fn default_view_model_list_source_item_count_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        self.data_bind_graph
            .default_view_model_list_source_item_count_for_data_bind(data_bind_index)
    }

    pub fn bindable_list_property_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        let global_id = self
            .data_bind_graph
            .list_target_global_id_for_data_bind(data_bind_index)?;
        self.bindable_lists
            .iter()
            .find(|bindable_list| bindable_list.global_id == global_id)
            .map(|bindable_list| bindable_list.property_value)
    }

    pub fn default_view_model_string_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<&[u8]> {
        self.data_bind_graph
            .default_view_model_string_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_string_value_for_data_bind(&self, data_bind_index: usize) -> Option<&[u8]> {
        if let Some(value) = self
            .data_bind_graph
            .string_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_string_value(&self.bindable_strings, global_id))
        {
            return Some(value);
        }
        self.bindable_strings
            .iter()
            .find(|bindable_string| bindable_string.has_data_bind_index(data_bind_index))
            .map(|bindable_string| bindable_string.value.as_slice())
    }

    pub fn default_view_model_color_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        self.data_bind_graph
            .default_view_model_color_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_color_value_for_data_bind(&self, data_bind_index: usize) -> Option<u32> {
        if let Some(value) = self
            .data_bind_graph
            .color_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_color_value(&self.bindable_colors, global_id))
        {
            return Some(value);
        }
        self.bindable_colors
            .iter()
            .find(|bindable_color| bindable_color.has_data_bind_index(data_bind_index))
            .map(|bindable_color| bindable_color.value)
    }

    pub fn default_view_model_enum_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_enum_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_enum_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .enum_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_enum_value(&self.bindable_enums, global_id))
        {
            return Some(value);
        }
        self.bindable_enums
            .iter()
            .find(|bindable_enum| bindable_enum.has_data_bind_index(data_bind_index))
            .map(|bindable_enum| bindable_enum.value)
    }

    pub fn default_view_model_symbol_list_index_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_symbol_list_index_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_integer_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .integer_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_integer_value(&self.bindable_integers, global_id))
        {
            return Some(value);
        }
        self.bindable_integers
            .iter()
            .find(|bindable_integer| bindable_integer.has_data_bind_index(data_bind_index))
            .map(|bindable_integer| bindable_integer.value)
    }

    pub fn default_view_model_asset_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_asset_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_asset_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .asset_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_asset_value(&self.bindable_assets, global_id))
        {
            return Some(value);
        }
        self.bindable_assets
            .iter()
            .find(|bindable_asset| bindable_asset.has_data_bind_index(data_bind_index))
            .map(|bindable_asset| bindable_asset.value.asset_index())
    }

    pub fn default_view_model_artboard_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_artboard_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_artboard_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .artboard_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_artboard_value(&self.bindable_artboards, global_id))
        {
            return Some(value);
        }
        self.bindable_artboards
            .iter()
            .find(|bindable_artboard| bindable_artboard.has_data_bind_index(data_bind_index))
            .map(|bindable_artboard| bindable_artboard.value)
    }

    pub fn default_view_model_trigger_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_trigger_source_value_for_data_bind(data_bind_index)
    }

    pub fn bindable_trigger_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .default_view_model_trigger_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_trigger_value(&self.bindable_triggers, global_id))
        {
            return Some(value);
        }
        self.bindable_triggers
            .iter()
            .find(|bindable_trigger| bindable_trigger.has_data_bind_index(data_bind_index))
            .map(|bindable_trigger| bindable_trigger.value)
    }

    fn set_key_frame_default_number_source_for_path(&mut self, path: &[u32], value: f32) -> bool {
        self.key_frame_data_bind_graphs
            .iter_mut()
            .flatten()
            .fold(false, |changed, graph| {
                graph.set_default_view_model_number_source_for_path(path, value) || changed
            })
    }

    fn set_key_frame_default_boolean_source_for_path(&mut self, path: &[u32], value: bool) -> bool {
        self.key_frame_data_bind_graphs
            .iter_mut()
            .flatten()
            .fold(false, |changed, graph| {
                graph.set_default_view_model_boolean_source_for_path(path, value) || changed
            })
    }

    fn set_key_frame_default_string_source_for_path(&mut self, path: &[u32], value: &[u8]) -> bool {
        self.key_frame_data_bind_graphs
            .iter_mut()
            .flatten()
            .fold(false, |changed, graph| {
                graph.set_default_view_model_string_source_for_path(path, value) || changed
            })
    }

    fn set_key_frame_default_color_source_for_path(&mut self, path: &[u32], value: u32) -> bool {
        self.key_frame_data_bind_graphs
            .iter_mut()
            .flatten()
            .fold(false, |changed, graph| {
                graph.set_default_view_model_color_source_for_path(path, value) || changed
            })
    }

    fn set_key_frame_active_source_for_path(
        &mut self,
        path: &[u32],
        value: RuntimeDataBindGraphValue,
    ) -> bool {
        self.key_frame_data_bind_graphs
            .iter_mut()
            .flatten()
            .fold(false, |changed, graph| {
                graph.set_active_view_model_source_for_path(path, value.clone()) || changed
            })
    }

    pub fn set_default_view_model_number_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_default_view_model_number_source_for_data_bind(data_bind_index, value);
        let key_frame_changed = path
            .is_some_and(|path| self.set_key_frame_default_number_source_for_path(&path, value));
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_number_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelNumberSourceHandle> {
        let path = runtime_default_view_model_number_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelNumberSourceHandle { path })
    }

    pub fn default_view_model_number_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelNumberSourceHandle> {
        let path =
            runtime_default_view_model_number_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelNumberSourceHandle { path })
    }

    pub fn set_default_view_model_number_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelNumberSourceHandle,
        value: f32,
    ) -> bool {
        let changed = self
            .data_bind_graph
            .set_default_view_model_number_source_for_path(&handle.path, value);
        let key_frame_changed =
            self.set_key_frame_default_number_source_for_path(&handle.path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_number_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: f32,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_number_property_path_for_name(file, property_name)
        else {
            return false;
        };
        let changed = self
            .data_bind_graph
            .set_default_view_model_number_source_for_path(&path, value);
        let key_frame_changed = self.set_key_frame_default_number_source_for_path(&path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_boolean_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: bool,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_boolean_property_path_for_name(file, property_name)
        else {
            return false;
        };
        let changed = self
            .data_bind_graph
            .set_default_view_model_boolean_source_for_path(&path, value);
        let key_frame_changed = self.set_key_frame_default_boolean_source_for_path(&path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_boolean_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelBooleanSourceHandle> {
        let path = runtime_default_view_model_boolean_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelBooleanSourceHandle { path })
    }

    pub fn default_view_model_boolean_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelBooleanSourceHandle> {
        let path =
            runtime_default_view_model_boolean_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelBooleanSourceHandle { path })
    }

    pub fn set_default_view_model_boolean_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelBooleanSourceHandle,
        value: bool,
    ) -> bool {
        let changed = self
            .data_bind_graph
            .set_default_view_model_boolean_source_for_path(&handle.path, value);
        let key_frame_changed =
            self.set_key_frame_default_boolean_source_for_path(&handle.path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_boolean_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_default_view_model_boolean_source_for_data_bind(data_bind_index, value);
        let key_frame_changed = path
            .is_some_and(|path| self.set_key_frame_default_boolean_source_for_path(&path, value));
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_string_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: &[u8],
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_string_property_path_for_name(file, property_name)
        else {
            return false;
        };
        let changed = self
            .data_bind_graph
            .set_default_view_model_string_source_for_path(&path, value);
        let key_frame_changed = self.set_key_frame_default_string_source_for_path(&path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_string_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelStringSourceHandle> {
        let path = runtime_default_view_model_string_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelStringSourceHandle { path })
    }

    pub fn default_view_model_string_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelStringSourceHandle> {
        let path =
            runtime_default_view_model_string_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelStringSourceHandle { path })
    }

    pub fn set_default_view_model_string_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelStringSourceHandle,
        value: &[u8],
    ) -> bool {
        let changed = self
            .data_bind_graph
            .set_default_view_model_string_source_for_path(&handle.path, value);
        let key_frame_changed =
            self.set_key_frame_default_string_source_for_path(&handle.path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_string_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_default_view_model_string_source_for_data_bind(data_bind_index, value);
        let key_frame_changed = path
            .is_some_and(|path| self.set_key_frame_default_string_source_for_path(&path, value));
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_color_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u32,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_color_property_path_for_name(file, property_name)
        else {
            return false;
        };
        let changed = self
            .data_bind_graph
            .set_default_view_model_color_source_for_path(&path, value);
        let key_frame_changed = self.set_key_frame_default_color_source_for_path(&path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_color_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelColorSourceHandle> {
        let path = runtime_default_view_model_color_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelColorSourceHandle { path })
    }

    pub fn default_view_model_color_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelColorSourceHandle> {
        let path =
            runtime_default_view_model_color_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelColorSourceHandle { path })
    }

    pub fn set_default_view_model_color_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelColorSourceHandle,
        value: u32,
    ) -> bool {
        let changed = self
            .data_bind_graph
            .set_default_view_model_color_source_for_path(&handle.path, value);
        let key_frame_changed =
            self.set_key_frame_default_color_source_for_path(&handle.path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_color_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u32,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_default_view_model_color_source_for_data_bind(data_bind_index, value);
        let key_frame_changed =
            path.is_some_and(|path| self.set_key_frame_default_color_source_for_path(&path, value));
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_enum_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_enum_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_enum_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_enum_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelEnumSourceHandle> {
        let path = runtime_default_view_model_enum_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelEnumSourceHandle { path })
    }

    pub fn default_view_model_enum_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelEnumSourceHandle> {
        let path =
            runtime_default_view_model_enum_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelEnumSourceHandle { path })
    }

    pub fn set_default_view_model_enum_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelEnumSourceHandle,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_enum_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_enum_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_enum_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_symbol_list_index_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) = runtime_default_view_model_symbol_list_index_property_path_for_name(
            file,
            property_name,
        ) else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_symbol_list_index_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_symbol_list_index_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelSymbolListIndexSourceHandle> {
        let path = runtime_default_view_model_symbol_list_index_property_path_for_name(
            file,
            property_name,
        )?;
        Some(RuntimeDefaultViewModelSymbolListIndexSourceHandle { path })
    }

    pub fn default_view_model_symbol_list_index_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelSymbolListIndexSourceHandle> {
        let path = runtime_default_view_model_symbol_list_index_property_path_for_name_path(
            file,
            property_path,
        )?;
        Some(RuntimeDefaultViewModelSymbolListIndexSourceHandle { path })
    }

    pub fn set_default_view_model_symbol_list_index_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelSymbolListIndexSourceHandle,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_symbol_list_index_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_symbol_list_index_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_symbol_list_index_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_asset_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_asset_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_asset_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_asset_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelAssetSourceHandle> {
        let path = runtime_default_view_model_asset_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelAssetSourceHandle { path })
    }

    pub fn default_view_model_asset_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelAssetSourceHandle> {
        let path =
            runtime_default_view_model_asset_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelAssetSourceHandle { path })
    }

    pub fn set_default_view_model_asset_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelAssetSourceHandle,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_asset_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_asset_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_asset_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_artboard_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_artboard_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_artboard_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_artboard_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_artboard_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_artboard_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelArtboardSourceHandle> {
        let path = runtime_default_view_model_artboard_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelArtboardSourceHandle { path })
    }

    pub fn default_view_model_artboard_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelArtboardSourceHandle> {
        let path =
            runtime_default_view_model_artboard_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelArtboardSourceHandle { path })
    }

    pub fn set_default_view_model_artboard_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelArtboardSourceHandle,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_artboard_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    fn set_default_view_model_trigger_cell_for_path(&mut self, path: &[u32], value: u64) -> bool {
        let Some(cell) = self
            .default_view_model_trigger_instance
            .as_ref()
            .and_then(|context| context.cell_for_source_path(path))
            .filter(|cell| matches!(cell.value(), RuntimeViewModelCellValue::Trigger(_)))
        else {
            return false;
        };
        if !cell.set_value(RuntimeViewModelCellValue::Trigger(value)) {
            return false;
        }
        self.data_bind_graph.collect_retained_source_dirt();
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.collect_retained_source_dirt();
        }
        true
    }

    pub fn set_default_view_model_trigger_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(path) = self
            .data_bind_graph
            .default_view_model_trigger_source_path_for_data_bind(data_bind_index)
        else {
            return false;
        };
        if !self.set_default_view_model_trigger_cell_for_path(&path, value) {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_trigger_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_trigger_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self.set_default_view_model_trigger_cell_for_path(&path, value) {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_trigger_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelTriggerSourceHandle> {
        let path = runtime_default_view_model_trigger_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelTriggerSourceHandle { path })
    }

    pub fn default_view_model_trigger_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelTriggerSourceHandle> {
        let path =
            runtime_default_view_model_trigger_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelTriggerSourceHandle { path })
    }

    pub fn set_default_view_model_trigger_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelTriggerSourceHandle,
        value: u64,
    ) -> bool {
        if !self.set_default_view_model_trigger_cell_for_path(&handle.path, value) {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_list_source_item_count_for_data_bind(
        &mut self,
        data_bind_index: usize,
        item_count: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_list_source_item_count_for_data_bind(
                data_bind_index,
                item_count,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_list_source_item_count_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        item_count: usize,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_list_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_list_source_item_count_for_path(&path, item_count)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_list_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelListSourceHandle> {
        let path = runtime_default_view_model_list_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelListSourceHandle { path })
    }

    pub fn default_view_model_list_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelListSourceHandle> {
        let path =
            runtime_default_view_model_list_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelListSourceHandle { path })
    }

    pub fn set_default_view_model_list_source_item_count_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelListSourceHandle,
        item_count: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_list_source_item_count_for_path(&handle.path, item_count)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_default_view_model_view_model_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_view_model_source_for_data_bind(data_bind_index, instance_index)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn relink_default_view_model_view_model_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_default_view_model_view_model_source_for_data_bind(
                data_bind_index,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn relink_default_view_model_view_model_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        instance_index: usize,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_view_model_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .relink_default_view_model_view_model_source_for_path(&path, instance_index)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn default_view_model_view_model_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelViewModelSourceHandle> {
        let path =
            runtime_default_view_model_view_model_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelViewModelSourceHandle { path })
    }

    pub fn default_view_model_view_model_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelViewModelSourceHandle> {
        let path =
            runtime_default_view_model_view_model_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelViewModelSourceHandle { path })
    }

    pub fn relink_default_view_model_view_model_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelViewModelSourceHandle,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_default_view_model_view_model_source_for_path(&handle.path, instance_index)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn relink_view_model_instance_view_model_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_view_model_instance_view_model_source_for_data_bind(
                data_bind_index,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn relink_imported_view_model_context_view_model_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_imported_view_model_context_view_model_source_for_data_bind(
                context,
                data_bind_index,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_number_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_imported_view_model_context_number_source_for_data_bind(
                context,
                data_bind_index,
                value,
            );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::Number(value),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_number_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_owned_view_model_context_number_source_for_data_bind(
                context,
                data_bind_index,
                value,
            );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::Number(value),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_symbol_list_index_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_symbol_list_index_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_boolean_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_owned_view_model_context_boolean_source_for_data_bind(
                context,
                data_bind_index,
                value,
            );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::Boolean(value),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_enum_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_enum_source_for_data_bind(context, data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_color_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u32,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_owned_view_model_context_color_source_for_data_bind(
                context,
                data_bind_index,
                value,
            );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::Color(value),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_string_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_owned_view_model_context_string_source_for_data_bind(
                context,
                data_bind_index,
                value,
            );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::String(value.to_vec()),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_trigger_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_trigger_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    fn fire_owned_view_model_context_trigger_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(source_path) = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index)
        else {
            return false;
        };
        let Some((&view_model_index, source_tail)) = source_path.split_first() else {
            return false;
        };
        if usize::try_from(view_model_index).ok() != Some(context.view_model_index) {
            return false;
        }
        let property_path = source_tail
            .iter()
            .map(|property_index| usize::try_from(*property_index).ok())
            .collect::<Option<Vec<_>>>();
        let Some(property_path) = property_path.filter(|path| !path.is_empty()) else {
            return false;
        };
        self.fire_owned_view_model_context_trigger_source_for_data_bind_at_property_path(
            context,
            data_bind_index,
            value,
            &property_path,
        )
    }

    fn fire_owned_view_model_context_trigger_source_for_data_bind_at_property_path(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
        property_path: &[usize],
    ) -> bool {
        let Some(bindable_trigger) = self
            .bindable_triggers
            .iter_mut()
            .find(|bindable_trigger| bindable_trigger.has_data_bind_index(data_bind_index))
        else {
            return false;
        };

        bindable_trigger.set_value(value);
        if !self
            .data_bind_graph
            .fire_owned_view_model_context_trigger_source_for_data_bind_at_property_path(
                context,
                data_bind_index,
                value,
                property_path,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_list_source_item_count_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        item_count: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_list_source_item_count_for_data_bind(
                context,
                data_bind_index,
                item_count,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_asset_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_asset_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_artboard_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_artboard_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_owned_view_model_context_view_model_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_view_model_source_for_data_bind(
                context,
                data_bind_index,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_boolean_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_imported_view_model_context_boolean_source_for_data_bind(
                context,
                data_bind_index,
                value,
            );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::Boolean(value),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_string_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_imported_view_model_context_string_source_for_data_bind(
                context,
                data_bind_index,
                value,
            );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::String(value.to_vec()),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_color_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u32,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_imported_view_model_context_color_source_for_data_bind(
                context,
                data_bind_index,
                value,
            );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::Color(value),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_enum_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_enum_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_symbol_list_index_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_symbol_list_index_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_asset_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_asset_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_artboard_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_artboard_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_trigger_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_trigger_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_imported_view_model_context_list_source_item_count_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        item_count: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_list_source_item_count_for_data_bind(
                context,
                data_bind_index,
                item_count,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn relink_view_model_instance_view_model_source_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_view_model_instance_view_model_source_by_property_name_path(
                file,
                property_path,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn bind_empty_data_context(&mut self) -> bool {
        self.clear_owned_view_model_context();
        if !self.data_bind_graph.bind_empty_data_context() {
            return false;
        }
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.bind_empty_data_context();
        }
        self.active_file_view_model_binding = None;
        self.needs_advance = true;
        true
    }

    pub fn bind_default_view_model_context(&mut self) -> bool {
        self.clear_owned_view_model_context();
        if !self.data_bind_graph.bind_default_view_model_context() {
            return false;
        }
        if let Some(context) = self.default_view_model_trigger_instance.as_ref() {
            self.data_bind_graph
                .bind_file_view_model_trigger_sources(context);
        }
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.bind_default_view_model_context();
            if let Some(context) = self.default_view_model_trigger_instance.as_ref() {
                graph.bind_file_view_model_trigger_sources(context);
            }
        }
        self.sync_bindable_font_assets_from_default_context();
        self.active_file_view_model_binding = self.default_view_model_index.map(|index| (index, 0));
        self.needs_advance = true;
        true
    }

    pub fn set_data_bind_formula_random_values(&mut self, values: &[f32]) {
        self.data_bind_graph.set_formula_random_values(values);
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.set_formula_random_values(values);
            graph.mark_default_view_model_bindings_dirty();
        }
    }

    pub fn data_bind_formula_random_call_count(&self) -> usize {
        self.data_bind_graph.formula_random_call_count()
    }

    /// Retained transition-duration DataBind occurrence count, in authored order.
    ///
    /// This is exposed for pinned-C++ differential evidence. Each occurrence
    /// has independent converter/source ownership even when several target the
    /// same transition.
    #[doc(hidden)]
    pub fn transition_duration_binding_count(&self) -> usize {
        self.transition_durations.len()
    }

    /// Current retained value of one authored transition-duration occurrence.
    #[doc(hidden)]
    pub fn transition_duration_binding_value(&self, index: usize) -> Option<f32> {
        self.transition_durations
            .get(index)
            .map(StateMachineTransitionDurationInstance::value)
    }

    pub fn bind_view_model_instance_context(
        &mut self,
        file: &RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
    ) -> bool {
        self.clear_owned_view_model_context();
        let Some(instance_cells) = self
            .file_view_model_instances
            .as_ref()
            .and_then(|catalog| catalog.instance(view_model_index, instance_index))
        else {
            return false;
        };
        if !self.data_bind_graph.bind_view_model_instance_context(
            file,
            view_model_index,
            instance_index,
            &instance_cells,
        ) {
            return false;
        }
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.bind_view_model_instance_context(
                file,
                view_model_index,
                instance_index,
                &instance_cells,
            );
        }
        self.sync_bindable_font_assets_from_imported_instance(
            file,
            view_model_index,
            instance_index,
        );
        self.active_file_view_model_binding = Some((view_model_index, instance_index));
        self.needs_advance = true;
        true
    }

    pub fn bind_imported_view_model_context(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeImportedViewModelInstanceContext,
    ) -> bool {
        let Some(instance) = self
            .file_view_model_instances
            .as_ref()
            .and_then(|catalog| catalog.instance(context.view_model_index, context.instance_index))
        else {
            return false;
        };
        if !context.adopt_file_trigger_instance(instance) {
            return false;
        }
        self.clear_owned_view_model_context();
        if !self
            .data_bind_graph
            .bind_imported_view_model_context(file, context)
        {
            return false;
        }
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.bind_imported_view_model_context(file, context);
        }
        self.sync_bindable_font_assets_from_imported_instance(
            file,
            context.view_model_index,
            context.instance_index,
        );
        self.active_file_view_model_binding =
            Some((context.view_model_index, context.instance_index));
        self.needs_advance = true;
        true
    }

    /// Snapshot an owned ViewModel context into this machine.
    ///
    /// ViewModel listeners dispatch their ordinary input/event actions, but an
    /// immutable borrow cannot receive listener-authored ViewModel writes. Use
    /// [`Self::bind_owned_view_model_context_mut`] or the owning artboard's
    /// context-aware advance API when those writes must update the host context.
    pub fn bind_owned_view_model_context(
        &mut self,
        context: &RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.clear_owned_view_model_handle();
        self.bind_owned_view_model_snapshot(context)
    }

    /// Bind and retain a shared owned view-model graph.
    ///
    /// Later mutations through any alias are refreshed at the next data
    /// context advance, so the state machine and host never fork identity.
    pub fn bind_owned_view_model_handle(&mut self, context: &RuntimeOwnedViewModelHandle) -> bool {
        let context = RuntimeOwnedViewModelContextHandle::root_without_file(context.clone());
        self.bind_owned_view_model_context_handle(&context)
    }

    pub fn bind_owned_view_model_context_handle(
        &mut self,
        context: &RuntimeOwnedViewModelContextHandle,
    ) -> bool {
        self.bind_owned_view_model_data_context(&RuntimeOwnedDataContext::from_context_handle(
            context,
        ))
    }

    /// Install a facade-supplied live context without eagerly walking any
    /// DataBind. The scripting facade then executes the one C++-ordered
    /// ordinary + cloned-ScriptedObject container before calling
    /// [`Self::finish_scripted_object_data_context_bind`].
    #[doc(hidden)]
    pub fn begin_scripted_object_data_context_bind(
        &mut self,
        context: &RuntimeOwnedViewModelHandle,
    ) -> bool {
        if self.script_error.is_some() {
            return false;
        }
        if self.scripted_facade_root_requires_rebind(Some(context)) {
            self.require_scripted_object_data_context_rebind();
        }
        self.active_file_view_model_binding = None;
        let data_context = self
            .owned_data_context
            .as_ref()
            .filter(|bound| bound.main_root_matches(context))
            .cloned()
            .unwrap_or_else(|| {
                let context_handle =
                    RuntimeOwnedViewModelContextHandle::root_without_file(context.clone());
                RuntimeOwnedDataContext::from_context_handle(&context_handle)
            });
        let identity_changed = self
            .owned_data_context
            .as_ref()
            .is_none_or(|bound| !bound.same_binding(&data_context));
        let structural_rebind = self
            .owned_view_model_rebind_sink
            .take_dirt()
            .contains(RuntimeCellDirt::BINDINGS);
        if !identity_changed && !structural_rebind && self.scripted_data_context_bind_complete {
            return false;
        }
        // C++ `dataContext()` clears every ListenerViewModel registration
        // before `internalDataContext()` enters the first DataBind/converter
        // callback (`state_machine_instance.cpp:2880-2913,2923-2933`).
        // Leaving the old cells attached until `finish_*` would let a
        // converter callback enqueue a report through the previous context.
        self.clear_view_model_listener_cell_bindings();
        self.owned_data_context = Some(data_context);
        self.scripted_data_context_bind_complete = false;
        if identity_changed {
            self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        }
        self.needs_advance = true;
        true
    }

    /// Begin a C++ `rebind()` against the exact retained DataContext.
    ///
    /// This is the no-root facade path: it must preserve authored local,
    /// global, scoped, and parent instances rather than rebuilding a
    /// main-only context from a convenience argument.
    #[doc(hidden)]
    pub fn begin_retained_scripted_object_data_context_rebind(&mut self) -> bool {
        if self.script_error.is_some() || self.owned_data_context.is_none() {
            return false;
        }
        let structural_rebind = self
            .owned_view_model_rebind_sink
            .take_dirt()
            .contains(RuntimeCellDirt::BINDINGS);
        if !structural_rebind && self.scripted_data_context_bind_complete {
            return false;
        }
        self.clear_view_model_listener_cell_bindings();
        self.scripted_data_context_bind_complete = false;
        self.needs_advance = true;
        true
    }

    /// Complete C++ `StateMachineInstance::internalDataContext` after the
    /// facade has walked every outer DataBind and converter occurrence.
    #[doc(hidden)]
    pub fn finish_scripted_object_data_context_bind(&mut self) -> bool {
        let Some(data_context) = self.owned_data_context.clone() else {
            return false;
        };
        #[cfg(test)]
        {
            self.owned_data_bind_context_bind_count += 1;
        }
        self.sync_bindable_font_assets_from_owned_data_context(&data_context);
        self.bind_view_model_listener_cells_for_data_context(&data_context);
        self.retain_owned_view_model_advance_context(&data_context);
        self.register_owned_view_model_rebind_dependents();
        self.scripted_data_context_bind_complete = true;
        self.needs_advance = true;
        true
    }

    #[doc(hidden)]
    pub fn bind_script_artboard_data_context(
        &mut self,
        context: &ScriptArtboardDataContext,
    ) -> bool {
        self.bind_owned_view_model_data_context(context.runtime_context())
    }

    fn bind_owned_view_model_snapshot(&mut self, context: &RuntimeOwnedViewModelInstance) -> bool {
        self.active_file_view_model_binding = None;
        let mut advance_context = RuntimeOwnedViewModelAdvanceContext::default();
        advance_context.extend(context);
        self.active_owned_view_model_advance_context = Some(advance_context);
        let mut changed = self.data_bind_graph.bind_owned_view_model_context(context);
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            changed |= graph.bind_owned_view_model_context(context);
        }
        self.sync_bindable_font_assets_from_owned_context(context);
        self.bind_view_model_listener_cells_for_context_chain(context, &[&[]]);
        if changed {
            self.needs_advance = true;
        }
        changed
    }

    /// Rebind an owned ViewModel context. Typed ViewModel-change listeners
    /// retain their condition cells here and dispatch at next-frame start.
    pub fn bind_owned_view_model_context_mut(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.active_file_view_model_binding = None;
        self.clear_owned_view_model_handle();
        let mut advance_context = RuntimeOwnedViewModelAdvanceContext::default();
        advance_context.extend(context);
        self.active_owned_view_model_advance_context = Some(advance_context);
        let mut changed = self.data_bind_graph.bind_owned_view_model_context(context);
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            changed |= graph.bind_owned_view_model_context(context);
        }
        self.sync_bindable_font_assets_from_owned_context(context);
        self.bind_view_model_listener_cells_for_context_chain(context, &[&[]]);
        if changed {
            self.needs_advance = true;
        }
        changed
    }

    fn bind_view_model_listener_cells_for_context_chain(
        &mut self,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) {
        let runtime_file = self.scripted_listener_runtime_file.as_deref();
        for (listener_index, listener) in self.view_model_listeners.iter_mut().enumerate() {
            let definition = &listener.listener_definitions[listener.listener_index];
            for binding in &mut listener.property_bindings {
                let path = match binding.source {
                    RuntimeViewModelListenerSource::Single => definition.view_model_path.as_ref(),
                    RuntimeViewModelListenerSource::Input(input_index) => definition
                        .view_model_input_types
                        .get(input_index)
                        .and_then(|input| input.path()),
                };
                let cell = path.and_then(|path| match path {
                    RuntimeListenerViewModelPath::Absolute {
                        view_model_index,
                        property_path,
                    } => context_chain.iter().find_map(|context_path| {
                        context.cell_by_scoped_property_path(
                            context_path,
                            *view_model_index,
                            property_path,
                        )
                    }),
                    RuntimeListenerViewModelPath::Relative {
                        resolved_name_ids,
                        absolute_fallback,
                    } => {
                        let file = runtime_file?;
                        if file.manifest().is_some() {
                            context_chain.iter().find_map(|context_path| {
                                let property_path = context
                                    .property_path_for_context_resolved_name_path(
                                        file,
                                        context_path,
                                        resolved_name_ids,
                                        false,
                                    )?;
                                context.cell_by_property_path(&property_path)
                            })
                        } else {
                            let (view_model_index, property_path) = absolute_fallback.as_ref()?;
                            context_chain.iter().find_map(|context_path| {
                                context.cell_by_scoped_property_path(
                                    context_path,
                                    *view_model_index,
                                    property_path,
                                )
                            })
                        }
                    }
                });
                relink_view_model_listener_cell(
                    binding,
                    cell,
                    &self.reported_listener_view_models,
                    listener_index,
                );
            }
        }
    }

    fn bind_view_model_listener_cells_for_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) {
        let runtime_file = self.scripted_listener_runtime_file.as_deref();
        for (listener_index, listener) in self.view_model_listeners.iter_mut().enumerate() {
            let definition = &listener.listener_definitions[listener.listener_index];
            for binding in &mut listener.property_bindings {
                let path = match binding.source {
                    RuntimeViewModelListenerSource::Single => definition.view_model_path.as_ref(),
                    RuntimeViewModelListenerSource::Input(input_index) => definition
                        .view_model_input_types
                        .get(input_index)
                        .and_then(|input| input.path()),
                };
                let cell = path.and_then(|path| {
                    let resolved = match path {
                        RuntimeListenerViewModelPath::Absolute {
                            view_model_index,
                            property_path,
                        } => {
                            let mut source_path = Vec::with_capacity(property_path.len() + 1);
                            source_path.push(u32::try_from(*view_model_index).ok()?);
                            source_path.extend(
                                property_path
                                    .iter()
                                    .copied()
                                    .map(u32::try_from)
                                    .collect::<Result<Vec<_>, _>>()
                                    .ok()?,
                            );
                            data_context.resolved_property_path(&source_path)
                        }
                        RuntimeListenerViewModelPath::Relative {
                            resolved_name_ids,
                            absolute_fallback,
                        } => {
                            let file = runtime_file?;
                            if file.manifest().is_some() {
                                data_context.resolved_property_path_for_resolved_name_path(
                                    file,
                                    resolved_name_ids,
                                )
                            } else {
                                let (view_model_index, property_path) =
                                    absolute_fallback.as_ref()?;
                                let mut source_path = Vec::with_capacity(property_path.len() + 1);
                                source_path.push(u32::try_from(*view_model_index).ok()?);
                                source_path.extend(
                                    property_path
                                        .iter()
                                        .copied()
                                        .map(u32::try_from)
                                        .collect::<Result<Vec<_>, _>>()
                                        .ok()?,
                                );
                                data_context.resolved_property_path(&source_path)
                            }
                        }
                    };
                    // C++ `DataContext::getViewModelProperty` returns the
                    // retained `ViewModelInstanceValue`; every authored
                    // ListenerInputTypeViewModel registers its own binding
                    // against the same parent ListenerViewModel
                    // (`state_machine_instance.cpp:1349-1372,1401-1407`).
                    resolved.and_then(|(context, property_path)| {
                        context.borrow().cell_by_property_path(&property_path)
                    })
                });
                relink_view_model_listener_cell(
                    binding,
                    cell,
                    &self.reported_listener_view_models,
                    listener_index,
                );
            }
        }
    }

    fn apply_listener_view_model_change_at_property_path(
        context: &mut RuntimeOwnedViewModelInstance,
        property_path: &[usize],
        value: &RuntimeListenerViewModelChangeValue,
        asset_value: Option<&RuntimeBindableAssetValue>,
    ) -> Option<bool> {
        match value {
            RuntimeListenerViewModelChangeValue::Number(value) => {
                context.number_value_by_property_path(property_path)?;
                Some(context.set_number_by_property_path(property_path, *value))
            }
            RuntimeListenerViewModelChangeValue::Integer(value) => {
                context.symbol_list_index_value_by_property_path(property_path)?;
                Some(context.set_symbol_list_index_by_property_path(property_path, *value))
            }
            RuntimeListenerViewModelChangeValue::Color(value) => {
                context.color_value_by_property_path(property_path)?;
                Some(context.set_color_by_property_path(property_path, *value))
            }
            RuntimeListenerViewModelChangeValue::String(value) => {
                context.string_value_by_property_path(property_path)?;
                Some(context.set_string_by_property_path(property_path, value))
            }
            RuntimeListenerViewModelChangeValue::Enum(value) => {
                context.enum_value_by_property_path(property_path)?;
                Some(context.set_enum_by_property_path(property_path, *value))
            }
            RuntimeListenerViewModelChangeValue::Asset(_) => {
                let asset_value = asset_value?;
                if context
                    .font_asset_value_by_property_path(property_path)
                    .is_some()
                {
                    let font_value = asset_value.font_data_bind_value().unwrap_or_else(|| {
                        RuntimeFontAssetValue::from_file_asset_index(asset_value.asset_index())
                    });
                    return Some(context.apply_font_asset_data_bind_value_by_property_path(
                        property_path,
                        &font_value,
                    ));
                }
                context.asset_value_by_property_path(property_path)?;
                Some(
                    context.set_asset_by_property_path(
                        property_path,
                        asset_value.data_bind_asset_index(),
                    ),
                )
            }
            RuntimeListenerViewModelChangeValue::Artboard(value) => {
                context.artboard_value_by_property_path(property_path)?;
                Some(context.set_artboard_by_property_path(property_path, *value))
            }
            RuntimeListenerViewModelChangeValue::Trigger(_) => None,
            RuntimeListenerViewModelChangeValue::Boolean(value) => {
                context.boolean_value_by_property_path(property_path)?;
                Some(context.set_boolean_by_property_path(property_path, *value))
            }
            RuntimeListenerViewModelChangeValue::List(value) => {
                Some(context.set_list_item_count_by_property_path(
                    property_path,
                    usize::try_from(*value).ok()?,
                ))
            }
            RuntimeListenerViewModelChangeValue::ViewModel(_) => None,
        }
    }

    fn perform_listener_view_model_change_for_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
        data_bind_index: usize,
        value: &RuntimeListenerViewModelChangeValue,
    ) -> bool {
        let Some(source_path) = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index)
        else {
            return false;
        };
        let asset_value =
            matches!(value, RuntimeListenerViewModelChangeValue::Asset(_)).then(|| {
                let RuntimeListenerViewModelChangeValue::Asset(fallback) = value else {
                    unreachable!("asset listener value was checked above")
                };
                self.listener_asset_value_for_data_bind(data_bind_index, fallback)
                    .clone()
            });

        let Some((context, property_path)) = data_context.resolved_property_path(&source_path)
        else {
            return false;
        };
        let mut context = context.borrow_mut();
        let changed = match value {
            RuntimeListenerViewModelChangeValue::Trigger(value) => Some(
                self.fire_owned_view_model_context_trigger_source_for_data_bind_at_property_path(
                    &mut context,
                    data_bind_index,
                    *value,
                    &property_path,
                ),
            ),
            _ => Self::apply_listener_view_model_change_at_property_path(
                &mut context,
                &property_path,
                value,
                asset_value.as_ref(),
            ),
        };
        changed.unwrap_or(false)
    }

    fn bind_owned_data_binds_from_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) -> bool {
        #[cfg(test)]
        {
            self.owned_data_bind_context_bind_count += 1;
        }
        let mut changed = self
            .data_bind_graph
            .bind_owned_view_model_data_context(data_context);
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            changed |= graph.bind_owned_view_model_data_context(data_context);
        }
        self.sync_bindable_font_assets_from_owned_data_context(data_context);
        changed
    }

    #[cfg(test)]
    pub(crate) fn owned_data_bind_context_bind_count(&self) -> usize {
        self.owned_data_bind_context_bind_count
    }

    fn perform_listener_actions_for_data_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        data_context: &RuntimeOwnedDataContext,
        listener_actions: &[RuntimeScheduledListenerAction],
        invocation: &ScriptListenerInvocation,
    ) -> Result<bool, ScriptError> {
        let mut changed = false;
        for action in listener_actions {
            if let RuntimeScheduledListenerAction::ViewModelChange(action) = action
                && let Some(bindable_global_id) = action.bindable_global_id
            {
                let value = {
                    let targets = RuntimeScheduledListenerActionTargetsMut {
                        inputs: &mut self.inputs,
                        reported_events: &mut self.reported_events,
                        bindable_numbers: &mut self.bindable_numbers,
                        bindable_integers: &mut self.bindable_integers,
                        bindable_colors: &mut self.bindable_colors,
                        bindable_strings: &mut self.bindable_strings,
                        bindable_enums: &mut self.bindable_enums,
                        bindable_assets: &mut self.bindable_assets,
                        bindable_artboards: &mut self.bindable_artboards,
                        bindable_lists: &mut self.bindable_lists,
                        bindable_triggers: &mut self.bindable_triggers,
                        bindable_view_models: &mut self.bindable_view_models,
                        bindable_booleans: &mut self.bindable_booleans,
                        transition_durations: &mut self.transition_durations,
                    };
                    action.occurrence_value(&targets, true)
                };
                let Some(value) = value else {
                    continue;
                };
                let source_changed = self
                    .data_bind_graph
                    .bindable_data_bind_to_source_index(bindable_global_id)
                    .is_some_and(|data_bind_index| {
                        self.perform_listener_view_model_change_for_data_context(
                            data_context,
                            data_bind_index,
                            &value,
                        )
                    });
                let target_dirtied = self
                    .data_bind_graph
                    .dirty_bindable_data_bind_to_target(bindable_global_id);
                changed |= source_changed || target_dirtied;
                // The retained source cell carries this mutation into the
                // next `updateDataBinds(false)` batch. Rebinding every source
                // here would spuriously reconcile unrelated two-way binds.
                continue;
            }
            changed |= self.perform_listener_actions(
                artboard,
                std::slice::from_ref(action),
                None,
                invocation,
                &mut NoopScriptHost,
            )?;
        }
        Ok(changed)
    }

    pub fn bind_owned_view_model_contexts(
        &mut self,
        context: &RuntimeOwnedViewModelContext,
    ) -> bool {
        self.bind_owned_view_model_data_context(&RuntimeOwnedDataContext::from_owned_context(
            context,
        ))
    }

    pub(crate) fn bind_owned_view_model_context_chain(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> bool {
        self.active_file_view_model_binding = None;
        self.clear_owned_view_model_handle();
        let mut advance_context = RuntimeOwnedViewModelAdvanceContext::default();
        advance_context.extend(context);
        self.active_owned_view_model_advance_context = Some(advance_context);
        let mut changed =
            self.data_bind_graph
                .bind_owned_view_model_context_chain(file, context, context_chain);
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            changed |= graph.bind_owned_view_model_context_chain(file, context, context_chain);
        }
        self.sync_bindable_font_assets_from_owned_context_chain(file, context, context_chain);
        self.bind_view_model_listener_cells_for_context_chain(context, context_chain);
        if changed {
            self.needs_advance = true;
        }
        changed
    }

    pub(crate) fn bind_owned_view_model_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) -> bool {
        self.active_file_view_model_binding = None;
        let identity_changed = self
            .owned_data_context
            .as_ref()
            .is_none_or(|bound| !bound.same_binding(data_context));
        let structural_rebind = self
            .owned_view_model_rebind_sink
            .take_dirt()
            .contains(RuntimeCellDirt::BINDINGS);
        if !identity_changed && !structural_rebind {
            // Explicit same-source bindFromContext is still C++
            // `DataBind::bind()`: the graph marks both supported directions
            // for reconcile in favor order (`data_bind_context.cpp:80-85`).
            // This path deliberately does not scan trigger values.
            self.scripted_data_context_bind_complete = false;
            let changed = self.bind_owned_data_binds_from_data_context(data_context);
            if changed {
                self.needs_advance = true;
            }
            return changed;
        }

        self.scripted_data_context_bind_complete = false;
        let changed = self.bind_owned_data_binds_from_data_context(data_context);
        self.bind_view_model_listener_cells_for_data_context(data_context);
        self.owned_data_context = Some(data_context.clone());
        self.retain_owned_view_model_advance_context(data_context);
        if identity_changed {
            self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
            self.register_owned_view_model_rebind_dependents();
        }
        if changed || identity_changed || structural_rebind {
            self.needs_advance = true;
        }
        changed || identity_changed || structural_rebind
    }

    fn register_owned_view_model_rebind_dependents(&self) {
        if let Some(data_context) = self.owned_data_context.as_ref() {
            data_context.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        }
    }

    fn retain_owned_view_model_advance_context(&mut self, data_context: &RuntimeOwnedDataContext) {
        if data_context.is_empty() {
            self.active_owned_view_model_advance_context = None;
            return;
        }
        let mut advance_context = RuntimeOwnedViewModelAdvanceContext::default();
        for context in data_context.root_handles() {
            advance_context.extend(&context.borrow());
        }
        self.active_owned_view_model_advance_context = Some(advance_context);
    }

    fn sync_bindable_font_assets<F>(&mut self, mut resolve: F)
    where
        F: FnMut(&RuntimeBindableAssetDefaultViewModelSource) -> Option<RuntimeFontAssetValue>,
    {
        for bindable in &mut self.bindable_assets {
            let value = bindable
                .default_view_model_sources
                .iter()
                .filter(|source| {
                    data_bind_flags_apply_source_to_target(source.flags)
                        && source.value.font_value().is_some()
                })
                .filter_map(&mut resolve)
                .last();
            if let Some(value) = value {
                bindable.apply_font_value(&value);
            }
        }
    }

    fn sync_bindable_font_assets_from_default_context(&mut self) {
        self.sync_bindable_font_assets(|source| source.value.font_value().cloned());
    }

    fn sync_bindable_font_assets_from_imported_instance(
        &mut self,
        file: &RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
    ) {
        let instance_object = file
            .view_model(view_model_index)
            .and_then(|view_model| view_model.instances.into_iter().nth(instance_index))
            .map(|instance| instance.object);
        self.sync_bindable_font_assets(|source| {
            let source_object =
                file.data_context_view_model_property_for_instance(instance_object?, &source.path)?;
            (source_object.type_name == "ViewModelInstanceAssetFont")
                .then(|| source_object.uint_property("propertyValue"))
                .flatten()
                .map(RuntimeFontAssetValue::from_file_asset_index)
        });
    }

    fn sync_bindable_font_assets_from_owned_context(
        &mut self,
        context: &RuntimeOwnedViewModelInstance,
    ) {
        self.sync_bindable_font_assets(|source| {
            runtime_owned_font_asset_value_for_state_machine_source(context, &source.path)
        });
    }

    fn sync_bindable_font_assets_from_owned_context_chain(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) {
        self.sync_bindable_font_assets(|source| {
            context_chain.iter().find_map(|context_path| {
                context.font_asset_value_by_context_source_path(
                    file,
                    context_path,
                    &source.path,
                    false,
                )
            })
        });
    }

    fn sync_bindable_font_assets_from_owned_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) {
        self.sync_bindable_font_assets(|source| {
            data_context.resolved_property_path(&source.path).and_then(
                |(context, property_path)| {
                    context
                        .borrow()
                        .font_asset_value_by_property_path(&property_path)
                },
            )
        });
    }

    pub fn advance_data_context(&mut self) -> bool {
        self.collect_retained_owned_view_model_dirt();
        if !self.data_bind_graph.data_context_present() {
            return false;
        }
        // Pinned `StateMachineInstance::advancedDataContext` only consumes the
        // live ViewModel trigger values through `DataContext::advanced`.
        // DataBindContainer work remains owned by `updateDataBinds` during an
        // ordinary advance or the explicit public update API; doing it here
        // incorrectly applies queued target edits before the trigger reset
        // (`state_machine_instance.cpp:2587-2593`).
        self.reset_advanced_data_context();
        true
    }

    pub(crate) fn reset_advanced_data_context(&mut self) {
        if !self.data_bind_graph.default_view_model_context_bound() {
            return;
        }
        let file_instance_advanced = self.active_file_view_model_binding.is_some_and(
            |(view_model_index, instance_index)| {
                self.file_view_model_instances
                    .as_ref()
                    .is_some_and(|catalog| {
                        catalog.advance_instance(view_model_index, instance_index)
                    })
            },
        );
        if file_instance_advanced {
            // C++ `ViewModelInstance::advanced()` walks the retained values;
            // the catalog precomputes the root's unique nested/list trigger
            // cells so cyclic authored topology stays allocation-free here.
        } else if let Some(context) = &self.active_owned_view_model_advance_context {
            context.advanced();
        }
        let mut changed = self.data_bind_graph.collect_retained_trigger_source_dirt();
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            changed |= graph.collect_retained_trigger_source_dirt();
        }
        // Cloned ScriptInput DataBinds belong to this same C++
        // DataBindContainer. `ViewModelInstanceTrigger::advanced()` dirties
        // them synchronously even though delegation is suppressed; fold that
        // retained-cell notification now so the next bounded outer pass can
        // project the target back to zero (`viewmodel_instance_trigger.cpp:
        // 20-27`; `state_machine_instance.cpp:2629-2647`).
        for binding in &mut self.scripted_object_bindings {
            changed |= binding.collect_source_dirt();
        }
        if changed {
            self.needs_advance = true;
        }
    }

    pub(crate) fn has_pending_data_bind_work(&self) -> bool {
        self.data_bind_container.has_pending_work()
    }

    /// Mirrors C++ `DataBindContainer::updateDataBinds(false)`. Dirty
    /// source-to-target values must be visible to event listeners and
    /// transition conditions without polling or writing target-to-source
    /// bindings.
    fn update_data_binds_false(
        &mut self,
        artboard: &ArtboardInstance,
        owned_context: Option<&RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<(), ScriptError> {
        // Retained cells cascade into per-bind sinks without borrowing this
        // machine. Fold that dirt before each applyEvents batch so listener-
        // authored chained writes have the same visibility as C++'s direct
        // `DataBind::addDirt` calls (`state_machine_instance.cpp:2328`).
        self.data_bind_graph.collect_retained_source_dirt();
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.collect_retained_source_dirt();
        }
        let occurrences = self.data_bind_occurrences.clone();
        let queued = {
            let graph = &self.data_bind_graph;
            let scripted = &self.scripted_object_bindings;
            self.data_bind_container
                .begin_update(|occurrence| match occurrences.get(occurrence) {
                    Some(RuntimeStateMachineDataBindOccurrence::Ordinary { data_bind_index }) => {
                        graph.data_bind_is_to_source(*data_bind_index)
                    }
                    Some(RuntimeStateMachineDataBindOccurrence::ScriptedObject {
                        action_binding_index,
                        input_index,
                    }) => scripted
                        .get(*action_binding_index)
                        .is_some_and(|binding| binding.data_bind_is_to_source(*input_index)),
                    None => false,
                })
        };
        let Some(queued) = queued else {
            return Ok(());
        };

        let file = artboard.runtime_file_arc();
        let resolver = self.scripted_listener_artboard_resolver.clone();
        let artboard_parent_context = self.scripted_listener_artboard_parent_context(None);
        let data_context = self.owned_data_context.clone();
        for (position, occurrence_index) in queued.iter().copied().enumerate() {
            self.data_bind_container.begin_occurrence(occurrence_index);
            let result = (|| -> Result<(), ScriptError> {
                let Some(occurrence) = occurrences.get(occurrence_index).copied() else {
                    return Ok(());
                };
                match occurrence {
                    RuntimeStateMachineDataBindOccurrence::Ordinary { data_bind_index } => {
                        if let Some(file) = file.as_ref() {
                            let mut apply =
                                |instance: &RuntimeScriptInstanceHandle,
                                 input_name: &ScriptCoreString,
                                 value: super::scripted_listener_action::RuntimeScriptedListenerBoundValue|
                                 -> Result<(), ScriptError> {
                                    apply_scripted_input_update(
                                        instance,
                                        input_name,
                                        value,
                                        resolver.as_deref(),
                                        artboard_parent_context.as_ref(),
                                        host,
                                    )?;
                                    Ok(())
                                };
                            if let Some(data_context) = data_context.as_ref() {
                                self.data_bind_graph
                                    .update_converter_data_binds_from_data_context_for_data_bind(
                                        data_bind_index,
                                        file,
                                        data_context,
                                        &mut apply,
                                    )?;
                            } else if let Some(owned_context) = owned_context {
                                self.data_bind_graph
                                    .update_converter_data_binds_for_data_bind(
                                        data_bind_index,
                                        file,
                                        owned_context,
                                        &mut apply,
                                    )?;
                            }
                        }
                        if self.data_bind_graph.default_view_model_context_bound() {
                            self.update_default_view_model_binding(
                                data_bind_index,
                                true,
                                RuntimeDataBindGraphApplyPhase::UpdateDataBindsFalse,
                            )?;
                        }
                    }
                    RuntimeStateMachineDataBindOccurrence::ScriptedObject {
                        action_binding_index,
                        input_index,
                    } => {
                        let Some(file) = file.as_ref() else {
                            return Ok(());
                        };
                        let Some(binding) =
                            self.scripted_object_bindings.get_mut(action_binding_index)
                        else {
                            return Ok(());
                        };
                        let owner_instance = self
                            .scripted_listener_action_instances
                            .get(&binding.action_global_id())
                            .cloned();
                        let mut apply =
                            |instance: &RuntimeScriptInstanceHandle,
                             input_name: &ScriptCoreString,
                             value: super::scripted_listener_action::RuntimeScriptedListenerBoundValue|
                             -> Result<(), ScriptError> {
                                apply_scripted_input_update(
                                    instance,
                                    input_name,
                                    value,
                                    resolver.as_deref(),
                                    artboard_parent_context.as_ref(),
                                    host,
                                )?;
                                Ok(())
                            };
                        if let Some(update) = binding.public_update_data_bind(
                            input_index,
                            file,
                            owner_instance.as_ref(),
                            false,
                            &mut apply,
                        )? && let Some(instance) = self
                            .scripted_listener_action_instances
                            .get(&update.action_global_id)
                            .cloned()
                        {
                            apply_scripted_input_update(
                                &instance,
                                &update.input_name,
                                update.value,
                                resolver.as_deref(),
                                artboard_parent_context.as_ref(),
                                host,
                            )?;
                        }
                    }
                }
                Ok(())
            })();
            if let Err(error) = result {
                self.data_bind_container
                    .abort_update(queued[position..].iter().copied());
                return Err(error);
            }
        }
        self.data_bind_container.finish_update();
        let _ = owned_context;
        Ok(())
    }

    fn collect_retained_owned_view_model_dirt(&mut self) -> bool {
        // Retained property cells refresh through their individual dirt
        // sinks. Structural ViewModel replacement separately pushes the
        // parent relay's DataContext-rebind sink; no root generation is
        // sampled or compared on the steady frame.
        let mut collected = self.data_bind_graph.collect_retained_source_dirt();
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            collected |= graph.collect_retained_source_dirt();
        }
        for binding in &mut self.scripted_object_bindings {
            collected |= binding.collect_source_dirt();
        }
        if self.owned_data_context.is_none() {
            if collected {
                self.needs_advance = true;
            }
            return collected;
        }
        let structural_rebind = self
            .owned_view_model_rebind_sink
            .take_dirt()
            .contains(RuntimeCellDirt::BINDINGS);
        if structural_rebind {
            // The legacy ordinary/keyframe walk consumed the shared
            // structural sink, but fixed ScriptedObject occurrences have not
            // crossed their C++ rebind yet. Keep that work pending for the
            // facade's source-corresponding pre-operation pass.
            self.scripted_data_context_bind_complete = false;
            let data_context = self
                .owned_data_context
                .clone()
                .expect("owned context checked above");
            self.bind_owned_data_binds_from_data_context(&data_context);
            self.bind_view_model_listener_cells_for_data_context(&data_context);
            self.retain_owned_view_model_advance_context(&data_context);
        }
        if collected || structural_rebind {
            self.needs_advance = true;
        }
        collected || structural_rebind
    }

    fn clear_owned_view_model_handle(&mut self) {
        self.owned_data_context = None;
        self.scripted_data_context_bind_complete = false;
        self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        self.active_owned_view_model_advance_context = None;
    }

    fn clear_owned_view_model_context(&mut self) {
        self.clear_owned_view_model_handle();
        // Leaving every owned-context path drops each listener's dependent
        // registration (C++ `ListenerViewModel::clearDataContext`). Owned
        // compatibility rebinds instead relink in place when identity is
        // unchanged.
        self.clear_view_model_listener_cell_bindings();
    }

    fn clear_view_model_listener_cell_bindings(&mut self) {
        for listener in &mut self.view_model_listeners {
            for binding in &mut listener.property_bindings {
                binding.cell_binding = None;
            }
        }
    }

    pub fn update_data_binds_apply_target_to_source(&mut self) -> bool {
        let has_bound_data_binds = self.data_bind_graph.data_context_present()
            || !self.scripted_object_bindings.is_empty();
        let occurrences = self.data_bind_occurrences.clone();
        let queued = {
            let graph = &self.data_bind_graph;
            let scripted = &self.scripted_object_bindings;
            self.data_bind_container
                .begin_update(|occurrence| match occurrences.get(occurrence) {
                    Some(RuntimeStateMachineDataBindOccurrence::Ordinary { data_bind_index }) => {
                        graph.data_bind_is_to_source(*data_bind_index)
                    }
                    Some(RuntimeStateMachineDataBindOccurrence::ScriptedObject {
                        action_binding_index,
                        input_index,
                    }) => scripted
                        .get(*action_binding_index)
                        .is_some_and(|binding| binding.data_bind_is_to_source(*input_index)),
                    None => false,
                })
        };
        let Some(queued) = queued else {
            return has_bound_data_binds;
        };

        let file = self.scripted_listener_runtime_file.clone();
        let resolver = self.scripted_listener_artboard_resolver.clone();
        let artboard_parent_context = self.scripted_listener_artboard_parent_context(None);
        let data_context = self.owned_data_context.clone();
        let mut host = NoopScriptHost;
        let mut changed = false;
        let mut result = Ok(());
        for (position, occurrence_index) in queued.iter().copied().enumerate() {
            self.data_bind_container.begin_occurrence(occurrence_index);
            let occurrence_result = (|| -> Result<(), ScriptError> {
                let Some(occurrence) = occurrences.get(occurrence_index).copied() else {
                    return Ok(());
                };
                match occurrence {
                    RuntimeStateMachineDataBindOccurrence::Ordinary { data_bind_index } => {
                        if let (Some(file), Some(data_context)) =
                            (file.as_ref(), data_context.as_ref())
                        {
                            let mut apply =
                                |instance: &RuntimeScriptInstanceHandle,
                                 input_name: &ScriptCoreString,
                                 value: super::scripted_listener_action::RuntimeScriptedListenerBoundValue|
                                 -> Result<(), ScriptError> {
                                    changed |= apply_scripted_input_update(
                                        instance,
                                        input_name,
                                        value,
                                        resolver.as_deref(),
                                        artboard_parent_context.as_ref(),
                                        &mut host,
                                    )?;
                                    Ok(())
                                };
                            self.data_bind_graph
                                .update_converter_data_binds_from_data_context_for_data_bind(
                                    data_bind_index,
                                    file,
                                    data_context,
                                    &mut apply,
                                )?;
                        }
                        if self.data_bind_graph.default_view_model_context_bound() {
                            self.public_update_default_view_model_binding(data_bind_index)?;
                        }
                    }
                    RuntimeStateMachineDataBindOccurrence::ScriptedObject {
                        action_binding_index,
                        input_index,
                    } => {
                        let Some(file) = file.as_ref() else {
                            return Ok(());
                        };
                        let Some(binding) =
                            self.scripted_object_bindings.get_mut(action_binding_index)
                        else {
                            return Ok(());
                        };
                        let owner_instance = self
                            .scripted_listener_action_instances
                            .get(&binding.action_global_id())
                            .cloned();
                        let mut apply =
                            |instance: &RuntimeScriptInstanceHandle,
                             input_name: &ScriptCoreString,
                             value: super::scripted_listener_action::RuntimeScriptedListenerBoundValue|
                             -> Result<(), ScriptError> {
                                changed |= apply_scripted_input_update(
                                    instance,
                                    input_name,
                                    value,
                                    resolver.as_deref(),
                                    artboard_parent_context.as_ref(),
                                    &mut host,
                                )?;
                                Ok(())
                            };
                        if let Some(update) = binding.public_update_data_bind(
                            input_index,
                            file,
                            owner_instance.as_ref(),
                            true,
                            &mut apply,
                        )? && let Some(instance) = self
                            .scripted_listener_action_instances
                            .get(&update.action_global_id)
                            .cloned()
                        {
                            changed |= apply_scripted_input_update(
                                &instance,
                                &update.input_name,
                                update.value,
                                resolver.as_deref(),
                                artboard_parent_context.as_ref(),
                                &mut host,
                            )?;
                        }
                    }
                }
                Ok(())
            })();
            if let Err(error) = occurrence_result {
                self.data_bind_container
                    .abort_update(queued[position..].iter().copied());
                result = Err(error);
                break;
            }
        }
        if result.is_ok() {
            self.data_bind_container.finish_update();
        }
        if let Err(error) = result {
            self.script_error = Some(error);
        }
        if changed {
            self.needs_advance = true;
        }
        has_bound_data_binds
    }

    pub fn current_animation_count(&self) -> usize {
        self.layers
            .iter()
            .filter(|layer| layer.has_current_animation())
            .count()
    }

    pub fn current_animation(&self, index: usize) -> Option<&LinearAnimationInstance> {
        self.layers
            .iter()
            .filter_map(StateMachineLayerInstance::current_animation)
            .nth(index)
    }

    pub fn reported_event_count(&self) -> usize {
        self.reported_events.len()
    }

    /// Whether retained ViewModel-listener mutations are queued for the next
    /// `applyEvents` frame. C++ includes this queue in both raw `advance` and
    /// `advanceAndApply` return values (`state_machine_instance.cpp:2583-2584,
    /// 2663-2665`).
    pub fn has_pending_listener_view_model_reports(&self) -> bool {
        !self.reported_listener_view_models.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn pending_listener_view_model_report_count(&self) -> usize {
        self.reported_listener_view_models.len()
    }

    /// Return one report after resolving its retained Event identity against
    /// the live Artboard occurrence, matching C++ `EventReport::event()`.
    pub fn reported_event(
        &mut self,
        artboard: &ArtboardInstance,
        index: usize,
    ) -> Option<&StateMachineReportedEvent> {
        let event = self.reported_events.get_mut(index)?;
        event.refresh_from_live_artboard(artboard);
        Some(event)
    }

    /// Immutable test/probe projection captured at the last live refresh.
    ///
    /// Runtime consumers must use [`Self::reported_event`] so the retained
    /// Event identity is resolved against its current Artboard occurrence.
    #[doc(hidden)]
    pub fn reported_event_snapshot(&self, index: usize) -> Option<&StateMachineReportedEvent> {
        self.reported_events.get(index)
    }

    pub(crate) fn next_unapplied_reported_event_index(&self) -> usize {
        self.reported_event_listener_index
            .min(self.reported_events.len())
    }

    pub(crate) fn discard_reported_event_prefix(&mut self, count: usize) {
        let count = count.min(self.reported_events.len());
        self.reported_events.drain(..count);
        self.reported_event_listener_index =
            self.reported_event_listener_index.saturating_sub(count);
        self.host_reported_event_index = self.host_reported_event_index.saturating_sub(count);
    }

    /// Drain reported events at a host operation seam without advancing.
    pub fn take_reported_events(
        &mut self,
        artboard: &ArtboardInstance,
    ) -> Vec<StateMachineReportedEvent> {
        let start = self
            .host_reported_event_index
            .min(self.reported_events.len());
        let events = &mut self.reported_events[start..];
        for event in events.iter_mut() {
            event.refresh_from_live_artboard(artboard);
        }
        let events = events.to_vec();
        self.host_reported_event_index = self.reported_events.len();
        events
    }

    pub fn view_model_trigger_count(&self, index: usize) -> Option<u64> {
        let trigger = self.default_view_model_triggers.get(index)?;
        let view_model_index = u32::try_from(self.default_view_model_index?).ok()?;
        let cell = self
            .default_view_model_trigger_instance
            .as_ref()?
            .cell_for_source_path(&[view_model_index, trigger.view_model_property_id])?;
        match cell.value() {
            RuntimeViewModelCellValue::Trigger(value) => Some(value),
            _ => None,
        }
    }

    /// #RB-1 e4 test seam: the retained cell a migrated listener condition is
    /// registered on, if any.
    #[cfg(test)]
    pub(crate) fn view_model_listener_condition_cell(
        &self,
        index: usize,
    ) -> Option<RuntimeViewModelCell> {
        self.view_model_listeners
            .get(index)?
            .property_bindings
            .first()?
            .cell_binding
            .as_ref()
            .map(|binding| binding.cell.clone())
    }

    pub fn view_model_trigger_value_count(&self) -> usize {
        self.default_view_model_triggers.len()
    }

    pub fn view_model_trigger_property_id(&self, index: usize) -> Option<u32> {
        self.default_view_model_triggers
            .get(index)
            .map(|trigger| trigger.view_model_property_id)
    }

    pub(crate) fn advance_with_owned_view_model_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        elapsed_seconds: f32,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.bind_owned_view_model_context_mut(context);
        let result = self.advance_with_report_mode(
            artboard,
            state_machine,
            elapsed_seconds,
            Some(context),
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    pub(crate) fn advance_preserving_reported_events(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        elapsed_seconds: f32,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        let result = self.advance_with_report_mode(
            artboard,
            state_machine,
            elapsed_seconds,
            owned_context,
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    pub(crate) fn advance_after_state_probe(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        elapsed_seconds: f32,
    ) -> bool {
        let probed_state_count = self.changed_state_count;
        let result = self.advance_with_report_mode(
            artboard,
            state_machine,
            elapsed_seconds,
            None,
            &mut NoopScriptHost,
        );
        let changed = self.retain_script_result(result);
        // The first state reported by the zero-delta apply is the transition
        // already counted by `try_change_state`. Keep the cumulative main and
        // outer count, then add only transitions chained after that duplicate.
        self.changed_state_count =
            probed_state_count.saturating_add(self.changed_state_count.saturating_sub(1));
        changed
    }

    /// Probe authored transitions without advancing or reapplying the current
    /// animations. This mirrors C++ `StateMachineInstance::tryChangeState`,
    /// which is used by the bounded outer update loop after component dirt can
    /// have changed data-bound transition inputs.
    pub(crate) fn try_change_state(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
    ) -> bool {
        let result =
            self.try_change_state_with_script_host(artboard, state_machine, &mut NoopScriptHost);
        self.retain_script_result(result)
    }

    pub(crate) fn try_change_state_with_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if let Some(error) = self.script_error.as_ref() {
            return Err(error.clone());
        }
        if self.scripted_data_context_prepare_pending() {
            return Ok(false);
        }
        self.post_update_probe_pending = false;
        if !self.focus.is_inert() {
            self.focus.sync(artboard);
        }
        self.collect_retained_owned_view_model_dirt();
        self.update_data_binds_false(artboard, None, host)?;

        let data_context_present = self.data_bind_graph.data_context_present();
        let file_data_context_instance =
            self.active_file_view_model_binding
                .and_then(|(view_model_index, instance_index)| {
                    self.file_view_model_instances
                        .as_ref()?
                        .instance(view_model_index, instance_index)
                });
        let mut changed_state = false;
        for (layer_index, layer) in state_machine
            .layers
            .iter()
            .enumerate()
            .take(self.layers.len())
        {
            let mut executor = RuntimeStateMachineListenerActionExecutor {
                needs_advance: &mut self.needs_advance,
                data_bind_graph: &mut self.data_bind_graph,
                data_bind_facilities_ready: true,
                owned_view_model_context: None,
                owned_data_context: self.owned_data_context.clone(),
                file_data_context_instance: file_data_context_instance.clone(),
                scripted_listener_action_instances: &self.scripted_listener_action_instances,
                scripted_instances_by_global: &self.scripted_instances_by_global,
                focus: &mut self.focus,
                host,
            };
            let layer_changed = self.layers[layer_index].update_state(
                artboard,
                layer,
                &self.key_frame_data_bind_graphs,
                data_context_present,
                layer_index,
                RuntimeScheduledListenerActionTargetsMut {
                    inputs: &mut self.inputs,
                    reported_events: &mut self.reported_events,
                    bindable_numbers: &mut self.bindable_numbers,
                    bindable_integers: &mut self.bindable_integers,
                    bindable_colors: &mut self.bindable_colors,
                    bindable_strings: &mut self.bindable_strings,
                    bindable_enums: &mut self.bindable_enums,
                    bindable_assets: &mut self.bindable_assets,
                    bindable_artboards: &mut self.bindable_artboards,
                    bindable_lists: &mut self.bindable_lists,
                    bindable_triggers: &mut self.bindable_triggers,
                    bindable_view_models: &mut self.bindable_view_models,
                    bindable_booleans: &mut self.bindable_booleans,
                    transition_durations: &mut self.transition_durations,
                },
                &mut executor,
            );
            drop(executor);
            let layer_changed = match layer_changed {
                Ok(layer_changed) => layer_changed,
                Err(error) => {
                    self.capture_focus_callbacks();
                    return Err(error);
                }
            };
            if layer_changed {
                changed_state = true;
                self.changed_state_count += 1;
            }
        }
        self.capture_focus_callbacks();
        if changed_state {
            // The zero-time advance following a successful probe must not be
            // elided by the steady-state fast path.
            self.needs_advance = true;
        }
        Ok(changed_state)
    }

    /// Whether C++'s one mandatory initial outer-update probe is still owed.
    pub(crate) fn post_update_probe_pending(&self) -> bool {
        self.post_update_probe_pending
    }

    pub(crate) fn schedule_post_update_probe(&mut self) {
        self.post_update_probe_pending = true;
    }

    fn advance_with_report_mode(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        elapsed_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if let Some(error) = self.script_error.as_ref() {
            return Err(error.clone());
        }
        if self.scripted_data_context_prepare_pending() {
            return Ok(false);
        }
        if !self.focus.is_inert() {
            self.focus.sync(artboard);
        }
        // A retained context can be mutated through any alias between
        // frames. Collect its pushed source dirt before the clean-frame fast
        // path so it schedules the ordinary updateDataBinds(false) work.
        self.collect_retained_owned_view_model_dirt();
        if self.has_advanced_once
            && elapsed_seconds == 0.0
            && !self.needs_advance
            && self.scripted_instances_by_global.is_empty()
        {
            self.changed_state_count = 0;
            return Ok(false);
        }
        self.has_advanced_once = true;
        self.changed_state_count = 0;
        self.needs_advance = false;
        // One C++ DataBindContainer pass owns both ordinary and cloned
        // ScriptInput occurrences before layer advancement. Do not run a
        // separate scripted full-list walk here: it loses the container's
        // authored cross-family partition/order.
        self.update_data_binds_false(artboard, owned_context.as_deref(), host)?;
        // C++ has no graph-wide Before/After sweep here. The pre-layer
        // selected false update above owns propagation; converter advance is
        // one post-layer DataBindContainer walk below.
        let data_context_present = self.data_bind_graph.data_context_present();
        let file_data_context_instance =
            self.active_file_view_model_binding
                .and_then(|(view_model_index, instance_index)| {
                    self.file_view_model_instances
                        .as_ref()?
                        .instance(view_model_index, instance_index)
                });
        let mut keep_going = false;
        for (layer_index, layer) in state_machine
            .layers
            .iter()
            .enumerate()
            .take(self.layers.len())
        {
            let mut executor = RuntimeStateMachineListenerActionExecutor {
                needs_advance: &mut self.needs_advance,
                data_bind_graph: &mut self.data_bind_graph,
                data_bind_facilities_ready: true,
                owned_view_model_context: owned_context.as_deref_mut(),
                owned_data_context: self.owned_data_context.clone(),
                file_data_context_instance: file_data_context_instance.clone(),
                scripted_listener_action_instances: &self.scripted_listener_action_instances,
                scripted_instances_by_global: &self.scripted_instances_by_global,
                focus: &mut self.focus,
                host: &mut *host,
            };
            let layer_result = {
                let layer_instance = &mut self.layers[layer_index];
                layer_instance.advance(
                    artboard,
                    layer,
                    &self.key_frame_data_bind_graphs,
                    elapsed_seconds,
                    data_context_present,
                    layer_index,
                    RuntimeScheduledListenerActionTargetsMut {
                        inputs: &mut self.inputs,
                        reported_events: &mut self.reported_events,
                        bindable_numbers: &mut self.bindable_numbers,
                        bindable_integers: &mut self.bindable_integers,
                        bindable_colors: &mut self.bindable_colors,
                        bindable_strings: &mut self.bindable_strings,
                        bindable_enums: &mut self.bindable_enums,
                        bindable_assets: &mut self.bindable_assets,
                        bindable_artboards: &mut self.bindable_artboards,
                        bindable_lists: &mut self.bindable_lists,
                        bindable_triggers: &mut self.bindable_triggers,
                        bindable_view_models: &mut self.bindable_view_models,
                        bindable_booleans: &mut self.bindable_booleans,
                        transition_durations: &mut self.transition_durations,
                    },
                    &mut executor,
                )
            };
            drop(executor);
            let layer_result = match layer_result {
                Ok(layer_result) => layer_result,
                Err(error) => {
                    self.capture_focus_callbacks();
                    return Err(error);
                }
            };
            if layer_result.changed_state {
                self.changed_state_count += 1;
            }
            keep_going |= layer_result.keep_going;
        }
        // `StateMachineInstance::advance` advances layers first, then every
        // retained DataBind converter. Converter dirt is consumed by the next
        // `updateDataBinds(false)` pass; raw advance does not perform a second
        // bind update after `advanceDataBinds`
        // (`state_machine_instance.cpp:2562-2574`).
        let mut data_bind_advance =
            crate::data_bind_graph::RuntimeDataBindGraphStatefulAdvance::default();
        for occurrence in self.data_bind_occurrences.clone() {
            let advance = match occurrence {
                RuntimeStateMachineDataBindOccurrence::Ordinary { data_bind_index } => self
                    .data_bind_graph
                    .advance_stateful_converter_with_scripted(
                        data_bind_index,
                        elapsed_seconds,
                        host,
                    )?,
                RuntimeStateMachineDataBindOccurrence::ScriptedObject {
                    action_binding_index,
                    input_index,
                } => match self.scripted_object_bindings.get_mut(action_binding_index) {
                    Some(binding) => {
                        binding.advance_stateful_converter(input_index, elapsed_seconds, host)?
                    }
                    None => Default::default(),
                },
            };
            data_bind_advance.changed |= advance.changed;
            data_bind_advance.keep_going |= advance.keep_going;
        }
        // A focus action performed by a state/transition entry callback calls
        // FocusListenerGroup immediately in C++. Retain that callback's
        // markNeedsAdvance result after the layer loop; queued actions execute
        // at the next new-frame boundary.
        self.capture_focus_callbacks();
        let focus_needs_advance = self.needs_advance;
        for input in &mut self.inputs {
            input.advanced();
        }
        self.needs_advance = focus_needs_advance
            || data_bind_advance.keep_going
            || keep_going
            || !self.reported_events.is_empty()
            || !self.reported_listener_view_models.is_empty();
        Ok(self.needs_advance)
    }

    fn apply_default_view_model_bindings(
        &mut self,
        include_view_models: bool,
        phase: RuntimeDataBindGraphApplyPhase,
    ) -> Result<(), ScriptError> {
        self.data_bind_graph.apply_default_view_model_bindings(
            RuntimeDataBindGraphTargetsMut {
                numbers: &mut self.bindable_numbers,
                integers: &mut self.bindable_integers,
                booleans: &mut self.bindable_booleans,
                strings: &mut self.bindable_strings,
                colors: &mut self.bindable_colors,
                enums: &mut self.bindable_enums,
                assets: &mut self.bindable_assets,
                artboards: &mut self.bindable_artboards,
                lists: &mut self.bindable_lists,
                triggers: &mut self.bindable_triggers,
                view_models: &mut self.bindable_view_models,
                transition_durations: &mut self.transition_durations,
                include_view_models,
            },
            phase,
        )
    }

    fn public_update_default_view_model_binding(
        &mut self,
        data_bind_index: usize,
    ) -> Result<(), ScriptError> {
        self.data_bind_graph
            .public_update_default_view_model_binding(
                data_bind_index,
                RuntimeDataBindGraphTargetsMut {
                    numbers: &mut self.bindable_numbers,
                    integers: &mut self.bindable_integers,
                    booleans: &mut self.bindable_booleans,
                    strings: &mut self.bindable_strings,
                    colors: &mut self.bindable_colors,
                    enums: &mut self.bindable_enums,
                    assets: &mut self.bindable_assets,
                    artboards: &mut self.bindable_artboards,
                    lists: &mut self.bindable_lists,
                    triggers: &mut self.bindable_triggers,
                    view_models: &mut self.bindable_view_models,
                    transition_durations: &mut self.transition_durations,
                    include_view_models: true,
                },
            )
    }

    fn update_default_view_model_binding(
        &mut self,
        data_bind_index: usize,
        include_view_models: bool,
        phase: RuntimeDataBindGraphApplyPhase,
    ) -> Result<(), ScriptError> {
        self.data_bind_graph.update_default_view_model_binding(
            data_bind_index,
            RuntimeDataBindGraphTargetsMut {
                numbers: &mut self.bindable_numbers,
                integers: &mut self.bindable_integers,
                booleans: &mut self.bindable_booleans,
                strings: &mut self.bindable_strings,
                colors: &mut self.bindable_colors,
                enums: &mut self.bindable_enums,
                assets: &mut self.bindable_assets,
                artboards: &mut self.bindable_artboards,
                lists: &mut self.bindable_lists,
                triggers: &mut self.bindable_triggers,
                view_models: &mut self.bindable_view_models,
                transition_durations: &mut self.transition_durations,
                include_view_models,
            },
            phase,
        )
    }
}

fn runtime_owned_font_asset_value_for_state_machine_source(
    context: &RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<RuntimeFontAssetValue> {
    if source_path.len() < 2 || usize::try_from(source_path[0]).ok()? != context.view_model_index {
        return None;
    }
    let property_path = source_path[1..]
        .iter()
        .map(|property_index| usize::try_from(*property_index).ok())
        .collect::<Option<Vec<_>>>()?;
    context.font_asset_value_by_property_path(&property_path)
}

#[cfg(test)]
mod view_model_listener_tests {
    use super::*;
    use crate::properties::property_key_for_name;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue, RuntimeFile};
    use nuxie_graph::GraphFile;

    #[test]
    fn one_listener_occurrence_binds_every_authored_view_model_source() {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("StateMachine", Vec::new()),
            record(
                "StateMachineListener",
                vec![uint_property("StateMachineListener", "targetId", 0)],
            ),
            record(
                "ListenerInputTypeViewModel",
                vec![
                    uint_property(
                        "ListenerInputTypeViewModel",
                        "listenerTypeValue",
                        RuntimeListenerType::ViewModel as u64,
                    ),
                    bytes_property("ListenerInputTypeViewModel", "viewModelPathIds", vec![0, 0]),
                ],
            ),
            record(
                "ListenerInputTypeViewModel",
                vec![
                    uint_property(
                        "ListenerInputTypeViewModel",
                        "listenerTypeValue",
                        RuntimeListenerType::ViewModel as u64,
                    ),
                    bytes_property("ListenerInputTypeViewModel", "viewModelPathIds", vec![0, 1]),
                ],
            ),
        ])
        .expect("view-model listener records import");
        let graph = GraphFile::from_runtime_file(&file).expect("listener graph builds");
        let authored = file.artboard_state_machine_graphs(0);
        let action_catalog = RuntimeFileStateMachineActionCatalog::new(&file);
        let action_owners = action_catalog
            .arena(authored[0].object.id)
            .expect("state-machine action owners");
        let definition = runtime_state_machine_listener(
            &file,
            graph.artboards.first().expect("artboard graph"),
            &authored[0].inputs,
            &[],
            &authored[0].listeners[0],
            &action_owners,
        )
        .expect("listener definition");
        let definitions = Arc::new(vec![definition]);
        let mut occurrence = RuntimeViewModelListenerInstance::new(Arc::clone(&definitions), 0)
            .expect("view-model listener occurrence");

        assert!(std::ptr::eq(occurrence.listener(), &definitions[0]));
        assert_eq!(occurrence.property_bindings.len(), 2);
        assert!(matches!(
            occurrence.property_bindings[0].source,
            RuntimeViewModelListenerSource::Input(0)
        ));
        assert!(matches!(
            occurrence.property_bindings[1].source,
            RuntimeViewModelListenerSource::Input(1)
        ));

        let queue = RuntimeCellNotificationQueue::default();
        let first = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(0.0));
        let second = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(0.0));
        relink_view_model_listener_cell(
            &mut occurrence.property_bindings[0],
            Some(first.clone()),
            &queue,
            0,
        );
        relink_view_model_listener_cell(
            &mut occurrence.property_bindings[1],
            Some(second.clone()),
            &queue,
            0,
        );

        assert!(first.set_value(RuntimeViewModelCellValue::Number(1.0)));
        assert!(second.set_value(RuntimeViewModelCellValue::Number(2.0)));
        let mut reporting = Vec::new();
        queue.swap_into(&mut reporting);

        // C++ has one ListenerViewModel parent and one property binding per
        // authored input. Either binding reports that same parent, preserving
        // mutation/FIFO order (`state_machine_instance.cpp:1324-1375,
        // 1377-1382,1454-1489,3021-3025`).
        assert_eq!(reporting, [0, 0]);
    }

    fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn uint_property(type_name: &str, name: &str, value: u64) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
            value: AuthoringValue::Uint(value),
        }
    }

    fn bytes_property(type_name: &str, name: &str, value: Vec<u8>) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
            value: AuthoringValue::Bytes(value),
        }
    }
}

#[cfg(test)]
mod scripted_listener_action_tests {
    use super::*;
    use crate::state_machine::focus_action_clear::RuntimeFocusActionClear;
    use crate::state_machine::focus_action_target::RuntimeFocusActionTarget;
    use crate::state_machine::focus_listener_group::RuntimeFocusListenerGroup;
    use crate::state_machine::gamepad_listener_group::RuntimeGamepadListenerGroup;
    use crate::state_machine::keyboard_listener_group::RuntimeKeyboardListenerGroup;
    use nuxie_binary::{
        AuthoringProperty, AuthoringRecord, AuthoringValue, RuntimeFile, read_runtime_file,
    };
    use nuxie_graph::GraphFile;
    use std::cell::{Cell, RefCell};
    use std::path::PathBuf;
    use std::rc::Rc;

    #[derive(Debug, Clone, PartialEq)]
    struct RecordedCall {
        label: &'static str,
        method: ScriptListenerActionMethod,
        invocation: ScriptListenerInvocation,
        state_before_call: usize,
    }

    struct RecordingListenerScript {
        label: &'static str,
        has_perform_action: bool,
        has_perform: bool,
        failure: ListenerFailure,
        state: usize,
        calls: Rc<RefCell<Vec<RecordedCall>>>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ListenerFailure {
        None,
        Ordinary,
        Terminal(&'static str),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ProjectionFailure {
        Ordinary,
        Resource,
    }

    struct InputProjectionScript {
        scalar_values: Rc<RefCell<Vec<(String, ScriptValue)>>>,
        trigger_calls: Rc<Cell<usize>>,
        trigger_failure: ProjectionFailure,
        artboard_widths: Rc<RefCell<Vec<f32>>>,
        lifetime_valid: bool,
    }

    impl ScriptInstance for InputProjectionScript {
        fn script_lifetime_valid(&self) -> bool {
            self.lifetime_valid
        }

        fn has_method(&self, _method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(false)
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn call_input_trigger(
            &mut self,
            _name: &str,
            _host: &mut dyn ScriptHost,
        ) -> Result<(), ScriptError> {
            self.trigger_calls.set(self.trigger_calls.get() + 1);
            match self.trigger_failure {
                ProjectionFailure::Ordinary => Err(ScriptError::new("ordinary trigger failure")),
                ProjectionFailure::Resource => Err(ScriptError::with_resource_code(
                    "terminal trigger resource failure",
                    "script.resource.test",
                )),
            }
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, name: &str, value: ScriptValue) -> Result<(), ScriptError> {
            self.scalar_values
                .borrow_mut()
                .push((name.to_owned(), value));
            Ok(())
        }

        fn set_artboard_input(
            &mut self,
            _name: &str,
            artboard: Box<dyn crate::ScriptArtboard>,
        ) -> Result<(), ScriptError> {
            self.artboard_widths.borrow_mut().push(artboard.width());
            Ok(())
        }
    }

    #[derive(Debug, Clone)]
    struct ProjectionArtboard {
        width: f32,
    }

    impl crate::ScriptArtboard for ProjectionArtboard {
        fn width(&self) -> f32 {
            self.width
        }

        fn height(&self) -> f32 {
            1.0
        }

        fn frame_origin(&self) -> bool {
            false
        }

        fn set_width(&mut self, width: f32) {
            self.width = width;
        }

        fn set_height(&mut self, _height: f32) {}

        fn set_frame_origin(&mut self, _frame_origin: bool) {}

        fn instance(
            &self,
            _view_model: Option<crate::ScriptViewModel>,
        ) -> Result<Box<dyn crate::ScriptArtboard>, ScriptError> {
            Ok(Box::new(self.clone()))
        }

        fn draw(
            &mut self,
            _factory: &mut dyn nuxie_render_api::Factory,
            _renderer: &mut dyn nuxie_render_api::Renderer,
        ) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct ProjectionArtboardResolver;

    impl ScriptArtboardResolver for ProjectionArtboardResolver {
        fn resolve_script_artboard(
            &self,
            artboard_id: u64,
            _parent_context: Option<&crate::ScriptArtboardParentContext>,
        ) -> Result<Box<dyn crate::ScriptArtboard>, ScriptError> {
            match artboard_id {
                7 => Ok(Box::new(ProjectionArtboard { width: 7.0 })),
                8 => Err(ScriptError::new("ordinary missing artboard")),
                _ => Err(ScriptError::with_resource_code(
                    "terminal artboard resource failure",
                    "script.resource.test",
                )),
            }
        }
    }

    struct HydrationTraceScript {
        trace: Rc<RefCell<Vec<String>>>,
        artboard_applied: Rc<Cell<bool>>,
    }

    impl ScriptInstance for HydrationTraceScript {
        fn set_context_view_model_chain(
            &mut self,
            _view_model: Option<ScriptViewModel>,
            _parents: Vec<ScriptViewModel>,
        ) -> Result<(), ScriptError> {
            self.trace.borrow_mut().push("context".to_owned());
            Ok(())
        }

        fn has_method(&self, _method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(false)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            if method == ScriptMethod::Init {
                self.trace.borrow_mut().push("init".to_owned());
                return Ok(ScriptValue::Bool(true));
            }
            Ok(ScriptValue::Nil)
        }

        fn user_init_pending(&mut self) -> Result<bool, ScriptError> {
            self.trace.borrow_mut().push("init-check".to_owned());
            Ok(true)
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            self.trace.borrow_mut().push(format!("set:{name}"));
            Ok(())
        }

        fn set_artboard_input(
            &mut self,
            name: &str,
            _artboard: Box<dyn crate::ScriptArtboard>,
        ) -> Result<(), ScriptError> {
            self.trace.borrow_mut().push(format!("set-artboard:{name}"));
            self.artboard_applied.set(true);
            Ok(())
        }
    }

    #[derive(Debug)]
    struct HydrationArtboardResolver {
        trace: Rc<RefCell<Vec<String>>>,
    }

    impl ScriptArtboardResolver for HydrationArtboardResolver {
        fn resolve_script_artboard(
            &self,
            artboard_id: u64,
            _parent_context: Option<&crate::ScriptArtboardParentContext>,
        ) -> Result<Box<dyn crate::ScriptArtboard>, ScriptError> {
            self.trace
                .borrow_mut()
                .push(format!("resolve-artboard:{artboard_id}"));
            if artboard_id == 7 {
                Ok(Box::new(ProjectionArtboard { width: 7.0 }))
            } else {
                Err(ScriptError::with_resource_code(
                    "terminal artboard resource failure",
                    "script.resource.test",
                ))
            }
        }
    }

    #[derive(Debug)]
    struct AfterArtboardViewModelResolver {
        trace: Rc<RefCell<Vec<String>>>,
        artboard_applied: Rc<Cell<bool>>,
    }

    impl crate::ScriptViewModelInputResolver for AfterArtboardViewModelResolver {
        fn resolve_script_view_model(
            &self,
            _input_global_id: u32,
            _path: &crate::ScriptInputViewModelPropertyPath,
        ) -> Result<Option<ScriptViewModel>, ScriptError> {
            self.trace
                .borrow_mut()
                .push("resolve-view-model".to_owned());
            assert!(
                self.artboard_applied.get(),
                "the earlier authored Artboard setter must run before the later ViewModel lookup"
            );
            Err(ScriptError::new("intentional late ViewModel miss"))
        }
    }

    #[derive(Debug)]
    struct NullViewModelResolver {
        trace: Rc<RefCell<Vec<String>>>,
    }

    impl crate::ScriptViewModelInputResolver for NullViewModelResolver {
        fn resolve_script_view_model(
            &self,
            _input_global_id: u32,
            _path: &crate::ScriptInputViewModelPropertyPath,
        ) -> Result<Option<ScriptViewModel>, ScriptError> {
            self.trace
                .borrow_mut()
                .push("resolve-null-view-model".to_owned());
            Ok(None)
        }
    }

    fn hydration_trace_machine(
        trace: &Rc<RefCell<Vec<String>>>,
        artboard_applied: &Rc<Cell<bool>>,
    ) -> (StateMachineInstance, u32) {
        let mut machine = scripted_listener_machine();
        let action_global_id = machine
            .scripted_listener_actions()
            .first()
            .expect("scripted listener fixture action")
            .action_global_id();
        machine
            .set_scripted_listener_action_instance(
                action_global_id,
                Box::new(HydrationTraceScript {
                    trace: Rc::clone(trace),
                    artboard_applied: Rc::clone(artboard_applied),
                }),
            )
            .expect("attach hydration trace script");
        (machine, action_global_id)
    }

    #[derive(Debug, Clone, PartialEq)]
    struct RecordedDrawableInputCall {
        label: &'static str,
        invocation: ScriptListenerInvocation,
    }

    struct RecordingDrawableInputScript {
        label: &'static str,
        methods: Vec<ScriptMethod>,
        handled: bool,
        calls: Rc<RefCell<Vec<RecordedDrawableInputCall>>>,
    }

    fn scripted_input_method(invocation: &ScriptListenerInvocation) -> Option<ScriptMethod> {
        match invocation {
            ScriptListenerInvocation::Keyboard { .. } => Some(ScriptMethod::KeyboardEvent),
            ScriptListenerInvocation::TextInput { .. } => Some(ScriptMethod::TextEvent),
            ScriptListenerInvocation::GamepadConnected { .. } => {
                Some(ScriptMethod::GamepadConnected)
            }
            ScriptListenerInvocation::GamepadEvent { .. } => Some(ScriptMethod::GamepadEvent),
            ScriptListenerInvocation::GamepadDisconnected { .. } => {
                Some(ScriptMethod::GamepadDisconnected)
            }
            ScriptListenerInvocation::Pointer { .. }
            | ScriptListenerInvocation::Focus { .. }
            | ScriptListenerInvocation::ReportedEvent { .. }
            | ScriptListenerInvocation::ViewModelChange { .. }
            | ScriptListenerInvocation::None
            | ScriptListenerInvocation::Semantic { .. } => None,
        }
    }

    struct ResourceFailingDrawableInputScript;

    impl ScriptInstance for ResourceFailingDrawableInputScript {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::KeyboardEvent)
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[crate::ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<crate::ScriptValue, ScriptError> {
            unreachable!("typed direct-input dispatch owns this callback")
        }

        fn call_scripted_drawable_input(
            &mut self,
            invocation: &ScriptListenerInvocation,
            _host: &mut dyn ScriptHost,
        ) -> Result<crate::ScriptedDrawableInputResult, ScriptError> {
            if scripted_input_method(invocation) != Some(ScriptMethod::KeyboardEvent) {
                return Ok(crate::ScriptedDrawableInputResult::default());
            }
            Err(ScriptError::with_resource_code(
                "script cycle exceeds 256 host commands",
                "script.resource.host_commands",
            ))
        }

        fn get_input(&self, _name: &str) -> Result<crate::ScriptValue, ScriptError> {
            Ok(crate::ScriptValue::Nil)
        }

        fn set_input(
            &mut self,
            _name: &str,
            _value: crate::ScriptValue,
        ) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    struct FailingDrawableInputScript {
        label: &'static str,
        methods: Vec<ScriptMethod>,
        resource_code: Option<&'static str>,
        calls: Rc<RefCell<Vec<RecordedDrawableInputCall>>>,
    }

    impl ScriptInstance for FailingDrawableInputScript {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(self.methods.contains(&method))
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[crate::ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<crate::ScriptValue, ScriptError> {
            unreachable!("typed direct-input dispatch owns this callback")
        }

        fn call_scripted_drawable_input(
            &mut self,
            invocation: &ScriptListenerInvocation,
            _host: &mut dyn ScriptHost,
        ) -> Result<crate::ScriptedDrawableInputResult, ScriptError> {
            if !scripted_input_method(invocation)
                .is_some_and(|method| self.methods.contains(&method))
            {
                return Ok(crate::ScriptedDrawableInputResult::default());
            }
            self.calls.borrow_mut().push(RecordedDrawableInputCall {
                label: self.label,
                invocation: invocation.clone(),
            });
            Err(match self.resource_code {
                Some(code) => ScriptError::with_resource_code("terminal resource fence", code),
                None => ScriptError::new("ordinary protected-call failure"),
            })
        }

        fn get_input(&self, _name: &str) -> Result<crate::ScriptValue, ScriptError> {
            Ok(crate::ScriptValue::Nil)
        }

        fn set_input(
            &mut self,
            _name: &str,
            _value: crate::ScriptValue,
        ) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for RecordingDrawableInputScript {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(self.methods.contains(&method))
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[crate::ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<crate::ScriptValue, ScriptError> {
            Err(ScriptError::new(
                "scripted drawable input must use the typed invocation seam",
            ))
        }

        fn call_scripted_drawable_input(
            &mut self,
            invocation: &ScriptListenerInvocation,
            _host: &mut dyn ScriptHost,
        ) -> Result<crate::ScriptedDrawableInputResult, ScriptError> {
            if !scripted_input_method(invocation)
                .is_some_and(|method| self.methods.contains(&method))
            {
                return Ok(crate::ScriptedDrawableInputResult::default());
            }
            self.calls.borrow_mut().push(RecordedDrawableInputCall {
                label: self.label,
                invocation: invocation.clone(),
            });
            let handled = matches!(
                invocation,
                ScriptListenerInvocation::GamepadConnected { .. }
                    | ScriptListenerInvocation::GamepadEvent { .. }
                    | ScriptListenerInvocation::GamepadDisconnected { .. }
            ) || self.handled;
            Ok(crate::ScriptedDrawableInputResult {
                invoked: true,
                handled,
            })
        }

        fn get_input(&self, _name: &str) -> Result<crate::ScriptValue, ScriptError> {
            Ok(crate::ScriptValue::Nil)
        }

        fn set_input(
            &mut self,
            _name: &str,
            _value: crate::ScriptValue,
        ) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for RecordingListenerScript {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(match method {
                ScriptMethod::PerformAction => self.has_perform_action,
                ScriptMethod::Perform => self.has_perform,
                _ => false,
            })
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[crate::ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<crate::ScriptValue, ScriptError> {
            Err(ScriptError::new(
                "listener dispatch must use the typed invocation seam",
            ))
        }

        fn call_listener_action(
            &mut self,
            method: ScriptListenerActionMethod,
            invocation: &ScriptListenerInvocation,
            _host: &mut dyn ScriptHost,
        ) -> Result<(), ScriptError> {
            self.calls.borrow_mut().push(RecordedCall {
                label: self.label,
                method,
                invocation: invocation.clone(),
                state_before_call: self.state,
            });
            self.state = self.state.checked_add(1).expect("bounded test call count");
            match self.failure {
                ListenerFailure::None => Ok(()),
                ListenerFailure::Ordinary => {
                    Err(ScriptError::new(format!("{} failed", self.label)))
                }
                ListenerFailure::Terminal(code) => Err(ScriptError::with_resource_code(
                    format!("{} exhausted a resource", self.label),
                    code,
                )),
            }
        }

        fn call_preferred_listener_action(
            &mut self,
            invocation: &ScriptListenerInvocation,
            host: &mut dyn ScriptHost,
        ) -> Result<bool, ScriptError> {
            let method = if self.has_perform_action {
                Some(ScriptListenerActionMethod::PerformAction)
            } else if self.has_perform {
                Some(ScriptListenerActionMethod::Perform)
            } else {
                None
            };
            let Some(method) = method else {
                return Ok(false);
            };
            self.call_listener_action(method, invocation, host)?;
            Ok(true)
        }

        fn get_input(&self, _name: &str) -> Result<crate::ScriptValue, ScriptError> {
            Ok(crate::ScriptValue::Nil)
        }

        fn set_input(
            &mut self,
            _name: &str,
            _value: crate::ScriptValue,
        ) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    #[test]
    fn scripted_input_scalar_trigger_and_artboard_projection_failures_match_cpp() {
        let scalar_values = Rc::new(RefCell::new(Vec::new()));
        let artboard_widths = Rc::new(RefCell::new(Vec::new()));
        let ordinary_trigger_calls = Rc::new(Cell::new(0));
        let ordinary = RuntimeScriptInstanceHandle::new(Box::new(InputProjectionScript {
            scalar_values: Rc::clone(&scalar_values),
            trigger_calls: Rc::clone(&ordinary_trigger_calls),
            trigger_failure: ProjectionFailure::Ordinary,
            artboard_widths: Rc::clone(&artboard_widths),
            lifetime_valid: true,
        }));
        let resolver = ProjectionArtboardResolver;
        let mut host = NoopScriptHost;

        for (name, value) in [
            ("enabled", ScriptValue::Bool(true)),
            ("amount", ScriptValue::Number(-0.0)),
            ("tint", ScriptValue::Color(0x1122_3344)),
            ("label", ScriptValue::String("ready".to_owned())),
        ] {
            assert!(
                apply_scripted_input_update(
                    &ordinary,
                    &ScriptCoreString::from(name),
                    crate::state_machine::RuntimeScriptedListenerBoundValue::Value(value),
                    Some(&resolver),
                    None,
                    &mut host,
                )
                .unwrap()
            );
        }
        assert_eq!(scalar_values.borrow().len(), 4);

        assert!(
            !apply_scripted_input_update(
                &ordinary,
                &ScriptCoreString::from("pulse"),
                crate::state_machine::RuntimeScriptedListenerBoundValue::Trigger(0),
                Some(&resolver),
                None,
                &mut host,
            )
            .unwrap(),
            "zero is not a trigger edge"
        );
        assert_eq!(ordinary_trigger_calls.get(), 0);
        assert!(
            !apply_scripted_input_update(
                &ordinary,
                &ScriptCoreString::from("pulse"),
                crate::state_machine::RuntimeScriptedListenerBoundValue::Trigger(1),
                Some(&resolver),
                None,
                &mut host,
            )
            .unwrap(),
            "an ordinary protected-call failure is swallowed and later inputs continue"
        );
        assert_eq!(ordinary_trigger_calls.get(), 1);

        assert!(
            apply_scripted_input_update(
                &ordinary,
                &ScriptCoreString::from("panel"),
                crate::state_machine::RuntimeScriptedListenerBoundValue::Artboard(7),
                Some(&resolver),
                None,
                &mut host,
            )
            .unwrap()
        );
        assert_eq!(&*artboard_widths.borrow(), &[7.0]);
        assert!(
            !apply_scripted_input_update(
                &ordinary,
                &ScriptCoreString::from("panel"),
                crate::state_machine::RuntimeScriptedListenerBoundValue::Artboard(8),
                Some(&resolver),
                None,
                &mut host,
            )
            .unwrap(),
            "an unresolved authored artboard leaves the prior table field untouched"
        );
        assert_eq!(&*artboard_widths.borrow(), &[7.0]);
        assert!(
            apply_scripted_input_update(
                &ordinary,
                &ScriptCoreString::from("after_panel"),
                crate::state_machine::RuntimeScriptedListenerBoundValue::Value(
                    ScriptValue::Number(5.0),
                ),
                Some(&resolver),
                None,
                &mut host,
            )
            .unwrap(),
            "an ordinary artboard resolution failure does not abort later authored inputs"
        );
        assert_eq!(scalar_values.borrow().last().unwrap().0, "after_panel");

        let resource_trigger_calls = Rc::new(Cell::new(0));
        let terminal = RuntimeScriptInstanceHandle::new(Box::new(InputProjectionScript {
            scalar_values: Rc::new(RefCell::new(Vec::new())),
            trigger_calls: Rc::clone(&resource_trigger_calls),
            trigger_failure: ProjectionFailure::Resource,
            artboard_widths: Rc::new(RefCell::new(Vec::new())),
            lifetime_valid: true,
        }));
        let trigger_error = apply_scripted_input_update(
            &terminal,
            &ScriptCoreString::from("pulse"),
            crate::state_machine::RuntimeScriptedListenerBoundValue::Trigger(1),
            Some(&resolver),
            None,
            &mut host,
        )
        .expect_err("typed resource failures are the Rust safety fence");
        assert_eq!(trigger_error.resource_code(), Some("script.resource.test"));
        assert_eq!(resource_trigger_calls.get(), 1);
        let artboard_error = apply_scripted_input_update(
            &terminal,
            &ScriptCoreString::from("panel"),
            crate::state_machine::RuntimeScriptedListenerBoundValue::Artboard(9),
            Some(&resolver),
            None,
            &mut host,
        )
        .expect_err("artboard construction resource failures remain terminal");
        assert_eq!(artboard_error.resource_code(), Some("script.resource.test"));

        let invalid_scalar_values = Rc::new(RefCell::new(Vec::new()));
        let invalid_trigger_calls = Rc::new(Cell::new(0));
        let invalid_artboard_widths = Rc::new(RefCell::new(Vec::new()));
        let invalid = RuntimeScriptInstanceHandle::new(Box::new(InputProjectionScript {
            scalar_values: Rc::clone(&invalid_scalar_values),
            trigger_calls: Rc::clone(&invalid_trigger_calls),
            trigger_failure: ProjectionFailure::Ordinary,
            artboard_widths: Rc::clone(&invalid_artboard_widths),
            lifetime_valid: false,
        }));
        for value in [
            crate::state_machine::RuntimeScriptedListenerBoundValue::Value(ScriptValue::Bool(true)),
            crate::state_machine::RuntimeScriptedListenerBoundValue::Trigger(1),
            crate::state_machine::RuntimeScriptedListenerBoundValue::Artboard(7),
        ] {
            assert!(
                !apply_scripted_input_update(
                    &invalid,
                    &ScriptCoreString::from("disposed"),
                    value,
                    Some(&resolver),
                    None,
                    &mut host,
                )
                .unwrap(),
                "a disposed C++ ScriptedObject has no state and rejects every ScriptInput update"
            );
        }
        assert!(invalid_scalar_values.borrow().is_empty());
        assert_eq!(invalid_trigger_calls.get(), 0);
        assert!(invalid_artboard_widths.borrow().is_empty());
    }

    #[test]
    fn scripted_hydration_validation_failure_applies_no_inputs_or_init() {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let artboard_applied = Rc::new(Cell::new(false));
        let (mut machine, action_global_id) = hydration_trace_machine(&trace, &artboard_applied);

        let error = machine
            .hydrate_and_initialize_scripted_listener_action_instance(
                action_global_id,
                crate::ScriptListenerActionHydration::new(None, Vec::new()),
                true,
                None,
                |_| {
                    trace.borrow_mut().push("validate".to_owned());
                    Err(ScriptError::new("intentional validation miss"))
                },
            )
            .expect_err("validation miss keeps the occurrence pending");

        assert_eq!(error.message(), "intentional validation miss");
        assert_eq!(
            trace.borrow().as_slice(),
            ["context", "validate"],
            "C++ installs the occurrence context before validation, but validation failure performs no input setter, resolver, or init work (`scripted_object.cpp:399-426`)"
        );
        assert!(!artboard_applied.get());
    }

    #[test]
    fn scripted_hydration_resolves_artboard_then_viewmodel_in_authored_apply_order() {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let artboard_applied = Rc::new(Cell::new(false));
        let (mut machine, action_global_id) = hydration_trace_machine(&trace, &artboard_applied);
        let artboard_resolver: Rc<dyn ScriptArtboardResolver> =
            Rc::new(HydrationArtboardResolver {
                trace: Rc::clone(&trace),
            });
        let view_model_resolver: Rc<dyn crate::ScriptViewModelInputResolver> =
            Rc::new(AfterArtboardViewModelResolver {
                trace: Rc::clone(&trace),
                artboard_applied: Rc::clone(&artboard_applied),
            });

        let error = machine
            .hydrate_and_initialize_scripted_listener_action_instance(
                action_global_id,
                crate::ScriptListenerActionHydration::new(None, Vec::new()),
                false,
                None,
                |_| {
                    trace.borrow_mut().push("validate".to_owned());
                    Ok(crate::ScriptListenerActionHydration::new(
                        None,
                        vec![
                            crate::ScriptListenerInputHydration::Artboard {
                                name: ScriptCoreString::from("panel"),
                                artboard_id: 7,
                                resolver: Rc::clone(&artboard_resolver),
                                parent_context: None,
                            },
                            crate::ScriptListenerInputHydration::ViewModel {
                                name: ScriptCoreString::from("child"),
                                input_global_id: 42,
                                path: crate::ScriptInputViewModelPropertyPath {
                                    path_ids: vec![1, 2],
                                    resolved_path_ids: vec![1, 2],
                                    is_relative: false,
                                },
                                resolver: Rc::clone(&view_model_resolver),
                            },
                        ],
                    ))
                },
            )
            .expect_err("the intentional late ViewModel miss ends phase two");

        assert_eq!(error.message(), "intentional late ViewModel miss");
        assert_eq!(
            trace.borrow().as_slice(),
            [
                "context",
                "validate",
                "resolve-artboard:7",
                "set-artboard:panel",
                "resolve-view-model",
            ],
            "phase two re-resolves each typed input at its authored position, so the later ViewModel lookup observes the earlier Artboard setter (`scripted_object.cpp:417-426`; `script_input_viewmodel_property.cpp:77-113`)"
        );
    }

    #[test]
    fn scripted_hydration_accepts_valid_null_viewmodel_and_continues_to_init() {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let artboard_applied = Rc::new(Cell::new(false));
        let (mut machine, action_global_id) = hydration_trace_machine(&trace, &artboard_applied);
        let resolver: Rc<dyn crate::ScriptViewModelInputResolver> =
            Rc::new(NullViewModelResolver {
                trace: Rc::clone(&trace),
            });

        let hydrated = machine
            .hydrate_and_initialize_scripted_listener_action_instance(
                action_global_id,
                crate::ScriptListenerActionHydration::new(None, Vec::new()),
                true,
                None,
                |_| {
                    trace.borrow_mut().push("validate".to_owned());
                    Ok(crate::ScriptListenerActionHydration::new(
                        None,
                        vec![
                            crate::ScriptListenerInputHydration::ViewModel {
                                name: ScriptCoreString::from("child"),
                                input_global_id: 42,
                                path: crate::ScriptInputViewModelPropertyPath {
                                    path_ids: vec![1, 2],
                                    resolved_path_ids: vec![1, 2],
                                    is_relative: false,
                                },
                                resolver: Rc::clone(&resolver),
                            },
                            crate::ScriptListenerInputHydration::Value {
                                name: ScriptCoreString::from("after"),
                                value: ScriptValue::Number(2.0),
                            },
                        ],
                    ))
                },
            )
            .expect("a valid nullable ViewModel property hydrates");

        assert!(hydrated);
        assert_eq!(
            trace.borrow().as_slice(),
            [
                "context",
                "validate",
                "resolve-null-view-model",
                "set:after",
                "init-check",
                "init",
            ],
            "C++ accepts the ViewModel-valued property cell, leaves its existing table field unchanged when referenceViewModelInstance is null, then hydrates later inputs and calls init (`script_input_viewmodel_property.cpp:60-113`; `scripted_object.cpp:399-426`)"
        );
    }

    #[test]
    fn scripted_hydration_typed_artboard_failure_stops_later_inputs_and_init() {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let artboard_applied = Rc::new(Cell::new(false));
        let (mut machine, action_global_id) = hydration_trace_machine(&trace, &artboard_applied);
        let artboard_resolver: Rc<dyn ScriptArtboardResolver> =
            Rc::new(HydrationArtboardResolver {
                trace: Rc::clone(&trace),
            });

        let error = machine
            .hydrate_and_initialize_scripted_listener_action_instance(
                action_global_id,
                crate::ScriptListenerActionHydration::new(None, Vec::new()),
                true,
                None,
                |_| {
                    trace.borrow_mut().push("validate".to_owned());
                    Ok(crate::ScriptListenerActionHydration::new(
                        None,
                        vec![
                            crate::ScriptListenerInputHydration::Value {
                                name: ScriptCoreString::from("before"),
                                value: ScriptValue::Number(1.0),
                            },
                            crate::ScriptListenerInputHydration::Artboard {
                                name: ScriptCoreString::from("panel"),
                                artboard_id: 9,
                                resolver: Rc::clone(&artboard_resolver),
                                parent_context: None,
                            },
                            crate::ScriptListenerInputHydration::Value {
                                name: ScriptCoreString::from("after"),
                                value: ScriptValue::Number(2.0),
                            },
                        ],
                    ))
                },
            )
            .expect_err("typed Artboard construction failure remains terminal");

        assert_eq!(error.resource_code(), Some("script.resource.test"));
        assert_eq!(
            trace.borrow().as_slice(),
            ["context", "validate", "set:before", "resolve-artboard:9"],
            "a phase-two failure preserves earlier authored writes but suppresses every later input and user init (`scripted_object.cpp:417-437`)"
        );
        assert!(!artboard_applied.get());
    }

    fn scripted_listener_artboard_and_machine() -> (ArtboardInstance, StateMachineInstance) {
        let fixture = PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets/scripted_listener_action.riv");
        let file = read_runtime_file(&std::fs::read(fixture).expect("read listener fixture"))
            .expect("import listener fixture");
        let graph = GraphFile::from_runtime_file(&file).expect("build listener graph");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &graph.artboards,
        )
        .expect("instantiate listener artboard");
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("fixture state machine");
        // This low-level fixture bypasses the facade that normally performs
        // C++'s complete synchronous ScriptedObject constructor pass before
        // exposing the instance. Treat the fixture as post-construction so
        // direct listener dispatch does not exercise Rust's otherwise
        // unobservable preparation seam.
        machine.mark_scripted_object_initialization_complete(None);
        (artboard, machine)
    }

    fn scripted_listener_machine() -> StateMachineInstance {
        scripted_listener_artboard_and_machine().1
    }

    fn reset_input_state_machine(
        listener_actions: Vec<RuntimeScheduledListenerAction>,
    ) -> RuntimeStateMachine {
        RuntimeStateMachine {
            global_id: 900,
            name: Some(Arc::from("reset inputs")),
            default_view_model_index: None,
            inputs: Arc::new(vec![
                Some(RuntimeStateMachineInput::new_bool(
                    901,
                    Some("enabled".to_owned()),
                    false,
                )),
                Some(RuntimeStateMachineInput::new_number(
                    902,
                    Some("amount".to_owned()),
                    0.0,
                )),
                Some(RuntimeStateMachineInput::new_trigger(
                    903,
                    Some("fire".to_owned()),
                )),
            ]),
            listeners: Arc::new(Vec::new()),
            layers: Arc::new(vec![RuntimeStateMachineLayer {
                global_id: 904,
                name: None,
                states: vec![RuntimeLayerState {
                    global_id: Some(905),
                    type_name: Some("EntryState"),
                    animation: None,
                    blend_state_1d: None,
                    blend_state_direct: None,
                    speed: 1.0,
                    flags: 0,
                    fire_actions: Vec::new(),
                    listener_actions,
                    transitions: Vec::new(),
                }],
                entry_state_index: Some(0),
                any_state_index: None,
                exit_state_index: None,
            }]),
            bindable_numbers: Arc::new(Vec::new()),
            bindable_integers: Arc::new(Vec::new()),
            bindable_colors: Arc::new(Vec::new()),
            bindable_strings: Arc::new(Vec::new()),
            bindable_enums: Arc::new(Vec::new()),
            bindable_assets: Arc::new(Vec::new()),
            bindable_artboards: Arc::new(Vec::new()),
            bindable_lists: Arc::new(Vec::new()),
            bindable_triggers: Arc::new(Vec::new()),
            bindable_view_models: Arc::new(Vec::new()),
            bindable_booleans: Arc::new(Vec::new()),
            view_model_triggers: Arc::new(Vec::new()),
            transition_duration_bindings: Arc::new(Vec::new()),
            data_bind_templates: Arc::new(Vec::new()),
            scripted_objects: Vec::new(),
            scripted_listener_actions: Vec::new(),
            scripted_object_bindings: Vec::new(),
            action_owners: RuntimeActionCoreArena::empty(),
        }
    }

    fn reset_input_actions() -> Vec<RuntimeScheduledListenerAction> {
        let direct = |index| RuntimeListenerInputTarget {
            direct_input_index: Some(index),
            nested_input_local_id: None,
        };
        vec![
            RuntimeScheduledListenerAction::BoolChange(RuntimeListenerBoolChange::for_test(
                StateMachineFireOccurrence::AtStart.value(),
                direct(0),
                1,
            )),
            RuntimeScheduledListenerAction::NumberChange(RuntimeListenerNumberChange::for_test(
                StateMachineFireOccurrence::AtStart.value(),
                direct(1),
                4.0,
            )),
            RuntimeScheduledListenerAction::TriggerChange(RuntimeListenerTriggerChange::for_test(
                StateMachineFireOccurrence::AtStart.value(),
                direct(2),
            )),
        ]
    }

    #[test]
    fn reset_state_marks_advance_only_for_genuine_entry_action_changes() {
        let (mut artboard, _) = scripted_listener_artboard_and_machine();

        artboard.state_machines = Arc::new(vec![reset_input_state_machine(Vec::new())]);
        let mut inert = artboard
            .state_machine_instance(0)
            .expect("inert reset state machine");
        inert.needs_advance = false;
        inert.reset_state(&mut artboard);
        assert!(
            !inert.needs_advance(),
            "StateMachineInstance::resetState itself does not call markNeedsAdvance"
        );

        artboard.state_machines = Arc::new(vec![reset_input_state_machine(reset_input_actions())]);
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("input reset state machine");
        assert_eq!(
            machine.input(0).and_then(|input| input.bool_value()),
            Some(true)
        );
        assert_eq!(
            machine.input(1).and_then(|input| input.number_value()),
            Some(4.0)
        );
        assert_eq!(
            machine.input(2).and_then(|input| input.trigger_fired()),
            Some(true)
        );

        machine.needs_advance = false;
        machine.reset_state(&mut artboard);
        assert!(
            !machine.needs_advance(),
            "equal bool/number writes and an already-fired trigger do not call SMIInput::valueChanged"
        );

        assert!(machine.set_bool(0, false));
        assert!(machine.set_number(1, 0.0));
        machine.inputs[2].advanced();
        machine.needs_advance = false;
        machine.reset_state(&mut artboard);
        assert!(
            machine.needs_advance(),
            "genuine bool/number/trigger entry-action changes call the owning StateMachineInstance::markNeedsAdvance"
        );
        assert_eq!(
            machine.input(0).and_then(|input| input.bool_value()),
            Some(true)
        );
        assert_eq!(
            machine.input(1).and_then(|input| input.number_value()),
            Some(4.0)
        );
        assert_eq!(
            machine.input(2).and_then(|input| input.trigger_fired()),
            Some(true)
        );
    }

    #[test]
    fn event_or_viewmodel_listener_excludes_other_constructor_groups() {
        // Pinned C++ continues the constructor loop immediately after either
        // report-only owner (`state_machine_instance.cpp:1829-1842`).
        assert!(listener_types_use_report_queue(&[
            RuntimeListenerType::Keyboard,
            RuntimeListenerType::Event,
            RuntimeListenerType::Focus,
        ]));
        assert!(listener_types_use_report_queue(&[
            RuntimeListenerType::Gamepad,
            RuntimeListenerType::ViewModel,
            RuntimeListenerType::SemanticAction,
        ]));
        assert!(!listener_types_use_report_queue(&[
            RuntimeListenerType::Keyboard,
            RuntimeListenerType::Focus,
            RuntimeListenerType::SemanticAction,
        ]));
    }

    #[test]
    fn malformed_local_event_listener_targeting_a_node_stays_blocked() {
        let (mut artboard, mut machine, _) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "event target",
                methods: Vec::new(),
                handled: false,
                calls: Rc::new(RefCell::new(Vec::new())),
            }));
        machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
            target_local_id: 1,
            is_single: false,
            listener_types: vec![RuntimeListenerType::Event],
            event_local_indices: vec![7],
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: vec![RuntimeScheduledListenerAction::FocusClear(
                RuntimeFocusActionClear::for_test(0),
            )],
        }]);
        let event = StateMachineReportedEvent {
            event_local_index: 7,
            event_core_type: 128,
            name: Some("local".to_owned()),
            url: None,
            target: None,
            properties: Vec::new(),
            string_properties: Vec::new(),
            seconds_delay: 0.0,
            context: None,
        };

        assert!(!machine.notify_events(&mut artboard, None, &[event]));
        assert!(
            machine.focus.target_has_focus(1),
            "pinned C++ does not deliver a local report to a listener whose target resolves to an ordinary Node"
        );
    }

    fn scripted_drawable_input_artboard_and_machine(
        script: Box<dyn ScriptInstance>,
    ) -> (ArtboardInstance, StateMachineInstance, u32) {
        scripted_drawable_subtype_input_artboard_and_machine("ScriptedDrawable", script)
    }

    fn scripted_drawable_subtype_input_artboard_and_machine(
        scripted_type_name: &str,
        script: Box<dyn ScriptInstance>,
    ) -> (ArtboardInstance, StateMachineInstance, u32) {
        scripted_drawable_subtype_input_artboard_and_machine_with_mount_order(
            scripted_type_name,
            script,
            true,
        )
    }

    fn scripted_drawable_subtype_input_artboard_and_machine_with_mount_order(
        scripted_type_name: &str,
        script: Box<dyn ScriptInstance>,
        mount_before_machine: bool,
    ) -> (ArtboardInstance, StateMachineInstance, u32) {
        scripted_drawable_subtype_input_artboard_and_machine_with_optional_script(
            scripted_type_name,
            Some(script),
            mount_before_machine,
        )
    }

    fn scripted_drawable_subtype_input_artboard_and_machine_with_optional_script(
        scripted_type_name: &str,
        script: Option<Box<dyn ScriptInstance>>,
        mount_before_machine: bool,
    ) -> (ArtboardInstance, StateMachineInstance, u32) {
        fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
            AuthoringRecord {
                type_key: nuxie_schema::definition_by_name(type_name)
                    .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                    .type_key
                    .int,
                properties,
            }
        }
        fn parent(type_name: &str, local_id: u64) -> AuthoringProperty {
            AuthoringProperty {
                key: crate::properties::property_key_for_name(type_name, "parentId")
                    .unwrap_or_else(|| panic!("missing {type_name}.parentId")),
                value: AuthoringValue::Uint(local_id),
            }
        }
        fn uint(type_name: &str, property_name: &str, value: u64) -> AuthoringProperty {
            AuthoringProperty {
                key: crate::properties::property_key_for_name(type_name, property_name)
                    .unwrap_or_else(|| panic!("missing {type_name}.{property_name}")),
                value: AuthoringValue::Uint(value),
            }
        }
        fn double(type_name: &str, property_name: &str, value: f32) -> AuthoringProperty {
            AuthoringProperty {
                key: crate::properties::property_key_for_name(type_name, property_name)
                    .unwrap_or_else(|| panic!("missing {type_name}.{property_name}")),
                value: AuthoringValue::Double(value),
            }
        }

        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record(
                scripted_type_name,
                vec![
                    parent(scripted_type_name, 0),
                    // Generated C++ `WorldTransformComponentBase` initializes
                    // `m_Opacity = 1`; make that omitted default explicit for
                    // this synthetic low-level record fixture.
                    double(scripted_type_name, "opacity", 1.0),
                ],
            ),
            record(
                "FocusData",
                vec![
                    parent("FocusData", 1),
                    // Generated C++ `FocusDataBase` initializes
                    // `m_FocusFlags = 7`; author it explicitly because this
                    // low-level synthetic record builder does not materialize
                    // omitted generated defaults.
                    uint("FocusData", "focusFlags", 7),
                ],
            ),
            record("SemanticData", vec![parent("SemanticData", 1)]),
            record("StateMachine", Vec::new()),
        ])
        .expect("scripted-input records import");
        let graph = GraphFile::from_runtime_file(&file).expect("scripted-input graph builds");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("scripted-input artboard"),
            &graph.artboards,
        )
        .expect("scripted-input artboard instantiates");
        let global_id = artboard
            .component(1)
            .expect("scripted drawable occurrence")
            .global_id;
        let mut script = script;
        if mount_before_machine && script.is_some() {
            artboard.set_script_instance_for_global(
                global_id,
                script.take().expect("script mounts exactly once"),
            );
        }
        // C++ constructs state-machine focus groups after the Artboard's
        // initial component update has produced world opacity. This synthetic
        // fixture bypasses the normal facade initialization, so perform that
        // same owner update explicitly.
        artboard.update_components();
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("scripted-input state machine");
        if !mount_before_machine && script.is_some() {
            artboard.set_script_instance_for_global(
                global_id,
                script.take().expect("script mounts exactly once"),
            );
        }
        assert!(machine.focus.set_focus_target(1));
        // The synthetic fixture is already at the post-constructor boundary:
        // its ScriptedDrawable table was mounted directly instead of through
        // the facade's synchronous C++ ScriptedObject initialization pass.
        machine.mark_scripted_object_initialization_complete(None);
        (artboard, machine, global_id)
    }

    #[test]
    fn facade_late_script_mount_completes_cpp_input_group_construction_once() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine, _) =
            scripted_drawable_subtype_input_artboard_and_machine_with_mount_order(
                "ScriptedDrawable",
                Box::new(RecordingDrawableInputScript {
                    label: "late",
                    methods: vec![ScriptMethod::KeyboardEvent, ScriptMethod::GamepadConnected],
                    handled: true,
                    calls: Rc::clone(&calls),
                }),
                false,
            );

        assert!(
            machine.key_input(&mut artboard, 66, 0, true, false),
            "first dispatch completes C++'s post-script input-group scan"
        );
        machine.synchronize_scripted_input_groups(&artboard);
        machine.synchronize_scripted_input_groups(&artboard);
        assert!(machine.gamepad_dispatch(
            &mut artboard,
            ScriptListenerInvocation::GamepadConnected {
                snapshot: ScriptGamepadSnapshot {
                    device_id: 7,
                    button_mask: 1,
                    button_values: vec![1.0],
                    axes: vec![0.25],
                    mapping: crate::ScriptGamepadMappingKind::Standard,
                },
            },
        ));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.invocation.clone())
                .collect::<Vec<_>>(),
            [
                ScriptListenerInvocation::Keyboard {
                    key: 66,
                    modifiers: 0,
                    is_pressed: true,
                    is_repeat: false,
                },
                ScriptListenerInvocation::GamepadConnected {
                    snapshot: ScriptGamepadSnapshot {
                        device_id: 7,
                        button_mask: 1,
                        button_values: vec![1.0],
                        axes: vec![0.25],
                        mapping: crate::ScriptGamepadMappingKind::Standard,
                    },
                },
            ],
            "idempotent completion must retain one authored input occurrence"
        );
    }

    #[test]
    fn replacing_scripted_input_occurrence_rebuilds_groups_without_duplicates() {
        let initial_calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine, global_id) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "initial",
                methods: vec![ScriptMethod::KeyboardEvent],
                handled: true,
                calls: Rc::clone(&initial_calls),
            }));

        let no_method_calls = Rc::new(RefCell::new(Vec::new()));
        artboard.set_script_instance_for_global(
            global_id,
            Box::new(RecordingDrawableInputScript {
                label: "no method",
                methods: Vec::new(),
                handled: true,
                calls: Rc::clone(&no_method_calls),
            }),
        );
        assert!(!machine.key_input(&mut artboard, 65, 0, true, false));
        assert!(no_method_calls.borrow().is_empty());

        let replacement_calls = Rc::new(RefCell::new(Vec::new()));
        artboard.set_script_instance_for_global(
            global_id,
            Box::new(RecordingDrawableInputScript {
                label: "replacement",
                methods: vec![ScriptMethod::KeyboardEvent],
                handled: true,
                calls: Rc::clone(&replacement_calls),
            }),
        );
        machine.synchronize_scripted_input_groups(&artboard);
        machine.synchronize_scripted_input_groups(&artboard);
        assert!(machine.key_input(&mut artboard, 66, 0, true, false));
        assert_eq!(
            replacement_calls.borrow().len(),
            1,
            "the replacement occurrence owns exactly one C++ KeyboardListenerGroup"
        );
        assert!(initial_calls.borrow().is_empty());
    }

    #[test]
    fn scripted_drawable_subtypes_register_keyboard_text_and_gamepad_paths() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine, _) = scripted_drawable_subtype_input_artboard_and_machine(
            "ScriptedLayout",
            Box::new(RecordingDrawableInputScript {
                label: "layout",
                methods: vec![
                    ScriptMethod::KeyboardEvent,
                    ScriptMethod::TextEvent,
                    ScriptMethod::GamepadConnected,
                ],
                handled: true,
                calls: Rc::clone(&calls),
            }),
        );

        assert!(machine.key_input(&mut artboard, 65, 0, true, false));
        assert!(machine.text_input(&mut artboard, "owned"));
        assert!(machine.gamepad_dispatch(
            &mut artboard,
            ScriptListenerInvocation::GamepadConnected {
                snapshot: ScriptGamepadSnapshot {
                    device_id: 7,
                    button_mask: 1,
                    button_values: vec![1.0],
                    axes: vec![0.25],
                    mapping: crate::ScriptGamepadMappingKind::Standard,
                },
            },
        ));
        assert_eq!(calls.borrow().len(), 3);
    }

    #[test]
    fn serialized_script_method_mask_controls_listenerless_input_membership() {
        let off_calls = Rc::new(RefCell::new(Vec::new()));
        let (mut off_artboard, mut off_machine, off_global_id) =
            scripted_drawable_subtype_input_artboard_and_machine(
                "ScriptedDrawable",
                Box::new(RecordingDrawableInputScript {
                    label: "mask off",
                    methods: vec![
                        ScriptMethod::KeyboardEvent,
                        ScriptMethod::TextEvent,
                        ScriptMethod::GamepadConnected,
                    ],
                    handled: true,
                    calls: Rc::clone(&off_calls),
                }),
            );
        off_artboard.set_script_instance_for_global_with_implemented_methods(
            off_global_id,
            Box::new(RecordingDrawableInputScript {
                label: "mask off",
                methods: vec![
                    ScriptMethod::KeyboardEvent,
                    ScriptMethod::TextEvent,
                    ScriptMethod::GamepadConnected,
                ],
                handled: true,
                calls: Rc::clone(&off_calls),
            }),
            0,
        );
        off_machine.synchronize_scripted_input_groups(&off_artboard);
        assert!(!off_machine.key_input(&mut off_artboard, 65, 0, true, false));
        assert!(!off_machine.text_input(&mut off_artboard, "masked"));
        assert!(!off_machine.gamepad_dispatch(
            &mut off_artboard,
            ScriptListenerInvocation::GamepadConnected {
                snapshot: ScriptGamepadSnapshot {
                    device_id: 7,
                    button_mask: 0,
                    button_values: Vec::new(),
                    axes: Vec::new(),
                    mapping: crate::ScriptGamepadMappingKind::Unknown,
                },
            },
        ));
        assert!(off_calls.borrow().is_empty());

        let missing_calls = Rc::new(RefCell::new(Vec::new()));
        let (mut missing_artboard, mut missing_machine, missing_global_id) =
            scripted_drawable_subtype_input_artboard_and_machine(
                "ScriptedDrawable",
                Box::new(RecordingDrawableInputScript {
                    label: "mask on, fields absent",
                    methods: Vec::new(),
                    handled: true,
                    calls: Rc::clone(&missing_calls),
                }),
            );
        let mask = crate::script_asset::RuntimeScriptImplementedMethods::KEYBOARD
            | crate::script_asset::RuntimeScriptImplementedMethods::TEXT
            | crate::script_asset::RuntimeScriptImplementedMethods::GAMEPAD_CONNECT;
        missing_artboard.set_script_instance_for_global_with_implemented_methods(
            missing_global_id,
            Box::new(RecordingDrawableInputScript {
                label: "mask on, fields absent",
                methods: Vec::new(),
                handled: true,
                calls: Rc::clone(&missing_calls),
            }),
            mask,
        );
        missing_machine.synchronize_scripted_input_groups(&missing_artboard);
        assert_eq!(missing_machine.keyboard_listener_groups.len(), 1);
        assert_eq!(missing_machine.gamepad_scripted_drawables.len(), 1);
        assert!(!missing_machine.key_input(&mut missing_artboard, 65, 0, true, false));
        assert!(!missing_machine.text_input(&mut missing_artboard, "missing"));
        assert!(!missing_machine.gamepad_dispatch(
            &mut missing_artboard,
            ScriptListenerInvocation::GamepadConnected {
                snapshot: ScriptGamepadSnapshot {
                    device_id: 7,
                    button_mask: 0,
                    button_values: Vec::new(),
                    axes: Vec::new(),
                    mapping: crate::ScriptGamepadMappingKind::Unknown,
                },
            },
        ));
        assert!(
            missing_calls.borrow().is_empty(),
            "C++ retains the group from serialized wants bits but missing Lua fields remain inert"
        );
    }

    fn nested_scripted_drawable_input_artboard_and_machine(
        ancestor_script: Box<dyn ScriptInstance>,
        leaf_script: Box<dyn ScriptInstance>,
    ) -> (ArtboardInstance, StateMachineInstance) {
        fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
            AuthoringRecord {
                type_key: nuxie_schema::definition_by_name(type_name)
                    .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                    .type_key
                    .int,
                properties,
            }
        }
        fn parent(type_name: &str, local_id: u64) -> AuthoringProperty {
            AuthoringProperty {
                key: crate::properties::property_key_for_name(type_name, "parentId")
                    .unwrap_or_else(|| panic!("missing {type_name}.parentId")),
                value: AuthoringValue::Uint(local_id),
            }
        }
        fn uint(type_name: &str, name: &str, value: u64) -> AuthoringProperty {
            AuthoringProperty {
                key: crate::properties::property_key_for_name(type_name, name)
                    .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
                value: AuthoringValue::Uint(value),
            }
        }
        fn double(type_name: &str, name: &str, value: f32) -> AuthoringProperty {
            AuthoringProperty {
                key: crate::properties::property_key_for_name(type_name, name)
                    .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
                value: AuthoringValue::Double(value),
            }
        }

        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record(
                "ScriptedDrawable",
                vec![
                    parent("ScriptedDrawable", 0),
                    double("ScriptedDrawable", "opacity", 1.0),
                ],
            ),
            record(
                "FocusData",
                vec![parent("FocusData", 1), uint("FocusData", "focusFlags", 7)],
            ),
            record(
                "ScriptedDrawable",
                vec![
                    parent("ScriptedDrawable", 1),
                    double("ScriptedDrawable", "opacity", 1.0),
                ],
            ),
            record(
                "FocusData",
                vec![parent("FocusData", 3), uint("FocusData", "focusFlags", 7)],
            ),
            record("StateMachine", Vec::new()),
        ])
        .expect("nested scripted-input records import");
        let graph =
            GraphFile::from_runtime_file(&file).expect("nested scripted-input graph builds");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph
                .artboards
                .first()
                .expect("nested scripted-input artboard"),
            &graph.artboards,
        )
        .expect("nested scripted-input artboard instantiates");
        let ancestor_global = artboard.component(1).expect("ancestor drawable").global_id;
        let leaf_global = artboard.component(3).expect("leaf drawable").global_id;
        artboard.set_script_instance_for_global(ancestor_global, ancestor_script);
        artboard.set_script_instance_for_global(leaf_global, leaf_script);
        artboard.update_components();
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("nested scripted-input state machine");
        assert!(machine.focus.set_focus_target(3));
        (artboard, machine)
    }

    #[test]
    fn listenerless_scripted_keyboard_and_text_dispatch_precede_listener_paths() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine, _) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "focused",
                methods: vec![ScriptMethod::KeyboardEvent, ScriptMethod::TextEvent],
                handled: true,
                calls: Rc::clone(&calls),
            }));

        assert!(machine.key_input(&mut artboard, 65, 3, true, true));
        assert!(machine.text_input(&mut artboard, "owned"));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.invocation.clone())
                .collect::<Vec<_>>(),
            [
                ScriptListenerInvocation::Keyboard {
                    key: 65,
                    modifiers: 3,
                    is_pressed: true,
                    is_repeat: true,
                },
                ScriptListenerInvocation::TextInput {
                    text: "owned".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn direct_scripted_input_retains_terminal_resource_failure() {
        let (mut artboard, mut machine, _) = scripted_drawable_input_artboard_and_machine(
            Box::new(ResourceFailingDrawableInputScript),
        );

        assert!(!machine.key_input(&mut artboard, 65, 0, true, false));
        assert_eq!(
            machine.script_error().and_then(ScriptError::resource_code),
            Some("script.resource.host_commands")
        );
    }

    #[test]
    fn terminal_resource_failure_stops_every_later_focused_input_callback() {
        let resource_code = "script.resource.host_commands";

        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
            Box::new(RecordingDrawableInputScript {
                label: "ancestor",
                methods: vec![ScriptMethod::KeyboardEvent],
                handled: true,
                calls: Rc::clone(&calls),
            }),
            Box::new(FailingDrawableInputScript {
                label: "leaf",
                methods: vec![ScriptMethod::KeyboardEvent],
                resource_code: Some(resource_code),
                calls: Rc::clone(&calls),
            }),
        );
        assert!(!machine.key_input(&mut artboard, 65, 0, true, false));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["leaf"]
        );
        assert_eq!(
            machine.script_error().and_then(ScriptError::resource_code),
            Some(resource_code)
        );

        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
            Box::new(RecordingDrawableInputScript {
                label: "ancestor",
                methods: vec![ScriptMethod::TextEvent],
                handled: true,
                calls: Rc::clone(&calls),
            }),
            Box::new(FailingDrawableInputScript {
                label: "leaf",
                methods: vec![ScriptMethod::TextEvent],
                resource_code: Some(resource_code),
                calls: Rc::clone(&calls),
            }),
        );
        assert!(!machine.text_input(&mut artboard, "owned"));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["leaf"]
        );
        assert_eq!(
            machine.script_error().and_then(ScriptError::resource_code),
            Some(resource_code)
        );

        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
            Box::new(FailingDrawableInputScript {
                label: "ancestor",
                methods: vec![ScriptMethod::GamepadConnected],
                resource_code: Some(resource_code),
                calls: Rc::clone(&calls),
            }),
            Box::new(RecordingDrawableInputScript {
                label: "leaf",
                methods: vec![ScriptMethod::GamepadConnected],
                handled: true,
                calls: Rc::clone(&calls),
            }),
        );
        assert!(!machine.gamepad_dispatch(
            &mut artboard,
            ScriptListenerInvocation::GamepadConnected {
                snapshot: ScriptGamepadSnapshot {
                    device_id: 7,
                    button_mask: 0,
                    button_values: Vec::new(),
                    axes: Vec::new(),
                    mapping: ScriptGamepadMappingKind::Standard,
                },
            },
        ));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["ancestor"]
        );
        assert_eq!(
            machine.script_error().and_then(ScriptError::resource_code),
            Some(resource_code)
        );
    }

    #[test]
    fn ordinary_protected_input_failure_continues_the_cpp_callback_order() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
            Box::new(RecordingDrawableInputScript {
                label: "ancestor",
                methods: vec![ScriptMethod::KeyboardEvent],
                handled: true,
                calls: Rc::clone(&calls),
            }),
            Box::new(FailingDrawableInputScript {
                label: "leaf",
                methods: vec![ScriptMethod::KeyboardEvent],
                resource_code: None,
                calls: Rc::clone(&calls),
            }),
        );
        assert!(machine.key_input(&mut artboard, 65, 0, true, false));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["leaf", "ancestor"]
        );
        assert!(machine.script_error().is_none());

        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
            Box::new(RecordingDrawableInputScript {
                label: "ancestor",
                methods: vec![ScriptMethod::TextEvent],
                handled: true,
                calls: Rc::clone(&calls),
            }),
            Box::new(FailingDrawableInputScript {
                label: "leaf",
                methods: vec![ScriptMethod::TextEvent],
                resource_code: None,
                calls: Rc::clone(&calls),
            }),
        );
        assert!(machine.text_input(&mut artboard, "owned"));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["leaf", "ancestor"]
        );
        assert!(machine.script_error().is_none());

        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
            Box::new(FailingDrawableInputScript {
                label: "ancestor",
                methods: vec![ScriptMethod::GamepadConnected],
                resource_code: None,
                calls: Rc::clone(&calls),
            }),
            Box::new(RecordingDrawableInputScript {
                label: "leaf",
                methods: vec![ScriptMethod::GamepadConnected],
                handled: true,
                calls: Rc::clone(&calls),
            }),
        );
        assert!(machine.gamepad_dispatch(
            &mut artboard,
            ScriptListenerInvocation::GamepadConnected {
                snapshot: ScriptGamepadSnapshot {
                    device_id: 7,
                    button_mask: 0,
                    button_values: Vec::new(),
                    axes: Vec::new(),
                    mapping: ScriptGamepadMappingKind::Standard,
                },
            },
        ));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["ancestor", "leaf"]
        );
        assert!(machine.script_error().is_none());
    }

    #[test]
    fn focused_keyboard_dispatch_bubbles_leaf_to_parent_and_stops_when_handled() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
            Box::new(RecordingDrawableInputScript {
                label: "ancestor",
                methods: vec![ScriptMethod::KeyboardEvent],
                handled: true,
                calls: Rc::clone(&calls),
            }),
            Box::new(RecordingDrawableInputScript {
                label: "leaf",
                methods: vec![ScriptMethod::KeyboardEvent],
                handled: true,
                calls: Rc::clone(&calls),
            }),
        );

        assert!(machine.key_input(&mut artboard, 65, 0, true, false));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["leaf"]
        );

        calls.borrow_mut().clear();
        let leaf_global = artboard.component(3).expect("leaf drawable").global_id;
        artboard.set_script_instance_for_global(
            leaf_global,
            Box::new(RecordingDrawableInputScript {
                label: "leaf",
                methods: vec![ScriptMethod::KeyboardEvent],
                handled: false,
                calls: Rc::clone(&calls),
            }),
        );
        assert!(machine.key_input(&mut artboard, 66, 0, true, false));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["leaf", "ancestor"]
        );
    }

    #[test]
    fn text_input_parent_precedes_scripted_and_listener_keyboard_dispatch() {
        fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
            AuthoringRecord {
                type_key: nuxie_schema::definition_by_name(type_name)
                    .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                    .type_key
                    .int,
                properties,
            }
        }
        fn property(type_name: &str, name: &str, value: AuthoringValue) -> AuthoringProperty {
            AuthoringProperty {
                key: crate::properties::property_key_for_name(type_name, name)
                    .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
                value,
            }
        }

        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record(
                "TextInput",
                vec![
                    property("TextInput", "parentId", AuthoringValue::Uint(0)),
                    property("TextInput", "opacity", AuthoringValue::Double(1.0)),
                    property("TextInput", "multiline", AuthoringValue::Bool(true)),
                    property(
                        "TextInput",
                        "text",
                        AuthoringValue::String("seed".to_owned()),
                    ),
                ],
            ),
            record(
                "FocusData",
                vec![
                    property("FocusData", "parentId", AuthoringValue::Uint(1)),
                    property("FocusData", "focusFlags", AuthoringValue::Uint(7)),
                ],
            ),
            record("StateMachine", Vec::new()),
        ])
        .expect("TextInput precedence records import");
        let graph = GraphFile::from_runtime_file(&file).expect("TextInput precedence graph builds");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph
                .artboards
                .first()
                .expect("TextInput precedence artboard"),
            &graph.artboards,
        )
        .expect("TextInput precedence artboard instantiates");
        artboard.update_components();
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("TextInput precedence state machine");
        assert!(machine.focus.set_focus_target(1));
        let listener = RuntimeStateMachineListener {
            target_local_id: 1,
            is_single: false,
            listener_types: vec![
                RuntimeListenerType::Keyboard,
                RuntimeListenerType::TextInput,
            ],
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: vec![RuntimeScheduledListenerAction::FocusClear(
                RuntimeFocusActionClear::for_test(0),
            )],
        };
        machine.keyboard_listener_groups = vec![
            RuntimeKeyboardListenerGroup::new(0, 2, &listener).expect("TextInput listener group"),
        ];
        machine.listener_definitions = Arc::new(vec![listener]);

        assert!(!machine.key_input(&mut artboard, 259, 0, true, false));
        assert!(!machine.focus.focused_listener_chain().is_empty());
        assert!(!machine.key_input(&mut artboard, 66, 0, true, false));
        assert!(!machine.focus.focused_listener_chain().is_empty());
        assert!(machine.text_input(&mut artboard, "owned"));
        assert!(!machine.focus.focused_listener_chain().is_empty());
    }

    #[test]
    fn mixed_report_listener_does_not_register_semantic_or_focus_groups() {
        let (mut artboard, mut machine, _) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "focused",
                methods: Vec::new(),
                handled: false,
                calls: Rc::new(RefCell::new(Vec::new())),
            }));
        machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
            target_local_id: 1,
            is_single: false,
            listener_types: vec![
                RuntimeListenerType::Event,
                RuntimeListenerType::SemanticAction,
                RuntimeListenerType::Focus,
                RuntimeListenerType::Keyboard,
                RuntimeListenerType::Gamepad,
                RuntimeListenerType::DragEnd,
            ],
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: Vec::new(),
        }]);
        machine.focus_listener_groups.clear();
        machine.keyboard_listener_groups.clear();
        machine.gamepad_listener_groups.clear();
        machine.semantic_listener_groups.clear();

        machine.initialize_listener_groups(&artboard);

        assert!(machine.focus_listener_groups.is_empty());
        assert!(
            machine
                .keyboard_listener_groups
                .iter()
                .all(|group| group.listener_index != Some(0)),
            "the mixed report listener must not register; an independent listener-less scripted group may still exist"
        );
        assert!(machine.gamepad_listener_groups.is_empty());
        assert!(machine.semantic_listener_groups.is_empty());

        machine
            .pointer_down_listener_hits
            .push(RuntimePointerDownListenerHit {
                pointer_id: 7,
                listener_index: 0,
                drag_phase: Some(RuntimePointerDragPhase::Dragging),
                event_context: None,
            });
        assert!(
            !machine
                .dispatch_captured_pointer_listener_type(
                    &mut artboard,
                    7,
                    RuntimeListenerType::DragEnd,
                    12.0,
                    34.0,
                    0.0,
                    None,
                    &mut NoopScriptHost,
                )
                .expect("mixed report listener dispatch")
        );
    }

    #[test]
    fn missing_pointer_hit_path_disables_only_pointer_not_focus_dispatch() {
        let (mut artboard, mut machine, _) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "mixed",
                methods: Vec::new(),
                handled: false,
                calls: Rc::new(RefCell::new(Vec::new())),
            }));
        machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
            target_local_id: 1,
            is_single: false,
            listener_types: vec![RuntimeListenerType::Down, RuntimeListenerType::Blur],
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: Vec::new(),
        }]);
        machine.focus_listener_groups.clear();
        machine.keyboard_listener_groups.clear();
        machine.gamepad_listener_groups.clear();
        machine.semantic_listener_groups.clear();
        machine.initialize_listener_groups(&artboard);

        assert!(
            !machine.pointer_down(&mut artboard, 0.0, 0.0, 1),
            "C++ retains the mixed listener but registers no pointer hit target"
        );
        assert!(
            machine.clear_focus(),
            "the independent focus channel remains registered"
        );
        assert_eq!(
            machine.queued_focus_events.len(),
            1,
            "C++ queues the matching non-pointer listener occurrence"
        );
    }

    #[test]
    fn semantic_callbacks_apply_constraints_preserve_duplicates_and_defer_actions() {
        fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
            AuthoringRecord {
                type_key: nuxie_schema::definition_by_name(type_name)
                    .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                    .type_key
                    .int,
                properties,
            }
        }
        fn uint(type_name: &str, name: &str, value: u64) -> AuthoringProperty {
            AuthoringProperty {
                key: crate::properties::property_key_for_name(type_name, name)
                    .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
                value: AuthoringValue::Uint(value),
            }
        }

        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("Node", vec![uint("Node", "parentId", 0)]),
            record("SemanticData", vec![uint("SemanticData", "parentId", 1)]),
            record("StateMachine", Vec::new()),
            record("StateMachineBool", Vec::new()),
            record(
                "StateMachineListener",
                vec![uint("StateMachineListener", "targetId", 1)],
            ),
            record(
                "ListenerInputTypeSemantic",
                vec![uint(
                    "ListenerInputTypeSemantic",
                    "listenerTypeValue",
                    RuntimeListenerType::SemanticAction as u64,
                )],
            ),
            record(
                "SemanticInput",
                vec![uint("SemanticInput", "actionType", 0)],
            ),
            record(
                "ListenerBoolChange",
                vec![
                    uint("ListenerBoolChange", "inputId", 0),
                    // Values other than 0/1 toggle in pinned C++.
                    uint("ListenerBoolChange", "value", 2),
                ],
            ),
        ])
        .expect("semantic listener records");
        let graph = GraphFile::from_runtime_file(&file).expect("semantic listener graph");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("semantic listener artboard"),
            &graph.artboards,
        )
        .expect("semantic listener instance");
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("semantic listener machine");

        assert!(
            !machine.semantic_action_for_target(1, 1),
            "a nonmatching action is not registered"
        );
        assert!(machine.semantic_action_for_target(1, 0));
        assert_eq!(
            machine
                .input(0)
                .and_then(StateMachineInputInstance::bool_value),
            Some(false),
            "C++ queues the callback instead of executing its actions inline"
        );
        assert!(machine.apply_local_event_listeners(&mut artboard, 0, None));
        assert_eq!(
            machine
                .input(0)
                .and_then(StateMachineInputInstance::bool_value),
            Some(true)
        );

        assert!(machine.semantic_action_for_target(1, 0));
        assert!(machine.semantic_action_for_target(1, 0));
        assert!(
            machine.apply_local_event_listeners(&mut artboard, 0, None),
            "both duplicate callback occurrences execute in FIFO order"
        );
        assert_eq!(
            machine
                .input(0)
                .and_then(StateMachineInputInstance::bool_value),
            Some(true),
            "two retained toggle callbacks leave the value unchanged"
        );
    }

    #[test]
    fn focus_listener_groups_queue_matching_duplicate_occurrences_in_registration_order() {
        let (mut artboard, mut machine, _) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "focused",
                methods: Vec::new(),
                handled: false,
                calls: Rc::new(RefCell::new(Vec::new())),
            }));
        // Discard the constructor fixture's unregistered initial focus event.
        machine.focus.take_owner_events();
        let listener = |listener_types| RuntimeStateMachineListener {
            target_local_id: 1,
            is_single: false,
            listener_types,
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: Vec::new(),
        };
        machine.listener_definitions = Arc::new(vec![
            listener(vec![RuntimeListenerType::Focus, RuntimeListenerType::Blur]),
            listener(vec![RuntimeListenerType::Focus, RuntimeListenerType::Blur]),
        ]);
        machine.focus_listener_groups = machine
            .listener_definitions
            .iter()
            .enumerate()
            .map(|(index, listener)| {
                RuntimeFocusListenerGroup::new(index, 2, listener).expect("focus listener group")
            })
            .collect();

        assert!(machine.focus.clear_focus());
        machine.capture_focus_callbacks();
        assert_eq!(
            machine.queued_focus_events,
            [
                ScriptListenerInvocation::Focus {
                    listener_index: 0,
                    is_focus: false,
                },
                ScriptListenerInvocation::Focus {
                    listener_index: 1,
                    is_focus: false,
                },
            ],
            "C++ queues one callback per registered group occurrence, in registration order"
        );
        machine.queued_focus_events.clear();

        assert!(machine.focus.set_focus_target(1));
        machine.capture_focus_callbacks();
        assert_eq!(
            machine.queued_focus_events,
            [
                ScriptListenerInvocation::Focus {
                    listener_index: 0,
                    is_focus: true,
                },
                ScriptListenerInvocation::Focus {
                    listener_index: 1,
                    is_focus: true,
                },
            ]
        );

        // Removing the occurrence-owned groups is Rust's exact registration
        // teardown boundary: later manager callbacks have no retained sink.
        machine.focus_listener_groups.clear();
        machine.queued_focus_events.clear();
        assert!(machine.focus.clear_focus());
        machine.capture_focus_callbacks();
        assert!(machine.queued_focus_events.is_empty());
        // Keep the artboard live through the teardown proof.
        artboard.update_components();
    }

    #[test]
    fn focus_action_marks_machine_only_through_a_registered_focus_callback() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine, _) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "focused",
                methods: Vec::new(),
                handled: false,
                calls,
            }));
        let action =
            RuntimeScheduledListenerAction::FocusClear(RuntimeFocusActionClear::for_test(0));
        let invocation = ScriptListenerInvocation::Keyboard {
            key: 0,
            modifiers: 0,
            is_pressed: true,
            is_repeat: false,
        };

        // FocusManager still changes focus when no group is registered, but
        // C++ has no queueFocusEvent callback and therefore does not mark the
        // owning StateMachineInstance.
        machine.focus_listener_groups.clear();
        machine.focus.take_owner_events();
        machine.needs_advance = false;
        assert!(
            machine
                .perform_listener_actions(
                    &mut artboard,
                    std::slice::from_ref(&action),
                    None,
                    &invocation,
                    &mut NoopScriptHost,
                )
                .expect("focus action")
        );
        assert!(!machine.needs_advance);
        assert!(machine.queued_focus_events.is_empty());

        assert!(machine.focus.set_focus_target(1));
        machine.focus.take_owner_events();
        machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
            target_local_id: 1,
            is_single: false,
            listener_types: vec![RuntimeListenerType::Focus, RuntimeListenerType::Blur],
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: Vec::new(),
        }]);
        machine.focus_listener_groups = vec![
            RuntimeFocusListenerGroup::new(0, 2, &machine.listener_definitions[0])
                .expect("focus group"),
        ];
        machine.needs_advance = false;
        assert!(
            machine
                .perform_listener_actions(
                    &mut artboard,
                    &[action],
                    None,
                    &invocation,
                    &mut NoopScriptHost,
                )
                .expect("focus action")
        );
        assert!(machine.needs_advance);
        assert_eq!(
            machine.queued_focus_events,
            [ScriptListenerInvocation::Focus {
                listener_index: 0,
                is_focus: false,
            }]
        );
    }

    #[test]
    fn completed_focus_callback_survives_a_later_terminal_action() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine, _) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "focused",
                methods: Vec::new(),
                handled: false,
                calls: Rc::new(RefCell::new(Vec::new())),
            }));
        machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
            target_local_id: 1,
            is_single: false,
            listener_types: vec![RuntimeListenerType::Focus, RuntimeListenerType::Blur],
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: Vec::new(),
        }]);
        machine.focus_listener_groups = vec![
            RuntimeFocusListenerGroup::new(0, 2, &machine.listener_definitions[0])
                .expect("focus group"),
        ];
        machine.focus.take_owner_events();
        machine.queued_focus_events.clear();
        let definition =
            ScriptListenerActionDefinition::new(777, 0, "terminal-after-focus".to_owned());
        machine.scripted_listener_action_definitions = vec![definition.clone()];
        machine
            .set_scripted_listener_action_instance(
                definition.action_global_id(),
                script(
                    "terminal",
                    true,
                    false,
                    ListenerFailure::Terminal("script.resource.host_commands"),
                    &calls,
                ),
            )
            .expect("attach terminal scripted action");
        let actions = [
            RuntimeScheduledListenerAction::FocusClear(RuntimeFocusActionClear::for_test(0)),
            RuntimeScheduledListenerAction::scripted_for_test(0, Some(definition)),
            RuntimeScheduledListenerAction::FocusTarget(RuntimeFocusActionTarget::for_test(
                0,
                Some(1),
            )),
        ];

        let error = machine
            .perform_listener_actions(
                &mut artboard,
                &actions,
                None,
                &ScriptListenerInvocation::None,
                &mut NoopScriptHost,
            )
            .expect_err("typed resource exhaustion remains terminal");

        assert_eq!(error.resource_code(), Some("script.resource.host_commands"));
        assert_eq!(
            machine.queued_focus_events,
            [ScriptListenerInvocation::Focus {
                listener_index: 0,
                is_focus: false,
            }],
            "the focus callback completed synchronously before the later action failed"
        );
        assert!(
            machine.focus.focused_listener_chain().is_empty(),
            "the action after the terminal fence must not run"
        );
        assert_eq!(calls.borrow().len(), 1);
    }

    #[test]
    fn gamepad_broadcast_uses_authored_script_identity_once() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine, global_id) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "focused",
                methods: vec![ScriptMethod::GamepadEvent],
                // C++ gamepad methods do not return a handled boolean. The
                // method's existence makes dispatch handled even if Rust's
                // shared test seam supplies false.
                handled: false,
                calls: Rc::clone(&calls),
            }));
        assert_eq!(
            machine
                .gamepad_scripted_drawables
                .iter()
                .map(|scripted| scripted.global_id)
                .collect::<Vec<_>>(),
            [global_id]
        );
        let invocation = ScriptListenerInvocation::GamepadEvent {
            full_state: ScriptGamepadSnapshot {
                device_id: 7,
                button_mask: 2,
                button_values: vec![0.0, 0.75],
                axes: vec![-0.5],
                mapping: crate::ScriptGamepadMappingKind::Standard,
            },
            change: crate::ScriptGamepadInputChange::Button {
                index: 1,
                value: 0.75,
            },
            standard_button_intent: Some(1),
            standard_axis_intent: None,
        };

        assert!(machine.gamepad_dispatch(&mut artboard, invocation.clone()));
        assert_eq!(
            calls.borrow().as_slice(),
            [RecordedDrawableInputCall {
                label: "focused",
                invocation
            }]
        );

        assert!(!machine.gamepad_dispatch(
            &mut artboard,
            ScriptListenerInvocation::GamepadConnected {
                snapshot: ScriptGamepadSnapshot {
                    device_id: 7,
                    button_mask: 0,
                    button_values: Vec::new(),
                    axes: Vec::new(),
                    mapping: crate::ScriptGamepadMappingKind::Standard,
                },
            },
        ));
        assert_eq!(
            calls.borrow().len(),
            1,
            "C++ broadcasts only the invocation methods the scripted drawable declared"
        );

        let event = ScriptListenerInvocation::GamepadEvent {
            full_state: ScriptGamepadSnapshot {
                device_id: 7,
                button_mask: 2,
                button_values: vec![0.0, 0.75],
                axes: vec![-0.5],
                mapping: crate::ScriptGamepadMappingKind::Standard,
            },
            change: crate::ScriptGamepadInputChange::Button {
                index: 1,
                value: 0.75,
            },
            standard_button_intent: Some(1),
            standard_axis_intent: None,
        };
        assert!(
            machine
                .broadcast_gamepad_to_scripted_drawables(
                    &mut artboard,
                    &event,
                    Some((u64::MAX, global_id)),
                )
                .handled,
            "the same authored id in another artboard occurrence is a distinct C++ pointer"
        );
        let owner_identity = artboard.instance_identity();
        assert!(
            !machine
                .broadcast_gamepad_to_scripted_drawables(
                    &mut artboard,
                    &event,
                    Some((owner_identity, global_id)),
                )
                .handled,
            "only the exact focused scripted-drawable occurrence is skipped"
        );
        assert_eq!(calls.borrow().len(), 2);
    }

    #[test]
    fn scripted_gamepad_parent_never_falls_through_to_listener_actions() {
        let (mut artboard, mut machine, global_id) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "no gamepad method",
                methods: Vec::new(),
                handled: false,
                calls: Rc::new(RefCell::new(Vec::new())),
            }));
        machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
            target_local_id: 1,
            is_single: false,
            listener_types: vec![RuntimeListenerType::Gamepad],
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: vec![RuntimeScheduledListenerAction::noop_for_test(0)],
        }]);
        machine.gamepad_listener_groups = vec![
            RuntimeGamepadListenerGroup::new(0, 2, &machine.listener_definitions[0])
                .expect("gamepad listener group"),
        ];
        machine.needs_advance = false;

        let (outcome, dispatched) = machine.gamepad_dispatch_at_focus_data(
            &mut artboard,
            2,
            &ScriptListenerInvocation::GamepadDisconnected { device_id: 7 },
        );

        assert!(!outcome.handled);
        assert!(!outcome.terminal_resource_failure);
        assert_eq!(dispatched, Some((artboard.instance_identity(), global_id)));
        assert!(
            !machine.needs_advance,
            "C++ returns the ScriptedDrawable result immediately; it never runs or marks the ordinary listener branch"
        );
    }

    #[test]
    fn scripted_drawable_without_attached_script_still_owns_gamepad_dispatch() {
        let (mut artboard, mut machine, _) =
            scripted_drawable_subtype_input_artboard_and_machine_with_optional_script(
                "ScriptedDrawable",
                None,
                true,
            );
        machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
            target_local_id: 1,
            is_single: false,
            listener_types: vec![RuntimeListenerType::Gamepad],
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: vec![
                super::listener_types::RuntimeListenerInputTypeGamepad::catch_all_for_test(1),
            ],
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: vec![RuntimeScheduledListenerAction::noop_for_test(0)],
        }]);
        machine.gamepad_listener_groups = vec![
            RuntimeGamepadListenerGroup::new(0, 2, &machine.listener_definitions[0])
                .expect("gamepad listener group"),
        ];
        machine.needs_advance = false;

        let (outcome, dispatched) = machine.gamepad_dispatch_at_focus_data(
            &mut artboard,
            2,
            &ScriptListenerInvocation::GamepadDisconnected { device_id: 7 },
        );

        assert!(!outcome.handled);
        assert!(!outcome.terminal_resource_failure);
        assert_eq!(
            dispatched,
            Some((
                artboard.instance_identity(),
                artboard.component(1).unwrap().global_id
            ))
        );
        assert!(
            !machine.needs_advance,
            "C++ selects the ScriptedDrawable branch from the concrete parent type; a null VM returns false without running the ordinary listener"
        );
    }

    #[test]
    fn gamepad_listener_dispatches_all_payloads_fifo_marks_advance_and_returns_false() {
        fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
            AuthoringRecord {
                type_key: nuxie_schema::definition_by_name(type_name)
                    .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                    .type_key
                    .int,
                properties,
            }
        }
        fn uint(type_name: &str, name: &str, value: u64) -> AuthoringProperty {
            AuthoringProperty {
                key: crate::properties::property_key_for_name(type_name, name)
                    .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
                value: AuthoringValue::Uint(value),
            }
        }

        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("Node", vec![uint("Node", "parentId", 0)]),
            record("FocusData", vec![uint("FocusData", "parentId", 1)]),
            record("StateMachine", Vec::new()),
            record("StateMachineBool", Vec::new()),
            record(
                "StateMachineListener",
                vec![uint("StateMachineListener", "targetId", 1)],
            ),
            record(
                "ListenerInputTypeGamepad",
                vec![uint(
                    "ListenerInputTypeGamepad",
                    "listenerTypeValue",
                    RuntimeListenerType::Gamepad as u64,
                )],
            ),
            record(
                "ListenerBoolChange",
                vec![
                    uint("ListenerBoolChange", "inputId", 0),
                    uint("ListenerBoolChange", "value", 1),
                ],
            ),
            record(
                "ListenerBoolChange",
                vec![
                    uint("ListenerBoolChange", "inputId", 0),
                    uint("ListenerBoolChange", "value", 2),
                ],
            ),
        ])
        .expect("gamepad listener records");
        let graph = GraphFile::from_runtime_file(&file).expect("gamepad listener graph");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("gamepad listener artboard"),
            &graph.artboards,
        )
        .expect("gamepad listener instance");
        artboard.update_components();
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("gamepad listener machine");
        assert!(machine.focus.set_focus_target(1));

        let invocations = [
            RuntimeGamepadListenerGroup::connected(ScriptGamepadSnapshot {
                device_id: 9,
                button_mask: 0,
                button_values: Vec::new(),
                axes: Vec::new(),
                mapping: crate::ScriptGamepadMappingKind::Unknown,
            }),
            ScriptListenerInvocation::GamepadEvent {
                full_state: ScriptGamepadSnapshot {
                    device_id: 9,
                    button_mask: 1,
                    button_values: vec![1.0],
                    axes: vec![0.25],
                    mapping: crate::ScriptGamepadMappingKind::Standard,
                },
                change: crate::ScriptGamepadInputChange::Axis {
                    index: 0,
                    value: 0.25,
                },
                standard_button_intent: None,
                standard_axis_intent: Some(0),
            },
            RuntimeGamepadListenerGroup::disconnected(9),
        ];
        for invocation in invocations {
            machine.needs_advance = false;
            assert!(
                !machine.gamepad_dispatch(&mut artboard, invocation),
                "the authored listener branch never handles propagation in C++"
            );
            assert!(machine.needs_advance());
            assert_eq!(
                machine
                    .input(0)
                    .and_then(StateMachineInputInstance::bool_value),
                Some(false),
                "set-true then toggle executes both authored actions in FIFO order"
            );
        }
    }

    #[test]
    fn draining_public_reports_leaves_the_core_queue_for_apply_events() {
        let (artboard, mut machine) = scripted_listener_artboard_and_machine();
        let event = StateMachineReportedEvent {
            event_local_index: 7,
            event_core_type: 128,
            name: Some("next-frame".to_owned()),
            url: None,
            target: None,
            properties: Vec::new(),
            string_properties: Vec::new(),
            seconds_delay: 0.0,
            context: None,
        };
        machine.reported_events.push(event.clone());

        let drained = machine.take_reported_events(&artboard);

        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].event_local_index(), event.event_local_index());
        assert!(machine.take_reported_events(&artboard).is_empty());
        assert_eq!(machine.reported_event_count(), 1);
        assert_eq!(machine.next_unapplied_reported_event_index(), 0);
    }

    #[test]
    fn apply_events_consumes_every_chained_listener_fire_event_in_the_same_frame() {
        fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
            AuthoringRecord {
                type_key: nuxie_schema::definition_by_name(type_name)
                    .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                    .type_key
                    .int,
                properties,
            }
        }
        fn uint(type_name: &str, property_name: &str, value: u64) -> AuthoringProperty {
            AuthoringProperty {
                key: crate::properties::property_key_for_name(type_name, property_name)
                    .unwrap_or_else(|| panic!("missing {type_name}.{property_name}")),
                value: AuthoringValue::Uint(value),
            }
        }
        fn event_listener(
            event_local_id: usize,
            fire_event_local_id: Option<usize>,
        ) -> RuntimeStateMachineListener {
            RuntimeStateMachineListener {
                target_local_id: 0,
                is_single: false,
                listener_types: vec![RuntimeListenerType::Event],
                event_local_indices: vec![event_local_id],
                view_model_path: None,
                view_model_input_types: Vec::new(),
                gamepad_input_types: Vec::new(),
                keyboard_input_types: Vec::new(),
                semantic_input_types: Vec::new(),
                hit_paths: Vec::new(),
                listener_actions: fire_event_local_id
                    .map(|event_local_id| {
                        vec![RuntimeScheduledListenerAction::FireEvent(
                            super::listener_fire_event::RuntimeListenerFireEvent::for_test(
                                0,
                                Some(event_local_id),
                            ),
                        )]
                    })
                    .unwrap_or_default(),
            }
        }

        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("Event", vec![uint("Event", "parentId", 0)]),
            record("Event", vec![uint("Event", "parentId", 0)]),
            record("Event", vec![uint("Event", "parentId", 0)]),
            record("StateMachine", Vec::new()),
        ])
        .expect("chained event records import");
        let graph = GraphFile::from_runtime_file(&file).expect("chained event graph builds");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("chained event artboard"),
            &graph.artboards,
        )
        .expect("chained event artboard instantiates");
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("chained event state machine");
        machine.listener_definitions = Arc::new(vec![
            event_listener(1, Some(2)),
            event_listener(2, Some(3)),
            event_listener(3, None),
        ]);
        machine.reported_events.push(StateMachineReportedEvent {
            event_local_index: 1,
            event_core_type: 128,
            name: Some("first".to_owned()),
            url: None,
            target: None,
            properties: Vec::new(),
            string_properties: Vec::new(),
            seconds_delay: 0.0,
            context: None,
        });

        assert!(
            !artboard.advance_state_machine_instance(&mut machine, 0.25),
            "C++ clears each applyEvents batch before notifying it, so a fully consumed fire-event chain leaves no pending-work return term"
        );
        assert_eq!(
            machine
                .reporting_events
                .iter()
                .map(StateMachineReportedEvent::event_local_index)
                .collect::<Vec<_>>(),
            [3],
            "E1 -> E2 -> E3 must reach the final listener in this one applyEvents call"
        );
        assert_eq!(machine.reported_event_count(), 0);
        assert_eq!(machine.next_unapplied_reported_event_index(), 0);
        assert!(machine.take_reported_events(&artboard).is_empty());
    }

    #[test]
    fn listener_fire_event_reports_live_payload_before_advance() {
        fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
            AuthoringRecord {
                type_key: nuxie_schema::definition_by_name(type_name)
                    .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                    .type_key
                    .int,
                properties,
            }
        }
        fn uint(type_name: &str, property_name: &str, value: u64) -> AuthoringProperty {
            AuthoringProperty {
                key: crate::properties::property_key_for_name(type_name, property_name)
                    .unwrap_or_else(|| panic!("missing {type_name}.{property_name}")),
                value: AuthoringValue::Uint(value),
            }
        }
        fn string(type_name: &str, property_name: &str, value: &str) -> AuthoringProperty {
            AuthoringProperty {
                key: crate::properties::property_key_for_name(type_name, property_name)
                    .unwrap_or_else(|| panic!("missing {type_name}.{property_name}")),
                value: AuthoringValue::String(value.to_owned()),
            }
        }

        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record(
                "Event",
                vec![
                    uint("Event", "parentId", 0),
                    string("Event", "name", "imported"),
                ],
            ),
            record(
                "CustomPropertyString",
                vec![
                    uint("CustomPropertyString", "parentId", 1),
                    string("CustomPropertyString", "name", "payload"),
                    string("CustomPropertyString", "propertyValue", "old"),
                ],
            ),
            record("StateMachine", Vec::new()),
        ])
        .expect("event listener records import");
        let graph = GraphFile::from_runtime_file(&file).expect("event listener graph builds");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("event listener artboard"),
            &graph.artboards,
        )
        .expect("event listener artboard instantiates");
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("event listener state machine");

        let event_name =
            crate::properties::property_key_for_name("Event", "name").expect("Event.name property");
        let property_value =
            crate::properties::property_key_for_name("CustomPropertyString", "propertyValue")
                .expect("CustomPropertyString.propertyValue property");
        assert!(artboard.set_string_property(1, event_name, b"live".to_vec()));
        assert!(artboard.set_string_property(2, property_value, b"new".to_vec()));

        let actions = [RuntimeScheduledListenerAction::FireEvent(
            super::listener_fire_event::RuntimeListenerFireEvent::for_test(
                StateMachineFireOccurrence::AtStart.value(),
                Some(1),
            ),
        )];
        let facade_hit_context = StateMachineEventContext {
            path: Vec::new(),
            occurrence: Vec::new(),
        };
        assert!(
            machine
                .perform_listener_actions_with_event_context(
                    &mut artboard,
                    &actions,
                    None,
                    &ScriptListenerInvocation::None,
                    &mut NoopScriptHost,
                    Some(&facade_hit_context),
                )
                .expect("fire live event")
        );
        // C++ EventReport retains Event*, so edits made after reportEvent and
        // before host observation are visible too.
        assert!(artboard.set_string_property(1, event_name, b"latest".to_vec()));
        assert!(artboard.set_string_property(2, property_value, b"after-fire".to_vec()));

        let mut snapshot = machine.clone();
        let drained = machine.take_reported_events(&artboard);
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].name(), Some("latest"));
        assert_eq!(
            drained[0].context(),
            Some(&facade_hit_context),
            "the Rust facade's rendered occurrence metadata is orthogonal to the ignored C++ ListenerInvocation payload"
        );
        assert_eq!(
            drained[0]
                .string_properties()
                .iter()
                .map(|property| (property.name(), property.value()))
                .collect::<Vec<_>>(),
            [("payload", "after-fire")]
        );
        assert_eq!(machine.reported_event_count(), 1);
        assert_eq!(machine.next_unapplied_reported_event_index(), 0);
        assert_eq!(
            snapshot.take_reported_events(&artboard).len(),
            1,
            "Rust's explicit Clone snapshot retains pending values in non-aliased storage"
        );
        assert!(
            snapshot.take_reported_events(&artboard).is_empty(),
            "draining the snapshot does not mutate the source cursor"
        );
        assert!(
            machine.take_reported_events(&artboard).is_empty(),
            "draining the source does not replay after the snapshot drain"
        );
    }

    fn script(
        label: &'static str,
        has_perform_action: bool,
        has_perform: bool,
        failure: ListenerFailure,
        calls: &Rc<RefCell<Vec<RecordedCall>>>,
    ) -> Box<dyn ScriptInstance> {
        Box::new(RecordingListenerScript {
            label,
            has_perform_action,
            has_perform,
            failure,
            state: 0,
            calls: Rc::clone(calls),
        })
    }

    fn scripted_test_listener(
        machine: &mut StateMachineInstance,
        action_global_id: u32,
        label: &'static str,
        failure: ListenerFailure,
        listener_types: Vec<RuntimeListenerType>,
        calls: &Rc<RefCell<Vec<RecordedCall>>>,
    ) -> RuntimeStateMachineListener {
        let definition = ScriptListenerActionDefinition::new(action_global_id, 1, label.to_owned());
        machine
            .scripted_listener_action_definitions
            .push(definition.clone());
        machine
            .set_scripted_listener_action_instance(
                action_global_id,
                script(label, true, false, failure, calls),
            )
            .expect("attach scripted test listener");
        RuntimeStateMachineListener {
            target_local_id: 1,
            is_single: false,
            listener_types,
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: vec![RuntimeScheduledListenerAction::scripted_for_test(
                0,
                Some(definition),
            )],
        }
    }

    #[test]
    fn deferred_focus_and_semantic_callbacks_continue_after_ordinary_script_failure() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let listeners = vec![
            scripted_test_listener(
                &mut machine,
                700,
                "focus ordinary",
                ListenerFailure::Ordinary,
                vec![RuntimeListenerType::Focus],
                &calls,
            ),
            scripted_test_listener(
                &mut machine,
                701,
                "focus later",
                ListenerFailure::None,
                vec![RuntimeListenerType::Focus],
                &calls,
            ),
            scripted_test_listener(
                &mut machine,
                702,
                "semantic later",
                ListenerFailure::None,
                vec![RuntimeListenerType::SemanticAction],
                &calls,
            ),
        ];
        machine.listener_definitions = Arc::new(listeners);
        machine.queued_focus_events = vec![
            ScriptListenerInvocation::Focus {
                listener_index: 0,
                is_focus: true,
            },
            ScriptListenerInvocation::Focus {
                listener_index: 1,
                is_focus: true,
            },
        ];
        machine.queued_semantic_events = vec![ScriptListenerInvocation::Semantic {
            listener_index: 2,
            action_type: 1,
        }];

        assert!(machine.process_deferred_listener_group_events(&mut artboard, None));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["focus ordinary", "focus later", "semantic later"]
        );
        assert!(machine.script_error().is_none());
    }

    #[test]
    fn terminal_focus_or_semantic_callback_stops_the_remaining_deferred_batch() {
        let resource_code = "script.resource.host_commands";

        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let listeners = vec![
            scripted_test_listener(
                &mut machine,
                710,
                "focus terminal",
                ListenerFailure::Terminal(resource_code),
                vec![RuntimeListenerType::Focus],
                &calls,
            ),
            scripted_test_listener(
                &mut machine,
                711,
                "focus skipped",
                ListenerFailure::None,
                vec![RuntimeListenerType::Focus],
                &calls,
            ),
            scripted_test_listener(
                &mut machine,
                712,
                "semantic skipped",
                ListenerFailure::None,
                vec![RuntimeListenerType::SemanticAction],
                &calls,
            ),
        ];
        machine.listener_definitions = Arc::new(listeners);
        machine.queued_focus_events = vec![
            ScriptListenerInvocation::Focus {
                listener_index: 0,
                is_focus: true,
            },
            ScriptListenerInvocation::Focus {
                listener_index: 1,
                is_focus: true,
            },
        ];
        machine.queued_semantic_events = vec![ScriptListenerInvocation::Semantic {
            listener_index: 2,
            action_type: 1,
        }];

        assert!(!machine.process_deferred_listener_group_events(&mut artboard, None));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["focus terminal"]
        );
        assert_eq!(
            machine.script_error().and_then(ScriptError::resource_code),
            Some(resource_code)
        );

        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let listeners = vec![
            scripted_test_listener(
                &mut machine,
                713,
                "focus first",
                ListenerFailure::None,
                vec![RuntimeListenerType::Focus],
                &calls,
            ),
            scripted_test_listener(
                &mut machine,
                714,
                "semantic terminal",
                ListenerFailure::Terminal(resource_code),
                vec![RuntimeListenerType::SemanticAction],
                &calls,
            ),
            scripted_test_listener(
                &mut machine,
                715,
                "semantic skipped",
                ListenerFailure::None,
                vec![RuntimeListenerType::SemanticAction],
                &calls,
            ),
        ];
        machine.listener_definitions = Arc::new(listeners);
        machine.queued_focus_events = vec![ScriptListenerInvocation::Focus {
            listener_index: 0,
            is_focus: true,
        }];
        machine.queued_semantic_events = vec![
            ScriptListenerInvocation::Semantic {
                listener_index: 1,
                action_type: 1,
            },
            ScriptListenerInvocation::Semantic {
                listener_index: 2,
                action_type: 1,
            },
        ];

        assert!(machine.process_deferred_listener_group_events(&mut artboard, None));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["focus first", "semantic terminal"]
        );
        assert_eq!(
            machine.script_error().and_then(ScriptError::resource_code),
            Some(resource_code)
        );
    }

    #[test]
    fn view_model_callback_fifo_continues_after_ordinary_failure_and_stops_on_terminal() {
        fn run(first_failure: ListenerFailure) -> (Vec<&'static str>, Option<String>) {
            let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
            let calls = Rc::new(RefCell::new(Vec::new()));
            let listeners = Arc::new(vec![
                scripted_test_listener(
                    &mut machine,
                    720,
                    "view model first",
                    first_failure,
                    vec![RuntimeListenerType::ViewModel],
                    &calls,
                ),
                scripted_test_listener(
                    &mut machine,
                    721,
                    "view model later",
                    ListenerFailure::None,
                    vec![RuntimeListenerType::ViewModel],
                    &calls,
                ),
            ]);
            machine.listener_definitions = Arc::clone(&listeners);
            machine.view_model_listeners = (0..listeners.len())
                .filter_map(|index| {
                    RuntimeViewModelListenerInstance::new(Arc::clone(&listeners), index)
                })
                .collect();
            machine.reported_listener_view_models.report_data_bind(0);
            machine.reported_listener_view_models.report_data_bind(1);

            let _ = machine.apply_local_event_listeners(&mut artboard, 0, None);
            let labels = calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>();
            (
                labels,
                machine
                    .script_error()
                    .and_then(ScriptError::resource_code)
                    .map(str::to_owned),
            )
        }

        assert_eq!(
            run(ListenerFailure::Ordinary),
            (vec!["view model first", "view model later"], None)
        );
        assert_eq!(
            run(ListenerFailure::Terminal("script.resource.host_commands")),
            (
                vec!["view model first"],
                Some("script.resource.host_commands".to_owned())
            )
        );
    }

    #[test]
    fn retained_terminal_input_error_blocks_every_later_apply_callback() {
        let (mut artboard, mut machine, _) = scripted_drawable_input_artboard_and_machine(
            Box::new(ResourceFailingDrawableInputScript),
        );
        assert!(!machine.key_input(&mut artboard, 65, 0, true, false));

        let calls = Rc::new(RefCell::new(Vec::new()));
        let listeners = Arc::new(vec![
            scripted_test_listener(
                &mut machine,
                730,
                "focus blocked",
                ListenerFailure::None,
                vec![RuntimeListenerType::Focus],
                &calls,
            ),
            scripted_test_listener(
                &mut machine,
                731,
                "semantic blocked",
                ListenerFailure::None,
                vec![RuntimeListenerType::SemanticAction],
                &calls,
            ),
            scripted_test_listener(
                &mut machine,
                732,
                "view model blocked",
                ListenerFailure::None,
                vec![RuntimeListenerType::ViewModel],
                &calls,
            ),
        ]);
        machine.listener_definitions = Arc::clone(&listeners);
        machine.view_model_listeners = vec![
            RuntimeViewModelListenerInstance::new(Arc::clone(&listeners), 2)
                .expect("ViewModel listener occurrence"),
        ];
        machine.queued_focus_events = vec![ScriptListenerInvocation::Focus {
            listener_index: 0,
            is_focus: true,
        }];
        machine.queued_semantic_events = vec![ScriptListenerInvocation::Semantic {
            listener_index: 1,
            action_type: 1,
        }];
        machine.reported_listener_view_models.report_data_bind(0);

        assert!(!machine.apply_local_event_listeners(&mut artboard, 0, None));
        assert!(calls.borrow().is_empty());
        assert_eq!(machine.queued_focus_events.len(), 1);
        assert_eq!(machine.queued_semantic_events.len(), 1);
        assert!(!machine.reported_listener_view_models.is_empty());
    }

    #[test]
    fn scripted_listener_actions_keep_authored_fifo_and_prefer_perform_action() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let first = machine
            .scripted_listener_actions()
            .first()
            .expect("fixture scripted listener action")
            .clone();
        let second = ScriptListenerActionDefinition::new(500, 1, "legacy".to_owned());
        machine.scripted_listener_action_definitions = vec![first.clone(), second.clone()];
        let calls = Rc::new(RefCell::new(Vec::new()));
        machine
            .set_scripted_listener_action_instance(
                first.action_global_id(),
                script("first", true, true, ListenerFailure::None, &calls),
            )
            .expect("attach first action");
        machine
            .set_scripted_listener_action_instance(
                second.action_global_id(),
                script("second", false, true, ListenerFailure::None, &calls),
            )
            .expect("attach second action");
        // Pinned C++ resolves one stateful clone from the authored
        // ScriptedListenerAction occurrence (`scripted_listener_action.cpp:
        // 88-99`). A same-id entry in the general scripted-object table must
        // not become a second callback.
        machine.set_script_instance_for_global(
            first.action_global_id(),
            script("wrong-table", true, false, ListenerFailure::None, &calls),
        );
        let actions = vec![
            RuntimeScheduledListenerAction::scripted_for_test(0, Some(first)),
            RuntimeScheduledListenerAction::scripted_for_test(0, Some(second)),
        ];
        let invocation = ScriptListenerInvocation::Pointer {
            x: 12.0,
            y: 34.0,
            previous_x: 12.0,
            previous_y: 34.0,
            pointer_id: 7,
            event: ScriptPointerEventKind::Click,
            timestamp_seconds: 0.0,
        };

        assert!(
            machine
                .perform_listener_actions(
                    &mut artboard,
                    &actions,
                    None,
                    &invocation,
                    &mut NoopScriptHost,
                )
                .expect("perform listener actions")
        );
        assert_eq!(
            calls.borrow().as_slice(),
            [
                RecordedCall {
                    label: "first",
                    method: ScriptListenerActionMethod::PerformAction,
                    invocation: invocation.clone(),
                    state_before_call: 0,
                },
                RecordedCall {
                    label: "second",
                    method: ScriptListenerActionMethod::Perform,
                    invocation,
                    state_before_call: 0,
                },
            ]
        );
    }

    #[test]
    fn successive_pointer_events_preserve_previous_position_and_timestamp() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let action = machine
            .scripted_listener_actions()
            .first()
            .expect("fixture scripted listener action")
            .clone();
        let calls = Rc::new(RefCell::new(Vec::new()));
        machine
            .set_scripted_listener_action_instance(
                action.action_global_id(),
                script("pointer", true, false, ListenerFailure::None, &calls),
            )
            .expect("attach pointer action");

        machine
            .try_pointer_down_with_timestamp_and_script_host(
                &mut artboard,
                200.0,
                20.0,
                1,
                1.25,
                &mut NoopScriptHost,
            )
            .expect("first pointer down");
        machine
            .try_pointer_up_with_timestamp_and_script_host(
                &mut artboard,
                200.0,
                20.0,
                1,
                1.5,
                &mut NoopScriptHost,
            )
            .expect("first pointer up");
        machine
            .try_pointer_down_with_timestamp_and_script_host(
                &mut artboard,
                205.0,
                20.0,
                1,
                2.25,
                &mut NoopScriptHost,
            )
            .expect("second pointer down");
        machine
            .try_pointer_up_with_timestamp_and_script_host(
                &mut artboard,
                210.0,
                20.0,
                1,
                2.5,
                &mut NoopScriptHost,
            )
            .expect("second pointer up");

        let invocations = calls
            .borrow()
            .iter()
            .map(|call| call.invocation.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            invocations,
            [
                ScriptListenerInvocation::Pointer {
                    x: 200.0,
                    y: 20.0,
                    previous_x: 200.0,
                    previous_y: 20.0,
                    pointer_id: 1,
                    event: ScriptPointerEventKind::Click,
                    timestamp_seconds: 1.5,
                },
                ScriptListenerInvocation::Pointer {
                    x: 210.0,
                    y: 20.0,
                    previous_x: 205.0,
                    previous_y: 20.0,
                    pointer_id: 1,
                    event: ScriptPointerEventKind::Click,
                    timestamp_seconds: 2.5,
                },
            ]
        );
    }

    #[test]
    fn matched_pointer_listener_marks_advance_even_when_actions_are_noops() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let mut listeners = machine.listener_definitions.as_ref().clone();
        assert!(
            listeners.iter_mut().any(|listener| {
                if !listener.has_listener(RuntimeListenerType::Click) {
                    return false;
                }
                listener.listener_actions.clear();
                true
            }),
            "fixture must retain a click listener"
        );
        machine.listener_definitions = Arc::new(listeners);
        machine.needs_advance = false;

        assert!(machine.pointer_down(&mut artboard, 200.0, 20.0, 1));
        machine.needs_advance = false;
        assert!(machine.pointer_up(&mut artboard, 200.0, 20.0, 1));
        assert!(
            machine.needs_advance(),
            "C++ ListenerGroup::processEvent marks the machine after every matched listener, \
             even when its action list is empty (`listener_group.cpp:218-225`)"
        );
    }

    #[test]
    fn pointer_history_is_listener_scoped_and_resets_on_first_entry_and_reentry() {
        let mut machine = scripted_listener_machine();

        let (first_entry, was_hovered) =
            machine.pointer_input_for_listener(0, 10.0, 20.0, 7, 1.0, true);
        assert!(!was_hovered);
        assert_eq!(
            (first_entry.previous_x, first_entry.previous_y),
            (10.0, 20.0)
        );
        machine.record_pointer_input_for_listener(0, first_entry);

        let (overlapping_entry, was_hovered) =
            machine.pointer_input_for_listener(1, 100.0, 200.0, 7, 1.1, true);
        assert!(!was_hovered);
        assert_eq!(
            (overlapping_entry.previous_x, overlapping_entry.previous_y),
            (100.0, 200.0),
            "a second listener group must not inherit the first group's history"
        );
        machine.record_pointer_input_for_listener(1, overlapping_entry);

        let (move_inside, was_hovered) =
            machine.pointer_input_for_listener(0, 15.0, 25.0, 7, 1.2, true);
        assert!(was_hovered);
        assert_eq!(
            (move_inside.previous_x, move_inside.previous_y),
            (10.0, 20.0)
        );
        machine.record_pointer_input_for_listener(0, move_inside);

        let (exit, was_hovered) = machine.pointer_input_for_listener(0, 30.0, 40.0, 7, 1.3, false);
        assert!(was_hovered);
        assert_eq!((exit.previous_x, exit.previous_y), (15.0, 25.0));
        machine.record_pointer_input_for_listener(0, exit);

        let (outside, was_hovered) =
            machine.pointer_input_for_listener(0, 50.0, 60.0, 7, 1.4, false);
        assert!(!was_hovered);
        machine.record_pointer_input_for_listener(0, outside);
        let (reentry, was_hovered) =
            machine.pointer_input_for_listener(0, 70.0, 80.0, 7, 1.5, true);
        assert!(!was_hovered);
        assert_eq!(
            (reentry.previous_x, reentry.previous_y),
            (70.0, 80.0),
            "reentry resets the prior outside position before dispatch"
        );
    }

    #[test]
    fn pointer_up_position_is_retained_for_exit_then_released() {
        let mut machine = scripted_listener_machine();

        let (down, _) = machine.pointer_input_for_listener(0, 10.0, 20.0, 9, 2.0, true);
        machine.record_pointer_input_for_listener(0, down);
        let (up, was_hovered) = machine.pointer_input_for_listener(0, 15.0, 25.0, 9, 2.1, true);
        assert!(was_hovered);
        assert_eq!((up.previous_x, up.previous_y), (10.0, 20.0));
        machine.record_pointer_input_for_listener(0, up);

        let (exit, was_hovered) = machine.pointer_input_for_listener(0, 30.0, 40.0, 9, 2.2, false);
        assert!(was_hovered);
        assert_eq!((exit.previous_x, exit.previous_y), (15.0, 25.0));
        machine.record_pointer_input_for_listener(0, exit);
        machine.release_pointer_input(9);

        let (next_entry, was_hovered) =
            machine.pointer_input_for_listener(0, 50.0, 60.0, 9, 3.0, true);
        assert!(!was_hovered);
        assert_eq!((next_entry.previous_x, next_entry.previous_y), (50.0, 60.0));
    }

    #[test]
    fn scripted_listener_failure_is_swallowed_and_later_actions_still_run() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let first = machine
            .scripted_listener_actions()
            .first()
            .expect("fixture scripted listener action")
            .clone();
        let second = ScriptListenerActionDefinition::new(500, 1, "later".to_owned());
        machine.scripted_listener_action_definitions = vec![first.clone(), second.clone()];
        let calls = Rc::new(RefCell::new(Vec::new()));
        machine
            .set_scripted_listener_action_instance(
                first.action_global_id(),
                script("first", true, false, ListenerFailure::Ordinary, &calls),
            )
            .expect("attach failing action");
        machine
            .set_scripted_listener_action_instance(
                second.action_global_id(),
                script("later", true, false, ListenerFailure::None, &calls),
            )
            .expect("attach later action");
        let actions = vec![
            RuntimeScheduledListenerAction::scripted_for_test(0, Some(first)),
            RuntimeScheduledListenerAction::scripted_for_test(0, Some(second)),
        ];

        assert!(
            machine
                .perform_listener_actions(
                    &mut artboard,
                    &actions,
                    None,
                    &ScriptListenerInvocation::None,
                    &mut NoopScriptHost,
                )
                .expect("C++ consumes the protected-call error")
        );
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["first", "later"],
            "an authored action after a failing script still runs"
        );
        assert_eq!(
            machine.script_error(),
            None,
            "listener protected-call errors do not poison the machine"
        );

        assert!(
            machine
                .perform_listener_actions(
                    &mut artboard,
                    &actions,
                    None,
                    &ScriptListenerInvocation::None,
                    &mut NoopScriptHost,
                )
                .expect("later dispatch remains live")
        );
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["first", "later", "first", "later"]
        );
    }

    #[test]
    fn each_state_machine_occurrence_retains_fresh_listener_script_state() {
        let (mut first_artboard, mut first_machine) = scripted_listener_artboard_and_machine();
        let (mut second_artboard, mut second_machine) = scripted_listener_artboard_and_machine();
        let definition = first_machine
            .scripted_listener_actions()
            .first()
            .expect("fixture scripted listener action")
            .clone();
        let calls = Rc::new(RefCell::new(Vec::new()));
        first_machine
            .set_scripted_listener_action_instance(
                definition.action_global_id(),
                script("occurrence", true, false, ListenerFailure::None, &calls),
            )
            .expect("attach first occurrence");
        second_machine
            .set_scripted_listener_action_instance(
                definition.action_global_id(),
                script("occurrence", true, false, ListenerFailure::None, &calls),
            )
            .expect("attach second occurrence");
        let actions = [RuntimeScheduledListenerAction::scripted_for_test(
            0,
            Some(definition),
        )];

        first_machine
            .perform_listener_actions(
                &mut first_artboard,
                &actions,
                None,
                &ScriptListenerInvocation::None,
                &mut NoopScriptHost,
            )
            .expect("run first occurrence");
        second_machine
            .perform_listener_actions(
                &mut second_artboard,
                &actions,
                None,
                &ScriptListenerInvocation::None,
                &mut NoopScriptHost,
            )
            .expect("run second occurrence");

        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.state_before_call)
                .collect::<Vec<_>>(),
            [0, 0]
        );
    }

    #[test]
    fn cloned_snapshot_rebuilds_mutable_listener_tables_without_aliasing() {
        let mut original = scripted_listener_machine();
        let original_layer_ids = original
            .layers
            .iter()
            .map(StateMachineLayerInstance::view_model_trigger_layer_id)
            .collect::<Vec<_>>();
        let definition = original
            .scripted_listener_actions()
            .first()
            .expect("fixture scripted listener action")
            .clone();
        let original_calls = Rc::new(RefCell::new(Vec::new()));
        original
            .set_scripted_listener_action_instance(
                definition.action_global_id(),
                script(
                    "original",
                    true,
                    false,
                    ListenerFailure::None,
                    &original_calls,
                ),
            )
            .expect("attach original occurrence");

        let mut cloned = original.clone();
        let cloned_layer_ids = cloned
            .layers
            .iter()
            .map(StateMachineLayerInstance::view_model_trigger_layer_id)
            .collect::<Vec<_>>();
        assert_eq!(original_layer_ids.len(), cloned_layer_ids.len());
        assert!(
            original_layer_ids
                .iter()
                .zip(&cloned_layer_ids)
                .all(|(original, cloned)| original != cloned),
            "a cloned state-machine occurrence has distinct C++ layer-pointer identities"
        );
        let cloned_calls = Rc::new(RefCell::new(Vec::new()));
        cloned
            .set_scripted_listener_action_instance(
                definition.action_global_id(),
                script("clone", true, false, ListenerFailure::None, &cloned_calls),
            )
            .expect("clone must accept a fresh table");
    }

    #[test]
    fn transactional_candidate_can_adopt_the_same_occurrence_listener_state() {
        let (mut artboard, mut original) = scripted_listener_artboard_and_machine();
        let definition = original
            .scripted_listener_actions()
            .first()
            .expect("fixture scripted listener action")
            .clone();
        let calls = Rc::new(RefCell::new(Vec::new()));
        original
            .set_scripted_listener_action_instance(
                definition.action_global_id(),
                script("transaction", true, false, ListenerFailure::None, &calls),
            )
            .expect("attach original occurrence");
        let mut candidate = original.clone();

        candidate
            .adopt_scripted_listener_action_state_from(&original)
            .expect("validated candidate represents the same occurrence");
        candidate
            .perform_listener_actions(
                &mut artboard,
                &[RuntimeScheduledListenerAction::scripted_for_test(
                    0,
                    Some(definition),
                )],
                None,
                &ScriptListenerInvocation::None,
                &mut NoopScriptHost,
            )
            .expect("committed candidate retains the listener table");

        assert_eq!(calls.borrow().len(), 1);
    }
}
