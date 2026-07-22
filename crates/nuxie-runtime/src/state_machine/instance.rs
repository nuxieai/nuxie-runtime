// Runtime instance orchestration for the C++ state machine path.
// Mirrors /Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp.
use super::*;
use crate::artboard_data_bind::RuntimeOwnedViewModelBindingCandidate;
use crate::data_bind_graph::data_bind_flags_apply_source_to_target;
use crate::focus::RuntimeFocusTree;
use crate::scripting::RuntimeScriptInstanceHandle;
use crate::view_model::RuntimeFontAssetValue;
use crate::view_model_cell::{
    RuntimeCellDirtSink, RuntimeCellNotificationQueue, RuntimeViewModelCell,
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
    RuntimeOwnedViewModelInstance, ScriptError, ScriptHost, ScriptInstance,
    ScriptListenerActionDefinition, ScriptListenerActionMethod, ScriptListenerInvocation,
    ScriptMethod, ScriptPointerEventKind, ScriptValue,
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

#[derive(Debug)]
pub struct StateMachineInstance {
    state_machine_index: usize,
    default_view_model_index: Option<usize>,
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
    default_view_model_triggers: Vec<StateMachineViewModelTriggerInstance>,
    view_model_triggers: Vec<StateMachineViewModelTriggerInstance>,
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
    pending_listener_events: Vec<StateMachineReportedEvent>,
    changed_state_count: usize,
    needs_advance: bool,
    has_advanced_once: bool,
    // A mounted NestedStateMachine is initialized before its parent binding
    // settles. C++ gives that occurrence one outer-update probe after the
    // mounted artboard updates, even when its authored conditions are stable.
    post_update_probe_pending: bool,
    data_bind_graph: RuntimeDataBindGraph,
    key_frame_data_bind_graphs: Vec<Option<RuntimeDataBindGraph>>,
    owned_view_model_candidates: Vec<RuntimeOwnedViewModelBindingCandidate>,
    owned_view_model_candidate_generations: Vec<u64>,
    pointer_down_listener_hits: Vec<RuntimePointerDownListenerHit>,
    pointer_listener_states: Vec<RuntimePointerListenerState>,
    pointer_positions: Vec<RuntimePointerPosition>,
    scripted_instances_by_global: BTreeMap<u32, RuntimeScriptInstanceHandle>,
    focus: RuntimeFocusTree,
    scripted_listener_action_definitions: Vec<ScriptListenerActionDefinition>,
    scripted_listener_action_instances: BTreeMap<u32, RuntimeScriptInstanceHandle>,
    script_error: Option<ScriptError>,
    view_model_listeners: Vec<RuntimeViewModelListenerInstance>,
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

struct RuntimeStateMachineListenerActionExecutor<'a> {
    data_bind_graph: &'a mut RuntimeDataBindGraph,
    default_view_model_index: Option<usize>,
    owned_view_model_context: Option<&'a mut RuntimeOwnedViewModelInstance>,
    owned_view_model_candidates: Vec<RuntimeOwnedViewModelBindingCandidate>,
    scripted_listener_action_instances: &'a BTreeMap<u32, RuntimeScriptInstanceHandle>,
    script_error: &'a mut Option<ScriptError>,
    focus: &'a mut RuntimeFocusTree,
    host: &'a mut dyn ScriptHost,
}

impl RuntimeStateMachineListenerActionExecutor<'_> {
    fn default_view_model_trigger_source_property_id(&self, data_bind_index: usize) -> Option<u32> {
        let source_view_model_index = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index)?
            .first()
            .copied()
            .and_then(|index| usize::try_from(index).ok());
        (source_view_model_index == self.default_view_model_index)
            .then(|| {
                self.data_bind_graph
                    .default_view_model_trigger_source_property_id_for_data_bind(data_bind_index)
            })
            .flatten()
    }

    fn perform_scripted_listener_action(
        &mut self,
        definition: &ScriptListenerActionDefinition,
        invocation: &ScriptListenerInvocation,
    ) -> Result<bool, ScriptError> {
        let Some(instance) = self
            .scripted_listener_action_instances
            .get(&definition.action_global_id())
            .cloned()
        else {
            // Visual-only imports deliberately leave script tables
            // unattached. Their ordinary listener actions keep working while
            // embedded bytecode remains inert.
            return Ok(false);
        };
        let result = (|| {
            let mut instance = instance.borrow_mut();
            let method = if instance.has_method(ScriptMethod::PerformAction)? {
                Some(ScriptListenerActionMethod::PerformAction)
            } else if instance.has_method(ScriptMethod::Perform)? {
                Some(ScriptListenerActionMethod::Perform)
            } else {
                None
            };
            let Some(method) = method else {
                return Ok(false);
            };
            instance.call_listener_action(method, invocation, self.host)?;
            Ok(true)
        })();
        result.map_err(|error: ScriptError| {
            let error = error.with_context(format!(
                "ScriptedListenerAction global {} asset ordinal {} name '{}' failed",
                definition.action_global_id(),
                definition.asset_ordinal(),
                definition.asset_name()
            ));
            if self.script_error.is_none() {
                *self.script_error = Some(error.clone());
            }
            self.script_error.clone().unwrap_or(error)
        })
    }

    fn perform_scheduled_view_model_change(
        &mut self,
        artboard: &mut ArtboardInstance,
        data_bind_index: usize,
        value: &RuntimeListenerViewModelChangeValue,
        mut targets: RuntimeScheduledListenerActionTargetsMut<'_>,
    ) -> bool {
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
        };
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = if let Some(context) = self.owned_view_model_context.take() {
            let changed = self.perform_owned_view_model_change(
                &mut *context,
                data_bind_index,
                value,
                &mut targets,
            );
            self.owned_view_model_context = Some(context);
            changed
        } else if !self.owned_view_model_candidates.is_empty() {
            self.perform_owned_view_model_candidates_change(data_bind_index, value, &mut targets)
        } else {
            self.data_bind_graph
                .set_active_view_model_source_for_data_bind(data_bind_index, artboard_value.clone())
        };
        if !changed {
            return false;
        }
        if let Some(path) = path {
            artboard.set_artboard_data_bind_value_for_path(&path, artboard_value);
        }
        self.data_bind_graph.apply_default_view_model_bindings(
            RuntimeDataBindGraphTargetsMut {
                numbers: targets.bindable_numbers,
                integers: targets.bindable_integers,
                booleans: targets.bindable_booleans,
                strings: targets.bindable_strings,
                colors: targets.bindable_colors,
                enums: targets.bindable_enums,
                assets: targets.bindable_assets,
                artboards: targets.bindable_artboards,
                lists: targets.bindable_lists,
                triggers: targets.bindable_triggers,
                view_models: targets.bindable_view_models,
                transition_durations: targets.transition_durations,
                include_view_models: true,
            },
            RuntimeDataBindGraphApplyPhase::Immediate,
        );
        true
    }

    fn perform_owned_view_model_candidates_change(
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
        let Some((candidate, property_path)) =
            self.owned_view_model_candidates
                .iter()
                .find_map(|candidate| {
                    candidate
                        .property_path_for_source_path(&source_path)
                        .map(|property_path| (candidate.clone(), property_path))
                })
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
            let mut context = candidate.context.borrow_mut();
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
            if let Some(trigger_property_id) =
                self.default_view_model_trigger_source_property_id(data_bind_index)
                && let Some(value) = context.trigger_value_by_property_path(&property_path)
                && let Some(trigger) = targets
                    .view_model_triggers
                    .iter_mut()
                    .find(|trigger| trigger.view_model_property_id() == trigger_property_id)
            {
                trigger.set_value(value);
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
            RuntimeListenerViewModelChangeValue::Trigger(_) => unreachable!(),
        };
        let mut context = candidate.context.borrow_mut();
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
        if let RuntimeListenerViewModelChangeValue::Number(value) = value {
            self.data_bind_graph
                .refresh_operation_view_model_number_dependents_for_owned_context_path(
                    &source_path,
                    *value,
                );
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
                let trigger_property_id =
                    self.default_view_model_trigger_source_property_id(data_bind_index);
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
                if let Some((trigger_property_id, value)) =
                    trigger_property_id.and_then(|trigger_property_id| {
                        let property_index = usize::try_from(trigger_property_id).ok()?;
                        let value = context.trigger_value_by_property_index(property_index)?;
                        Some((trigger_property_id, value))
                    })
                    && let Some(trigger) = targets
                        .view_model_triggers
                        .iter_mut()
                        .find(|trigger| trigger.view_model_property_id() == trigger_property_id)
                {
                    trigger.set_value(value);
                }
                true
            }
        }
    }
}

impl RuntimeScheduledListenerActionExecutor for RuntimeStateMachineListenerActionExecutor<'_> {
    fn perform_instance_action(
        &mut self,
        artboard: &mut ArtboardInstance,
        action: &RuntimeScheduledListenerAction,
        targets: RuntimeScheduledListenerActionTargetsMut<'_>,
    ) -> Result<bool, ScriptError> {
        match action {
            RuntimeScheduledListenerAction::ViewModelChange {
                data_bind_index,
                value,
                ..
            } => Ok(self.perform_scheduled_view_model_change(
                artboard,
                *data_bind_index,
                value,
                targets,
            )),
            RuntimeScheduledListenerAction::Scripted { definition, .. } => {
                self.perform_scripted_listener_action(definition, &ScriptListenerInvocation::None)
            }
            RuntimeScheduledListenerAction::FocusTarget {
                target_local_id, ..
            } => Ok(self.focus.set_focus_target(*target_local_id)),
            RuntimeScheduledListenerAction::FocusClear { .. } => Ok(self.focus.clear_focus()),
            RuntimeScheduledListenerAction::FocusTraversal { traversal_kind, .. } => {
                Ok(self.focus.traverse(*traversal_kind))
            }
            RuntimeScheduledListenerAction::FireEvent { .. }
            | RuntimeScheduledListenerAction::BoolChange { .. }
            | RuntimeScheduledListenerAction::NumberChange { .. }
            | RuntimeScheduledListenerAction::TriggerChange { .. } => Err(ScriptError::new(
                "ordinary listener action reached the instance-owned executor",
            )),
        }
    }
}

impl Clone for StateMachineInstance {
    fn clone(&self) -> Self {
        let reported_listener_view_models = RuntimeCellNotificationQueue::default();
        let view_model_listeners = self
            .view_model_listeners
            .iter()
            .enumerate()
            .map(|(listener_index, listener)| {
                listener.clone_for_queue(&reported_listener_view_models, listener_index)
            })
            .collect();
        Self {
            state_machine_index: self.state_machine_index,
            default_view_model_index: self.default_view_model_index,
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
            default_view_model_triggers: self.default_view_model_triggers.clone(),
            view_model_triggers: self.view_model_triggers.clone(),
            transition_durations: self.transition_durations.clone(),
            layers: self.layers.clone(),
            reported_events: self.reported_events.clone(),
            reported_event_listener_index: self.reported_event_listener_index,
            host_reported_event_index: self.host_reported_event_index,
            reporting_events: self.reporting_events.clone(),
            reported_listener_view_models,
            reporting_listener_view_models: Vec::new(),
            pending_listener_events: self.pending_listener_events.clone(),
            changed_state_count: self.changed_state_count,
            needs_advance: self.needs_advance,
            has_advanced_once: self.has_advanced_once,
            post_update_probe_pending: self.post_update_probe_pending,
            data_bind_graph: self.data_bind_graph.clone(),
            key_frame_data_bind_graphs: self.key_frame_data_bind_graphs.clone(),
            owned_view_model_candidates: self.owned_view_model_candidates.clone(),
            owned_view_model_candidate_generations: self
                .owned_view_model_candidate_generations
                .clone(),
            pointer_down_listener_hits: self.pointer_down_listener_hits.clone(),
            pointer_listener_states: self.pointer_listener_states.clone(),
            pointer_positions: self.pointer_positions.clone(),
            scripted_instances_by_global: BTreeMap::new(),
            focus: self.focus.clone(),
            scripted_listener_action_definitions: self.scripted_listener_action_definitions.clone(),
            // A cloned artboard/state-machine occurrence is intentionally
            // cold. VM table handles carry mutable per-occurrence state and
            // must be regenerated by the facade before script execution.
            scripted_listener_action_instances: BTreeMap::new(),
            script_error: None,
            view_model_listeners,
        }
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
    view_model_index: usize,
    property_path: Vec<usize>,
    actions: Vec<RuntimeScheduledListenerAction>,
    /// Last-delivered condition value. Migrated (cell-backed) listeners keep
    /// this as the C++ "consume a value change once" record; unmigrated
    /// conditions still use it as the polled observed copy. Deletion belongs
    /// to slice (f).
    observed: Option<RuntimeViewModelListenerValue>,
    /// #RB-1 e4: the retained scalar cell this listener's condition currently
    /// reads on the owned-candidates path, with this listener's dirt sink
    /// registered as a dependent (C++ `ListenerViewModelPropertyBinding`,
    /// src/animation/state_machine_instance.cpp:1281-1299 at pin d788e8ec:
    /// `vmProp->addDependent(this)`). `None` while the condition is
    /// unmigrated: list/view-model conditions, unresolved paths, and every
    /// non-candidates context bind.
    cell_binding: Option<RuntimeViewModelListenerCellBinding>,
}

impl RuntimeViewModelListenerInstance {
    /// A cloned machine re-registers a FRESH reporting sink on the same
    /// retained cell and the clone's own queue. Pending reports stay with the
    /// original machine, matching C++ instance-local listener vectors.
    fn clone_for_queue(&self, queue: &RuntimeCellNotificationQueue, listener_index: usize) -> Self {
        Self {
            view_model_index: self.view_model_index,
            property_path: self.property_path.clone(),
            actions: self.actions.clone(),
            observed: self.observed.clone(),
            cell_binding: self.cell_binding.as_ref().map(|binding| {
                RuntimeViewModelListenerCellBinding::new(
                    binding.cell.clone(),
                    queue,
                    listener_index,
                )
            }),
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum RuntimeViewModelListenerValue {
    Number(u32),
    Boolean(bool),
    String(Vec<u8>),
    Color(u32),
    Enum(u64),
    SymbolListIndex(u64),
    Asset(u64),
    Trigger(u64),
}

/// C++ `ListenerViewModel::reportToStateMachine` ignores the suppressed
/// trigger-zero write performed by `ViewModelInstanceTrigger::advanced()`
/// (`state_machine_instance.cpp:1374-1380`). Other changed values report.
fn runtime_view_model_listener_should_dispatch(
    previous: &RuntimeViewModelListenerValue,
    current: &RuntimeViewModelListenerValue,
) -> bool {
    previous != current && !matches!(current, RuntimeViewModelListenerValue::Trigger(0))
}

fn runtime_view_model_listener_value(
    context: &RuntimeOwnedViewModelInstance,
    property_path: &[usize],
) -> Option<RuntimeViewModelListenerValue> {
    context
        .number_value_by_property_path(property_path)
        .map(|value| RuntimeViewModelListenerValue::Number(value.to_bits()))
        .or_else(|| {
            context
                .boolean_value_by_property_path(property_path)
                .map(RuntimeViewModelListenerValue::Boolean)
        })
        .or_else(|| {
            context
                .string_value_by_property_path(property_path)
                .map(|value| RuntimeViewModelListenerValue::String(value.to_vec()))
        })
        .or_else(|| {
            context
                .color_value_by_property_path(property_path)
                .map(RuntimeViewModelListenerValue::Color)
        })
        .or_else(|| {
            context
                .enum_value_by_property_path(property_path)
                .map(RuntimeViewModelListenerValue::Enum)
        })
        .or_else(|| {
            context
                .symbol_list_index_value_by_property_path(property_path)
                .map(RuntimeViewModelListenerValue::SymbolListIndex)
        })
        .or_else(|| {
            context
                .asset_value_by_property_path(property_path)
                .map(RuntimeViewModelListenerValue::Asset)
        })
        .or_else(|| {
            context
                .trigger_value_by_property_path(property_path)
                .map(RuntimeViewModelListenerValue::Trigger)
        })
}

fn runtime_view_model_listener_value_for_context_path(
    context: &RuntimeOwnedViewModelInstance,
    context_path: &[usize],
    view_model_index: usize,
    property_path: &[usize],
) -> Option<RuntimeViewModelListenerValue> {
    let context_view_model_index = context.view_model_index_by_property_path(context_path)?;
    if context_view_model_index != view_model_index {
        return None;
    }
    let mut resolved_path =
        Vec::with_capacity(context_path.len().saturating_add(property_path.len()));
    resolved_path.extend_from_slice(context_path);
    resolved_path.extend_from_slice(property_path);
    runtime_view_model_listener_value(context, &resolved_path)
}

fn runtime_owned_view_model_trigger_value_for_context_path(
    context: &RuntimeOwnedViewModelInstance,
    context_path: &[usize],
    view_model_index: usize,
    property_index: usize,
) -> Option<u64> {
    if context.view_model_index_by_property_path(context_path)? != view_model_index {
        return None;
    }
    let mut property_path = context_path.to_vec();
    property_path.push(property_index);
    context.trigger_value_by_property_path(&property_path)
}

impl StateMachineInstance {
    pub(crate) fn new(
        state_machine_index: usize,
        state_machine: &RuntimeStateMachine,
        artboard: &ArtboardInstance,
    ) -> Self {
        let inputs = state_machine
            .inputs
            .iter()
            .enumerate()
            .map(|(index, input)| StateMachineInputInstance::new(index, input))
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
        let view_model_triggers = state_machine
            .view_model_triggers
            .iter()
            .map(|trigger| {
                StateMachineViewModelTriggerInstance::new(
                    trigger.global_id,
                    trigger.view_model_property_id,
                    trigger.value,
                )
            })
            .collect::<Vec<_>>();
        let default_view_model_triggers = view_model_triggers.clone();
        let transition_durations = state_machine
            .transition_duration_bindings
            .iter()
            .map(StateMachineTransitionDurationInstance::new)
            .collect::<Vec<_>>();
        let layers = state_machine
            .layers
            .iter()
            .map(|layer| {
                StateMachineLayerInstance::new(layer, artboard, &inputs, &bindable_numbers)
            })
            .collect();
        let mut data_bind_graph = RuntimeDataBindGraph::new(state_machine);
        data_bind_graph
            .attach_scripted_instances(&artboard.scripted_data_converter_instances_by_global);
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
        let view_model_listeners = state_machine
            .listeners
            .iter()
            .filter(|listener| listener.has_listener(RuntimeListenerType::ViewModel))
            .filter_map(|listener| {
                Some(RuntimeViewModelListenerInstance {
                    view_model_index: listener.view_model_index?,
                    property_path: listener.view_model_property_path.clone()?,
                    actions: listener.listener_actions.clone(),
                    observed: None,
                    cell_binding: None,
                })
            })
            .collect();
        Self {
            state_machine_index,
            default_view_model_index: state_machine.default_view_model_index,
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
            view_model_triggers,
            transition_durations,
            layers,
            reported_events: Vec::new(),
            reported_event_listener_index: 0,
            host_reported_event_index: 0,
            reporting_events: Vec::new(),
            reported_listener_view_models: RuntimeCellNotificationQueue::default(),
            reporting_listener_view_models: Vec::new(),
            pending_listener_events: Vec::new(),
            changed_state_count: 0,
            needs_advance: false,
            has_advanced_once: false,
            post_update_probe_pending: false,
            data_bind_graph,
            key_frame_data_bind_graphs,
            owned_view_model_candidates: Vec::new(),
            owned_view_model_candidate_generations: Vec::new(),
            pointer_down_listener_hits: Vec::new(),
            pointer_listener_states: Vec::new(),
            pointer_positions: Vec::new(),
            scripted_instances_by_global: BTreeMap::new(),
            focus: RuntimeFocusTree::from_artboard(artboard),
            scripted_listener_action_definitions: state_machine.scripted_listener_actions.clone(),
            scripted_listener_action_instances: BTreeMap::new(),
            script_error: None,
            view_model_listeners,
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
        if !self
            .scripted_listener_action_definitions
            .iter()
            .any(|definition| definition.action_global_id() == action_global_id)
        {
            return Err(ScriptError::new(format!(
                "state machine has no scripted listener action global {action_global_id}",
            )));
        }
        if self
            .scripted_listener_action_instances
            .contains_key(&action_global_id)
        {
            return Err(ScriptError::new(format!(
                "scripted listener action global {action_global_id} is already attached",
            )));
        }
        self.scripted_listener_action_instances
            .insert(action_global_id, RuntimeScriptInstanceHandle::new(instance));
        Ok(())
    }

    /// Apply a prevalidated data-context/input hydration batch to one retained
    /// scripted listener occurrence.
    pub fn hydrate_scripted_listener_action_instance(
        &mut self,
        action_global_id: u32,
        hydration: crate::ScriptListenerActionHydration,
    ) -> Result<(), ScriptError> {
        let handle = self
            .scripted_listener_action_instances
            .get(&action_global_id)
            .cloned()
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "scripted listener action global {action_global_id} is not attached",
                ))
            })?;
        hydration.apply(&mut **handle.borrow_mut(), &mut NoopScriptHost)
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
            || self.scripted_listener_action_definitions
                != source.scripted_listener_action_definitions
        {
            return Err(ScriptError::new(
                "cannot adopt scripted listener state from a different state-machine occurrence shape",
            ));
        }
        if !self.scripted_listener_action_instances.is_empty() || self.script_error.is_some() {
            return Err(ScriptError::new(
                "transaction candidate already has scripted listener state",
            ));
        }
        if source
            .scripted_listener_action_instances
            .keys()
            .any(|global_id| {
                !source
                    .scripted_listener_action_definitions
                    .iter()
                    .any(|definition| definition.action_global_id() == *global_id)
            })
        {
            return Err(ScriptError::new(
                "source occurrence has an unknown scripted listener attachment",
            ));
        }

        self.scripted_listener_action_instances = source.scripted_listener_action_instances.clone();
        self.script_error = source.script_error.clone();
        Ok(())
    }

    /// The first listener-script failure retained by compatibility entry
    /// points that cannot return a `Result`.
    pub fn script_error(&self) -> Option<&ScriptError> {
        self.script_error.as_ref()
    }

    pub fn state_machine_index(&self) -> usize {
        self.state_machine_index
    }

    pub(crate) fn requires_post_update_state_probe(&self) -> bool {
        self.requires_post_update_state_probe
    }

    pub fn changed_state_count(&self) -> usize {
        self.changed_state_count
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
        self.inputs.get(index)
    }

    pub fn input_named(&self, name: &str) -> Option<&StateMachineInputInstance> {
        self.inputs.iter().find(|input| input.name() == Some(name))
    }

    pub fn input_index_named(&self, name: &str) -> Option<usize> {
        self.inputs
            .iter()
            .position(|input| input.name() == Some(name))
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
        self.focus.traverse(0)
    }

    pub fn focus_previous(&mut self) -> bool {
        self.focus.traverse(1)
    }

    pub fn focus_up(&mut self) -> bool {
        self.focus.traverse(2)
    }

    pub fn focus_down(&mut self) -> bool {
        self.focus.traverse(3)
    }

    pub fn focus_left(&mut self) -> bool {
        self.focus.traverse(4)
    }

    pub fn focus_right(&mut self) -> bool {
        self.focus.traverse(5)
    }

    pub fn clear_focus(&mut self) -> bool {
        self.focus.clear_focus()
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

    pub fn pointer_down(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        self.try_pointer_down_with_script_host(artboard, x, y, pointer_id, &mut NoopScriptHost)
            .unwrap_or(false)
    }

    pub fn pointer_down_with_event_context(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        event_context: &StateMachineEventContext,
    ) -> bool {
        self.pointer_down_with_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            0.0,
            None,
            Some(event_context),
            &mut NoopScriptHost,
        )
        .unwrap_or(false)
    }

    pub fn pointer_down_with_owned_view_model_context(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.try_pointer_down_with_owned_view_model_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            context,
            &mut NoopScriptHost,
        )
        .unwrap_or(false)
    }

    pub fn pointer_down_with_owned_view_model_and_event_context(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
        event_context: &StateMachineEventContext,
    ) -> bool {
        self.pointer_down_with_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            0.0,
            Some(context),
            Some(event_context),
            &mut NoopScriptHost,
        )
        .unwrap_or(false)
    }

    pub fn try_pointer_down_with_script_host(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.try_pointer_down_with_timestamp_and_script_host(artboard, x, y, pointer_id, 0.0, host)
    }

    pub fn try_pointer_down_with_timestamp_and_script_host(
        &mut self,
        artboard: &ArtboardInstance,
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
        artboard: &ArtboardInstance,
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
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if !x.is_finite() || !y.is_finite() {
            return Ok(false);
        }
        if !self.focus.is_inert() {
            self.focus.sync(artboard);
        }
        self.pointer_down_listener_hits
            .retain(|hit| hit.pointer_id != pointer_id);
        let Some(state_machine) = artboard.state_machine(self.state_machine_index) else {
            return Ok(false);
        };
        let mut hit = false;
        for (listener_index, listener) in state_machine.listeners.iter().enumerate() {
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
            if let Some(action_type) = action_type
                && self.perform_listener_actions_with_event_context(
                    &listener.listener_actions,
                    owned_context.as_deref_mut(),
                    &script_pointer_invocation(pointer, action_type),
                    host,
                    event_context,
                )?
            {
                self.needs_advance = true;
            }
            self.record_pointer_input_for_listener(listener_index, pointer);
        }
        self.remember_pointer_position(pointer_id, x, y);
        Ok(hit)
    }

    pub fn pointer_move(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        seconds: f32,
        pointer_id: i32,
    ) -> bool {
        self.try_pointer_move_with_timestamp_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            seconds,
            &mut NoopScriptHost,
        )
        .unwrap_or(false)
    }

    pub fn pointer_move_with_owned_view_model_context(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        seconds: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        validate_pointer_timestamp(seconds)
            .and_then(|()| {
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
            })
            .unwrap_or(false)
    }

    pub fn try_pointer_move_with_script_host(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.try_pointer_move_with_timestamp_and_script_host(artboard, x, y, pointer_id, 0.0, host)
    }

    pub fn try_pointer_move_with_timestamp_and_script_host(
        &mut self,
        artboard: &ArtboardInstance,
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
        artboard: &ArtboardInstance,
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
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        self.try_pointer_up_with_script_host(artboard, x, y, pointer_id, &mut NoopScriptHost)
            .unwrap_or(false)
    }

    pub fn pointer_up_with_event_context(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        event_context: &StateMachineEventContext,
    ) -> bool {
        self.pointer_up_with_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            0.0,
            None,
            Some(event_context),
            &mut NoopScriptHost,
        )
        .unwrap_or(false)
    }

    pub fn pointer_up_with_owned_view_model_context(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.try_pointer_up_with_owned_view_model_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            context,
            &mut NoopScriptHost,
        )
        .unwrap_or(false)
    }

    pub fn pointer_up_with_owned_view_model_and_event_context(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
        event_context: &StateMachineEventContext,
    ) -> bool {
        self.pointer_up_with_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            0.0,
            Some(context),
            Some(event_context),
            &mut NoopScriptHost,
        )
        .unwrap_or(false)
    }

    pub fn try_pointer_up_with_script_host(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.try_pointer_up_with_timestamp_and_script_host(artboard, x, y, pointer_id, 0.0, host)
    }

    pub fn try_pointer_up_with_timestamp_and_script_host(
        &mut self,
        artboard: &ArtboardInstance,
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
        artboard: &ArtboardInstance,
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
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if !x.is_finite() || !y.is_finite() {
            return Ok(false);
        }
        if !self.focus.is_inert() {
            self.focus.sync(artboard);
        }
        let Some(state_machine) = artboard.state_machine(self.state_machine_index) else {
            self.pointer_down_listener_hits
                .retain(|hit| hit.pointer_id != pointer_id);
            return Ok(false);
        };
        let mut hit = false;
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
        for (listener_index, listener) in state_machine.listeners.iter().enumerate() {
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
            if let Some(action_type) = action_type
                && self.perform_listener_actions_with_event_context(
                    &listener.listener_actions,
                    owned_context.as_deref_mut(),
                    &script_pointer_invocation(pointer, action_type),
                    host,
                    action_event_context,
                )?
            {
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
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        self.try_pointer_exit_with_script_host(artboard, x, y, pointer_id, &mut NoopScriptHost)
            .unwrap_or(false)
    }

    pub fn pointer_exit_with_owned_view_model_context(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.try_pointer_exit_with_owned_view_model_context_and_script_host(
            artboard,
            x,
            y,
            pointer_id,
            context,
            &mut NoopScriptHost,
        )
        .unwrap_or(false)
    }

    pub fn try_pointer_exit_with_script_host(
        &mut self,
        artboard: &ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.try_pointer_exit_with_timestamp_and_script_host(artboard, x, y, pointer_id, 0.0, host)
    }

    pub fn try_pointer_exit_with_timestamp_and_script_host(
        &mut self,
        artboard: &ArtboardInstance,
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
        artboard: &ArtboardInstance,
        pointer_id: i32,
        listener_type: RuntimeListenerType,
        x: f32,
        y: f32,
        timestamp_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        let Some(state_machine) = artboard.state_machine(self.state_machine_index) else {
            return Ok(false);
        };
        let mut captured_targets = Vec::new();
        for capture in self.pointer_down_listener_hits.iter().filter(|capture| {
            capture.pointer_id == pointer_id
                && capture.drag_phase == Some(RuntimePointerDragPhase::Dragging)
        }) {
            let Some(target_local_id) = state_machine
                .listeners
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
        for listener in state_machine.listeners.iter() {
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
            if self.perform_listener_actions_with_event_context(
                &listener.listener_actions,
                owned_context.as_deref_mut(),
                &script_pointer_invocation(pointer, listener_type),
                host,
                event_context.as_ref(),
            )? {
                self.needs_advance = true;
            }
        }
        Ok(hit)
    }

    pub fn try_pointer_exit_with_owned_view_model_context_and_script_host(
        &mut self,
        artboard: &ArtboardInstance,
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
        artboard: &ArtboardInstance,
        listener_type: RuntimeListenerType,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if !x.is_finite() || !y.is_finite() {
            return Ok(false);
        }
        if !self.focus.is_inert() {
            self.focus.sync(artboard);
        }
        let Some(state_machine) = artboard.state_machine(self.state_machine_index) else {
            if listener_type == RuntimeListenerType::Exit {
                self.release_pointer_input(pointer_id);
                self.pointer_down_listener_hits
                    .retain(|capture| capture.pointer_id != pointer_id);
            }
            return Ok(false);
        };

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

        let mut hit = false;
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
        for (listener_index, listener) in state_machine.listeners.iter().enumerate() {
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
            if let Some(action_type) = action_type
                && self.perform_listener_actions_with_event_context(
                    &listener.listener_actions,
                    owned_context.as_deref_mut(),
                    &script_pointer_invocation(pointer, action_type),
                    host,
                    captured_event_context.as_ref(),
                )?
            {
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
        artboard: &ArtboardInstance,
        source_local_id: Option<usize>,
        events: &[StateMachineReportedEvent],
    ) -> bool {
        self.try_notify_events_with_script_host(
            artboard,
            source_local_id,
            events,
            &mut NoopScriptHost,
        )
        .unwrap_or(false)
    }

    pub(crate) fn try_notify_events_with_script_host(
        &mut self,
        artboard: &ArtboardInstance,
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
        artboard: &ArtboardInstance,
        source_local_id: Option<usize>,
        events: &[StateMachineReportedEvent],
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.notify_events_with_context(artboard, source_local_id, events, Some(context))
    }

    fn notify_events_with_context(
        &mut self,
        artboard: &ArtboardInstance,
        source_local_id: Option<usize>,
        events: &[StateMachineReportedEvent],
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        self.notify_events_with_context_and_script_host(
            artboard,
            source_local_id,
            events,
            owned_context.as_deref_mut(),
            &mut NoopScriptHost,
        )
        .unwrap_or(false)
    }

    fn notify_events_with_context_and_script_host(
        &mut self,
        artboard: &ArtboardInstance,
        source_local_id: Option<usize>,
        events: &[StateMachineReportedEvent],
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if events.is_empty() {
            return Ok(false);
        }
        let Some(state_machine) = artboard.state_machine(self.state_machine_index) else {
            return Ok(false);
        };

        let mut changed = false;
        for listener in state_machine.listeners.iter() {
            if !listener.has_listener(RuntimeListenerType::Event) {
                continue;
            }
            if source_local_id
                .is_some_and(|source_local_id| listener.target_local_id != source_local_id)
            {
                continue;
            }
            if let Some(event) = events.iter().find(|event| {
                listener
                    .event_local_indices
                    .contains(&event.event_local_index())
            }) {
                changed |= self.perform_listener_actions_with_event_context(
                    &listener.listener_actions,
                    owned_context.as_deref_mut(),
                    &ScriptListenerInvocation::ReportedEvent {
                        event_local_index: event.event_local_index(),
                        seconds_delay: event.seconds_delay(),
                    },
                    host,
                    event.context.as_ref(),
                )?;
            }
        }
        if changed {
            self.needs_advance = true;
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

        if self.pending_listener_events.is_empty()
            && next_event_index >= self.reported_events.len()
            && self.reported_listener_view_models.is_empty()
        {
            return false;
        }

        let mut changed = false;
        for _ in 0..MAX_EVENT_ITERATIONS {
            self.reported_events
                .append(&mut self.pending_listener_events);
            if next_event_index >= self.reported_events.len()
                && self.reported_listener_view_models.is_empty()
            {
                break;
            }

            // Mirrors C++ `StateMachineInstance::applyEvents()` updating
            // data binds before each queued notification batch
            // (`state_machine_instance.cpp:2320-2335`). Layer advancement is
            // deliberately left to the caller's single ordinary advance.
            self.update_data_binds_false();
            let mut events = std::mem::take(&mut self.reporting_events);
            events.clear();
            events.extend_from_slice(&self.reported_events[next_event_index..]);
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
            if event_changed && let Some(context) = owned_context.as_deref_mut() {
                changed |= self.bind_owned_view_model_context_mut(context);
            }

            // C++ reports the listener pointer once per genuine mutation and
            // preserves duplicates/FIFO order (`reportListenerViewModel`,
            // state_machine_instance.cpp:3021-3025,3048-3058). Temporarily
            // take both retained tables so actions can mutate this machine
            // and enqueue chained reports without cloning action vectors.
            let candidates = std::mem::take(&mut self.owned_view_model_candidates);
            let listeners = std::mem::take(&mut self.view_model_listeners);
            for &listener_index in &listener_indices {
                let Some(listener) = listeners.get(listener_index)
                else {
                    continue;
                };
                changed |= self
                    .perform_listener_actions_for_candidates(&candidates, &listener.actions)
                    .unwrap_or(false);
            }
            self.view_model_listeners = listeners;
            self.owned_view_model_candidates = candidates;
            listener_indices.clear();
            self.reporting_listener_view_models = listener_indices;
            self.reported_events
                .append(&mut self.pending_listener_events);
        }
        self.reported_event_listener_index = next_event_index.min(self.reported_events.len());
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

    fn perform_listener_actions(
        &mut self,
        listener_actions: &[RuntimeScheduledListenerAction],
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        invocation: &ScriptListenerInvocation,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.perform_listener_actions_with_event_context(
            listener_actions,
            owned_context,
            invocation,
            host,
            None,
        )
    }

    fn perform_listener_actions_with_event_context(
        &mut self,
        listener_actions: &[RuntimeScheduledListenerAction],
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        invocation: &ScriptListenerInvocation,
        host: &mut dyn ScriptHost,
        event_context: Option<&StateMachineEventContext>,
    ) -> Result<bool, ScriptError> {
        if let Some(error) = self.script_error.as_ref() {
            return Err(error.clone());
        }
        let mut changed = false;
        for action in listener_actions {
            match action {
                RuntimeScheduledListenerAction::FireEvent { event, .. } => {
                    let mut reported = event.clone();
                    reported.context = event_context.cloned();
                    self.pending_listener_events.push(reported);
                    changed = true;
                }
                RuntimeScheduledListenerAction::BoolChange {
                    input_index, value, ..
                } => {
                    if let Some(input) = self.inputs.get_mut(*input_index) {
                        changed |= input.apply_listener_bool_change(*value);
                    }
                }
                RuntimeScheduledListenerAction::NumberChange {
                    input_index, value, ..
                } => {
                    if let Some(input) = self.inputs.get_mut(*input_index) {
                        changed |= input.set_number(*value);
                    }
                }
                RuntimeScheduledListenerAction::TriggerChange { input_index, .. } => {
                    if let Some(input) = self.inputs.get_mut(*input_index) {
                        changed |= input.fire_trigger();
                    }
                }
                RuntimeScheduledListenerAction::ViewModelChange {
                    data_bind_index,
                    value,
                    ..
                } => {
                    changed |= if let Some(context) = owned_context.as_deref_mut() {
                        self.perform_listener_view_model_change(
                            *data_bind_index,
                            value,
                            Some(context),
                        )
                    } else if !self.owned_view_model_candidates.is_empty() {
                        let candidates = self.owned_view_model_candidates.clone();
                        let context_changed = self
                            .perform_listener_view_model_change_for_candidates(
                                &candidates,
                                *data_bind_index,
                                value,
                            );
                        if context_changed {
                            self.rebind_owned_view_model_context_candidates(&candidates, false);
                        }
                        context_changed
                    } else {
                        self.perform_listener_view_model_change(*data_bind_index, value, None)
                    };
                }
                RuntimeScheduledListenerAction::Scripted { definition, .. } => {
                    changed |= self.perform_script_object_listener_action(
                        definition.action_global_id(),
                        invocation,
                        host,
                    );
                    changed |=
                        self.perform_scripted_listener_action(definition, invocation, host)?;
                }
                RuntimeScheduledListenerAction::FocusTarget {
                    target_local_id, ..
                } => {
                    changed |= self.focus.set_focus_target(*target_local_id);
                }
                RuntimeScheduledListenerAction::FocusClear { .. } => {
                    changed |= self.focus.clear_focus();
                }
                RuntimeScheduledListenerAction::FocusTraversal { traversal_kind, .. } => {
                    changed |= self.focus.traverse(*traversal_kind);
                }
            }
        }
        if changed {
            self.needs_advance = true;
        }
        Ok(changed)
    }

    fn perform_script_object_listener_action(
        &mut self,
        global_id: u32,
        invocation: &ScriptListenerInvocation,
        host: &mut dyn ScriptHost,
    ) -> bool {
        let Some(instance) = self.scripted_instances_by_global.get(&global_id).cloned() else {
            return false;
        };
        // C++ consumes script errors inside ScriptedListenerAction::perform;
        // a failing callback never aborts state-machine dispatch.
        let result = (|| {
            let mut instance = instance.borrow_mut();
            let method = if instance.has_method(ScriptMethod::PerformAction)? {
                Some(ScriptListenerActionMethod::PerformAction)
            } else if instance.has_method(ScriptMethod::Perform)? {
                Some(ScriptListenerActionMethod::Perform)
            } else {
                None
            };
            let Some(method) = method else {
                return Ok(false);
            };
            instance.call_listener_action(method, invocation, host)?;
            Ok::<bool, ScriptError>(true)
        })();
        result.unwrap_or(false)
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

    fn perform_scripted_listener_action(
        &mut self,
        definition: &ScriptListenerActionDefinition,
        invocation: &ScriptListenerInvocation,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        let Some(instance) = self
            .scripted_listener_action_instances
            .get(&definition.action_global_id())
            .cloned()
        else {
            // Visual-only imports deliberately leave script tables
            // unattached. Their ordinary listener actions keep working while
            // embedded bytecode remains inert.
            return Ok(false);
        };
        let result = (|| {
            let mut instance = instance.borrow_mut();
            let method = if instance.has_method(ScriptMethod::PerformAction)? {
                Some(ScriptListenerActionMethod::PerformAction)
            } else if instance.has_method(ScriptMethod::Perform)? {
                Some(ScriptListenerActionMethod::Perform)
            } else {
                None
            };
            let Some(method) = method else {
                return Ok(false);
            };
            instance.call_listener_action(method, invocation, host)?;
            Ok(true)
        })();
        result.map_err(|error: ScriptError| {
            let error = error.with_context(format!(
                "ScriptedListenerAction global {} asset ordinal {} name '{}' failed",
                definition.action_global_id(),
                definition.asset_ordinal(),
                definition.asset_name()
            ));
            if self.script_error.is_none() {
                self.script_error = Some(error.clone());
            }
            self.script_error.clone().unwrap_or(error)
        })
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
        if !self.update_data_binds_apply_target_to_source() {
            return false;
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
        self.needs_advance = true;
        true
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

    fn set_default_view_model_trigger_target_mirror(
        &mut self,
        bindable_global_id: u32,
        value: u64,
    ) {
        let Some(trigger_global_id) = self
            .bindable_triggers
            .iter()
            .find(|trigger| trigger.global_id == bindable_global_id)
            .and_then(|trigger| match trigger.source {
                RuntimeBindableTriggerSource::DefaultViewModelTrigger { trigger_global_id } => {
                    Some(trigger_global_id)
                }
                RuntimeBindableTriggerSource::None => None,
            })
        else {
            return;
        };
        if let Some(trigger) = self
            .default_view_model_triggers
            .iter_mut()
            .find(|trigger| trigger.global_id() == trigger_global_id)
        {
            trigger.set_value(value);
        }
        if self
            .data_bind_graph
            .default_view_model_source_context_bound()
            && let Some(trigger) = self
                .view_model_triggers
                .iter_mut()
                .find(|trigger| trigger.global_id() == trigger_global_id)
        {
            trigger.set_value(value);
        }
    }

    fn set_default_view_model_trigger_source_mirror(
        &mut self,
        view_model_property_id: u32,
        value: u64,
    ) {
        if let Some(trigger) = self
            .default_view_model_triggers
            .iter_mut()
            .find(|trigger| trigger.view_model_property_id() == view_model_property_id)
        {
            trigger.set_value(value);
        }
        if self
            .data_bind_graph
            .default_view_model_source_context_bound()
            && let Some(trigger) = self
                .view_model_triggers
                .iter_mut()
                .find(|trigger| trigger.view_model_property_id() == view_model_property_id)
        {
            trigger.set_value(value);
        }
    }

    fn sync_default_view_model_trigger_mirrors_from_data_bind_graph(&mut self) {
        let updates: Vec<(u32, u64)> = self
            .bindable_triggers
            .iter()
            .filter_map(|bindable_trigger| {
                if !matches!(
                    bindable_trigger.source,
                    RuntimeBindableTriggerSource::DefaultViewModelTrigger { .. }
                ) {
                    return None;
                }
                bindable_trigger
                    .data_bind_indices
                    .iter()
                    .find_map(|data_bind_index| {
                        self.data_bind_graph
                            .default_view_model_trigger_source_value_for_data_bind(*data_bind_index)
                    })
                    .map(|value| (bindable_trigger.global_id, value))
            })
            .collect();
        for (bindable_global_id, value) in updates {
            self.set_default_view_model_trigger_target_mirror(bindable_global_id, value);
        }
    }

    pub fn set_default_view_model_trigger_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let bindable_global_id = self
            .data_bind_graph
            .default_view_model_trigger_target_global_id_for_data_bind(data_bind_index);
        let trigger_property_id = self
            .data_bind_graph
            .default_view_model_trigger_source_property_id_for_data_bind(data_bind_index);
        if !self
            .data_bind_graph
            .set_default_view_model_trigger_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        if let Some(trigger_property_id) = trigger_property_id {
            self.set_default_view_model_trigger_source_mirror(trigger_property_id, value);
        }
        if let Some(bindable_global_id) = bindable_global_id {
            self.set_default_view_model_trigger_target_mirror(bindable_global_id, value);
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
        let bindable_global_ids = self
            .data_bind_graph
            .default_view_model_trigger_target_global_ids_for_source_path(&path);
        if !self
            .data_bind_graph
            .set_default_view_model_trigger_source_for_path(&path, value)
        {
            return false;
        }
        if let Some(trigger_property_id) = path.last().copied() {
            self.set_default_view_model_trigger_source_mirror(trigger_property_id, value);
        }
        for bindable_global_id in bindable_global_ids {
            self.set_default_view_model_trigger_target_mirror(bindable_global_id, value);
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
        let bindable_global_ids = self
            .data_bind_graph
            .default_view_model_trigger_target_global_ids_for_source_path(&handle.path);
        if !self
            .data_bind_graph
            .set_default_view_model_trigger_source_for_path(&handle.path, value)
        {
            return false;
        }
        if let Some(trigger_property_id) = handle.path.last().copied() {
            self.set_default_view_model_trigger_source_mirror(trigger_property_id, value);
        }
        for bindable_global_id in bindable_global_ids {
            self.set_default_view_model_trigger_target_mirror(bindable_global_id, value);
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
        let source_view_model_index = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index)
            .and_then(|source_path| source_path.first().copied())
            .and_then(|view_model_index| usize::try_from(view_model_index).ok());
        let Some(bindable_trigger) = self
            .bindable_triggers
            .iter_mut()
            .find(|bindable_trigger| bindable_trigger.has_data_bind_index(data_bind_index))
        else {
            return false;
        };

        bindable_trigger.set_value(value);
        let trigger_property_id = source_view_model_index
            .is_some_and(|view_model_index| Some(view_model_index) == self.default_view_model_index)
            .then(|| {
                self.data_bind_graph
                    .default_view_model_trigger_source_property_id_for_data_bind(data_bind_index)
            })
            .flatten();
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
        if let Some((trigger_property_id, value)) =
            trigger_property_id.and_then(|trigger_property_id| {
                let value = context.trigger_value_by_property_path(property_path)?;
                Some((trigger_property_id, value))
            })
            && let Some(trigger) = self
                .view_model_triggers
                .iter_mut()
                .find(|trigger| trigger.view_model_property_id() == trigger_property_id)
        {
            trigger.set_value(value);
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
        let bindable_global_id = self
            .data_bind_graph
            .default_view_model_trigger_target_global_id_for_data_bind(data_bind_index);
        let trigger_property_id = self
            .data_bind_graph
            .default_view_model_trigger_source_property_id_for_data_bind(data_bind_index);
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
        if let Some(trigger_property_id) = trigger_property_id {
            if let Some(trigger) = self
                .default_view_model_triggers
                .iter_mut()
                .find(|trigger| trigger.view_model_property_id() == trigger_property_id)
            {
                trigger.set_value(value);
            }
            if let Some(trigger) = self
                .view_model_triggers
                .iter_mut()
                .find(|trigger| trigger.view_model_property_id() == trigger_property_id)
            {
                trigger.set_value(value);
            }
        }
        if let Some(trigger_global_id) = bindable_global_id.and_then(|bindable_global_id| {
            self.bindable_triggers
                .iter()
                .find(|trigger| trigger.global_id == bindable_global_id)
                .and_then(|trigger| match trigger.source {
                    RuntimeBindableTriggerSource::DefaultViewModelTrigger { trigger_global_id } => {
                        Some(trigger_global_id)
                    }
                    RuntimeBindableTriggerSource::None => None,
                })
        }) && let Some(trigger) = self
            .view_model_triggers
            .iter_mut()
            .find(|trigger| trigger.global_id() == trigger_global_id)
        {
            trigger.set_value(value);
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
        self.clear_owned_view_model_handle();
        if !self.data_bind_graph.bind_empty_data_context() {
            return false;
        }
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.bind_empty_data_context();
        }
        self.needs_advance = true;
        true
    }

    pub fn bind_default_view_model_context(&mut self) -> bool {
        self.clear_owned_view_model_handle();
        if !self.data_bind_graph.bind_default_view_model_context() {
            return false;
        }
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.bind_default_view_model_context();
        }
        self.sync_bindable_font_assets_from_default_context();
        self.bind_active_default_view_model_triggers();
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

    pub fn bind_view_model_instance_context(
        &mut self,
        file: &RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
    ) -> bool {
        self.clear_owned_view_model_handle();
        if !self.data_bind_graph.bind_view_model_instance_context(
            file,
            view_model_index,
            instance_index,
        ) {
            return false;
        }
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.bind_view_model_instance_context(file, view_model_index, instance_index);
        }
        self.sync_bindable_font_assets_from_imported_instance(
            file,
            view_model_index,
            instance_index,
        );
        self.bind_active_imported_view_model_triggers(file, view_model_index, instance_index, None);
        self.needs_advance = true;
        true
    }

    pub fn bind_imported_view_model_context(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeImportedViewModelInstanceContext,
    ) -> bool {
        self.clear_owned_view_model_handle();
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
        self.bind_active_imported_view_model_triggers(
            file,
            context.view_model_index,
            context.instance_index,
            Some(context),
        );
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
        self.bind_owned_view_model_context_candidates(&[
            RuntimeOwnedViewModelBindingCandidate::context_handle(context),
        ])
    }

    fn bind_owned_view_model_snapshot(&mut self, context: &RuntimeOwnedViewModelInstance) -> bool {
        let mut changed = self.data_bind_graph.bind_owned_view_model_context(context);
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            changed |= graph.bind_owned_view_model_context(context);
        }
        self.sync_bindable_font_assets_from_owned_context(context);
        self.bind_active_owned_view_model_triggers(context);
        for listener_actions in self.changed_view_model_listener_actions(context) {
            changed |= self
                .perform_listener_actions(
                    &listener_actions,
                    None,
                    &ScriptListenerInvocation::None,
                    &mut NoopScriptHost,
                )
                .unwrap_or(false);
        }
        if changed {
            self.needs_advance = true;
        }
        changed
    }

    /// Rebind an owned ViewModel context and dispatch typed ViewModel-change
    /// listeners into that same retained context.
    pub fn bind_owned_view_model_context_mut(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.bind_owned_view_model_context_mut_with_listener_change(context)
            .0
    }

    pub(crate) fn bind_owned_view_model_context_mut_with_listener_change(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> (bool, bool) {
        self.clear_owned_view_model_handle();
        let mut changed = self.data_bind_graph.bind_owned_view_model_context(context);
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            changed |= graph.bind_owned_view_model_context(context);
        }
        self.sync_bindable_font_assets_from_owned_context(context);
        self.bind_active_owned_view_model_triggers(context);
        let actions = self.changed_view_model_listener_actions(context);
        let mut listener_changed = false;
        for listener_actions in actions {
            listener_changed |= self
                .perform_listener_actions(
                    &listener_actions,
                    Some(context),
                    &ScriptListenerInvocation::None,
                    &mut NoopScriptHost,
                )
                .unwrap_or(false);
        }
        changed |= listener_changed;
        if changed {
            self.needs_advance = true;
        }
        (changed, listener_changed)
    }

    fn changed_view_model_listener_actions(
        &mut self,
        context: &RuntimeOwnedViewModelInstance,
    ) -> Vec<Vec<RuntimeScheduledListenerAction>> {
        self.changed_view_model_listener_actions_for_context_chain(context, &[&[]])
    }

    fn changed_view_model_listener_actions_for_context_chain(
        &mut self,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> Vec<Vec<RuntimeScheduledListenerAction>> {
        let mut changed = Vec::new();
        for listener in &mut self.view_model_listeners {
            let current = context_chain.iter().find_map(|context_path| {
                runtime_view_model_listener_value_for_context_path(
                    context,
                    context_path,
                    listener.view_model_index,
                    &listener.property_path,
                )
            });
            let should_dispatch = listener
                .observed
                .as_ref()
                .zip(current.as_ref())
                .is_some_and(|(previous, current)| {
                    runtime_view_model_listener_should_dispatch(previous, current)
                });
            listener.observed = current;
            if should_dispatch {
                changed.push(listener.actions.clone());
            }
        }
        changed
    }

    fn bind_view_model_listener_cells_for_candidates(
        &mut self,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
    ) {
        for (listener_index, listener) in self.view_model_listeners.iter_mut().enumerate() {
            let mut source_path = Vec::with_capacity(listener.property_path.len() + 1);
            let Ok(view_model_index) = u32::try_from(listener.view_model_index) else {
                listener.cell_binding = None;
                continue;
            };
            source_path.push(view_model_index);
            source_path.extend(
                listener
                    .property_path
                    .iter()
                    .filter_map(|property_index| u32::try_from(*property_index).ok()),
            );
            // C++ `DataContext::getViewModelProperty` returns the retained
            // `ViewModelInstanceValue`; the listener registers itself as a
            // dependent and does not snapshot or dispatch the initial value
            // (`state_machine_instance.cpp:1331-1372,1401-1407`).
            let cell = candidates.iter().find_map(|candidate| {
                let property_path = candidate.property_path_for_source_path(&source_path)?;
                candidate
                    .context
                    .borrow()
                    .cell_by_property_path(&property_path)
            });
            if let Some(cell) = cell {
                let same_cell = listener
                    .cell_binding
                    .as_ref()
                    .is_some_and(|binding| binding.cell.ptr_eq(&cell));
                if !same_cell {
                    listener.cell_binding = Some(RuntimeViewModelListenerCellBinding::new(
                        cell,
                        &self.reported_listener_view_models,
                        listener_index,
                    ));
                }
            } else {
                listener.cell_binding = None;
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
        }
    }

    fn perform_listener_view_model_change_for_candidates(
        &mut self,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
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

        for candidate in candidates {
            let Some(property_path) = candidate.property_path_for_source_path(&source_path) else {
                continue;
            };
            let mut context = candidate.context.borrow_mut();
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
            if let Some(changed) = changed {
                return changed;
            }
        }
        false
    }

    fn rebind_owned_view_model_context_candidates(
        &mut self,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
        sync_active_triggers: bool,
    ) -> bool {
        let mut changed = self
            .data_bind_graph
            .bind_owned_view_model_context_candidates(candidates);
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            changed |= graph.bind_owned_view_model_context_candidates(candidates);
        }
        self.sync_bindable_font_assets_from_owned_candidates(candidates);
        if sync_active_triggers {
            self.bind_active_owned_view_model_triggers_for_candidates(candidates);
        }
        changed
    }

    fn perform_listener_actions_for_candidates(
        &mut self,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
        listener_actions: &[RuntimeScheduledListenerAction],
    ) -> Result<bool, ScriptError> {
        let mut changed = false;
        for action in listener_actions {
            if let RuntimeScheduledListenerAction::ViewModelChange {
                data_bind_index,
                value,
                ..
            } = action
            {
                let context_changed = self.perform_listener_view_model_change_for_candidates(
                    candidates,
                    *data_bind_index,
                    value,
                );
                changed |= context_changed;
                if context_changed {
                    changed |= self.rebind_owned_view_model_context_candidates(candidates, false);
                }
                continue;
            }
            changed |= self.perform_listener_actions(
                std::slice::from_ref(action),
                None,
                &ScriptListenerInvocation::None,
                &mut NoopScriptHost,
            )?;
        }
        Ok(changed)
    }

    pub fn bind_owned_view_model_contexts(
        &mut self,
        context: &RuntimeOwnedViewModelContext,
    ) -> bool {
        let mut candidates = Vec::new();
        if let Some(main) = context.main_handle() {
            candidates.push(RuntimeOwnedViewModelBindingCandidate::root_handle(main));
        }
        candidates.extend(context.global_slot_handles().map(|(slot, handle)| {
            RuntimeOwnedViewModelBindingCandidate::declared_global_slot(handle, slot)
        }));
        self.bind_owned_view_model_context_candidates(&candidates)
    }

    pub(crate) fn bind_owned_view_model_context_chain(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> bool {
        self.clear_owned_view_model_handle();
        let mut changed =
            self.data_bind_graph
                .bind_owned_view_model_context_chain(file, context, context_chain);
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            changed |= graph.bind_owned_view_model_context_chain(file, context, context_chain);
        }
        self.sync_bindable_font_assets_from_owned_context_chain(file, context, context_chain);
        self.bind_active_owned_view_model_triggers_for_context_chain(context, context_chain);
        for listener_actions in
            self.changed_view_model_listener_actions_for_context_chain(context, context_chain)
        {
            changed |= self
                .perform_listener_actions(
                    &listener_actions,
                    None,
                    &ScriptListenerInvocation::None,
                    &mut NoopScriptHost,
                )
                .unwrap_or(false);
        }
        if changed {
            self.needs_advance = true;
        }
        changed
    }

    pub(crate) fn bind_owned_view_model_context_candidates(
        &mut self,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
    ) -> bool {
        let identity_changed = self.owned_view_model_candidates.len() != candidates.len()
            || self
                .owned_view_model_candidates
                .iter()
                .zip(candidates)
                .any(|(bound, candidate)| !bound.same_binding(candidate));
        let generations = candidates
            .iter()
            .map(RuntimeOwnedViewModelBindingCandidate::mutation_generation)
            .collect::<Vec<_>>();
        if !identity_changed && self.owned_view_model_candidate_generations == generations {
            return false;
        }

        let changed = self.rebind_owned_view_model_context_candidates(candidates, false);
        if identity_changed {
            self.bind_active_owned_view_model_triggers_for_candidates(candidates);
        } else {
            self.refresh_active_owned_view_model_triggers_for_candidates(candidates);
        }
        self.bind_view_model_listener_cells_for_candidates(candidates);
        self.owned_view_model_candidates = candidates.to_vec();
        if candidates.is_empty() {
            self.reset_active_view_model_triggers();
        }
        self.owned_view_model_candidate_generations = generations;
        if changed || identity_changed {
            self.needs_advance = true;
        }
        changed || identity_changed
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
            runtime_owned_font_asset_value_for_state_machine_source(context, &source.path).cloned()
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
                context
                    .font_asset_value_by_context_source_path(
                        file,
                        context_path,
                        &source.path,
                        false,
                    )
                    .cloned()
            })
        });
    }

    fn sync_bindable_font_assets_from_owned_candidates(
        &mut self,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
    ) {
        self.sync_bindable_font_assets(|source| {
            candidates.iter().find_map(|candidate| {
                let property_path = candidate.property_path_for_source_path(&source.path)?;
                candidate
                    .context
                    .borrow()
                    .font_asset_value_by_property_path(&property_path)
                    .cloned()
            })
        });
    }

    pub fn advance_data_context(&mut self) -> bool {
        self.advance_data_context_with_trigger_reset(true)
    }

    fn advance_data_context_with_trigger_reset(&mut self, reset_triggers: bool) -> bool {
        self.refresh_owned_view_model_candidates();
        if !self.data_bind_graph.data_context_present() {
            return false;
        }
        if self.data_bind_graph.default_view_model_context_bound() {
            self.data_bind_graph
                .apply_default_view_model_number_targets_to_sources(&self.bindable_numbers);
            self.data_bind_graph
                .apply_default_view_model_symbol_list_index_targets_to_sources(
                    &self.bindable_integers,
                );
            self.data_bind_graph
                .apply_default_view_model_boolean_targets_to_sources(&self.bindable_booleans);
            self.data_bind_graph
                .apply_default_view_model_string_targets_to_sources(&self.bindable_strings);
            self.data_bind_graph
                .apply_default_view_model_color_targets_to_sources(&self.bindable_colors);
            self.data_bind_graph
                .apply_default_view_model_enum_targets_to_sources(&self.bindable_enums);
            self.data_bind_graph
                .apply_default_view_model_asset_targets_to_sources(&self.bindable_assets);
            self.data_bind_graph
                .apply_default_view_model_artboard_targets_to_sources(&self.bindable_artboards);
            self.data_bind_graph
                .apply_default_view_model_list_targets_to_sources();
            self.data_bind_graph
                .apply_default_view_model_trigger_targets_to_sources(&self.bindable_triggers);
            self.data_bind_graph
                .apply_default_view_model_view_model_targets_to_sources(&self.bindable_view_models);
            self.apply_default_view_model_bindings(true, RuntimeDataBindGraphApplyPhase::Immediate);
            if reset_triggers {
                self.reset_advanced_data_context();
            }
        }
        true
    }

    pub(crate) fn reset_advanced_data_context(&mut self) {
        if !self.data_bind_graph.default_view_model_context_bound() {
            return;
        }
        for trigger in &mut self.view_model_triggers {
            // C++ `ViewModelInstance::advanced()` walks the retained values;
            // `ViewModelInstanceTrigger::advanced()` zeroes `propertyValue`
            // under SuppressDelegation, then clears its changed flag.
            trigger.advanced();
        }
        self.data_bind_graph.reset_bound_trigger_sources();
        if self
            .data_bind_graph
            .default_view_model_source_context_bound()
        {
            self.sync_default_view_model_triggers_from_active();
        }
    }

    /// Mirrors C++ `DataBindContainer::updateDataBinds(false)`. Dirty
    /// source-to-target values must be visible to event listeners and
    /// transition conditions without polling or writing target-to-source
    /// bindings.
    fn update_data_binds_false(&mut self) {
        // Retained cells cascade into per-bind sinks without borrowing this
        // machine. Fold that dirt before each applyEvents batch so listener-
        // authored chained writes have the same visibility as C++'s direct
        // `DataBind::addDirt` calls (`state_machine_instance.cpp:2328`).
        self.data_bind_graph.collect_retained_owned_source_dirt();
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.collect_retained_owned_source_dirt();
        }
        if self.data_bind_graph.default_view_model_context_bound() {
            self.apply_default_view_model_bindings(
                true,
                RuntimeDataBindGraphApplyPhase::UpdateDataBindsFalse,
            );
        }
    }

    fn refresh_owned_view_model_candidates(&mut self) -> bool {
        if self.owned_view_model_candidates.is_empty() {
            return false;
        }
        // #RB-1 e3: migrated sources refresh from retained-cell dirt — the
        // direction-engine channel — BEFORE the mutation-clock poll below, so
        // cell writes that never bump the clock still propagate, and stale
        // sink dirt cannot flip a fresh reconcile's origin afterwards.
        // Unmigrated sources still rely entirely on the generation rebind.
        let mut collected = self.data_bind_graph.collect_retained_owned_source_dirt();
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            collected |= graph.collect_retained_owned_source_dirt();
        }
        let candidates = self.owned_view_model_candidates.clone();
        let bound = self.bind_owned_view_model_context_candidates(&candidates);
        if collected {
            self.needs_advance = true;
        }
        collected || bound
    }

    fn clear_owned_view_model_handle(&mut self) {
        self.owned_view_model_candidates.clear();
        self.owned_view_model_candidate_generations.clear();
        // Leaving the candidates path drops every listener's dependent
        // registration (C++ `ListenerViewModel::clearDataContext`); snapshot
        // and context-chain binds poll the observed copy instead.
        for listener in &mut self.view_model_listeners {
            listener.cell_binding = None;
        }
    }

    pub fn update_data_binds_apply_target_to_source(&mut self) -> bool {
        if !self.data_bind_graph.data_context_present() {
            return false;
        }
        if self.data_bind_graph.default_view_model_context_bound() {
            self.data_bind_graph
                .apply_default_view_model_number_public_update_targets_to_sources(
                    &self.bindable_numbers,
                );
            self.data_bind_graph
                .apply_default_view_model_symbol_list_index_public_update_targets_to_sources(
                    &self.bindable_integers,
                );
            self.data_bind_graph
                .apply_default_view_model_boolean_public_update_targets_to_sources(
                    &self.bindable_booleans,
                );
            self.data_bind_graph
                .apply_default_view_model_string_public_update_targets_to_sources(
                    &self.bindable_strings,
                );
            self.data_bind_graph
                .apply_default_view_model_color_public_update_targets_to_sources(
                    &self.bindable_colors,
                );
            self.data_bind_graph
                .apply_default_view_model_enum_public_update_targets_to_sources(
                    &self.bindable_enums,
                );
            self.data_bind_graph
                .apply_default_view_model_asset_public_update_targets_to_sources(
                    &self.bindable_assets,
                );
            self.data_bind_graph
                .apply_default_view_model_artboard_public_update_targets_to_sources(
                    &self.bindable_artboards,
                );
            self.data_bind_graph
                .apply_default_view_model_list_public_update_targets_to_sources();
            self.data_bind_graph
                .apply_default_view_model_trigger_public_update_targets_to_sources(
                    &self.bindable_triggers,
                );
            self.data_bind_graph
                .apply_default_view_model_view_model_public_update_targets_to_sources(
                    &self.bindable_view_models,
                );
            self.apply_default_view_model_bindings(
                true,
                RuntimeDataBindGraphApplyPhase::PublicUpdate,
            );
            self.sync_default_view_model_trigger_mirrors_from_data_bind_graph();
        }
        true
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

    pub fn reported_event(&self, index: usize) -> Option<&StateMachineReportedEvent> {
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
    pub fn take_reported_events(&mut self) -> Vec<StateMachineReportedEvent> {
        let start = self
            .host_reported_event_index
            .min(self.reported_events.len());
        let events = self.reported_events[start..].to_vec();
        self.host_reported_event_index = self.reported_events.len();
        events
    }

    pub fn view_model_trigger_count(&self, index: usize) -> Option<u64> {
        self.default_view_model_triggers
            .get(index)
            .map(StateMachineViewModelTriggerInstance::value)
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
            .cell_binding
            .as_ref()
            .map(|binding| binding.cell.clone())
    }

    #[cfg(test)]
    pub(crate) fn active_view_model_trigger_count(&self, index: usize) -> Option<u64> {
        self.view_model_triggers
            .get(index)
            .map(StateMachineViewModelTriggerInstance::value)
    }

    #[cfg(test)]
    pub(crate) fn active_view_model_trigger_is_fireable_for_layer(
        &self,
        index: usize,
        layer_index: usize,
    ) -> Option<bool> {
        self.view_model_triggers
            .get(index)
            .map(|trigger| trigger.is_fireable_for_layer(layer_index))
    }

    pub fn view_model_trigger_value_count(&self) -> usize {
        self.default_view_model_triggers.len()
    }

    pub fn view_model_trigger_property_id(&self, index: usize) -> Option<u32> {
        self.default_view_model_triggers
            .get(index)
            .map(StateMachineViewModelTriggerInstance::view_model_property_id)
    }

    pub(crate) fn advance_with_owned_view_model_context(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        elapsed_seconds: f32,
        context: &mut RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.bind_owned_view_model_context_mut(context);
        self.advance_with_report_mode(
            artboard,
            state_machine,
            elapsed_seconds,
            Some(context),
            &mut NoopScriptHost,
        )
        .unwrap_or(false)
    }

    pub(crate) fn advance_preserving_reported_events(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        elapsed_seconds: f32,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        self.advance_with_report_mode(
            artboard,
            state_machine,
            elapsed_seconds,
            owned_context,
            &mut NoopScriptHost,
        )
        .unwrap_or(false)
    }

    pub(crate) fn advance_after_state_probe(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        elapsed_seconds: f32,
    ) -> bool {
        let probed_state_count = self.changed_state_count;
        let changed = self
            .advance_with_report_mode(
                artboard,
                state_machine,
                elapsed_seconds,
                None,
                &mut NoopScriptHost,
            )
            .unwrap_or(false);
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
        self.try_change_state_with_script_host(artboard, state_machine, &mut NoopScriptHost)
            .unwrap_or(false)
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
        self.post_update_probe_pending = false;
        if !self.focus.is_inert() {
            self.focus.sync(artboard);
        }
        self.refresh_owned_view_model_candidates();
        self.update_data_binds_false();

        let data_context_present = self.data_bind_graph.data_context_present();
        let data_context_view_model_bound = self.data_bind_graph.default_view_model_context_bound();
        let mut changed_state = false;
        for (layer_index, layer) in state_machine
            .layers
            .iter()
            .enumerate()
            .take(self.layers.len())
        {
            let scripted_instances = self.scripted_instances_by_global.clone();
            let focus = self.focus.clone();
            let bindable_numbers = self.bindable_numbers.clone();
            let bindable_integers = self.bindable_integers.clone();
            let bindable_colors = self.bindable_colors.clone();
            let bindable_strings = self.bindable_strings.clone();
            let bindable_enums = self.bindable_enums.clone();
            let bindable_assets = self.bindable_assets.clone();
            let bindable_artboards = self.bindable_artboards.clone();
            let bindable_triggers = self.bindable_triggers.clone();
            let bindable_view_models = self.bindable_view_models.clone();
            let bindable_booleans = self.bindable_booleans.clone();
            let evaluation_context = TransitionEvaluationContext {
                scripted_instances: &scripted_instances,
                focus: &focus,
                bindable_numbers: &bindable_numbers,
                bindable_integers: &bindable_integers,
                bindable_colors: &bindable_colors,
                bindable_strings: &bindable_strings,
                bindable_enums: &bindable_enums,
                bindable_assets: &bindable_assets,
                bindable_artboards: &bindable_artboards,
                bindable_triggers: &bindable_triggers,
                bindable_view_models: &bindable_view_models,
                bindable_booleans: &bindable_booleans,
                data_context_present,
                data_context_view_model_bound,
                layer_index,
            };
            let mut executor = RuntimeStateMachineListenerActionExecutor {
                data_bind_graph: &mut self.data_bind_graph,
                default_view_model_index: self.default_view_model_index,
                owned_view_model_context: None,
                owned_view_model_candidates: self.owned_view_model_candidates.clone(),
                scripted_listener_action_instances: &self.scripted_listener_action_instances,
                script_error: &mut self.script_error,
                focus: &mut self.focus,
                host,
            };
            let layer_changed = self.layers[layer_index].update_state(
                &evaluation_context,
                artboard,
                layer,
                &mut self.inputs,
                &mut self.bindable_numbers,
                &mut self.bindable_integers,
                &mut self.bindable_colors,
                &mut self.bindable_strings,
                &mut self.bindable_enums,
                &mut self.bindable_assets,
                &mut self.bindable_artboards,
                &mut self.bindable_lists,
                &mut self.bindable_triggers,
                &mut self.bindable_view_models,
                &mut self.bindable_booleans,
                &mut self.transition_durations,
                data_context_present,
                data_context_view_model_bound,
                &mut self.view_model_triggers,
                &mut self.reported_events,
                &mut executor,
            )?;
            if layer_changed {
                changed_state = true;
                self.changed_state_count += 1;
            }
        }
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
        if !self.focus.is_inert() {
            self.focus.sync(artboard);
        }
        // A retained context can be mutated through any alias between
        // frames. Refresh it before the clean-frame fast path so the new
        // generation schedules the ordinary updateDataBinds(false) work.
        self.refresh_owned_view_model_candidates();
        self.reported_events
            .append(&mut self.pending_listener_events);
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
        let has_data_bindings = self.data_bind_graph.has_bindings();
        if has_data_bindings {
            self.apply_default_view_model_bindings(
                true,
                RuntimeDataBindGraphApplyPhase::BeforeStatefulAdvance,
            );
        }
        let data_bind_advance = if has_data_bindings {
            self.data_bind_graph
                .advance_stateful_converters(elapsed_seconds)
        } else {
            Default::default()
        };
        if has_data_bindings {
            self.apply_default_view_model_bindings(
                true,
                RuntimeDataBindGraphApplyPhase::AfterStatefulAdvance,
            );
        }
        let data_context_present = self.data_bind_graph.data_context_present();
        let data_context_view_model_bound = self.data_bind_graph.default_view_model_context_bound();
        let mut keep_going = false;
        for (layer_index, layer) in state_machine
            .layers
            .iter()
            .enumerate()
            .take(self.layers.len())
        {
            let scripted_instances = self.scripted_instances_by_global.clone();
            let focus = self.focus.clone();
            let bindable_numbers = self.bindable_numbers.clone();
            let bindable_integers = self.bindable_integers.clone();
            let bindable_colors = self.bindable_colors.clone();
            let bindable_strings = self.bindable_strings.clone();
            let bindable_enums = self.bindable_enums.clone();
            let bindable_assets = self.bindable_assets.clone();
            let bindable_artboards = self.bindable_artboards.clone();
            let bindable_triggers = self.bindable_triggers.clone();
            let bindable_view_models = self.bindable_view_models.clone();
            let bindable_booleans = self.bindable_booleans.clone();
            let evaluation_context = TransitionEvaluationContext {
                scripted_instances: &scripted_instances,
                focus: &focus,
                bindable_numbers: &bindable_numbers,
                bindable_integers: &bindable_integers,
                bindable_colors: &bindable_colors,
                bindable_strings: &bindable_strings,
                bindable_enums: &bindable_enums,
                bindable_assets: &bindable_assets,
                bindable_artboards: &bindable_artboards,
                bindable_triggers: &bindable_triggers,
                bindable_view_models: &bindable_view_models,
                bindable_booleans: &bindable_booleans,
                data_context_present,
                data_context_view_model_bound,
                layer_index,
            };
            let mut executor = RuntimeStateMachineListenerActionExecutor {
                data_bind_graph: &mut self.data_bind_graph,
                default_view_model_index: self.default_view_model_index,
                owned_view_model_context: owned_context.as_deref_mut(),
                owned_view_model_candidates: self.owned_view_model_candidates.clone(),
                scripted_listener_action_instances: &self.scripted_listener_action_instances,
                script_error: &mut self.script_error,
                focus: &mut self.focus,
                host: &mut *host,
            };
            let layer_result = {
                let layer_instance = &mut self.layers[layer_index];
                layer_instance.advance(
                    &evaluation_context,
                    artboard,
                    layer,
                    &self.key_frame_data_bind_graphs,
                    elapsed_seconds,
                    &mut self.inputs,
                    &mut self.bindable_numbers,
                    &mut self.bindable_integers,
                    &mut self.bindable_colors,
                    &mut self.bindable_strings,
                    &mut self.bindable_enums,
                    &mut self.bindable_assets,
                    &mut self.bindable_artboards,
                    &mut self.bindable_lists,
                    &mut self.bindable_triggers,
                    &mut self.bindable_view_models,
                    &mut self.bindable_booleans,
                    &mut self.transition_durations,
                    data_context_present,
                    data_context_view_model_bound,
                    &mut self.view_model_triggers,
                    &mut self.reported_events,
                    &mut executor,
                )
            }?;
            if layer_result.changed_state {
                self.changed_state_count += 1;
            }
            keep_going |= layer_result.keep_going;
        }
        for input in &mut self.inputs {
            input.advanced();
        }
        if self
            .data_bind_graph
            .default_view_model_source_context_bound()
        {
            self.sync_default_view_model_triggers_from_active();
        }
        self.needs_advance = data_bind_advance.keep_going
            || keep_going
            || !self.reported_events.is_empty()
            || !self.reported_listener_view_models.is_empty()
            || !self.pending_listener_events.is_empty();
        Ok(self.needs_advance)
    }

    fn apply_default_view_model_bindings(
        &mut self,
        include_view_models: bool,
        phase: RuntimeDataBindGraphApplyPhase,
    ) {
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
        );
    }

    fn bind_active_default_view_model_triggers(&mut self) {
        self.view_model_triggers = self.default_view_model_triggers.clone();
    }

    fn bind_active_imported_view_model_triggers(
        &mut self,
        file: &RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
        context: Option<&RuntimeImportedViewModelInstanceContext>,
    ) {
        let instance = file
            .view_model(view_model_index)
            .and_then(|view_model| view_model.instances.into_iter().nth(instance_index));
        let Ok(view_model_path_id) = u32::try_from(view_model_index) else {
            self.reset_active_view_model_triggers();
            return;
        };
        let Some(instance) = instance else {
            self.reset_active_view_model_triggers();
            return;
        };

        let mut active = self.default_view_model_triggers.clone();
        for trigger in &mut active {
            let path = [view_model_path_id, trigger.view_model_property_id()];
            let value = context
                .and_then(|context| context.trigger_overrides.get(path.as_slice()).copied())
                .or_else(|| {
                    file.data_context_view_model_property_for_instance(instance.object, &path)
                        .and_then(|source| {
                            file.view_model_instance_trigger_count_for_object(source)
                        })
                });
            if let Some(value) = value {
                trigger.replace_value(value);
            } else {
                trigger.reset();
            }
        }
        self.view_model_triggers = active;
    }

    fn bind_active_owned_view_model_triggers(&mut self, context: &RuntimeOwnedViewModelInstance) {
        self.bind_active_owned_view_model_triggers_for_context(context, &[]);
    }

    fn bind_active_owned_view_model_triggers_for_context_chain(
        &mut self,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) {
        let default_view_model_index = self.default_view_model_index;
        let mut active = self.default_view_model_triggers.clone();
        for trigger in &mut active {
            let value = default_view_model_index.and_then(|view_model_index| {
                usize::try_from(trigger.view_model_property_id())
                    .ok()
                    .and_then(|property_index| {
                        context_chain.iter().find_map(|context_path| {
                            runtime_owned_view_model_trigger_value_for_context_path(
                                context,
                                context_path,
                                view_model_index,
                                property_index,
                            )
                        })
                    })
            });
            if let Some(value) = value {
                trigger.replace_value(value);
            } else {
                trigger.reset();
            }
        }
        self.view_model_triggers = active;
    }

    fn bind_active_owned_view_model_triggers_for_candidates(
        &mut self,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
    ) {
        let default_view_model_index = self.default_view_model_index;
        let mut active = self.default_view_model_triggers.clone();
        for trigger in &mut active {
            let resolved = default_view_model_index.and_then(|view_model_index| {
                let view_model_index = u32::try_from(view_model_index).ok()?;
                let source_path = [view_model_index, trigger.view_model_property_id()];
                candidates.iter().find_map(|candidate| {
                    candidate.resolve_value_and_cell_for_source_path(
                        &RuntimeDataBindGraphValue::Trigger(trigger.value()),
                        &source_path,
                    )
                })
            });
            match resolved {
                Some((RuntimeDataBindGraphValue::Trigger(_), Some(cell))) => {
                    trigger.bind_cell(cell)
                }
                Some((RuntimeDataBindGraphValue::Trigger(value), None)) => {
                    trigger.replace_value(value)
                }
                _ => trigger.reset(),
            }
        }
        self.view_model_triggers = active;
    }

    fn refresh_active_owned_view_model_triggers_for_candidates(
        &mut self,
        candidates: &[RuntimeOwnedViewModelBindingCandidate],
    ) {
        let default_view_model_index = self.default_view_model_index;
        for trigger in &mut self.view_model_triggers {
            let resolved = default_view_model_index.and_then(|view_model_index| {
                let view_model_index = u32::try_from(view_model_index).ok()?;
                let source_path = [view_model_index, trigger.view_model_property_id()];
                candidates.iter().find_map(|candidate| {
                    candidate.resolve_value_and_cell_for_source_path(
                        &RuntimeDataBindGraphValue::Trigger(trigger.value()),
                        &source_path,
                    )
                })
            });
            match resolved {
                Some((RuntimeDataBindGraphValue::Trigger(_), Some(cell))) => {
                    trigger.bind_cell(cell)
                }
                Some((RuntimeDataBindGraphValue::Trigger(value), None)) => {
                    trigger.replace_value(value)
                }
                _ => trigger.reset(),
            }
        }
    }

    fn bind_active_owned_view_model_triggers_for_context(
        &mut self,
        context: &RuntimeOwnedViewModelInstance,
        context_path: &[usize],
    ) {
        let default_view_model_index = self.default_view_model_index;
        let mut active = self.default_view_model_triggers.clone();
        for trigger in &mut active {
            let value = default_view_model_index.and_then(|view_model_index| {
                usize::try_from(trigger.view_model_property_id())
                    .ok()
                    .and_then(|property_index| {
                        runtime_owned_view_model_trigger_value_for_context_path(
                            context,
                            context_path,
                            view_model_index,
                            property_index,
                        )
                    })
            });
            if let Some(value) = value {
                trigger.replace_value(value);
            } else {
                trigger.reset();
            }
        }
        self.view_model_triggers = active;
    }

    fn reset_active_view_model_triggers(&mut self) {
        self.view_model_triggers = self.default_view_model_triggers.clone();
        for trigger in &mut self.view_model_triggers {
            trigger.reset();
        }
    }

    fn sync_default_view_model_triggers_from_active(&mut self) {
        for default_trigger in &mut self.default_view_model_triggers {
            if let Some(active_trigger) = self
                .view_model_triggers
                .iter()
                .find(|trigger| trigger.global_id() == default_trigger.global_id())
            {
                *default_trigger = active_trigger.clone();
            }
        }
    }
}

fn runtime_owned_font_asset_value_for_state_machine_source<'a>(
    context: &'a RuntimeOwnedViewModelInstance,
    source_path: &[u32],
) -> Option<&'a RuntimeFontAssetValue> {
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
mod scripted_listener_action_tests {
    use super::*;
    use nuxie_binary::read_runtime_file;
    use nuxie_graph::GraphFile;
    use std::cell::RefCell;
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
        fail: bool,
        state: usize,
        calls: Rc<RefCell<Vec<RecordedCall>>>,
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
            if self.fail {
                Err(ScriptError::new(format!("{} failed", self.label)))
            } else {
                Ok(())
            }
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

    fn scripted_listener_artboard_and_machine() -> (ArtboardInstance, StateMachineInstance) {
        let fixture = PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets/scripted_listener_action.riv");
        let file = read_runtime_file(&std::fs::read(fixture).expect("read listener fixture"))
            .expect("import listener fixture");
        let graph = GraphFile::from_runtime_file(&file).expect("build listener graph");
        let artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &graph.artboards,
        )
        .expect("instantiate listener artboard");
        let machine = artboard
            .state_machine_instance(0)
            .expect("fixture state machine");
        (artboard, machine)
    }

    fn scripted_listener_machine() -> StateMachineInstance {
        scripted_listener_artboard_and_machine().1
    }

    #[test]
    fn draining_public_reports_leaves_the_core_queue_for_apply_events() {
        let mut machine = scripted_listener_machine();
        let event = StateMachineReportedEvent {
            event_local_index: 7,
            event_core_type: 128,
            name: Some("next-frame".to_owned()),
            url: None,
            target: None,
            string_properties: Vec::new(),
            seconds_delay: 0.0,
            context: None,
        };
        machine.reported_events.push(event.clone());

        let drained = machine.take_reported_events();

        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].event_local_index(), event.event_local_index());
        assert!(machine.take_reported_events().is_empty());
        assert_eq!(machine.reported_event_count(), 1);
        assert_eq!(machine.next_unapplied_reported_event_index(), 0);
    }

    fn script(
        label: &'static str,
        has_perform_action: bool,
        has_perform: bool,
        fail: bool,
        calls: &Rc<RefCell<Vec<RecordedCall>>>,
    ) -> Box<dyn ScriptInstance> {
        Box::new(RecordingListenerScript {
            label,
            has_perform_action,
            has_perform,
            fail,
            state: 0,
            calls: Rc::clone(calls),
        })
    }

    #[test]
    fn scripted_listener_actions_keep_authored_fifo_and_prefer_perform_action() {
        let mut machine = scripted_listener_machine();
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
                script("first", true, true, false, &calls),
            )
            .expect("attach first action");
        machine
            .set_scripted_listener_action_instance(
                second.action_global_id(),
                script("second", false, true, false, &calls),
            )
            .expect("attach second action");
        let actions = vec![
            RuntimeScheduledListenerAction::Scripted {
                flags: 0,
                definition: first,
            },
            RuntimeScheduledListenerAction::Scripted {
                flags: 0,
                definition: second,
            },
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
                .perform_listener_actions(&actions, None, &invocation, &mut NoopScriptHost)
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
        let (artboard, mut machine) = scripted_listener_artboard_and_machine();
        let action = machine
            .scripted_listener_actions()
            .first()
            .expect("fixture scripted listener action")
            .clone();
        let calls = Rc::new(RefCell::new(Vec::new()));
        machine
            .set_scripted_listener_action_instance(
                action.action_global_id(),
                script("pointer", true, false, false, &calls),
            )
            .expect("attach pointer action");

        machine
            .try_pointer_down_with_timestamp_and_script_host(
                &artboard,
                200.0,
                20.0,
                1,
                1.25,
                &mut NoopScriptHost,
            )
            .expect("first pointer down");
        machine
            .try_pointer_up_with_timestamp_and_script_host(
                &artboard,
                200.0,
                20.0,
                1,
                1.5,
                &mut NoopScriptHost,
            )
            .expect("first pointer up");
        machine
            .try_pointer_down_with_timestamp_and_script_host(
                &artboard,
                205.0,
                20.0,
                1,
                2.25,
                &mut NoopScriptHost,
            )
            .expect("second pointer down");
        machine
            .try_pointer_up_with_timestamp_and_script_host(
                &artboard,
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
    fn first_script_failure_stops_later_actions_and_remains_observable() {
        let mut machine = scripted_listener_machine();
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
                script("first", true, false, true, &calls),
            )
            .expect("attach failing action");
        machine
            .set_scripted_listener_action_instance(
                second.action_global_id(),
                script("later", true, false, false, &calls),
            )
            .expect("attach later action");
        let actions = vec![
            RuntimeScheduledListenerAction::Scripted {
                flags: 0,
                definition: first,
            },
            RuntimeScheduledListenerAction::Scripted {
                flags: 0,
                definition: second,
            },
        ];

        let error = machine
            .perform_listener_actions(
                &actions,
                None,
                &ScriptListenerInvocation::None,
                &mut NoopScriptHost,
            )
            .expect_err("first action must fail");
        assert!(error.message().contains("first failed"));
        assert_eq!(calls.borrow().len(), 1, "later action must not run");
        assert_eq!(machine.script_error(), Some(&error));

        let repeated = machine
            .perform_listener_actions(
                &actions,
                None,
                &ScriptListenerInvocation::None,
                &mut NoopScriptHost,
            )
            .expect_err("the first error poisons later dispatch");
        assert_eq!(repeated, error);
        assert_eq!(calls.borrow().len(), 1, "poisoned dispatch stays inert");
    }

    #[test]
    fn each_state_machine_occurrence_retains_fresh_listener_script_state() {
        let mut first_machine = scripted_listener_machine();
        let mut second_machine = scripted_listener_machine();
        let definition = first_machine
            .scripted_listener_actions()
            .first()
            .expect("fixture scripted listener action")
            .clone();
        let calls = Rc::new(RefCell::new(Vec::new()));
        first_machine
            .set_scripted_listener_action_instance(
                definition.action_global_id(),
                script("occurrence", true, false, false, &calls),
            )
            .expect("attach first occurrence");
        second_machine
            .set_scripted_listener_action_instance(
                definition.action_global_id(),
                script("occurrence", true, false, false, &calls),
            )
            .expect("attach second occurrence");
        let actions = [RuntimeScheduledListenerAction::Scripted {
            flags: 0,
            definition,
        }];

        first_machine
            .perform_listener_actions(
                &actions,
                None,
                &ScriptListenerInvocation::None,
                &mut NoopScriptHost,
            )
            .expect("run first occurrence");
        second_machine
            .perform_listener_actions(
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
    fn cloned_occurrence_is_cold_instead_of_sharing_listener_tables() {
        let mut original = scripted_listener_machine();
        let definition = original
            .scripted_listener_actions()
            .first()
            .expect("fixture scripted listener action")
            .clone();
        let original_calls = Rc::new(RefCell::new(Vec::new()));
        original
            .set_scripted_listener_action_instance(
                definition.action_global_id(),
                script("original", true, false, false, &original_calls),
            )
            .expect("attach original occurrence");

        let mut cloned = original.clone();
        let cloned_calls = Rc::new(RefCell::new(Vec::new()));
        cloned
            .set_scripted_listener_action_instance(
                definition.action_global_id(),
                script("clone", true, false, false, &cloned_calls),
            )
            .expect("clone must accept a fresh table");
    }

    #[test]
    fn transactional_candidate_can_adopt_the_same_occurrence_listener_state() {
        let mut original = scripted_listener_machine();
        let definition = original
            .scripted_listener_actions()
            .first()
            .expect("fixture scripted listener action")
            .clone();
        let calls = Rc::new(RefCell::new(Vec::new()));
        original
            .set_scripted_listener_action_instance(
                definition.action_global_id(),
                script("transaction", true, false, false, &calls),
            )
            .expect("attach original occurrence");
        let mut candidate = original.clone();

        candidate
            .adopt_scripted_listener_action_state_from(&original)
            .expect("validated candidate represents the same occurrence");
        candidate
            .perform_listener_actions(
                &[RuntimeScheduledListenerAction::Scripted {
                    flags: 0,
                    definition,
                }],
                None,
                &ScriptListenerInvocation::None,
                &mut NoopScriptHost,
            )
            .expect("committed candidate retains the listener table");

        assert_eq!(calls.borrow().len(), 1);
    }
}
