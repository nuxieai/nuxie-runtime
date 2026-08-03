// Runtime instance orchestration for the C++ state machine path.
// Mirrors /Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp.
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

#[cfg(any(test, feature = "tools"))]
thread_local! {
    static RUNTIME_NESTED_EVENT_CHAIN_TRACE:
        RefCell<Option<Vec<RuntimeNestedEventChainStep>>> = const { RefCell::new(None) };
    static RUNTIME_NESTED_NOTIFY_BATCH_TRACE:
        RefCell<Option<Vec<RuntimeNestedNotifyBatchEntry>>> = const { RefCell::new(None) };
}

/// Tools-only chronology for the production nested-reporter policy.
#[cfg(any(test, feature = "tools"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeNestedEventChainPhase {
    SourceLocal,
    AncestorDispatch,
    AudioUnwind,
}

/// One phase reached by one authored nested-artboard source.
#[cfg(any(test, feature = "tools"))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeNestedEventChainStep {
    pub source_local_id: usize,
    pub phase: RuntimeNestedEventChainPhase,
    pub seconds_delay: Option<f32>,
}

/// Scoped, thread-local recorder used by cross-runtime production probes.
#[cfg(any(test, feature = "tools"))]
pub struct RuntimeNestedEventChainTrace {
    active: bool,
}

#[cfg(any(test, feature = "tools"))]
impl RuntimeNestedEventChainTrace {
    pub fn start() -> Self {
        RUNTIME_NESTED_EVENT_CHAIN_TRACE.with(|trace| {
            assert!(
                trace.borrow().is_none(),
                "nested event-chain tracing is already active on this thread"
            );
            *trace.borrow_mut() = Some(Vec::new());
        });
        Self { active: true }
    }

    pub fn finish(mut self) -> Vec<RuntimeNestedEventChainStep> {
        self.active = false;
        RUNTIME_NESTED_EVENT_CHAIN_TRACE.with(|trace| trace.borrow_mut().take().unwrap_or_default())
    }
}

#[cfg(any(test, feature = "tools"))]
impl Drop for RuntimeNestedEventChainTrace {
    fn drop(&mut self) {
        if self.active {
            RUNTIME_NESTED_EVENT_CHAIN_TRACE.with(|trace| {
                trace.borrow_mut().take();
            });
        }
    }
}

/// Scoped recorder for batches entering the real Rust event-notify seam.
#[cfg(any(test, feature = "tools"))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeNestedNotifyBatchEntry {
    pub size: usize,
    pub source_layer_value: Option<f32>,
}

/// Scoped recorder for batches entering the real Rust event-notify seam.
#[cfg(any(test, feature = "tools"))]
pub struct RuntimeNestedNotifyBatchTrace {
    active: bool,
}

#[cfg(any(test, feature = "tools"))]
impl RuntimeNestedNotifyBatchTrace {
    pub fn start() -> Self {
        RUNTIME_NESTED_NOTIFY_BATCH_TRACE.with(|trace| {
            assert!(
                trace.borrow().is_none(),
                "nested notify-batch tracing is already active on this thread"
            );
            *trace.borrow_mut() = Some(Vec::new());
        });
        Self { active: true }
    }

    pub fn finish(mut self) -> Vec<RuntimeNestedNotifyBatchEntry> {
        self.active = false;
        RUNTIME_NESTED_NOTIFY_BATCH_TRACE
            .with(|trace| trace.borrow_mut().take().unwrap_or_default())
    }
}

#[cfg(any(test, feature = "tools"))]
impl Drop for RuntimeNestedNotifyBatchTrace {
    fn drop(&mut self) {
        if self.active {
            RUNTIME_NESTED_NOTIFY_BATCH_TRACE.with(|trace| {
                trace.borrow_mut().take();
            });
        }
    }
}

#[cfg(any(test, feature = "tools"))]
fn record_runtime_nested_notify_batch(size: usize, source_layer_value: Option<f32>) {
    RUNTIME_NESTED_NOTIFY_BATCH_TRACE.with(|trace| {
        if let Some(trace) = trace.borrow_mut().as_mut() {
            trace.push(RuntimeNestedNotifyBatchEntry {
                size,
                source_layer_value,
            });
        }
    });
}

#[cfg(any(test, feature = "tools"))]
fn record_runtime_nested_event_chain_step(
    source_local_id: usize,
    phase: RuntimeNestedEventChainPhase,
) {
    RUNTIME_NESTED_EVENT_CHAIN_TRACE.with(|trace| {
        if let Some(trace) = trace.borrow_mut().as_mut() {
            trace.push(RuntimeNestedEventChainStep {
                source_local_id,
                phase,
                seconds_delay: None,
            });
        }
    });
}

#[cfg(any(test, feature = "tools"))]
fn record_runtime_nested_event_report_step(source_local_id: usize, seconds_delay: f32) {
    RUNTIME_NESTED_EVENT_CHAIN_TRACE.with(|trace| {
        if let Some(trace) = trace.borrow_mut().as_mut() {
            trace.push(RuntimeNestedEventChainStep {
                source_local_id,
                phase: RuntimeNestedEventChainPhase::SourceLocal,
                seconds_delay: Some(seconds_delay),
            });
        }
    });
}

#[cfg(any(test, feature = "tools"))]
fn runtime_nested_source_layer_value(artboard: &ArtboardInstance) -> Option<f32> {
    artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Node")
        .and_then(|component| artboard.transform_property(component.local_id, TransformProperty::X))
        .or_else(|| {
            artboard
                .nested_artboards
                .values()
                .find_map(|nested| runtime_nested_source_layer_value(&nested.child))
        })
}

#[cfg(not(any(test, feature = "tools")))]
fn runtime_nested_source_layer_value(_artboard: &ArtboardInstance) -> Option<f32> {
    None
}

/// Instance-owned boundary for C++ `semanticManager()->nodeById(id)`.
///
/// The deferred semantic-manager family installs the production resolver.
/// FL-C5 owns only the node-id lookup contract and the action switch after a
/// resolver returns the authored `SemanticData` occurrence identity.
pub(crate) trait SemanticNodeResolver: std::fmt::Debug {
    fn semantic_data_local_id(&self, semantic_node_id: u32) -> Option<usize>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AudioEventOccurrence {
    event_local_index: usize,
    event_core_type: u32,
}

struct RuntimeNestedApplyEventsPhase {
    next_event_index: usize,
    event_iterations: usize,
}

struct RuntimeLocalEventListenerBatch {
    next_event_index: usize,
    changed: bool,
    bubbled_events: Vec<StateMachineReportedEvent>,
    listener_indices: Vec<usize>,
    resume_view_model_listeners: bool,
}

/// Instance-owned production boundary for `AudioEvent::play`.
trait AudioEventSeam: std::fmt::Debug {
    fn selected(
        &self,
        occurrence: AudioEventOccurrence,
        selection_count: &mut usize,
        last_occurrence: &mut Option<AudioEventOccurrence>,
    );
}

#[derive(Debug)]
struct PlaybackAudioEventSeam {
    playback: crate::audio_event::RuntimeAudioEventPlayback,
}

impl AudioEventSeam for PlaybackAudioEventSeam {
    fn selected(
        &self,
        occurrence: AudioEventOccurrence,
        selection_count: &mut usize,
        last_occurrence: &mut Option<AudioEventOccurrence>,
    ) {
        *selection_count = selection_count.saturating_add(1);
        *last_occurrence = Some(occurrence);
        let _ = self.playback.play(occurrence.event_local_index);
    }
}

/// Exact C++ pointer result strength (`HitResult`, `hit_result.hpp`).
///
/// The established Rust `bool` facade projects this via [`Self::is_hit`]; the
/// tri-state itself is public so hosts and the golden side-channel can record
/// what C++ `Scene::pointerDown/Move/Up/Exit` return (`scene.hpp:55-60`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimeHitResult {
    #[default]
    None,
    Hit,
    HitOpaque,
}

impl RuntimeHitResult {
    pub fn is_hit(self) -> bool {
        self != Self::None
    }

    fn strongest(self, other: Self) -> Self {
        self.max(other)
    }
}

// Internal shorthand: the FL-ported listener pipeline reads like the pinned
// C++ when the local name matches C++'s `HitResult`.
use RuntimeHitResult as HitResult;

trait HitComponent: std::fmt::Debug {
    fn clone_box(&self) -> Box<dyn HitComponent>;
    fn component(&self) -> Option<ComponentHandle>;
    fn prepare_event(
        &mut self,
        artboard: &ArtboardInstance,
        groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        pointer_id: i32,
    );
    #[allow(clippy::too_many_arguments)]
    fn process_event(
        &mut self,
        instance: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        can_hit: bool,
        timestamp_seconds: f32,
        pointer_id: i32,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<HitResult, ScriptError>;
    fn process_gamepad_invocation(
        &mut self,
        _instance: &mut StateMachineInstance,
        _artboard: &mut ArtboardInstance,
        _invocation: &ScriptListenerInvocation,
        _already_dispatched: Option<(u64, u32)>,
    ) -> HitResult {
        HitResult::None
    }
    fn hit_test(
        &self,
        instance: &StateMachineInstance,
        artboard: &ArtboardInstance,
        position: (f32, f32),
    ) -> bool;
    fn enable_pointer_events(&mut self, _groups: &mut [ListenerGroup], _pointer_id: i32) {}
    fn disable_pointer_events(&mut self, _groups: &mut [ListenerGroup], _pointer_id: i32) {}
    fn add_listener(
        &mut self,
        _group_index: usize,
        _groups: &[ListenerGroup],
        _listeners: &[RuntimeStateMachineListener],
    ) -> bool {
        false
    }
    fn set_explicit_opaque(&mut self, _opaque: bool) {}
    fn scripted_global_id(&self) -> Option<u32> {
        None
    }
}

impl Clone for Box<dyn HitComponent> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
struct HitDrawable {
    component: Option<ComponentHandle>,
    drawable: Option<ComponentHandle>,
    listeners: Vec<usize>,
    is_hovered: bool,
    can_early_out: bool,
    needs_down_listener: bool,
    needs_up_listener: bool,
    is_opaque: bool,
}

impl HitDrawable {
    fn new(
        artboard: &ArtboardInstance,
        drawable: Option<ComponentHandle>,
        component: Option<ComponentHandle>,
        is_opaque: bool,
    ) -> Self {
        let dynamic_opaque = drawable.is_some_and(|drawable| {
            StateMachineInstance::drawable_is_target_opaque(artboard, drawable)
        });
        Self {
            component,
            drawable,
            listeners: Vec::new(),
            is_hovered: false,
            can_early_out: !dynamic_opaque,
            needs_down_listener: false,
            needs_up_listener: false,
            is_opaque,
        }
    }

    fn event_is_required(&self, hit_type: RuntimeListenerType) -> bool {
        !self.can_early_out
            || (hit_type == RuntimeListenerType::Down && self.needs_down_listener)
            || (hit_type == RuntimeListenerType::Up && self.needs_up_listener)
    }

    fn prepare_with(
        &mut self,
        artboard: &ArtboardInstance,
        groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        pointer_id: i32,
        hit_test: impl FnOnce(&ArtboardInstance, (f32, f32)) -> bool,
    ) {
        if !self.event_is_required(hit_type) {
            return;
        }
        self.is_hovered = hit_type != RuntimeListenerType::Exit && hit_test(artboard, position);
        if self.is_hovered {
            for &group_index in &self.listeners {
                if let Some(group) = groups.get_mut(group_index) {
                    group.hover(pointer_id);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_with(
        &mut self,
        instance: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        can_hit: bool,
        timestamp_seconds: f32,
        pointer_id: i32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<HitResult, ScriptError> {
        if !self.event_is_required(hit_type) {
            return Ok(HitResult::None);
        }
        let mut blocking = false;
        for &group_index in &self.listeners {
            let Some(group) = groups.get_mut(group_index) else {
                continue;
            };
            if group.is_consumed {
                continue;
            }
            blocking |= instance.process_listener_group_event(
                group,
                artboard,
                position,
                hit_type,
                can_hit,
                timestamp_seconds,
                pointer_id,
                owned_context.as_deref_mut(),
                event_context,
                host,
            )?;
        }
        if !(self.is_hovered && can_hit) {
            return Ok(HitResult::None);
        }
        let dynamic_opaque = self.drawable.is_some_and(|drawable| {
            StateMachineInstance::drawable_is_target_opaque(artboard, drawable)
        });
        Ok(if self.is_opaque || dynamic_opaque || blocking {
            HitResult::HitOpaque
        } else {
            HitResult::Hit
        })
    }

    fn add_listener_impl(
        &mut self,
        group_index: usize,
        groups: &[ListenerGroup],
        listeners: &[RuntimeStateMachineListener],
    ) -> bool {
        let Some(group) = groups.get(group_index) else {
            return false;
        };
        if let ListenerGroupKind::Authored { listener_index } = group.kind {
            let Some(listener) = listeners.get(listener_index) else {
                return false;
            };
            let has_continuous = listener.listener_types.iter().any(|kind| {
                matches!(
                    kind,
                    RuntimeListenerType::Enter
                        | RuntimeListenerType::Exit
                        | RuntimeListenerType::Move
                        | RuntimeListenerType::Drag
                )
            });
            if has_continuous {
                self.can_early_out = false;
            } else {
                self.needs_down_listener |= listener.listener_types.iter().any(|kind| {
                    matches!(
                        kind,
                        RuntimeListenerType::Down
                            | RuntimeListenerType::Click
                            | RuntimeListenerType::Drag
                    )
                });
                self.needs_up_listener |= listener.listener_types.iter().any(|kind| {
                    matches!(
                        kind,
                        RuntimeListenerType::Up
                            | RuntimeListenerType::Click
                            | RuntimeListenerType::Drag
                    )
                });
            }
        } else {
            self.can_early_out = false;
        }
        self.listeners.push(group_index);
        true
    }

    fn add_text_input_listener(&mut self, group_index: usize) {
        self.can_early_out = false;
        self.needs_down_listener = true;
        self.needs_up_listener = true;
        self.listeners.push(group_index);
    }

    fn enable_groups(&mut self, groups: &mut [ListenerGroup], pointer_id: i32) {
        for &group_index in &self.listeners {
            if let Some(group) = groups.get_mut(group_index) {
                group.enable(pointer_id);
            }
        }
    }

    fn disable_groups(&mut self, groups: &mut [ListenerGroup], pointer_id: i32) {
        for &group_index in &self.listeners {
            if let Some(group) = groups.get_mut(group_index) {
                group.disable(pointer_id);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct HitScriptedDrawable {
    component: Option<ComponentHandle>,
    global_id: u32,
    implemented_methods: RuntimeScriptImplementedMethods,
}

impl HitScriptedDrawable {
    fn method_for_event(
        &self,
        can_hit: bool,
        hit_type: RuntimeListenerType,
    ) -> Option<ScriptMethod> {
        if !can_hit {
            return self
                .implemented_methods
                .wants_pointer_exit()
                .then_some(ScriptMethod::PointerExit);
        }
        match hit_type {
            RuntimeListenerType::Down => self
                .implemented_methods
                .wants_pointer_down()
                .then_some(ScriptMethod::PointerDown),
            RuntimeListenerType::Up => self
                .implemented_methods
                .wants_pointer_up()
                .then_some(ScriptMethod::PointerUp),
            RuntimeListenerType::DragStart | RuntimeListenerType::DragEnd => None,
            _ => self
                .implemented_methods
                .wants_pointer_move()
                .then_some(ScriptMethod::PointerMove),
        }
    }
}

impl HitComponent for HitScriptedDrawable {
    fn clone_box(&self) -> Box<dyn HitComponent> {
        Box::new(self.clone())
    }

    fn component(&self) -> Option<ComponentHandle> {
        self.component
    }

    fn prepare_event(
        &mut self,
        _artboard: &ArtboardInstance,
        _groups: &mut [ListenerGroup],
        _position: (f32, f32),
        _hit_type: RuntimeListenerType,
        _pointer_id: i32,
    ) {
    }

    fn process_event(
        &mut self,
        _instance: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        _groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        can_hit: bool,
        _timestamp_seconds: f32,
        pointer_id: i32,
        _owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        _event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<HitResult, ScriptError> {
        let Some(method) = self.method_for_event(can_hit, hit_type) else {
            return Ok(HitResult::None);
        };
        let Some(component) = self.component else {
            return Ok(HitResult::None);
        };
        let owner = artboard.component_at(component);
        let world = artboard
            .runtime_graph()
            .map(|graph| artboard.runtime_component_world_transform(owner.local_id, graph))
            .unwrap_or(owner.transform.world_transform);
        if world.determinant() == 0.0 {
            return Ok(HitResult::None);
        }
        let local = world
            .invert_or_identity()
            .transform_point(position.0, position.1);
        let Some(script) = artboard.script_instance_for_global(self.global_id) else {
            return Ok(HitResult::None);
        };
        let outcome = script
            .borrow_mut()
            .call_scripted_drawable_pointer(method, pointer_id, local.0, local.1, host)?;
        if outcome.invoked {
            artboard.wake_script_advance_for_global(self.global_id);
        }
        Ok(match outcome.hit {
            ScriptedDrawablePointerHit::None => HitResult::None,
            ScriptedDrawablePointerHit::Hit => HitResult::Hit,
            ScriptedDrawablePointerHit::HitOpaque => HitResult::HitOpaque,
        })
    }

    fn hit_test(
        &self,
        _instance: &StateMachineInstance,
        _artboard: &ArtboardInstance,
        _position: (f32, f32),
    ) -> bool {
        true
    }

    fn scripted_global_id(&self) -> Option<u32> {
        Some(self.global_id)
    }
}

#[derive(Debug, Clone)]
struct HitExpandable {
    drawable: HitDrawable,
}

#[derive(Debug, Clone)]
struct HitTextRun {
    expandable: HitExpandable,
}

#[derive(Debug, Clone)]
struct HitLayout {
    drawable: HitDrawable,
}

#[derive(Debug, Clone)]
struct HitNestedArtboard {
    component: Option<ComponentHandle>,
}

#[derive(Debug, Clone)]
struct HitComponentList {
    component: Option<ComponentHandle>,
}

impl HitComponent for HitDrawable {
    fn clone_box(&self) -> Box<dyn HitComponent> {
        Box::new(self.clone())
    }

    fn component(&self) -> Option<ComponentHandle> {
        self.component
    }

    fn prepare_event(
        &mut self,
        artboard: &ArtboardInstance,
        groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        pointer_id: i32,
    ) {
        self.prepare_with(artboard, groups, position, hit_type, pointer_id, |_, _| {
            false
        });
    }

    fn process_event(
        &mut self,
        instance: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        can_hit: bool,
        timestamp_seconds: f32,
        pointer_id: i32,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<HitResult, ScriptError> {
        self.process_with(
            instance,
            artboard,
            groups,
            position,
            hit_type,
            can_hit,
            timestamp_seconds,
            pointer_id,
            owned_context,
            event_context,
            host,
        )
    }

    fn hit_test(
        &self,
        _instance: &StateMachineInstance,
        _artboard: &ArtboardInstance,
        _position: (f32, f32),
    ) -> bool {
        false
    }

    fn enable_pointer_events(&mut self, groups: &mut [ListenerGroup], pointer_id: i32) {
        self.enable_groups(groups, pointer_id);
    }

    fn disable_pointer_events(&mut self, groups: &mut [ListenerGroup], pointer_id: i32) {
        self.disable_groups(groups, pointer_id);
    }

    fn add_listener(
        &mut self,
        group_index: usize,
        groups: &[ListenerGroup],
        listeners: &[RuntimeStateMachineListener],
    ) -> bool {
        self.add_listener_impl(group_index, groups, listeners)
    }

    fn set_explicit_opaque(&mut self, opaque: bool) {
        self.is_opaque |= opaque;
    }
}

impl HitComponent for HitExpandable {
    fn clone_box(&self) -> Box<dyn HitComponent> {
        Box::new(self.clone())
    }

    fn component(&self) -> Option<ComponentHandle> {
        self.drawable.component
    }

    fn prepare_event(
        &mut self,
        artboard: &ArtboardInstance,
        groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        pointer_id: i32,
    ) {
        let component = self.drawable.component;
        self.drawable.prepare_with(
            artboard,
            groups,
            position,
            hit_type,
            pointer_id,
            |artboard, position| {
                component.is_some_and(|component| {
                    StateMachineInstance::hit_expandable(artboard, component, position)
                })
            },
        );
    }

    fn process_event(
        &mut self,
        instance: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        can_hit: bool,
        timestamp_seconds: f32,
        pointer_id: i32,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<HitResult, ScriptError> {
        self.drawable.process_with(
            instance,
            artboard,
            groups,
            position,
            hit_type,
            can_hit,
            timestamp_seconds,
            pointer_id,
            owned_context,
            event_context,
            host,
        )
    }

    fn hit_test(
        &self,
        _instance: &StateMachineInstance,
        artboard: &ArtboardInstance,
        position: (f32, f32),
    ) -> bool {
        self.drawable.component.is_some_and(|component| {
            StateMachineInstance::hit_expandable(artboard, component, position)
        })
    }

    fn enable_pointer_events(&mut self, groups: &mut [ListenerGroup], pointer_id: i32) {
        self.drawable.enable_groups(groups, pointer_id);
    }

    fn disable_pointer_events(&mut self, groups: &mut [ListenerGroup], pointer_id: i32) {
        self.drawable.disable_groups(groups, pointer_id);
    }

    fn add_listener(
        &mut self,
        group_index: usize,
        groups: &[ListenerGroup],
        listeners: &[RuntimeStateMachineListener],
    ) -> bool {
        self.drawable
            .add_listener_impl(group_index, groups, listeners)
    }

    fn set_explicit_opaque(&mut self, opaque: bool) {
        self.drawable.is_opaque |= opaque;
    }
}

impl HitComponent for HitTextRun {
    fn clone_box(&self) -> Box<dyn HitComponent> {
        Box::new(self.clone())
    }

    fn component(&self) -> Option<ComponentHandle> {
        self.expandable.component()
    }

    fn prepare_event(
        &mut self,
        artboard: &ArtboardInstance,
        groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        pointer_id: i32,
    ) {
        self.expandable
            .prepare_event(artboard, groups, position, hit_type, pointer_id);
    }

    fn process_event(
        &mut self,
        instance: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        can_hit: bool,
        timestamp_seconds: f32,
        pointer_id: i32,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<HitResult, ScriptError> {
        self.expandable.process_event(
            instance,
            artboard,
            groups,
            position,
            hit_type,
            can_hit,
            timestamp_seconds,
            pointer_id,
            owned_context,
            event_context,
            host,
        )
    }

    fn hit_test(
        &self,
        instance: &StateMachineInstance,
        artboard: &ArtboardInstance,
        position: (f32, f32),
    ) -> bool {
        self.expandable.hit_test(instance, artboard, position)
    }

    fn enable_pointer_events(&mut self, groups: &mut [ListenerGroup], pointer_id: i32) {
        self.expandable.enable_pointer_events(groups, pointer_id);
    }

    fn disable_pointer_events(&mut self, groups: &mut [ListenerGroup], pointer_id: i32) {
        self.expandable.disable_pointer_events(groups, pointer_id);
    }

    fn add_listener(
        &mut self,
        group_index: usize,
        groups: &[ListenerGroup],
        listeners: &[RuntimeStateMachineListener],
    ) -> bool {
        self.expandable.add_listener(group_index, groups, listeners)
    }
}

impl HitComponent for HitLayout {
    fn clone_box(&self) -> Box<dyn HitComponent> {
        Box::new(self.clone())
    }

    fn component(&self) -> Option<ComponentHandle> {
        self.drawable.component
    }

    fn prepare_event(
        &mut self,
        artboard: &ArtboardInstance,
        groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        pointer_id: i32,
    ) {
        let component = self.drawable.component;
        self.drawable.prepare_with(
            artboard,
            groups,
            position,
            hit_type,
            pointer_id,
            |artboard, position| {
                component.is_some_and(|component| {
                    artboard.component_hit_test_point(component, position, false, true)
                })
            },
        );
    }

    fn process_event(
        &mut self,
        instance: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        can_hit: bool,
        timestamp_seconds: f32,
        pointer_id: i32,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<HitResult, ScriptError> {
        self.drawable.process_with(
            instance,
            artboard,
            groups,
            position,
            hit_type,
            can_hit,
            timestamp_seconds,
            pointer_id,
            owned_context,
            event_context,
            host,
        )
    }

    fn hit_test(
        &self,
        _instance: &StateMachineInstance,
        artboard: &ArtboardInstance,
        position: (f32, f32),
    ) -> bool {
        self.drawable.component.is_some_and(|component| {
            artboard.component_hit_test_point(component, position, false, true)
        })
    }

    fn enable_pointer_events(&mut self, groups: &mut [ListenerGroup], pointer_id: i32) {
        self.drawable.enable_groups(groups, pointer_id);
    }

    fn disable_pointer_events(&mut self, groups: &mut [ListenerGroup], pointer_id: i32) {
        self.drawable.disable_groups(groups, pointer_id);
    }

    fn add_listener(
        &mut self,
        group_index: usize,
        groups: &[ListenerGroup],
        listeners: &[RuntimeStateMachineListener],
    ) -> bool {
        self.drawable
            .add_listener_impl(group_index, groups, listeners)
    }

    fn set_explicit_opaque(&mut self, opaque: bool) {
        self.drawable.is_opaque |= opaque;
    }
}

impl HitComponent for HitNestedArtboard {
    fn clone_box(&self) -> Box<dyn HitComponent> {
        Box::new(self.clone())
    }

    fn component(&self) -> Option<ComponentHandle> {
        self.component
    }

    fn prepare_event(
        &mut self,
        _artboard: &ArtboardInstance,
        _groups: &mut [ListenerGroup],
        _position: (f32, f32),
        _hit_type: RuntimeListenerType,
        _pointer_id: i32,
    ) {
    }

    fn process_event(
        &mut self,
        instance: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        _groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        can_hit: bool,
        timestamp_seconds: f32,
        pointer_id: i32,
        _owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        _event_context: Option<&StateMachineEventContext>,
        _host: &mut dyn ScriptHost,
    ) -> Result<HitResult, ScriptError> {
        Ok(instance.process_nested_artboard_event(
            artboard,
            self.component,
            position,
            hit_type,
            can_hit,
            timestamp_seconds,
            pointer_id,
        ))
    }

    fn process_gamepad_invocation(
        &mut self,
        _instance: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        invocation: &ScriptListenerInvocation,
        already_dispatched: Option<(u64, u32)>,
    ) -> HitResult {
        let Some(component) = self.component else {
            return HitResult::None;
        };
        let local_id = artboard.component_at(component).local_id;
        let Some(nested) = artboard.nested_artboards.get_mut(&local_id) else {
            return HitResult::None;
        };
        for animation in &mut nested.animations {
            let crate::artboard::RuntimeNestedAnimationInstance::StateMachine(occurrence) =
                animation
            else {
                continue;
            };
            if let Some(state_machine) = occurrence.state_machine_mut() {
                let _ = state_machine.broadcast_gamepad_to_scripted_drawables(
                    &mut nested.child,
                    invocation,
                    already_dispatched,
                );
            }
        }
        // C++ deliberately ignores every nested child's result here.
        HitResult::None
    }

    fn hit_test(
        &self,
        instance: &StateMachineInstance,
        artboard: &ArtboardInstance,
        position: (f32, f32),
    ) -> bool {
        instance.hit_test_nested_artboard(artboard, self.component, position)
    }
}

impl HitComponent for HitComponentList {
    fn clone_box(&self) -> Box<dyn HitComponent> {
        Box::new(self.clone())
    }

    fn component(&self) -> Option<ComponentHandle> {
        self.component
    }

    fn prepare_event(
        &mut self,
        _artboard: &ArtboardInstance,
        _groups: &mut [ListenerGroup],
        _position: (f32, f32),
        _hit_type: RuntimeListenerType,
        _pointer_id: i32,
    ) {
    }

    fn process_event(
        &mut self,
        instance: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        _groups: &mut [ListenerGroup],
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        can_hit: bool,
        timestamp_seconds: f32,
        pointer_id: i32,
        _owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        _event_context: Option<&StateMachineEventContext>,
        _host: &mut dyn ScriptHost,
    ) -> Result<HitResult, ScriptError> {
        Ok(instance.process_component_list_event(
            artboard,
            self.component,
            position,
            hit_type,
            can_hit,
            timestamp_seconds,
            pointer_id,
        ))
    }

    fn process_gamepad_invocation(
        &mut self,
        _instance: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        invocation: &ScriptListenerInvocation,
        already_dispatched: Option<(u64, u32)>,
    ) -> HitResult {
        let Some(component) = self.component else {
            return HitResult::None;
        };
        let owner = artboard.component_at(component);
        if owner.is_collapsed() {
            return HitResult::None;
        }
        let list_local_id = owner.local_id;
        let order = {
            let Some(list) = artboard.component_list_state(list_local_id) else {
                return HitResult::None;
            };
            if let Some(runtime) = artboard.runtime_file() {
                runtime_component_list_order(runtime, list).indices.clone()
            } else {
                (0..list.items.len()).collect()
            }
        };
        let Some(items) = artboard.component_list_items_mut(list_local_id) else {
            return HitResult::None;
        };
        let mut result = HitResult::None;
        let mut running_can_hit = true;
        for item_index in order.into_iter().rev() {
            let Some(item) = items.get_mut(item_index) else {
                continue;
            };
            if !running_can_hit {
                continue;
            }
            let mut item_result = HitResult::None;
            for state_machine in &mut item.state_machines {
                let outcome = state_machine.broadcast_gamepad_to_scripted_drawables(
                    &mut item.child,
                    invocation,
                    already_dispatched,
                );
                if outcome.handled {
                    item_result = item_result.strongest(HitResult::Hit);
                }
            }
            result = result.strongest(item_result);
            if result == HitResult::HitOpaque {
                running_can_hit = false;
            }
        }
        result
    }

    fn hit_test(
        &self,
        instance: &StateMachineInstance,
        artboard: &ArtboardInstance,
        position: (f32, f32),
    ) -> bool {
        instance.hit_test_component_list(artboard, self.component, position)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeNestedEventNotifierKind {
    StateMachine,
    LinearAnimation,
}

/// One value-owned registration corresponding to one C++ nested animation
/// notifier. Rust polls nested reports rather than storing a raw listener
/// back-pointer in the child, so retaining the exact source/notifier identity
/// here is the ownership-safe adaptation. `dispose` explicitly removes every
/// occurrence before this owner can receive another nested report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeNestedEventRegistration {
    source_local_id: usize,
    notifier_local_id: usize,
    kind: RuntimeNestedEventNotifierKind,
}

/// Cheap, owner-safe host projection of the selected retained focus manager.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FocusState {
    pub has_focus: bool,
    pub expects_keyboard_input: bool,
}

/// Selection projection for the semantic-manager boundary. Internal selection
/// owns `RuntimeSemanticTree`; external selection remains identity-only until
/// the shared external-manager boundary is integrated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeSemanticManagerSelection {
    None,
    InternalRecorded,
    ExternalRecorded(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RuntimeSemanticOccurrenceKey {
    owner_identity: u64,
    data_local_id: usize,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeSemanticRoute {
    owner_identity: u64,
    target_local_id: usize,
    data_local_id: usize,
}

/// One retained semantic domain shared by every mounted artboard occurrence
/// beneath a state-machine instance. The interface stays on
/// `StateMachineInstance`; occurrence keys and recursive Artboard traversal
/// remain implementation details at this seam.
#[derive(Debug, Default)]
struct RuntimeSemanticTree {
    manager: SemanticManager,
    data: BTreeMap<RuntimeSemanticOccurrenceKey, RuntimeSemanticData>,
    routes: BTreeMap<u32, RuntimeSemanticRoute>,
    registered_listener_groups: BTreeSet<(RuntimeSemanticOccurrenceKey, usize)>,
    pending_focus_scroll: Option<RuntimeSemanticRoute>,
}

impl RuntimeSemanticTree {
    fn synchronize(
        &mut self,
        artboard: &mut ArtboardInstance,
        listener_groups: &[semantic_listener_group::RuntimeSemanticListenerGroup],
    ) {
        let root_owner_identity = artboard.instance_identity();
        let mut live = BTreeSet::new();
        self.visit_artboard(artboard, None, Mat2D::IDENTITY, &mut live);

        let stale = self
            .data
            .keys()
            .filter(|key| !live.contains(*key))
            .copied()
            .collect::<Vec<_>>();
        for key in stale {
            if let Some(mut data) = self.data.remove(&key) {
                data.detach(&mut self.manager);
            }
        }
        self.registered_listener_groups
            .retain(|(key, _)| live.contains(key));
        for (group_index, group) in listener_groups.iter().enumerate() {
            let key = RuntimeSemanticOccurrenceKey {
                owner_identity: root_owner_identity,
                data_local_id: group.semantic_data_local_id,
            };
            if self.data.contains_key(&key)
                && self.registered_listener_groups.insert((key, group_index))
            {
                group.register(
                    self.data
                        .get_mut(&key)
                        .expect("registered semantic data remains retained"),
                );
            }
        }
        self.rebuild_routes();
    }

    fn visit_artboard(
        &mut self,
        artboard: &mut ArtboardInstance,
        inherited_parent: Option<SemanticNodeHandle>,
        root_transform: Mat2D,
        live: &mut BTreeSet<RuntimeSemanticOccurrenceKey>,
    ) {
        let owner_identity = artboard.instance_identity();
        let semantic_locals = artboard
            .components()
            .iter()
            .filter(|component| component.type_name == "SemanticData")
            .map(|component| component.local_id)
            .collect::<Vec<_>>();

        for local_id in &semantic_locals {
            let key = RuntimeSemanticOccurrenceKey {
                owner_identity,
                data_local_id: *local_id,
            };
            live.insert(key);
            if !self.data.contains_key(&key) {
                let mut data = RuntimeSemanticData::from_artboard(artboard, *local_id);
                data.prepare_for_tree(artboard);
                self.data.insert(key, data);
            }
        }

        let nodes_by_target = semantic_locals
            .iter()
            .filter_map(|local_id| {
                let key = RuntimeSemanticOccurrenceKey {
                    owner_identity,
                    data_local_id: *local_id,
                };
                let data = self.data.get(&key)?;
                Some((data.parent_local_id?, data.node_handle()?))
            })
            .collect::<BTreeMap<_, _>>();

        for local_id in &semantic_locals {
            let key = RuntimeSemanticOccurrenceKey {
                owner_identity,
                data_local_id: *local_id,
            };
            let target_local = self.data.get(&key).and_then(|data| data.parent_local_id);
            let parent = target_local
                .and_then(|target| artboard.component_parent_local(target))
                .and_then(|parent| closest_semantic_node(artboard, parent, &nodes_by_target))
                .or_else(|| inherited_parent.clone());
            let data = self.data.get_mut(&key).expect("semantic data was retained");
            data.synchronize_from_artboard(artboard, &mut self.manager, root_transform);
            data.reconcile_tree_membership(
                &mut self.manager,
                parent.as_ref(),
                artboard,
                root_transform,
            );
        }

        let nested_hosts = artboard
            .nested_artboards
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for host_local in nested_hosts {
            let parent = closest_semantic_node(artboard, host_local, &nodes_by_target)
                .or_else(|| inherited_parent.clone());
            let host_world = artboard.runtime_component_world_transform_with_scroll(host_local);
            if let Some(nested) = artboard.nested_artboards.get_mut(&host_local) {
                let child_root = nested
                    .child
                    .mounted_root_transform(root_transform.multiply(host_world));
                self.visit_artboard(&mut nested.child, parent, child_root, live);
            }
        }

        let list_locals = artboard.component_list_locals().collect::<Vec<_>>();
        let list_root_transforms =
            artboard.runtime_component_list_child_root_transforms(root_transform);
        for list_local in list_locals {
            let parent = closest_semantic_node(artboard, list_local, &nodes_by_target)
                .or_else(|| inherited_parent.clone());
            let Some(items) = artboard.component_list_items_mut(list_local) else {
                continue;
            };
            for (item_index, item) in items.iter_mut().enumerate() {
                let child_root = list_root_transforms
                    .get(&list_local)
                    .and_then(|roots| roots.get(item_index))
                    .copied()
                    .unwrap_or(root_transform);
                self.visit_artboard(&mut item.child, parent.clone(), child_root, live);
            }
        }
    }

    fn rebuild_routes(&mut self) {
        self.routes.clear();
        for (key, data) in &self.data {
            let Some(target_local_id) = data.parent_local_id else {
                continue;
            };
            let id = data.semantic_id();
            if id != 0 {
                self.routes.insert(
                    id,
                    RuntimeSemanticRoute {
                        owner_identity: key.owner_identity,
                        target_local_id,
                        data_local_id: key.data_local_id,
                    },
                );
            }
        }
    }
}

fn closest_semantic_node(
    artboard: &ArtboardInstance,
    mut local_id: usize,
    nodes_by_target: &BTreeMap<usize, SemanticNodeHandle>,
) -> Option<SemanticNodeHandle> {
    loop {
        if let Some(node) = nodes_by_target.get(&local_id) {
            return Some(node.clone());
        }
        local_id = artboard.component_parent_local(local_id)?;
    }
}

/// C++ aggregate fields have no header initializers. These constructors force
/// both values to be supplied and intentionally do not implement `Default`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeQueuedFocusEvent {
    listener_index: usize,
    is_focus: bool,
}

impl RuntimeQueuedFocusEvent {
    fn from_invocation(invocation: ScriptListenerInvocation) -> Option<Self> {
        let ScriptListenerInvocation::Focus {
            listener_index,
            is_focus,
        } = invocation
        else {
            return None;
        };
        Some(Self {
            listener_index,
            is_focus,
        })
    }

    fn into_invocation(self) -> ScriptListenerInvocation {
        ScriptListenerInvocation::Focus {
            listener_index: self.listener_index,
            is_focus: self.is_focus,
        }
    }
}

/// C++ aggregate fields have no header initializers. These constructors force
/// both values to be supplied and intentionally do not implement `Default`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeQueuedSemanticEvent {
    listener_index: Option<usize>,
    action_type: u32,
}

impl RuntimeQueuedSemanticEvent {
    fn from_invocation(invocation: ScriptListenerInvocation) -> Option<Self> {
        let ScriptListenerInvocation::Semantic {
            listener_index,
            action_type,
        } = invocation
        else {
            return None;
        };
        Some(Self {
            listener_index: Some(listener_index),
            action_type,
        })
    }

    fn into_invocation(self) -> Option<ScriptListenerInvocation> {
        Some(ScriptListenerInvocation::Semantic {
            listener_index: self.listener_index?,
            action_type: self.action_type,
        })
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeDeferredCallbackProbe {
    FocusQueuesSemantic {
        listener_index: Option<usize>,
        action_type: u32,
    },
    SemanticQueuesSemantic {
        listener_index: Option<usize>,
        action_type: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeConstructorPhase {
    Inputs,
    LayersAnyEntry,
    MachineBinds,
    AuthoredListenerCategories,
    ComponentProvidedGroups,
    NestedListTextHits,
    ScriptedClonesAndFacilities,
    HitSort,
    FocusTree,
}

fn listener_property_path_for_resolved_name_path(
    context: &RuntimeOwnedViewModelInstance,
    file: &RuntimeFile,
    scope_path: &[usize],
    resolved_name_ids: &[u32],
) -> Option<Vec<usize>> {
    if resolved_name_ids.is_empty() {
        return None;
    }
    let manifest = file.manifest()?;
    let mut property_path = Vec::with_capacity(scope_path.len() + resolved_name_ids.len());
    property_path.extend_from_slice(scope_path);
    for name_id in resolved_name_ids {
        // C++ `ManifestAsset::resolveName` returns `""` for an unmapped id,
        // and `tryGetRelativeViewModelProperty` performs the ordinary lookup
        // for every resulting segment (`manifest_asset.cpp:146-153`;
        // `data_context.cpp:300-330`).
        let property_name = manifest.resolve_name(*name_id).unwrap_or("");
        let property_index = if property_path.is_empty() {
            context.property_index_by_name(property_name)?
        } else {
            context
                .view_model_by_property_path(&property_path)?
                .property_index_by_name(property_name)?
        };
        property_path.push(property_index);
    }
    Some(property_path)
}

fn resolved_listener_property_path_for_data_context(
    data_context: &RuntimeOwnedDataContext,
    file: &RuntimeFile,
    resolved_name_ids: &[u32],
) -> Option<(RuntimeOwnedViewModelHandle, Vec<usize>)> {
    data_context.resolve_instance(&mut |handle, context, scope_path| {
        let property_path = listener_property_path_for_resolved_name_path(
            context,
            file,
            scope_path,
            resolved_name_ids,
        )?;
        context.cell_by_property_path(&property_path)?;
        Some((handle.clone(), property_path))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeDataContextBindError {
    NullDataContext,
    NullDataContextWithViewModelListeners,
    NullDataBind,
}

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
            RuntimeListenerViewModelChangeValue::Asset(value) => {
                if let Some(blob_value) = value.blob_data_bind_value() {
                    self.data_bind_graph
                        .set_owned_view_model_context_blob_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            &blob_value,
                        )
                } else if let Some(font_value) = value.font_data_bind_value() {
                    self.data_bind_graph
                        .set_owned_view_model_context_font_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            &font_value,
                        )
                } else {
                    self.data_bind_graph
                        .set_owned_view_model_context_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            value.data_bind_asset_index(),
                        )
                }
            }
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
            profile_name: Arc::clone(&self.profile_name),
            state_machine_definitions: self.state_machine_definitions.as_ref().map(Arc::clone),
            listener_definitions: Arc::clone(&self.listener_definitions),
            default_view_model_index: self.default_view_model_index,
            file_view_model_instances,
            default_view_model_trigger_instance,
            active_file_view_model_binding,
            active_owned_view_model_advance_context: self
                .active_owned_view_model_advance_context
                .clone(),
            focus: self.focus.clone(),
            internal_focus: self.internal_focus.clone(),
            external_focus_manager_selected: self.external_focus_manager_selected,
            owns_focus_domain: self.owns_focus_domain,
            #[cfg(test)]
            focus_manager_phase_trace: self.focus_manager_phase_trace.clone(),
            internal_semantic_manager_enabled: self.internal_semantic_manager_enabled,
            // A Rust snapshot owns a fresh semantic domain. The next
            // Artboard-backed semantic operation repopulates it from the
            // cloned occurrence rather than aliasing retained node handles.
            semantic_tree: self
                .internal_semantic_manager_enabled
                .then(RuntimeSemanticTree::default),
            external_semantic_manager_identity: self.external_semantic_manager_identity,
            semantic_node_resolver: self.semantic_node_resolver.clone(),
            #[cfg(test)]
            semantic_manager_phase_trace: self.semantic_manager_phase_trace.clone(),
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
            layer_count: self.layer_count,
            data_bind_graph: self.data_bind_graph.clone_for_state_machine_snapshot(),
            data_bind_container: RuntimeDataBindContainerQueue::default(),
            data_bind_occurrences: Vec::new(),
            key_frame_data_bind_graphs: self.key_frame_data_bind_graphs.clone(),
            next_key_frame_data_bind_occurrence_id: 0,
            layers: self.layers.clone(),
            // Public Clone is Rust's explicit state-snapshot adaptation (C++
            // has no StateMachineInstance copy constructor). Copy report
            // values and cursors without aliasing their Vec storage; a fresh
            // C++-shaped occurrence is created through `new`, not `clone`.
            reported_events: self.reported_events.clone(),
            reported_event_listener_index: self.reported_event_listener_index,
            host_reported_event_index: self.host_reported_event_index,
            reporting_events: self.reporting_events.clone(),
            events_applied_during_loop: self.events_applied_during_loop.clone(),
            host_events_applied_during_loop_index: self
                .host_events_applied_during_loop_index,
            bubbled_event_reports: self.bubbled_event_reports.clone(),
            bubbled_event_report_index: self.bubbled_event_report_index,
            deferred_owner_audio_occurrences: self.deferred_owner_audio_occurrences.clone(),
            event_bubble_owner_attached: self.event_bubble_owner_attached,
            notifying_event_listeners: false,
            reported_listener_view_models,
            reporting_listener_view_models: Vec::new(),
            post_apply_listener_view_models: self.post_apply_listener_view_models.clone(),
            needs_advance: self.needs_advance,
            // Public Rust Clone is an approved non-aliasing snapshot. Rebuild
            // the primary carrier so a clone cannot join the source
            // occurrence's dependent-container identity.
            primary_data_context: self
                .primary_data_context
                .as_ref()
                .map(RuntimeStateMachineDataContext::detached_snapshot),
            owned_data_context: self.owned_data_context.clone(),
            #[cfg(test)]
            owned_data_bind_context_bind_count: self.owned_data_bind_context_bind_count,
            #[cfg(test)]
            bind_phase_trace: self.bind_phase_trace.clone(),
            #[cfg(test)]
            event_dispatch_phase_trace: self.event_dispatch_phase_trace.clone(),
            #[cfg(test)]
            event_total_order_trace: self.event_total_order_trace.clone(),
            #[cfg(test)]
            event_settlement_total_order_trace: self.event_settlement_total_order_trace.clone(),
            #[cfg(test)]
            nested_event_forward_test: self.nested_event_forward_test.clone(),
            audio_event_seam: self.audio_event_seam.clone(),
            audio_event_selection_count: self.audio_event_selection_count,
            audio_event_last_occurrence: self.audio_event_last_occurrence,
            #[cfg(test)]
            advance_phase_trace: self.advance_phase_trace.clone(),
            #[cfg(test)]
            raw_advance_call_count: self.raw_advance_call_count,
            #[cfg(test)]
            transition_probe_count: self.transition_probe_count,
            #[cfg(test)]
            data_context_advance_call_count: self.data_context_advance_call_count,
            #[cfg(test)]
            bind_advance_test_report: self.bind_advance_test_report.clone(),
            owned_view_model_rebind_sink: RuntimeCellDirtSink::new(),
            draggable_proxies: self
                .draggable_proxies
                .iter()
                .map(RuntimeDraggableProxy::clone_cold)
                .collect(),
            hit_components: self.hit_components.clone(),
            listener_groups: self.listener_groups.clone(),
            // Registration identities are snapshot state, but the owning Vec
            // must never alias the source instance.
            nested_event_registrations: self.nested_event_registrations.clone(),
            disposed: self.disposed,
            draw_order_change_counter: self.draw_order_change_counter,
            #[cfg(test)]
            constructor_phases: self.constructor_phases.clone(),
            #[cfg(test)]
            drop_phase_receipt: None,
            scripted_instances_by_global: BTreeMap::new(),
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
            embedder_gamepads: self.embedder_gamepads.clone(),
            scripted_input_group_generation: self.scripted_input_group_generation,
            semantic_listener_groups: self.semantic_listener_groups.clone(),
            // Snapshot pending callback values without aliasing their queues.
            // `StateMachineInstance::new` remains the cold-remount boundary.
            queued_focus_events: self.queued_focus_events.clone(),
            queued_semantic_events: self.queued_semantic_events.clone(),
            #[cfg(test)]
            deferred_callback_probe: self.deferred_callback_probe,
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

impl Drop for StateMachineInstance {
    fn drop(&mut self) {
        // C++ cleans only its internally owned focus/semantic trees, then
        // unbinds/deletes bind occurrences, deletes layers, and finally
        // deletes scripted clones (`state_machine_instance.cpp:2141-2199`).
        // An internal manager performs focus cleanup before its value is
        // released; an external projection releases only its `Rc` reference.
        // The explicit calls below make the observable Rust order deterministic.
        self.record_drop_phase("focus");
        if self.owns_focus_domain {
            self.focus.clear_focus();
        }
        drop(std::mem::take(&mut self.focus));
        self.dispose();
        self.unbind();
        self.teardown_bind_occurrences();
        self.teardown_layers();
        self.teardown_script_occurrences();
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

    fn report_pending_trigger_bindings(
        &self,
        queue: &RuntimeCellNotificationQueue,
        listener_index: usize,
    ) {
        for binding in &self.property_bindings {
            if binding.cell_binding.as_ref().is_some_and(|binding| {
                matches!(binding.cell.value(), RuntimeViewModelCellValue::Trigger(value) if value != 0)
            }) {
                // A trigger can fire before this listener binding exists
                // (for example during script initialization). Preserve that
                // pending fire for the first applyEvents boundary.
                queue.report_data_bind(listener_index);
            }
        }
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
    fn record_bind_phase(&mut self, phase: &'static str) {
        #[cfg(test)]
        self.bind_phase_trace.push(phase);
        #[cfg(not(test))]
        let _ = phase;
    }

    fn record_event_dispatch_phase(&mut self, phase: &'static str) {
        #[cfg(test)]
        {
            self.event_dispatch_phase_trace.push(phase);
            if let Some((local, audio, total_order)) = &self.event_total_order_trace {
                match phase {
                    "local-dispatch" => total_order.borrow_mut().push(local),
                    "recorded-audio-seam" => total_order.borrow_mut().push(audio),
                    _ => {}
                }
            }
        }
        #[cfg(not(test))]
        let _ = phase;
    }

    fn record_advance_phase(&mut self, phase: &'static str) {
        #[cfg(test)]
        self.advance_phase_trace.push(phase);
        #[cfg(not(test))]
        let _ = phase;
    }

    fn record_constructor_phase(&mut self, phase: RuntimeConstructorPhase) {
        #[cfg(test)]
        self.constructor_phases.push(phase);
        #[cfg(not(test))]
        let _ = phase;
    }

    fn record_drop_phase(&self, phase: &'static str) {
        #[cfg(test)]
        if let Some(receipt) = self.drop_phase_receipt.as_ref() {
            receipt.borrow_mut().push(phase);
        }
        #[cfg(not(test))]
        let _ = phase;
    }

    /// Explicitly detach this occurrence from every nested notifier identity.
    ///
    /// Rust polls nested reports rather than installing a raw child→parent
    /// pointer, but the lifetime boundary remains observable: after dispose,
    /// nested reports are rejected and repeated disposal is a no-op.
    pub fn dispose(&mut self) {
        self.detach_nested_event_registrations();
        self.disposed = true;
    }

    fn detach_nested_event_registrations(&mut self) {
        if self.nested_event_registrations.is_empty() {
            return;
        }
        self.record_drop_phase("nested-detach");
        self.nested_event_registrations.clear();
    }

    fn nested_event_source_registered(&self, source_local_id: usize) -> bool {
        self.nested_event_registrations
            .iter()
            .any(|registration| registration.source_local_id == source_local_id)
    }

    fn teardown_bind_occurrences(&mut self) {
        self.record_drop_phase("binds");
        for layer in &mut self.layers {
            layer.remove_key_frame_data_binds();
        }
        self.owned_data_context = None;
        self.data_bind_occurrences.clear();
        self.data_bind_container = RuntimeDataBindContainerQueue::default();
        self.key_frame_data_bind_graphs.clear();
        self.data_bind_graph.sources.clear();
        self.data_bind_graph.targets.clear();
        self.data_bind_graph.default_view_model_bindings.clear();
        self.data_bind_graph.imported_view_model_overrides.clear();
        self.scripted_object_bindings.clear();
    }

    fn teardown_layers(&mut self) {
        self.record_drop_phase("layers");
        self.layers.clear();
    }

    fn teardown_script_occurrences(&mut self) {
        self.record_drop_phase("scripts");
        self.scripted_listener_action_instances.clear();
        self.scripted_instances_by_global.clear();
        self.scripted_facade_root_view_model = None;
        self.script_error = None;
    }

    fn key_frame_data_bind_occurrence_ids(
        &mut self,
        enrollment: crate::animation::RuntimeKeyFrameDataBindEnrollment,
    ) -> Vec<crate::animation::RuntimeKeyFrameDataBindOccurrenceId> {
        let (layers, graphs, next_id) = (
            &mut self.layers,
            &self.key_frame_data_bind_graphs,
            &mut self.next_key_frame_data_bind_occurrence_id,
        );
        for layer in &mut *layers {
            // Snapshot Clone deliberately drops mutable graph occurrences.
            // Rebuild them from the immutable prototype before collecting
            // typed owner-local enrollment identities.
            layer.ensure_key_frame_data_binds(graphs);
            layer.enroll_unassigned_key_frame_data_binds(next_id);
        }
        let mut ids = Vec::new();
        for layer in layers {
            layer.collect_key_frame_data_bind_occurrence_ids(enrollment, &mut ids);
        }
        ids.sort_unstable();
        ids
    }

    fn prepare_key_frame_data_bind_enrollment(
        &mut self,
        enrollment: crate::animation::RuntimeKeyFrameDataBindEnrollment,
    ) -> bool {
        let ids = self.key_frame_data_bind_occurrence_ids(enrollment);
        let (layers, graphs) = (&mut self.layers, &self.key_frame_data_bind_graphs);
        let mut changed = false;
        for id in ids {
            for layer in &mut *layers {
                if let Some(result) = layer.prepare_key_frame_data_bind_occurrence(id, graphs) {
                    changed |= result;
                    break;
                }
            }
        }
        changed
    }

    fn advance_key_frame_data_bind_enrollment(
        &mut self,
        enrollment: crate::animation::RuntimeKeyFrameDataBindEnrollment,
        elapsed_seconds: f32,
    ) -> bool {
        let ids = self.key_frame_data_bind_occurrence_ids(enrollment);
        let (layers, graphs) = (&mut self.layers, &self.key_frame_data_bind_graphs);
        let mut keep_going = false;
        for id in ids {
            for layer in &mut *layers {
                if let Some(result) =
                    layer.advance_key_frame_data_bind_occurrence(id, graphs, elapsed_seconds)
                {
                    keep_going |= result;
                    break;
                }
            }
        }
        keep_going
    }

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
        if self.external_focus_manager_selected && self.focus.shares_manager(parent_focus) {
            return;
        }

        // C++ calls cleanupFocusTree before assigning the new manager. The
        // retained manager clears this occurrence's selected focus and queues
        // its observable callback before selection changes.
        self.clean_selected_focus_before_manager_switch();
        self.record_focus_manager_phase("clean-retained-tree");
        if !self.external_focus_manager_selected {
            // The private manager is no longer selected, so retaining its
            // fallback tree cannot affect the parent domain. On the reverse
            // switch it is refreshed from the live external subtree when that
            // occurrence exists, and remains available for embedders that
            // install an owner without an Artboard mount.
            self.internal_focus = Some(std::mem::take(&mut self.focus));
        } else {
            self.focus.cleanup_owner_occurrence();
        }
        self.focus = parent_focus.external_for_owner(owner_identity);
        self.publish_focusable_keyboard_capabilities();
        self.external_focus_manager_selected = true;
        self.owns_focus_domain = false;
        self.record_focus_manager_phase("assign-external");
        // The parent Artboard build already placed this child occurrence's
        // retained nodes in the shared domain before the nested machine is
        // pointed at it; no descriptor reconstruction is necessary here.
        self.record_focus_manager_phase("select-retained-tree");
    }

    /// Owner-safe external-to-null form of C++ `setExternalFocusManager`.
    ///
    /// The retained internal tree is selected again; no manager pointer
    /// crosses the public Rust API.
    pub fn clear_external_focus_manager(&mut self) -> bool {
        if !self.external_focus_manager_selected {
            return false;
        }
        self.clean_selected_focus_before_manager_switch();
        self.record_focus_manager_phase("clean-retained-tree");
        let mut internal_focus = self
            .internal_focus
            .take()
            .expect("external focus selection retains its internal fallback");
        internal_focus.replace_with_owner_occurrence_from(&self.focus);
        self.focus.cleanup_owner_occurrence();
        self.focus = internal_focus;
        self.external_focus_manager_selected = false;
        self.owns_focus_domain = true;
        self.publish_focusable_keyboard_capabilities();
        self.record_focus_manager_phase("assign-internal");
        self.record_focus_manager_phase("select-retained-tree");
        true
    }

    fn record_focus_manager_phase(&mut self, phase: &'static str) {
        #[cfg(test)]
        self.focus_manager_phase_trace.push(phase);
        #[cfg(not(test))]
        let _ = phase;
    }

    fn clean_selected_focus_before_manager_switch(&mut self) {
        // Queue callbacks that this occurrence can translate owner-safely.
        // Cross-owner nodes stay with the selected retained manager; only the
        // switching occurrence translates its own focus callback.
        let selected_owner = self.focus.owner_identity();
        let focused_by_this_tree = self
            .focus
            .focused_listener_chain()
            .first()
            .is_some_and(|(owner, _, _)| *owner == selected_owner);
        if focused_by_this_tree && self.focus.clear_focus() {
            self.capture_focus_callbacks();
        }
    }

    fn publish_focusable_keyboard_capabilities(&self) {
        self.focus.clear_keyboard_input_capabilities();
        for group in &self.keyboard_listener_groups {
            self.focus
                .set_accepts_keyboard_input(group.focus_data_local_id, true);
        }
    }

    #[cfg(test)]
    pub(crate) fn set_focus_target_for_test(&mut self, target_local: usize) -> bool {
        self.focus.set_focus_target(target_local)
    }

    #[cfg(test)]
    pub(crate) fn sync_focus_for_test(&mut self, artboard: &ArtboardInstance) {
        self.focus.build_focus_tree(artboard);
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
        let layer_count = state_machine.layers.len();
        let listener_definitions = Arc::clone(&state_machine.listeners);
        // Pinned C++ retains the FocusManager identity during layer entry
        // callbacks but does not build the complete artboard focus topology
        // until every layer has initialized
        // (`state_machine_instance.cpp:1747-1752,2123-2127`).
        let focus = RuntimeFocusTree::new_unsynchronized(artboard);
        let mut instance = Self {
            state_machine_index,
            profile_name: state_machine.name.clone().unwrap_or_default(),
            state_machine_definitions,
            listener_definitions,
            default_view_model_index: state_machine.default_view_model_index,
            file_view_model_instances,
            default_view_model_trigger_instance,
            active_file_view_model_binding: None,
            active_owned_view_model_advance_context: None,
            focus,
            internal_focus: None,
            external_focus_manager_selected: false,
            owns_focus_domain: true,
            #[cfg(test)]
            focus_manager_phase_trace: Vec::new(),
            internal_semantic_manager_enabled: false,
            semantic_tree: None,
            external_semantic_manager_identity: None,
            semantic_node_resolver: None,
            #[cfg(test)]
            semantic_manager_phase_trace: Vec::new(),
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
            layer_count,
            data_bind_graph,
            data_bind_container: RuntimeDataBindContainerQueue::default(),
            data_bind_occurrences: Vec::new(),
            key_frame_data_bind_graphs,
            next_key_frame_data_bind_occurrence_id: 0,
            layers: Vec::with_capacity(layer_capacity),
            reported_events: Vec::new(),
            reported_event_listener_index: 0,
            host_reported_event_index: 0,
            reporting_events: Vec::new(),
            events_applied_during_loop: Vec::new(),
            host_events_applied_during_loop_index: 0,
            bubbled_event_reports: Vec::new(),
            bubbled_event_report_index: 0,
            deferred_owner_audio_occurrences: Vec::new(),
            // Capture the mounted-owner boundary once, while the occurrence
            // is constructed. `frame_origin` is later reused as mutable draw
            // state, so consulting it from `advance` could accidentally turn
            // a root machine into an upward bubbling source.
            event_bubble_owner_attached: !artboard.frame_origin(),
            notifying_event_listeners: false,
            reported_listener_view_models: RuntimeCellNotificationQueue::default(),
            reporting_listener_view_models: Vec::new(),
            post_apply_listener_view_models: Vec::new(),
            needs_advance: false,
            primary_data_context: None,
            owned_data_context: None,
            #[cfg(test)]
            owned_data_bind_context_bind_count: 0,
            #[cfg(test)]
            bind_phase_trace: Vec::new(),
            #[cfg(test)]
            event_dispatch_phase_trace: Vec::new(),
            #[cfg(test)]
            event_total_order_trace: None,
            #[cfg(test)]
            event_settlement_total_order_trace: None,
            #[cfg(test)]
            nested_event_forward_test: None,
            audio_event_seam: Rc::new(PlaybackAudioEventSeam {
                playback: artboard.audio_event_playback(),
            }),
            audio_event_selection_count: 0,
            audio_event_last_occurrence: None,
            #[cfg(test)]
            advance_phase_trace: Vec::new(),
            #[cfg(test)]
            raw_advance_call_count: 0,
            #[cfg(test)]
            transition_probe_count: 0,
            #[cfg(test)]
            data_context_advance_call_count: 0,
            #[cfg(test)]
            bind_advance_test_report: None,
            owned_view_model_rebind_sink: RuntimeCellDirtSink::new(),
            draggable_proxies: Vec::new(),
            hit_components: Vec::new(),
            listener_groups: Vec::new(),
            nested_event_registrations: Vec::new(),
            disposed: false,
            draw_order_change_counter: 0,
            #[cfg(test)]
            constructor_phases: Vec::new(),
            #[cfg(test)]
            drop_phase_receipt: None,
            scripted_instances_by_global: BTreeMap::new(),
            scripted_object_definitions: state_machine.scripted_objects.clone(),
            scripted_listener_action_definitions: state_machine.scripted_listener_actions.clone(),
            scripted_object_bindings: Vec::new(),
            scripted_listener_action_instances: BTreeMap::new(),
            scripted_object_initialization_complete: false,
            scripted_constructor_context_was_prebound: false,
            scripted_data_context_bind_complete: false,
            scripted_facade_root_view_model: None,
            scripted_listener_runtime_file: artboard.runtime_file_arc(),
            scripted_listener_artboard_resolver: None,
            script_error: None,
            view_model_listeners: Vec::new(),
            focus_listener_groups: Vec::new(),
            keyboard_listener_groups: Vec::new(),
            gamepad_listener_groups: Vec::new(),
            gamepad_scripted_drawables: Vec::new(),
            embedder_gamepads: BTreeMap::new(),
            scripted_input_group_generation: 0,
            semantic_listener_groups: Vec::new(),
            queued_focus_events: Vec::new(),
            queued_semantic_events: Vec::new(),
            #[cfg(test)]
            deferred_callback_probe: None,
        };
        instance.record_constructor_phase(RuntimeConstructorPhase::Inputs);
        instance.initialize_layers_in_authored_order(artboard, state_machine);
        instance.record_constructor_phase(RuntimeConstructorPhase::LayersAnyEntry);
        instance.initialize_ordinary_data_bind_container();
        instance.record_constructor_phase(RuntimeConstructorPhase::MachineBinds);
        // Entry focus actions ran before C++ constructs listener groups. Do
        // not replay their manager callbacks into groups registered below.
        instance.focus.discard_unregistered_events();
        instance.initialize_authored_listener_categories(artboard);
        instance.record_constructor_phase(RuntimeConstructorPhase::AuthoredListenerCategories);
        instance.initialize_component_provided_groups(artboard);
        instance.record_constructor_phase(RuntimeConstructorPhase::ComponentProvidedGroups);
        instance.initialize_nested_list_text_hit_ownership(artboard);
        instance.record_constructor_phase(RuntimeConstructorPhase::NestedListTextHits);
        instance.initialize_scripted_clones_and_facilities(artboard, state_machine);
        instance.record_constructor_phase(RuntimeConstructorPhase::ScriptedClonesAndFacilities);
        instance.sort_hit_components(artboard);
        instance.record_constructor_phase(RuntimeConstructorPhase::HitSort);
        instance.build_initial_focus_tree(artboard);
        instance.record_constructor_phase(RuntimeConstructorPhase::FocusTree);
        // `Artboard::buildFocusTree` installs the parent's manager while it
        // visits nested artboards. Rust's retained tree is built above,
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

    fn initialize_component_provided_groups(&mut self, artboard: &mut ArtboardInstance) {
        self.draggable_proxies = runtime_draggable_proxies(artboard);
        let targets = self
            .draggable_proxies
            .iter()
            .enumerate()
            .map(|(proxy_index, proxy)| (proxy_index, proxy.hittable, proxy.opaque))
            .collect::<Vec<_>>();
        for (proxy_index, target, opaque) in targets {
            let group_index = self.listener_groups.len();
            self.listener_groups
                .push(ListenerGroup::draggable(proxy_index));
            self.add_to_hit_lookup(artboard, target, true, group_index, opaque);
        }
    }

    fn initialize_nested_list_text_hit_ownership(&mut self, artboard: &mut ArtboardInstance) {
        // Pinned C++ appends every nested-artboard owner (and its notifier
        // registrations) before component-list owners, then appends the
        // optional TextInput owner. Keep those category passes distinct even
        // when local IDs from different categories are interleaved.
        for source_local_id in artboard.nested_artboards.keys().copied() {
            self.hit_components.push(Box::new(HitNestedArtboard {
                component: artboard.component_handle(source_local_id),
            }));
            let Some(nested) = artboard.nested_artboards.get(&source_local_id) else {
                continue;
            };
            for animation in &nested.animations {
                let (notifier_local_id, kind) = match animation {
                    crate::artboard::RuntimeNestedAnimationInstance::StateMachine(occurrence) => (
                        occurrence.local_id(),
                        RuntimeNestedEventNotifierKind::StateMachine,
                    ),
                    crate::artboard::RuntimeNestedAnimationInstance::Simple {
                        local_id, ..
                    }
                    | crate::artboard::RuntimeNestedAnimationInstance::Remap { local_id, .. } => {
                        (*local_id, RuntimeNestedEventNotifierKind::LinearAnimation)
                    }
                };
                self.nested_event_registrations
                    .push(RuntimeNestedEventRegistration {
                        source_local_id,
                        notifier_local_id,
                        kind,
                    });
            }
        }
        for component in artboard
            .components()
            .iter()
            .filter(|component| component.type_name == "ArtboardComponentList")
        {
            self.hit_components.push(Box::new(HitComponentList {
                component: artboard.component_handle(component.local_id),
            }));
        }
        for component in artboard
            .components()
            .iter()
            .filter(|component| component.type_name == "TextInput")
        {
            let Some(handle) = artboard.component_handle(component.local_id) else {
                continue;
            };
            let group_index = self.listener_groups.len();
            self.listener_groups
                .push(ListenerGroup::text_input(component.local_id));
            let mut drawable = HitDrawable::new(artboard, Some(handle), Some(handle), true);
            drawable.add_text_input_listener(group_index);
            self.hit_components
                .push(Box::new(HitExpandable { drawable }));
        }
    }

    fn initialize_scripted_clones_and_facilities(
        &mut self,
        artboard: &ArtboardInstance,
        state_machine: &RuntimeStateMachine,
    ) {
        self.scripted_object_bindings = state_machine
            .scripted_object_bindings
            .iter()
            .map(|binding| binding.instantiate())
            .collect();
        // Scripted-object binds join only after listener/hit/TextInput
        // facilities, matching cloneScriptedObject's C++ constructor phase.
        self.append_scripted_data_binds_to_container();
        self.initialize_scripted_input_groups(artboard);
        self.scripted_input_group_generation = artboard.script_attachment_generation();
    }

    fn sort_hit_components(&mut self, artboard: &ArtboardInstance) {
        let mut current_sorted_index = 0;
        for index in 0..self.hit_components.len() {
            let is_artboard = self.hit_components[index]
                .component()
                .is_some_and(|component| artboard.component_at(component).type_name == "Artboard");
            if is_artboard {
                self.hit_components.swap(current_sorted_index, index);
                current_sorted_index += 1;
            }
        }
        for drawable in artboard.runtime_hit_component_order() {
            let mut index = current_sorted_index;
            while index < self.hit_components.len() {
                if self.hit_components[index].component() == Some(drawable) {
                    self.hit_components.swap(current_sorted_index, index);
                    current_sorted_index += 1;
                }
                index += 1;
            }
            if current_sorted_index == self.hit_components.len() {
                break;
            }
        }
    }

    fn add_to_hit_lookup(
        &mut self,
        artboard: &mut ArtboardInstance,
        target: ComponentHandle,
        is_layout_component: bool,
        group_index: usize,
        is_opaque: bool,
    ) {
        if let Some(existing) = self
            .hit_components
            .iter_mut()
            .find(|hit| hit.component() == Some(target))
            && existing.add_listener(
                group_index,
                &self.listener_groups,
                &self.listener_definitions,
            )
        {
            if is_layout_component && is_opaque {
                existing.set_explicit_opaque(true);
            }
            return;
        }

        let type_name = artboard.component_at(target).type_name;
        let definition = nuxie_schema::definition_by_name(type_name);
        if is_layout_component
            || definition.is_some_and(|definition| definition.is_a("LayoutComponent"))
        {
            let mut hit = HitLayout {
                drawable: HitDrawable::new(artboard, Some(target), Some(target), is_opaque),
            };
            hit.add_listener(
                group_index,
                &self.listener_groups,
                &self.listener_definitions,
            );
            self.hit_components.push(Box::new(hit));
            return;
        }

        if type_name == "Shape" {
            if let Some(shape) = artboard.component_at(target).concrete.shape.as_ref() {
                shape.add_flags(RuntimeShapeState::NEVER_DEFER_UPDATE);
            }
            let local_id = artboard.component_at(target).local_id;
            artboard.add_dirt(local_id, ComponentDirt::PATH, true);
            let mut hit = HitExpandable {
                drawable: HitDrawable::new(artboard, Some(target), Some(target), false),
            };
            hit.add_listener(
                group_index,
                &self.listener_groups,
                &self.listener_definitions,
            );
            self.hit_components.push(Box::new(hit));
            return;
        }

        if type_name == "TextValueRun" {
            let drawable = artboard.component_parent_handle(target).or(Some(target));
            if let Some(drawable) = drawable {
                let local_id = artboard.component_at(drawable).local_id;
                artboard.add_dirt(local_id, ComponentDirt::PATH, true);
            }
            let mut hit = HitTextRun {
                expandable: HitExpandable {
                    drawable: HitDrawable::new(artboard, drawable, Some(target), false),
                },
            };
            hit.add_listener(
                group_index,
                &self.listener_groups,
                &self.listener_definitions,
            );
            self.hit_components.push(Box::new(hit));
            return;
        }

        if definition.is_some_and(|definition| definition.is_a("ContainerComponent")) {
            let children = (0..artboard.component_child_len(target))
                .filter_map(|index| artboard.component_child_at(target, index))
                .collect::<Vec<_>>();
            for child in children {
                let child_is_layout =
                    nuxie_schema::definition_by_name(artboard.component_at(child).type_name)
                        .is_some_and(|definition| definition.is_a("LayoutComponent"));
                self.add_to_hit_lookup(artboard, child, child_is_layout, group_index, is_opaque);
            }
        }
    }

    fn build_initial_focus_tree(&mut self, artboard: &ArtboardInstance) {
        self.focus.synchronize_after_layer_initialization(artboard);
        self.publish_focusable_keyboard_capabilities();
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

    fn initialize_authored_listener_categories(&mut self, artboard: &mut ArtboardInstance) {
        self.view_model_listeners = (0..self.listener_definitions.len())
            .filter_map(|listener_index| {
                let listener = self.listener_definitions.get(listener_index)?;
                if listener.has_listener(RuntimeListenerType::Event) {
                    return None;
                }
                RuntimeViewModelListenerInstance::new(
                    Arc::clone(&self.listener_definitions),
                    listener_index,
                )
            })
            .collect();

        let listener_definitions = Arc::clone(&self.listener_definitions);
        for (listener_index, listener) in listener_definitions.iter().enumerate() {
            // Pinned C++ gives reported-event and ViewModel listeners their
            // own constructor paths and immediately continues the listener
            // loop. Even a malformed mixed listener therefore does not also
            // register focus, keyboard, gamepad, semantic, or pointer groups
            // (`state_machine_instance.cpp:1829-1842`).
            if listener_uses_report_queue(listener) {
                continue;
            }
            if listener.listener_types.iter().any(|listener_type| {
                matches!(
                    listener_type,
                    RuntimeListenerType::Enter
                        | RuntimeListenerType::Exit
                        | RuntimeListenerType::Down
                        | RuntimeListenerType::Up
                        | RuntimeListenerType::Move
                        | RuntimeListenerType::Click
                        | RuntimeListenerType::DragStart
                        | RuntimeListenerType::DragEnd
                        | RuntimeListenerType::Drag
                )
            }) {
                let group_index = self.listener_groups.len();
                self.listener_groups
                    .push(ListenerGroup::authored(listener_index));
                if let Some(target) = artboard.component_handle(listener.target_local_id) {
                    let is_layout_component =
                        nuxie_schema::definition_by_name(artboard.component_at(target).type_name)
                            .is_some_and(|definition| definition.is_a("LayoutComponent"));
                    self.add_to_hit_lookup(
                        artboard,
                        target,
                        is_layout_component,
                        group_index,
                        false,
                    );
                }
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
        for (listener_index, listener) in listener_definitions.iter().enumerate() {
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
            if implemented.listens_to_pointer_events() {
                self.hit_components.push(Box::new(HitScriptedDrawable {
                    component: artboard.component_handle(component.local_id),
                    global_id: component.global_id,
                    implemented_methods: implemented,
                }));
            }
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
        for group in &self.keyboard_listener_groups {
            if group.scripted_global_id.is_some() {
                self.focus
                    .set_accepts_keyboard_input(group.focus_data_local_id, false);
            }
        }
        self.keyboard_listener_groups
            .retain(|group| group.scripted_global_id.is_none());
        self.gamepad_scripted_drawables.clear();
        self.hit_components
            .retain(|hit| hit.scripted_global_id().is_none());
        self.initialize_scripted_input_groups(artboard);
        self.publish_focusable_keyboard_capabilities();
        self.sort_hit_components(artboard);
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
                state_machine.name.as_deref().unwrap_or_default(),
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
    ) -> (Option<ScriptViewModel>, Vec<Option<ScriptViewModel>>) {
        if let Some(data_context) = self.owned_data_context.as_ref() {
            let mut contexts = data_context.main_context_slots(file).into_iter();
            if let Some(main) = contexts.next() {
                let main = main.and_then(|context| {
                    crate::script_view_model_from_owned_context(file, &context)
                });
                let parents = contexts
                    .map(|context| {
                        context.and_then(|context| {
                            crate::script_view_model_from_owned_context(file, &context)
                        })
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

    pub(crate) fn retain_script_result<T: Default>(&mut self, result: Result<T, ScriptError>) -> T {
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

    fn profile_name(&self) -> &str {
        &self.profile_name
    }

    pub(crate) fn retained_state_machine_definitions(
        &self,
    ) -> Option<Arc<Vec<RuntimeStateMachine>>> {
        self.state_machine_definitions
            .as_ref()?
            .get(self.state_machine_index)?;
        self.state_machine_definitions.as_ref().map(Arc::clone)
    }

    pub fn changed_state_count(&self) -> usize {
        self.layers
            .iter()
            .filter(|layer| layer.state_changed_on_advance())
            .count()
    }

    /// Return the current state of the Nth changed layer in compressed
    /// authored-layer order, matching C++ `stateChangedByIndex`.
    pub fn changed_state(&self, index: usize) -> Option<&RuntimeLayerState> {
        let definitions = self.state_machine_definitions.as_deref()?;
        let state_machine = definitions.get(self.state_machine_index)?;
        let mut changed_index = 0;
        for (layer, layer_definition) in self.layers.iter().zip(state_machine.layers.iter()) {
            if !layer.state_changed_on_advance() {
                continue;
            }
            if changed_index == index {
                return layer.current_state(layer_definition);
            }
            changed_index += 1;
        }
        None
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
        for layer in &mut self.layers {
            layer.begin_new_frame();
        }
    }

    pub fn needs_advance(&self) -> bool {
        self.needs_advance
    }

    pub fn has_listeners(&self) -> bool {
        !self.hit_components.is_empty()
    }

    #[cfg(test)]
    fn hit_components_count(&self) -> usize {
        self.hit_components.len()
    }

    #[cfg(test)]
    fn hit_component(&self, index: usize) -> Option<&dyn HitComponent> {
        self.hit_components.get(index).map(Box::as_ref)
    }

    /// C++ exposes this only under TESTING. Bound against the retained machine
    /// definition before reading the occurrence, matching `layerState`.
    #[cfg(test)]
    fn layer_state(&self, index: usize) -> Option<&RuntimeLayerState> {
        let definitions = self.state_machine_definitions.as_deref()?;
        let state_machine = definitions.get(self.state_machine_index)?;
        let layer_definition = state_machine.layers.get(index)?;
        self.layers.get(index)?.current_state(layer_definition)
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

    fn get_named_input(
        &self,
        name: &str,
        kind: StateMachineInputKind,
    ) -> Option<&StateMachineInputInstance> {
        self.inputs
            .iter()
            .find(|input| !input.is_null() && input.kind() == kind && input.name() == Some(name))
    }

    /// Exact type-and-name first-match lookup matching C++ `getBool`.
    ///
    /// A same-name Number or Trigger occurrence does not shadow a later Bool
    /// (`state_machine_instance.cpp:2689-2706`).
    pub fn get_bool(&self, name: &str) -> Option<&StateMachineInputInstance> {
        self.get_named_input(name, StateMachineInputKind::Bool)
    }

    /// Exact type-and-name first-match lookup matching C++ `getNumber`
    /// (`state_machine_instance.cpp:2689-2710`).
    pub fn get_number(&self, name: &str) -> Option<&StateMachineInputInstance> {
        self.get_named_input(name, StateMachineInputKind::Number)
    }

    /// Exact type-and-name first-match lookup matching C++ `getTrigger`
    /// (`state_machine_instance.cpp:2689-2714`).
    pub fn get_trigger(&self, name: &str) -> Option<&StateMachineInputInstance> {
        self.get_named_input(name, StateMachineInputKind::Trigger)
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

    /// Owner-safe adaptation of C++ `setFocus(FocusData*)`.
    ///
    /// Rust uses the target's mounted local identity instead of exposing a
    /// borrowed `FocusData*`. `None`, or a retained FocusData whose node is no
    /// longer present in this retained tree, follows the C++ clear-focus branch.
    pub fn set_focus(&mut self, target_local_id: Option<usize>) -> bool {
        self.change_focus(|focus| match target_local_id {
            Some(target_local_id) if focus.has_focus_target(target_local_id) => {
                focus.set_focus_target(target_local_id)
            }
            Some(_) | None => focus.clear_focus(),
        })
    }

    /// C++ always owns an internal manager even while an external manager is
    /// selected. Returning availability rather than a borrowed manager keeps
    /// the Rust ownership boundary explicit.
    pub fn internal_focus_manager(&self) -> bool {
        true
    }

    pub fn has_external_focus_manager(&self) -> bool {
        self.external_focus_manager_selected
    }

    /// Poll the selected manager without exposing manager/node ownership.
    pub fn focus_state(&self) -> FocusState {
        let has_focus = self.focus.has_primary_focus();
        let expects_keyboard_input = self.focus.primary_accepts_keyboard_input();
        FocusState {
            has_focus,
            expects_keyboard_input,
        }
    }

    fn change_focus(&mut self, change: impl FnOnce(&mut RuntimeFocusTree) -> bool) -> bool {
        let changed = change(&mut self.focus);
        if changed {
            self.capture_focus_callbacks();
        }
        changed
    }

    fn capture_focus_callbacks(&mut self) {
        let mut captured = Vec::new();
        for (target_local_id, focus_data_local_id, kind) in self.focus.take_owner_events() {
            for group in &self.focus_listener_groups {
                if let Some(invocation) =
                    group.invocation_for(target_local_id, focus_data_local_id, kind)
                {
                    let event = RuntimeQueuedFocusEvent::from_invocation(invocation)
                        .expect("focus group creates a focus invocation");
                    captured.push(event);
                }
            }
        }
        for event in captured {
            self.queue_focus_event(event.listener_index, event.is_focus);
        }
    }

    fn queue_focus_event(&mut self, listener_index: usize, is_focus: bool) {
        self.queued_focus_events.push(RuntimeQueuedFocusEvent {
            listener_index,
            is_focus,
        });
        self.needs_advance = true;
    }

    fn queue_semantic_event(&mut self, listener_index: Option<usize>, action_type: u32) {
        self.queued_semantic_events
            .push(RuntimeQueuedSemanticEvent {
                listener_index,
                action_type,
            });
        self.needs_advance = true;
    }

    fn semantic_manager_selection(&self) -> RuntimeSemanticManagerSelection {
        if let Some(identity) = self.external_semantic_manager_identity {
            RuntimeSemanticManagerSelection::ExternalRecorded(identity)
        } else if self.internal_semantic_manager_enabled {
            RuntimeSemanticManagerSelection::InternalRecorded
        } else {
            RuntimeSemanticManagerSelection::None
        }
    }

    /// Owner-safe availability projection of C++ `semanticManager()`.
    ///
    /// The selected manager pointer and its internals remain at the RECORDED
    /// absent-manifest-row B6-0329 boundary.
    pub fn semantic_manager(&self) -> bool {
        self.semantic_manager_selection() != RuntimeSemanticManagerSelection::None
    }

    /// Preserve the C++ idempotent create-then-build orchestration without
    /// inventing semantic manager or tree internals in this owner.
    pub fn enable_semantics(&mut self) -> bool {
        if self.semantic_manager() {
            return false;
        }

        self.internal_semantic_manager_enabled = true;
        self.semantic_tree = Some(RuntimeSemanticTree::default());
        self.record_semantic_manager_phase("create-internal-recorded-seam");
        // Rust cannot retain an Artboard borrow on the instance. The exact
        // build is deferred to the first Artboard-backed semantic operation.
        self.record_semantic_manager_phase("build-tree-recorded-seam");
        true
    }

    /// Drain the selected internal semantic manager after synchronizing every
    /// retained mounted Artboard occurrence. This is the owner-safe Rust
    /// projection of `semanticManager()->drainDiff()`.
    pub fn drain_semantics_diff(
        &mut self,
        artboard: &mut ArtboardInstance,
    ) -> Result<SemanticsDiff, SemanticDrainError> {
        if self.external_semantic_manager_identity.is_some() {
            return Err(SemanticDrainError::NotEnabled);
        }
        let Some(tree) = self.semantic_tree.as_mut() else {
            return Err(SemanticDrainError::NotEnabled);
        };
        tree.synchronize(artboard, &self.semantic_listener_groups);
        tree.manager.drain_diff()
    }

    /// Route one semantic node through the retained RB-2 focus domain.
    pub fn request_semantic_focus(&mut self, semantic_node_id: u32) -> bool {
        let Some(tree) = self.semantic_tree.as_mut() else {
            return false;
        };
        let Some(route) = tree.routes.get(&semantic_node_id).copied() else {
            return false;
        };
        let changed = self
            .focus
            .set_focus_target_for_owner(route.owner_identity, route.target_local_id);
        if !changed {
            return false;
        }
        for (key, data) in &mut tree.data {
            let focused = key.owner_identity == route.owner_identity
                && key.data_local_id == route.data_local_id;
            data.set_focused_state(focused, Some(&mut tree.manager));
        }
        tree.pending_focus_scroll = Some(route);
        true
    }

    pub fn clear_semantic_focus(&mut self) -> bool {
        let changed = self.change_focus(RuntimeFocusTree::clear_focus);
        if !changed {
            return false;
        }
        if let Some(tree) = self.semantic_tree.as_mut() {
            for data in tree.data.values_mut() {
                data.set_focused_state(false, Some(&mut tree.manager));
            }
        }
        true
    }

    /// Owner-safe form of C++ `setExternalSemanticManager`.
    ///
    /// Only external manager identity and clean/assign/rebuild order are
    /// represented here. Concrete external tree ownership and parent-node
    /// attachment remain a shared integration item.
    pub fn set_external_semantic_manager(
        &mut self,
        manager_identity: Option<u64>,
        parent_node_id: Option<u32>,
    ) -> bool {
        if self.external_semantic_manager_identity == manager_identity {
            return false;
        }

        if self.semantic_manager() {
            self.record_semantic_manager_phase("clean-tree-recorded-seam");
        }
        self.external_semantic_manager_identity = manager_identity;
        self.record_semantic_manager_phase("assign-external");
        let _ = parent_node_id;
        self.record_semantic_manager_phase("build-tree-recorded-seam");
        true
    }

    /// Dispatch through the retained internal manager/data boundary. The
    /// compatibility resolver below preserves the recorded external-manager
    /// seam. Missing manager, node, or data and out-of-range actions are
    /// silent no-ops, matching C++'s missing-manager branch.
    pub fn fire_semantic_action(&mut self, semantic_node_id: u32, action_type: u32) -> bool {
        let retained_route = if let Some(tree) = self.semantic_tree.as_mut()
            && let Some(action) = SemanticActionType::from_raw(action_type)
            && let Some(route) = tree.routes.get(&semantic_node_id).copied()
            && let Some(data) = tree.data.get(&RuntimeSemanticOccurrenceKey {
                owner_identity: route.owner_identity,
                data_local_id: route.data_local_id,
            }) {
            data.fire(action);
            Some(route)
        } else {
            None
        };
        if let Some(route) = retained_route {
            self.capture_registered_semantic_callbacks(route.data_local_id);
            return true;
        }
        let Some(resolver) = self.semantic_node_resolver.clone() else {
            return false;
        };
        self.record_semantic_manager_phase("node-by-id-recorded-seam");
        let Some(semantic_data_local_id) = resolver.semantic_data_local_id(semantic_node_id) else {
            return false;
        };
        let Some(target_local_id) = self
            .semantic_listener_groups
            .iter()
            .find(|group| group.semantic_data_local_id == semantic_data_local_id)
            .map(|group| group.target_local_id)
        else {
            return false;
        };
        self.record_semantic_manager_phase("semantic-data-recorded-seam");
        let phase = match action_type {
            0 => "fire-tap-recorded-data-seam",
            1 => "fire-increase-recorded-data-seam",
            2 => "fire-decrease-recorded-data-seam",
            _ => return false,
        };

        // The switch itself is family-owned (`state_machine_instance.cpp:
        // 2509-2544`). Concrete manager/node lookup and SemanticData fire
        // internals stay at their recorded rows, while the existing listener
        // seam makes the selected action observable.
        self.record_semantic_manager_phase(phase);
        self.semantic_action_for_target(target_local_id, action_type);
        true
    }

    fn capture_registered_semantic_callbacks(&mut self, semantic_data_local_id: usize) {
        let mut captured = Vec::new();
        for group in &self.semantic_listener_groups {
            if group.semantic_data_local_id != semantic_data_local_id {
                continue;
            }
            let Some(listener) = self.listener_definitions.get(group.listener_index) else {
                continue;
            };
            captured.extend(
                group
                    .drain_registered_invocations(listener)
                    .into_iter()
                    .map(|invocation| {
                        RuntimeQueuedSemanticEvent::from_invocation(invocation)
                            .expect("semantic group creates a semantic invocation")
                    }),
            );
        }
        for event in captured {
            self.queue_semantic_event(event.listener_index, event.action_type);
        }
    }

    #[cfg(test)]
    fn set_semantic_node_resolver(&mut self, resolver: Option<Rc<dyn SemanticNodeResolver>>) {
        self.semantic_node_resolver = resolver;
    }

    fn record_semantic_manager_phase(&mut self, phase: &'static str) {
        #[cfg(test)]
        self.semantic_manager_phase_trace.push(phase);
        #[cfg(not(test))]
        let _ = phase;
    }

    /// Queue one callback from an already-resolved SemanticData occurrence.
    ///
    /// C++ `SemanticListenerGroup` receives this callback from SemanticData.
    /// The public node-id lookup above stops at the explicitly recorded
    /// manager/data seams; do not expose this local-id callback as that API.
    pub(crate) fn semantic_action_for_target(
        &mut self,
        target_local_id: usize,
        action_type: u32,
    ) -> bool {
        let mut captured = Vec::new();
        for group in &self.semantic_listener_groups {
            if group.target_local_id != target_local_id {
                continue;
            }
            let Some(listener) = self.listener_definitions.get(group.listener_index) else {
                continue;
            };
            if let Some(invocation) = group.invocation(listener, action_type) {
                let event = RuntimeQueuedSemanticEvent::from_invocation(invocation)
                    .expect("semantic group creates a semantic invocation");
                captured.push(event);
            }
        }
        for event in &captured {
            self.queue_semantic_event(event.listener_index, event.action_type);
        }
        !captured.is_empty()
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
        if self.scripted_data_context_rebind_pending() {
            return false;
        }
        self.ensure_scripted_input_groups_current(artboard);
        if self.script_error.is_some() {
            return false;
        }
        if !self.focus.is_inert() {
            self.focus.drop_hidden_focus_target();
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
        if self.scripted_data_context_rebind_pending() {
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
        if self.scripted_data_context_rebind_pending() {
            return false;
        }
        self.ensure_scripted_input_groups_current(artboard);
        if self.script_error.is_some() {
            return false;
        }
        if !self.focus.is_inert() {
            self.focus.drop_hidden_focus_target();
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
        if self.scripted_data_context_rebind_pending() {
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
        if self.scripted_data_context_rebind_pending() {
            return false;
        }
        self.ensure_scripted_input_groups_current(artboard);
        if self.script_error.is_some() {
            return false;
        }
        if !self.focus.is_inert() {
            self.focus.drop_hidden_focus_target();
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
        if self.scripted_data_context_rebind_pending() {
            return RuntimeInputDispatchOutcome::default();
        }
        self.ensure_scripted_input_groups_current(artboard);
        if self.script_error.is_some() {
            return RuntimeInputDispatchOutcome::terminal();
        }
        let mut hit_components = std::mem::take(&mut self.hit_components);
        let mut hit_result = HitResult::None;
        for hit in &mut hit_components {
            let item =
                hit.process_gamepad_invocation(self, artboard, invocation, already_dispatched);
            hit_result = hit_result.strongest(item);
            if hit_result == HitResult::HitOpaque {
                break;
            }
        }
        self.hit_components = hit_components;
        if self.script_error.is_some() {
            return RuntimeInputDispatchOutcome::terminal();
        }
        let mut handled = hit_result.is_hit();
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
        if self.scripted_data_context_rebind_pending() {
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

    fn normalized_hit_position(artboard: &ArtboardInstance, x: f32, y: f32) -> (f32, f32) {
        let position = if artboard.frame_origin() {
            (
                x - artboard.origin_x * artboard.width,
                y - artboard.origin_y * artboard.height,
            )
        } else {
            (x, y)
        };
        if artboard.has_self_transform() {
            artboard
                .self_transform()
                .invert_or_identity()
                .transform_point(position.0, position.1)
        } else {
            position
        }
    }

    fn drawable_is_target_opaque(artboard: &ArtboardInstance, drawable: ComponentHandle) -> bool {
        let Some(flags_key) = property_key_for_name("Drawable", "drawableFlags") else {
            return false;
        };
        artboard
            .component_local_id(drawable)
            .and_then(|local_id| artboard.uint_property(local_id, flags_key))
            .is_some_and(|flags| flags & (1 << 3) != 0)
    }

    fn hit_expandable(
        artboard: &ArtboardInstance,
        component: ComponentHandle,
        position: (f32, f32),
    ) -> bool {
        let owner = artboard.component_at(component);
        if owner.type_name == "TextValueRun" {
            return artboard.component_hit_test_point(component, position, true, true)
                && runtime_text_value_run_hit_test(artboard, owner.local_id, position);
        }
        if owner.type_name == "TextInput" {
            let Some((min_x, min_y, width, height)) =
                artboard.text_input_local_bounds_retained(owner.local_id)
            else {
                return false;
            };
            let Some(graph) = artboard.runtime_graph() else {
                return false;
            };
            let world = artboard.runtime_component_world_transform(owner.local_id, graph);
            if world.determinant() == 0.0 {
                return false;
            }
            let local = world
                .invert_or_identity()
                .transform_point(position.0, position.1);
            return local.0 >= min_x
                && local.0 <= min_x + width
                && local.1 >= min_y
                && local.1 <= min_y + height
                && artboard.component_hit_test_point(component, position, true, true);
        }
        if owner.type_name != "Shape" {
            return artboard.component_hit_test_point(component, position, true, true);
        }
        if !artboard.component_hit_test_point(component, position, true, true) {
            return false;
        }
        let Some(shape) = owner.concrete.shape.as_ref() else {
            return false;
        };
        let Some(graph) = artboard.runtime_graph() else {
            return false;
        };
        for path in &shape.paths {
            let path_owner = artboard.component_at(*path);
            let path_world = artboard.runtime_component_world_transform(path_owner.local_id, graph);
            if path_owner.is_collapsed() || path_world.determinant() == 0.0 {
                continue;
            }
            let Some(path) = graph
                .paths
                .iter()
                .find(|path| path.local_id == path_owner.local_id)
            else {
                continue;
            };
            if runtime_path_geometry_hit_test(artboard, path, path_world, position) {
                return true;
            }
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    fn process_listener_group_event(
        &mut self,
        group: &mut ListenerGroup,
        artboard: &mut ArtboardInstance,
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        can_hit: bool,
        timestamp_seconds: f32,
        pointer_id: i32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if group.disabled(pointer_id) {
            return Ok(false);
        }
        match group.kind {
            ListenerGroupKind::Draggable { proxy_index } => {
                let pointer_state = group.process(
                    pointer_id,
                    position,
                    can_hit,
                    hit_type == RuntimeListenerType::Down,
                    hit_type == RuntimeListenerType::Up,
                );
                let Some(proxy) = self.draggable_proxies.get_mut(proxy_index) else {
                    return Ok(false);
                };
                let hovered = pointer_state.current_hovered;
                let mut blocking = false;
                match hit_type {
                    RuntimeListenerType::Down if hovered => {
                        if !proxy.active_pointers.contains(&pointer_id) {
                            proxy.active_pointers.push(pointer_id);
                        }
                        runtime_draggable_proxy_start(artboard, proxy, position, timestamp_seconds);
                        group.is_consumed = true;
                    }
                    RuntimeListenerType::Move if proxy.active_pointers.contains(&pointer_id) => {
                        if runtime_draggable_proxy_drag(
                            artboard,
                            proxy,
                            position,
                            timestamp_seconds,
                        ) {
                            proxy.has_scrolled = true;
                            group.mark_dragged();
                            blocking = true;
                            group.is_consumed = true;
                        }
                    }
                    RuntimeListenerType::Up => {
                        if let Some(index) = proxy
                            .active_pointers
                            .iter()
                            .position(|active| *active == pointer_id)
                        {
                            proxy.active_pointers.remove(index);
                            runtime_draggable_proxy_end(artboard, proxy);
                            proxy.has_scrolled = false;
                            group.is_consumed = true;
                        }
                    }
                    RuntimeListenerType::Exit => {
                        proxy.active_pointers.retain(|active| *active != pointer_id);
                        proxy.has_scrolled = false;
                    }
                    _ => {}
                }
                return Ok(blocking);
            }
            ListenerGroupKind::Authored { listener_index } => {
                let Some(listener) = self.listener_definitions.get(listener_index).cloned() else {
                    return Ok(false);
                };
                let pointer_state = group.process(
                    pointer_id,
                    position,
                    can_hit,
                    hit_type == RuntimeListenerType::Down,
                    hit_type == RuntimeListenerType::Up,
                );
                let is_hovered = pointer_state.current_hovered;
                let hover_action = match (
                    pointer_state.previous_hovered,
                    pointer_state.current_hovered,
                ) {
                    (false, true) if listener.has_listener(RuntimeListenerType::Enter) => {
                        Some(RuntimeListenerType::Enter)
                    }
                    (true, false) if listener.has_listener(RuntimeListenerType::Exit) => {
                        Some(RuntimeListenerType::Exit)
                    }
                    _ => None,
                };
                let pointer = RuntimePointerInput {
                    x: position.0,
                    y: position.1,
                    previous_x: pointer_state.previous_position.0,
                    previous_y: pointer_state.previous_position.1,
                    timestamp_seconds,
                    id: pointer_id,
                };
                let captured_drag = hit_type == RuntimeListenerType::Move
                    && listener.has_listener(RuntimeListenerType::Drag)
                    && pointer_state.phase_is_down;
                let drag_started = captured_drag && !group.has_dragged();
                if captured_drag {
                    group.mark_dragged();
                }
                let click_matched = pointer_state.clicked
                    && !pointer_state.drag_ended
                    && listener.has_listener(RuntimeListenerType::Click);
                if hit_type == RuntimeListenerType::Down && is_hovered {
                    if listener.has_listener(RuntimeListenerType::Click)
                        || listener.has_listener(RuntimeListenerType::Drag)
                    {
                        group.begin_capture(pointer_id, event_context);
                    }
                }
                let captured_event_context = group
                    .captured_event_context(pointer_id)
                    .cloned()
                    .or_else(|| event_context.cloned());
                if drag_started {
                    self.dispatch_pointer_listener_type_for_target(
                        artboard,
                        listener.target_local_id,
                        pointer,
                        RuntimeListenerType::DragStart,
                        owned_context.as_deref_mut(),
                        host,
                        captured_event_context.as_ref(),
                    )?;
                }
                if pointer_state.drag_ended {
                    self.dispatch_pointer_listener_type_for_target(
                        artboard,
                        listener.target_local_id,
                        pointer,
                        RuntimeListenerType::DragEnd,
                        owned_context.as_deref_mut(),
                        host,
                        captured_event_context.as_ref(),
                    )?;
                }
                let direct_action =
                    (is_hovered && listener.has_listener(hit_type)).then_some(hit_type);
                let action_type = select_listener_action(
                    hover_action,
                    click_matched,
                    direct_action,
                    captured_drag,
                );
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
                    group.is_consumed = true;
                    crate::profiler::record_global_listener_perform_change(
                        &artboard.profile_name,
                        self.profile_name(),
                        listener.name.as_deref().unwrap_or_default(),
                        action_type.value(),
                        hit_type.value(),
                        pointer_id as u32,
                    );
                }
                group.record_position(pointer_id, position);
                Ok(captured_drag || pointer_state.drag_ended)
            }
            ListenerGroupKind::TextInput => {
                let pointer_state = group.process(
                    pointer_id,
                    position,
                    can_hit,
                    hit_type == RuntimeListenerType::Down,
                    hit_type == RuntimeListenerType::Up,
                );
                let Some(mut text_input) = group.text_input.take() else {
                    return Ok(false);
                };
                let result = text_input.process_event(
                    artboard,
                    pointer_state,
                    position,
                    hit_type,
                    timestamp_seconds,
                );
                if result.focus_requested {
                    let focus_data = artboard
                        .component_handle(text_input.text_input_local_id)
                        .and_then(|owner| {
                            (0..artboard.component_child_len(owner)).find_map(|index| {
                                let child = artboard.component_child_at(owner, index)?;
                                let local = artboard.component_local_id(child)?;
                                (artboard.runtime_object_type_name(local) == Some("FocusData"))
                                    .then_some(local)
                            })
                        });
                    if let Some(focus_data) = focus_data {
                        self.focus.set_focus_target_before_topology(
                            artboard,
                            text_input.text_input_local_id,
                            focus_data,
                        );
                    }
                }
                group.text_input = Some(text_input);
                group.record_position(pointer_id, position);
                group.is_consumed |= result.blocks;
                Ok(result.blocks)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn update_listeners(
        &mut self,
        artboard: &mut ArtboardInstance,
        hit_type: RuntimeListenerType,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<HitResult, ScriptError> {
        // Direct callbacks fail closed while the split facade still owes the
        // synchronous C++ `internalDataContext` work for this occurrence.
        if self.scripted_data_context_rebind_pending() {
            return Ok(HitResult::None);
        }
        if !self.focus.is_inert() {
            self.focus.drop_hidden_focus_target();
        }
        let position = Self::normalized_hit_position(artboard, x, y);
        let mut groups = std::mem::take(&mut self.listener_groups);
        for group in &mut groups {
            group.reset(pointer_id);
        }
        let mut hit_components = std::mem::take(&mut self.hit_components);
        for hit in &mut hit_components {
            hit.prepare_event(artboard, &mut groups, position, hit_type, pointer_id);
        }
        if hit_type == RuntimeListenerType::Up
            && groups
                .iter()
                .any(|group| group.phase_is_down(pointer_id) && group.has_dragged())
        {
            // C++ ListenerGroup::processEvent calls StateMachineInstance::dragEnd
            // before it evaluates Clicked. dragEnd recursively resets every
            // listener group and its pointerMove turns the clicked phases back
            // to Out (`listener_group.cpp:171-210`;
            // `state_machine_instance.cpp:1598-1607`). The direct FL-D owner
            // avoids that recursive traversal, so retain its one observable
            // effect explicitly for every group participating in this up.
            for group in &mut groups {
                group.suppress_click_once(pointer_id);
            }
        }
        let mut result = HitResult::None;
        if hit_type == RuntimeListenerType::Move
            && groups.iter().any(|group| {
                matches!(group.kind, ListenerGroupKind::Authored { .. })
                    && group.phase_is_down(pointer_id)
            })
        {
            result = HitResult::Hit;
        }

        let mut callback_error = None;
        for hit in &mut hit_components {
            let item = hit.process_event(
                self,
                artboard,
                &mut groups,
                position,
                hit_type,
                result != HitResult::HitOpaque,
                timestamp_seconds,
                pointer_id,
                owned_context.as_deref_mut(),
                event_context,
                host,
            );
            match item {
                Ok(item) => result = result.strongest(item),
                Err(error) => {
                    callback_error = Some(error);
                    break;
                }
            }
        }
        self.hit_components = hit_components;
        self.listener_groups = groups;
        if let Some(error) = callback_error {
            return Err(error);
        }
        if hit_type == RuntimeListenerType::Exit {
            for group in &mut self.listener_groups {
                group.release_event(pointer_id);
            }
            self.release_draggable_pointer(pointer_id);
        }
        self.sync_text_input_focus(artboard);
        Ok(result)
    }

    fn sync_text_input_focus(&self, artboard: &mut ArtboardInstance) -> bool {
        let artboard_identity = artboard.instance_identity();
        let focused_local_id = self.focus.focused_listener_chain().into_iter().find_map(
            |(owner_identity, target_local_id, _)| {
                (owner_identity == artboard_identity
                    && artboard.runtime_object_type_name(target_local_id) == Some("TextInput"))
                .then_some(target_local_id)
            },
        );
        artboard.sync_text_input_focus(focused_local_id)
    }

    fn enable_pointer_events(&mut self, pointer_id: i32) {
        let mut groups = std::mem::take(&mut self.listener_groups);
        for hit in &mut self.hit_components {
            hit.enable_pointer_events(&mut groups, pointer_id);
        }
        self.listener_groups = groups;
    }

    fn disable_pointer_events(&mut self, pointer_id: i32) {
        let mut groups = std::mem::take(&mut self.listener_groups);
        for hit in &mut self.hit_components {
            hit.disable_pointer_events(&mut groups, pointer_id);
        }
        self.listener_groups = groups;
    }

    fn nested_local_position(
        artboard: &ArtboardInstance,
        component: Option<ComponentHandle>,
        position: (f32, f32),
    ) -> Option<(usize, (f32, f32))> {
        let component = component?;
        let owner = artboard.component_at(component);
        let world = artboard
            .runtime_graph()
            .map(|graph| artboard.runtime_component_world_transform(owner.local_id, graph))
            .unwrap_or(owner.transform.world_transform);
        if owner.is_collapsed() || world.determinant() == 0.0 {
            return None;
        }
        let paused = property_key_for_name("NestedArtboard", "isPaused")
            .and_then(|key| artboard.bool_property(owner.local_id, key))
            .unwrap_or(false);
        if paused {
            return None;
        }
        Some((
            owner.local_id,
            world
                .invert_or_identity()
                .transform_point(position.0, position.1),
        ))
    }

    fn nested_host_ancestors_hit(
        artboard: &ArtboardInstance,
        component: Option<ComponentHandle>,
        position: (f32, f32),
    ) -> bool {
        let Some(host) = component else {
            return false;
        };
        let Some(parent) = artboard.component_parent_handle(host) else {
            return true;
        };
        // A hit inside the mounted Artboard eventually reaches
        // `Artboard::hitTestPoint`, which crosses the ArtboardHost boundary.
        // `NestedArtboard::hitTestHost` deliberately resumes at the host's
        // parent (not the host itself), preserving ancestor layout clipping
        // (`artboard.cpp:1575-1599`; `nested_artboard.cpp:529-535`).
        artboard.component_hit_test_point(parent, position, false, false)
    }

    fn hit_test_nested_artboard(
        &self,
        artboard: &ArtboardInstance,
        component: Option<ComponentHandle>,
        position: (f32, f32),
    ) -> bool {
        if !Self::nested_host_ancestors_hit(artboard, component, position) {
            return false;
        }
        let Some((local_id, position)) = Self::nested_local_position(artboard, component, position)
        else {
            return false;
        };
        let Some(nested) = artboard.nested_artboards.get(&local_id) else {
            return false;
        };
        nested.animations.iter().any(|animation| {
            let crate::artboard::RuntimeNestedAnimationInstance::StateMachine(occurrence) =
                animation
            else {
                return false;
            };
            occurrence.state_machine().is_some_and(|state_machine| {
                state_machine.hit_test(&nested.child, position.0, position.1)
            })
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn process_nested_artboard_event(
        &mut self,
        artboard: &mut ArtboardInstance,
        component: Option<ComponentHandle>,
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        can_hit: bool,
        timestamp_seconds: f32,
        pointer_id: i32,
    ) -> HitResult {
        let can_hit = can_hit && Self::nested_host_ancestors_hit(artboard, component, position);
        let Some((local_id, position)) = Self::nested_local_position(artboard, component, position)
        else {
            return HitResult::None;
        };
        let result = {
            let Some(nested) = artboard.nested_artboards.get_mut(&local_id) else {
                return HitResult::None;
            };
            let mut result = HitResult::None;
            for animation in &mut nested.animations {
                let crate::artboard::RuntimeNestedAnimationInstance::StateMachine(occurrence) =
                    animation
                else {
                    continue;
                };
                let Some(state_machine) = occurrence.state_machine_mut() else {
                    continue;
                };
                let routed = if can_hit {
                    match hit_type {
                        RuntimeListenerType::Down
                        | RuntimeListenerType::Up
                        | RuntimeListenerType::Move
                        | RuntimeListenerType::Exit => state_machine
                            .update_listeners(
                                &mut nested.child,
                                hit_type,
                                position.0,
                                position.1,
                                pointer_id,
                                if hit_type == RuntimeListenerType::Move {
                                    timestamp_seconds
                                } else {
                                    0.0
                                },
                                None,
                                None,
                                &mut NoopScriptHost,
                            )
                            .unwrap_or(HitResult::None),
                        RuntimeListenerType::DragStart => {
                            let _ = state_machine.drag_start(
                                &mut nested.child,
                                position.0,
                                position.1,
                                timestamp_seconds,
                                pointer_id,
                            );
                            HitResult::None
                        }
                        RuntimeListenerType::DragEnd => {
                            let _ = state_machine.drag_end(
                                &mut nested.child,
                                position.0,
                                position.1,
                                timestamp_seconds,
                                pointer_id,
                            );
                            HitResult::None
                        }
                        _ => HitResult::None,
                    }
                } else if matches!(
                    hit_type,
                    RuntimeListenerType::Down
                        | RuntimeListenerType::Up
                        | RuntimeListenerType::Move
                        | RuntimeListenerType::Exit
                ) {
                    let _ = state_machine.update_listeners(
                        &mut nested.child,
                        RuntimeListenerType::Exit,
                        position.0,
                        position.1,
                        pointer_id,
                        0.0,
                        None,
                        None,
                        &mut NoopScriptHost,
                    );
                    HitResult::None
                } else {
                    HitResult::None
                };
                // Pinned nested routing deliberately lets a later child
                // overwrite an earlier hit/opaque result.
                result = routed;
            }
            result
        };
        artboard.publish_nested_view_model_context_mutations(local_id);
        result
    }

    fn component_list_order_and_positions(
        artboard: &ArtboardInstance,
        component: Option<ComponentHandle>,
        position: (f32, f32),
    ) -> Option<(usize, Vec<(usize, (f32, f32))>)> {
        let component = component?;
        let owner = artboard.component_at(component);
        if owner.is_collapsed() {
            return None;
        }
        let list_local_id = owner.local_id;
        let list = artboard.component_list_state(list_local_id)?;
        let order = if let Some(runtime) = artboard.runtime_file() {
            runtime_component_list_order(runtime, list).indices.clone()
        } else {
            (0..list.items.len()).collect()
        };
        let list_world = owner.transform.world_transform;
        let mut positions = Vec::with_capacity(order.len());
        for item_index in order.into_iter().rev() {
            let Some(item) = list.items.get(item_index) else {
                continue;
            };
            let transform = list_world.multiply(item.transform);
            if transform.determinant() == 0.0 {
                continue;
            }
            positions.push((
                item_index,
                transform
                    .invert_or_identity()
                    .transform_point(position.0, position.1),
            ));
        }
        Some((list_local_id, positions))
    }

    fn hit_test_component_list(
        &self,
        artboard: &ArtboardInstance,
        component: Option<ComponentHandle>,
        position: (f32, f32),
    ) -> bool {
        let Some((list_local_id, positions)) =
            Self::component_list_order_and_positions(artboard, component, position)
        else {
            return false;
        };
        let Some(items) = artboard.component_list_items(list_local_id) else {
            return false;
        };
        positions.into_iter().any(|(item_index, position)| {
            items.get(item_index).is_some_and(|item| {
                item.state_machines.iter().any(|state_machine| {
                    state_machine.hit_test(&item.child, position.0, position.1)
                })
            })
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn process_component_list_event(
        &mut self,
        artboard: &mut ArtboardInstance,
        component: Option<ComponentHandle>,
        position: (f32, f32),
        hit_type: RuntimeListenerType,
        can_hit: bool,
        timestamp_seconds: f32,
        pointer_id: i32,
    ) -> HitResult {
        let Some((list_local_id, positions)) =
            Self::component_list_order_and_positions(artboard, component, position)
        else {
            return HitResult::None;
        };
        let Some(items) = artboard.component_list_items_mut(list_local_id) else {
            return HitResult::None;
        };
        let mut result = HitResult::None;
        let mut running_can_hit = can_hit;
        for (item_index, position) in positions {
            let Some(item) = items.get_mut(item_index) else {
                continue;
            };
            let mut item_result = HitResult::None;
            for state_machine in &mut item.state_machines {
                let routed = if running_can_hit {
                    match hit_type {
                        RuntimeListenerType::Down
                        | RuntimeListenerType::Up
                        | RuntimeListenerType::Move
                        | RuntimeListenerType::Exit => state_machine
                            .update_listeners(
                                &mut item.child,
                                hit_type,
                                position.0,
                                position.1,
                                pointer_id,
                                if hit_type == RuntimeListenerType::Move {
                                    timestamp_seconds
                                } else {
                                    0.0
                                },
                                None,
                                None,
                                &mut NoopScriptHost,
                            )
                            .unwrap_or(HitResult::None),
                        RuntimeListenerType::DragStart => {
                            let _ = state_machine.drag_start(
                                &mut item.child,
                                position.0,
                                position.1,
                                0.0,
                                pointer_id,
                            );
                            HitResult::None
                        }
                        RuntimeListenerType::DragEnd => {
                            let _ = state_machine.drag_end(
                                &mut item.child,
                                position.0,
                                position.1,
                                0.0,
                                pointer_id,
                            );
                            HitResult::None
                        }
                        _ => HitResult::None,
                    }
                } else if matches!(
                    hit_type,
                    RuntimeListenerType::Down
                        | RuntimeListenerType::Up
                        | RuntimeListenerType::Move
                        | RuntimeListenerType::Exit
                ) {
                    let _ = state_machine.update_listeners(
                        &mut item.child,
                        RuntimeListenerType::Exit,
                        position.0,
                        position.1,
                        pointer_id,
                        0.0,
                        None,
                        None,
                        &mut NoopScriptHost,
                    );
                    HitResult::None
                } else {
                    HitResult::None
                };
                item_result = item_result.strongest(routed);
            }
            result = result.strongest(item_result);
            if item_result == HitResult::HitOpaque {
                running_can_hit = false;
            }
        }
        result
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
        crate::scene::pointer_down(self, artboard, x, y, pointer_id)
    }

    pub(crate) fn hit_test(&self, artboard: &ArtboardInstance, x: f32, y: f32) -> bool {
        let position = Self::normalized_hit_position(artboard, x, y);
        self.hit_components
            .iter()
            .any(|hit| hit.hit_test(self, artboard, position))
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
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.update_listeners(
            artboard,
            RuntimeListenerType::Down,
            x,
            y,
            pointer_id,
            timestamp_seconds,
            owned_context,
            event_context,
            host,
        )
        .map(HitResult::is_hit)
    }

    pub fn pointer_move(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        seconds: f32,
        pointer_id: i32,
    ) -> bool {
        crate::scene::pointer_move(self, artboard, x, y, seconds, pointer_id)
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
        crate::scene::pointer_up(self, artboard, x, y, pointer_id)
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
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        event_context: Option<&StateMachineEventContext>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.update_listeners(
            artboard,
            RuntimeListenerType::Up,
            x,
            y,
            pointer_id,
            timestamp_seconds,
            owned_context,
            event_context,
            host,
        )
        .map(HitResult::is_hit)
    }

    pub fn pointer_exit(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        crate::scene::pointer_exit(self, artboard, x, y, pointer_id)
    }

    pub(crate) fn drag_start(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        timestamp_seconds: f32,
        pointer_id: i32,
    ) -> bool {
        let _ = timestamp_seconds;
        self.disable_pointer_events(pointer_id);
        let result = self
            .update_listeners(
                artboard,
                RuntimeListenerType::DragStart,
                x,
                y,
                pointer_id,
                0.0,
                None,
                None,
                &mut NoopScriptHost,
            )
            .map(HitResult::is_hit);
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
        self.enable_pointer_events(pointer_id);
        let result = self.update_listeners(
            artboard,
            RuntimeListenerType::DragEnd,
            x,
            y,
            pointer_id,
            0.0,
            None,
            None,
            &mut NoopScriptHost,
        );
        let drag_result = self.retain_script_result(result.map(HitResult::is_hit));
        let _ = self.pointer_move(artboard, x, y, timestamp_seconds, pointer_id);
        return drag_result;
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

    fn dispatch_pointer_listener_type_for_target(
        &mut self,
        artboard: &mut ArtboardInstance,
        target_local_id: usize,
        pointer: RuntimePointerInput,
        listener_type: RuntimeListenerType,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
        event_context: Option<&StateMachineEventContext>,
    ) -> Result<bool, ScriptError> {
        let listener_definitions = Arc::clone(&self.listener_definitions);
        let mut hit = false;
        for listener in listener_definitions.iter() {
            if listener_uses_report_queue(listener) {
                continue;
            }
            if listener.target_local_id != target_local_id {
                continue;
            }
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
                event_context,
            )?;
            self.needs_advance = true;
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

    /// C++ `Scene::pointerDown` returns the tri-state `HitResult`
    /// (`scene.hpp:55`; computed by `updateListeners`,
    /// `state_machine_instance.cpp:1494-1545`). This is that return for hosts
    /// that need more than the established `bool` projection; script errors
    /// are retained exactly like the `bool` facade and report `None`. The
    /// argument chain matches `pointer_down`/
    /// `pointer_down_with_owned_view_model_context` (timestamp 0, no event
    /// context).
    pub fn pointer_down_hit_result(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> RuntimeHitResult {
        let result = self.update_listeners(
            artboard,
            RuntimeListenerType::Down,
            x,
            y,
            pointer_id,
            0.0,
            owned_context,
            None,
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    /// Tri-state twin of `pointer_move`/
    /// `pointer_move_with_owned_view_model_context` (`scene.hpp:56-58`). The
    /// owned-context chain validates the timestamp exactly like the `bool`
    /// facade; the plain chain forwards it unvalidated, also like the `bool`
    /// facade.
    pub fn pointer_move_hit_result(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        seconds: f32,
        pointer_id: i32,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> RuntimeHitResult {
        let validated = if owned_context.is_some() {
            validate_pointer_timestamp(seconds)
        } else {
            Ok(())
        };
        let result = validated.and_then(|()| {
            self.update_listeners(
                artboard,
                RuntimeListenerType::Move,
                x,
                y,
                pointer_id,
                seconds,
                owned_context,
                None,
                &mut NoopScriptHost,
            )
        });
        self.retain_script_result(result)
    }

    /// Tri-state twin of `pointer_up`/
    /// `pointer_up_with_owned_view_model_context` (`scene.hpp:59`).
    pub fn pointer_up_hit_result(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> RuntimeHitResult {
        let result = self.update_listeners(
            artboard,
            RuntimeListenerType::Up,
            x,
            y,
            pointer_id,
            0.0,
            owned_context,
            None,
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    /// Tri-state twin of `pointer_exit`/
    /// `pointer_exit_with_owned_view_model_context` (`scene.hpp:60`).
    pub fn pointer_exit_hit_result(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> RuntimeHitResult {
        let result = self.update_listeners(
            artboard,
            RuntimeListenerType::Exit,
            x,
            y,
            pointer_id,
            0.0,
            owned_context,
            None,
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
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
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.update_listeners(
            artboard,
            listener_type,
            x,
            y,
            pointer_id,
            timestamp_seconds,
            owned_context,
            None,
            host,
        )
        .map(HitResult::is_hit)
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
        if source_local_id
            .is_some_and(|source_local_id| !self.nested_event_source_registered(source_local_id))
            || self.script_error.is_some()
            || events.is_empty()
        {
            return Ok(false);
        }
        #[cfg(any(test, feature = "tools"))]
        record_runtime_nested_notify_batch(
            events.len(),
            runtime_nested_source_layer_value(artboard),
        );
        self.notifying_event_listeners = true;
        self.record_event_dispatch_phase("local-dispatch");
        let listener_definitions = Arc::clone(&self.listener_definitions);
        let mut changed = false;
        let mut listener_error = None;
        'listeners: for listener in listener_definitions.iter() {
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
                let action_changed = self.perform_listener_actions_with_event_context(
                    artboard,
                    &listener.listener_actions,
                    owned_context.as_deref_mut(),
                    &ScriptListenerInvocation::ReportedEvent {
                        event_local_index: event.event_local_index(),
                        seconds_delay: event.seconds_delay(),
                    },
                    host,
                    event.context.as_ref(),
                );
                match action_changed {
                    Ok(action_changed) => changed |= action_changed,
                    Err(error) => {
                        listener_error = Some(error);
                        break 'listeners;
                    }
                }
                if listener.is_single {
                    break;
                }
            }
        }
        #[cfg(test)]
        if listener_error.is_none()
            && let Some(event) = self.nested_event_forward_test.clone()
        {
            self.reported_events.push(event);
        }
        self.notifying_event_listeners = false;
        if self.bubble_events_to_owner_seam(events) {
            self.defer_recorded_audio_event_seam(events);
        } else {
            self.reach_recorded_audio_event_seam(events);
        }
        listener_error.map_or(Ok(changed), Err)
    }

    /// C++ immediately forwards a nested report to the owning artboard after
    /// local listeners. Rust's owner-safe adaptation exposes that batch from
    /// this dispatch seam without retaining a raw parent pointer.
    fn bubble_events_to_owner_seam(&mut self, events: &[StateMachineReportedEvent]) -> bool {
        if self.event_bubble_owner_attached && !events.is_empty() {
            if self.bubbled_event_report_index != 0 {
                self.bubbled_event_reports
                    .drain(..self.bubbled_event_report_index);
                self.bubbled_event_report_index = 0;
            }
            self.bubbled_event_reports.extend_from_slice(events);
            self.record_event_dispatch_phase("bubble-to-owner");
            true
        } else {
            false
        }
    }

    pub(crate) fn attach_event_bubble_owner(&mut self) {
        self.event_bubble_owner_attached = true;
    }

    /// Drain the owner-safe bubbling FIFO without touching host reports or
    /// the local `applyEvents` queue.
    #[cfg(test)]
    pub(crate) fn take_bubbled_event_reports(&mut self) -> Vec<StateMachineReportedEvent> {
        self.drain_bubbled_event_reports()
    }

    fn drain_bubbled_event_reports(&mut self) -> Vec<StateMachineReportedEvent> {
        self.bubbled_event_report_index = 0;
        std::mem::take(&mut self.bubbled_event_reports)
    }

    /// Select C++ `AudioEvent` occurrences after local dispatch and bubbling,
    /// then invoke the production handoff once per occurrence.
    ///
    /// The production seam retains selection receipts for order probes and
    /// invokes the resolved `AudioEvent::play` owner
    /// (`state_machine_instance.cpp:3155-3169`).
    fn reach_recorded_audio_event_seam(&mut self, events: &[StateMachineReportedEvent]) {
        for event in events.iter().filter(|event| event.is_audio_event()) {
            let occurrence = AudioEventOccurrence {
                event_local_index: event.event_local_index(),
                event_core_type: event.event_core_type(),
            };
            self.deliver_recorded_audio_occurrence(occurrence);
        }
    }

    fn defer_recorded_audio_event_seam(&mut self, events: &[StateMachineReportedEvent]) {
        self.deferred_owner_audio_occurrences.extend(
            events
                .iter()
                .filter(|event| event.is_audio_event())
                .map(|event| AudioEventOccurrence {
                    event_local_index: event.event_local_index(),
                    event_core_type: event.event_core_type(),
                }),
        );
    }

    fn deliver_recorded_audio_occurrence(&mut self, occurrence: AudioEventOccurrence) {
        self.record_event_dispatch_phase("recorded-audio-seam");
        self.audio_event_seam.selected(
            occurrence,
            &mut self.audio_event_selection_count,
            &mut self.audio_event_last_occurrence,
        );
    }

    /// Complete the audio tail after the owner-mediated ancestor dispatch.
    /// The owning Artboard invokes this in ancestor-to-descendant order, which
    /// is the unwind order of C++'s synchronous recursive call.
    pub(crate) fn flush_deferred_owner_audio_events(&mut self) {
        let deferred = std::mem::take(&mut self.deferred_owner_audio_occurrences);
        for occurrence in deferred {
            self.deliver_recorded_audio_occurrence(occurrence);
        }
    }

    fn flush_deferred_owner_audio_event(&mut self, event: &StateMachineReportedEvent) {
        if !event.is_audio_event() {
            return;
        }
        let occurrence = AudioEventOccurrence {
            event_local_index: event.event_local_index(),
            event_core_type: event.event_core_type(),
        };
        let Some(index) = self
            .deferred_owner_audio_occurrences
            .iter()
            .position(|deferred| *deferred == occurrence)
        else {
            return;
        };
        self.deferred_owner_audio_occurrences.remove(index);
        self.deliver_recorded_audio_occurrence(occurrence);
    }

    #[cfg(test)]
    pub(crate) fn audio_event_seam_receipt(&self) -> (usize, Option<(usize, u32)>) {
        (
            self.audio_event_selection_count,
            self.audio_event_last_occurrence
                .map(|occurrence| (occurrence.event_local_index, occurrence.event_core_type)),
        )
    }

    #[cfg(test)]
    pub(crate) fn configure_nested_event_source_test(
        &mut self,
        local_phase: &'static str,
        audio_phase: &'static str,
        total_order: Rc<RefCell<Vec<&'static str>>>,
        event: StateMachineReportedEvent,
    ) {
        self.event_total_order_trace = Some((local_phase, audio_phase, total_order));
        self.attach_event_bubble_owner();
        self.reported_events.push(event);
    }

    #[cfg(test)]
    pub(crate) fn configure_nested_event_root_test(
        &mut self,
        local_phase: &'static str,
        audio_phase: &'static str,
        total_order: Rc<RefCell<Vec<&'static str>>>,
        source_local_ids: impl IntoIterator<Item = usize>,
    ) {
        self.event_total_order_trace = Some((local_phase, audio_phase, total_order));
        self.nested_event_registrations
            .extend(source_local_ids.into_iter().map(|source_local_id| {
                RuntimeNestedEventRegistration {
                    source_local_id,
                    notifier_local_id: source_local_id,
                    kind: RuntimeNestedEventNotifierKind::StateMachine,
                }
            }));
    }

    #[cfg(test)]
    pub(crate) fn configure_nested_event_settlement_test(
        &mut self,
        phase: &'static str,
        total_order: Rc<RefCell<Vec<&'static str>>>,
    ) {
        self.event_settlement_total_order_trace = Some((phase, total_order));
    }

    #[cfg(test)]
    pub(crate) fn configure_nested_event_forwarder_test(
        &mut self,
        local_phase: &'static str,
        audio_phase: &'static str,
        total_order: Rc<RefCell<Vec<&'static str>>>,
        source_local_id: usize,
        event: StateMachineReportedEvent,
    ) {
        self.configure_nested_event_root_test(
            local_phase,
            audio_phase,
            total_order,
            [source_local_id],
        );
        self.attach_event_bubble_owner();
        self.nested_event_forward_test = Some(event);
    }

    fn finish_listener_view_model_firing_boundary(&mut self) {
        for listener_index in std::mem::take(&mut self.post_apply_listener_view_models) {
            self.reported_listener_view_models
                .report_data_bind(listener_index);
        }
    }

    pub(crate) fn apply_local_event_listeners(
        &mut self,
        artboard: &mut ArtboardInstance,
        mut next_event_index: usize,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        const MAX_EVENT_ITERATIONS: usize = 100;

        self.events_applied_during_loop.clear();
        self.host_events_applied_during_loop_index = 0;

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
        self.record_advance_phase("apply-events");
        if next_event_index >= self.reported_events.len()
            && self.reported_listener_view_models.is_empty()
        {
            return changed;
        }

        let mut event_iterations = 0;
        for iteration in 0..MAX_EVENT_ITERATIONS {
            if next_event_index >= self.reported_events.len()
                && self.reported_listener_view_models.is_empty()
            {
                break;
            }
            event_iterations += 1;
            let (next, batch_changed) = self.apply_local_event_listener_batch(
                artboard,
                next_event_index,
                owned_context.as_deref_mut(),
                iteration > 0,
            );
            next_event_index = next;
            changed |= batch_changed;
            if self.script_error.is_some() {
                break;
            }
        }
        if event_iterations >= MAX_EVENT_ITERATIONS {
            eprintln!("StateMachine exceeded max event iterations");
        }
        self.reported_event_listener_index = next_event_index.min(self.reported_events.len());
        changed
    }

    /// Execute one C++ `applyEvents()` queue snapshot. Nested state-machine
    /// owners use the returned bubble batch to complete the synchronous
    /// ancestor chain before asking this instance for the next snapshot.
    fn apply_local_event_listener_batch(
        &mut self,
        artboard: &mut ArtboardInstance,
        next_event_index: usize,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        preserve_host_events: bool,
    ) -> (usize, bool) {
        let batch = self.begin_local_event_listener_batch(
            artboard,
            next_event_index,
            owned_context.as_deref_mut(),
            preserve_host_events,
            false,
        );
        self.finish_local_event_listener_batch(artboard, batch, owned_context.as_deref_mut())
    }

    fn begin_local_event_listener_batch(
        &mut self,
        artboard: &mut ArtboardInstance,
        mut next_event_index: usize,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        preserve_host_events: bool,
        capture_bubbled_events: bool,
    ) -> RuntimeLocalEventListenerBatch {
        // Mirrors C++ `StateMachineInstance::applyEvents()` updating data
        // binds before each queued notification batch
        // (`state_machine_instance.cpp:2320-2335`).
        let mut binding_host = NoopScriptHost;
        if let Err(error) =
            self.update_data_binds_false(artboard, owned_context.as_deref(), &mut binding_host)
        {
            self.script_error = Some(error);
            return RuntimeLocalEventListenerBatch {
                next_event_index,
                changed: false,
                bubbled_events: Vec::new(),
                listener_indices: Vec::new(),
                resume_view_model_listeners: false,
            };
        }
        let mut events = std::mem::take(&mut self.reporting_events);
        events.clear();
        events.extend_from_slice(&self.reported_events[next_event_index..]);
        for event in &mut events {
            event.refresh_from_live_artboard(artboard);
        }
        if preserve_host_events {
            self.events_applied_during_loop.extend_from_slice(&events);
        }
        next_event_index = self.reported_events.len();
        // The reporting snapshot is no longer pending before either callback
        // family runs. Count/At inspection from a callback sees only reports
        // appended for a later batch.
        self.reported_event_listener_index = next_event_index;
        // C++ swaps BOTH queues before notifying either one. Event actions
        // that mutate a listener cell therefore enqueue the next batch rather
        // than joining this reporting batch.
        let mut newly_reported = std::mem::take(&mut self.reporting_listener_view_models);
        self.reported_listener_view_models
            .swap_into(&mut newly_reported);
        let mut listener_indices = newly_reported;
        let changed =
            self.notify_events_with_context(artboard, None, &events, owned_context.as_deref_mut());
        let bubbled_events = if capture_bubbled_events {
            self.drain_bubbled_event_reports()
        } else {
            Vec::new()
        };
        self.reporting_events = events;
        let resume_view_model_listeners = self.script_error.is_none();
        if !resume_view_model_listeners {
            self.reporting_listener_view_models = std::mem::take(&mut listener_indices);
        }
        RuntimeLocalEventListenerBatch {
            next_event_index,
            changed,
            bubbled_events,
            listener_indices,
            resume_view_model_listeners,
        }
    }

    fn finish_local_event_listener_batch(
        &mut self,
        artboard: &mut ArtboardInstance,
        mut batch: RuntimeLocalEventListenerBatch,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> (usize, bool) {
        if !batch.resume_view_model_listeners {
            return (batch.next_event_index, batch.changed);
        }
        if batch.changed
            && let Some(context) = owned_context.as_deref_mut()
        {
            batch.changed |= self.bind_owned_view_model_context_mut(context);
        }

        // C++ reports the listener pointer once per genuine mutation and
        // preserves duplicates/FIFO order. Temporarily take both retained
        // tables so actions can enqueue chained reports without cloning.
        let data_context = self.owned_data_context.take();
        let listeners = std::mem::take(&mut self.view_model_listeners);
        for &listener_index in &batch.listener_indices {
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
            batch.changed |= self.retain_script_result(action_result);
            if self.script_error.is_some() {
                break;
            }
        }
        self.view_model_listeners = listeners;
        self.owned_data_context = data_context;
        batch.listener_indices.clear();
        self.reporting_listener_view_models = batch.listener_indices;
        (batch.next_event_index, batch.changed)
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
        self.record_advance_phase("focus-snapshot");
        let mut changed = self.process_focus_events(artboard, owned_context.as_deref_mut());
        if self.script_error.is_some() {
            return changed;
        }

        // Pinned C++ snapshots semantic events only after the focus batch has
        // finished (`state_machine_instance.cpp:2557-2558,2449-2490`).
        // Therefore a focus action that queues a semantic callback reaches
        // this same frame, while another semantic callback queued during this
        // loop waits for the next frame.
        self.record_advance_phase("semantic-snapshot");
        changed |= self.process_semantic_events(artboard, owned_context);
        changed
    }

    fn process_focus_events(
        &mut self,
        artboard: &mut ArtboardInstance,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        // Snapshot-and-clear before the first callback. Focus generated while
        // this batch runs remains in the member queue for a later new frame.
        let focus_events = std::mem::take(&mut self.queued_focus_events);
        let mut changed = false;
        for event in focus_events {
            let Some(listener) = self.listener_definitions.get(event.listener_index) else {
                continue;
            };
            if (event.is_focus && !listener.has_listener(RuntimeListenerType::Focus))
                || (!event.is_focus && !listener.has_listener(RuntimeListenerType::Blur))
            {
                continue;
            }
            let actions = listener.listener_actions.clone();
            let invocation = event.into_invocation();
            let result = self.perform_listener_actions(
                artboard,
                &actions,
                owned_context.as_deref_mut(),
                &invocation,
                &mut NoopScriptHost,
            );
            changed |= self.retain_script_result(result);
            #[cfg(test)]
            self.run_deferred_callback_probe(true);
            if self.script_error.is_some() {
                break;
            }
        }
        changed
    }

    fn process_semantic_events(
        &mut self,
        artboard: &mut ArtboardInstance,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        // Snapshot-and-clear before the first callback. Null group/listener
        // adaptations are retained in the batch and skipped in FIFO order;
        // semantic work queued by callbacks waits for a later new frame.
        let semantic_events = std::mem::take(&mut self.queued_semantic_events);
        let mut changed = false;
        for event in semantic_events {
            let Some(invocation) = event.into_invocation() else {
                continue;
            };
            let ScriptListenerInvocation::Semantic { listener_index, .. } = invocation else {
                unreachable!("typed semantic queue creates semantic invocations");
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
            #[cfg(test)]
            self.run_deferred_callback_probe(false);
            if self.script_error.is_some() {
                break;
            }
        }
        changed
    }

    #[cfg(test)]
    fn run_deferred_callback_probe(&mut self, focus_phase: bool) {
        let Some(probe) = self.deferred_callback_probe else {
            return;
        };
        let event = match (focus_phase, probe) {
            (
                true,
                RuntimeDeferredCallbackProbe::FocusQueuesSemantic {
                    listener_index,
                    action_type,
                },
            )
            | (
                false,
                RuntimeDeferredCallbackProbe::SemanticQueuesSemantic {
                    listener_index,
                    action_type,
                },
            ) => Some((listener_index, action_type)),
            _ => None,
        };
        let Some((listener_index, action_type)) = event else {
            return;
        };
        self.deferred_callback_probe = None;
        self.queue_semantic_event(listener_index, action_type);
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
                    if self.scripted_data_context_prepare_pending()
                        && !self
                            .scripted_listener_action_instances
                            .contains_key(&definition.action_global_id())
                    {
                        // Only the unavailable script-facing callback is inert.
                        // Ordinary actions in this same listener occurrence have
                        // already run and later ordinary actions must continue.
                        continue;
                    }
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
                let blob_value = value.blob_data_bind_value();
                match (owned_context, font_value.as_ref(), blob_value.as_ref()) {
                    (Some(context), Some(font_value), _) => self
                        .set_owned_view_model_context_font_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            font_value,
                        ),
                    (Some(context), _, Some(blob_value)) => self
                        .set_owned_view_model_context_blob_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            blob_value,
                        ),
                    (Some(context), None, None) => self
                        .set_owned_view_model_context_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            value.asset_index(),
                        ),
                    (None, _, _) => self.set_default_view_model_asset_source_for_data_bind(
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

    fn set_owned_view_model_context_blob_asset_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: &RuntimeBlobAssetValue,
    ) -> bool {
        self.data_bind_graph
            .set_owned_view_model_context_blob_asset_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
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

    fn owned_context_listener_report_waits_for_nested_relative_relink(
        &self,
        listener_index: usize,
        changed_cell: Option<&RuntimeViewModelCell>,
    ) -> bool {
        let Some(changed_cell) = changed_cell else {
            return false;
        };
        let Some(listener) = self.view_model_listeners.get(listener_index) else {
            return false;
        };
        let Some(file) = self.scripted_listener_runtime_file.as_deref() else {
            return false;
        };
        let Some(manifest) = file.manifest() else {
            return false;
        };
        let definition = &listener.listener_definitions[listener.listener_index];
        listener.property_bindings.iter().any(|binding| {
            let binding_matches_changed_cell = binding
                .cell_binding
                .as_ref()
                .is_some_and(|bound| bound.cell.ptr_eq(changed_cell));
            if !binding_matches_changed_cell {
                return false;
            }
            let path = match binding.source {
                RuntimeViewModelListenerSource::Single => definition.view_model_path.as_ref(),
                RuntimeViewModelListenerSource::Input(input_index) => definition
                    .view_model_input_types
                    .get(input_index)
                    .and_then(|input| input.path()),
            };
            matches!(
                path,
                Some(RuntimeListenerViewModelPath::Relative {
                    resolved_name_ids,
                    ..
                }) if resolved_name_ids.len() > 1
                    && resolved_name_ids
                        .iter()
                        .all(|name_id| manifest.resolve_name(*name_id).is_some())
            )
        })
    }

    fn write_owned_view_model_context_with_listener_boundary(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        write: impl FnOnce(&mut RuntimeDataBindGraph, &mut RuntimeOwnedViewModelInstance, usize) -> bool,
    ) -> bool {
        let changed_cell = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index)
            .and_then(|source_path| {
                let (&view_model_index, property_path) = source_path.split_first()?;
                (usize::try_from(view_model_index).ok()? == context.view_model_index)
                    .then_some(property_path)?
                    .iter()
                    .map(|property_index| usize::try_from(*property_index).ok())
                    .collect::<Option<Vec<_>>>()
            })
            .and_then(|property_path| context.cell_by_property_path(&property_path));
        let mut previously_reported = Vec::new();
        self.reported_listener_view_models
            .swap_into(&mut previously_reported);
        let changed = write(&mut self.data_bind_graph, context, data_bind_index);
        let mut reports_from_write = Vec::new();
        self.reported_listener_view_models
            .swap_into(&mut reports_from_write);
        for listener_index in previously_reported {
            self.reported_listener_view_models
                .report_data_bind(listener_index);
        }
        for listener_index in reports_from_write {
            if self.owned_context_listener_report_waits_for_nested_relative_relink(
                listener_index,
                changed_cell.as_ref(),
            ) {
                self.post_apply_listener_view_models.push(listener_index);
            } else {
                self.reported_listener_view_models
                    .report_data_bind(listener_index);
            }
        }
        changed
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
        // C++ discovers a fully resolved nested-relative listener report
        // during the later DataBind occurrence pass. Capture only the exact
        // listener/cell reports produced by this external write; flat and
        // unrelated listeners retain their immediate pending boundary.
        let changed = self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_number_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
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
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_symbol_list_index_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        ) {
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
        let changed = self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_boolean_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
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
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_enum_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        ) {
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
        let changed = self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_color_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
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
        let changed = self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_string_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
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
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_trigger_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        ) {
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
        let changed = self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.fire_owned_view_model_context_trigger_source_for_data_bind_at_property_path(
                    context,
                    data_bind_index,
                    value,
                    property_path,
                )
            },
        );
        if !changed {
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
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_list_source_item_count_for_data_bind(
                    context,
                    data_bind_index,
                    item_count,
                )
            },
        ) {
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
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_asset_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        ) {
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
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_artboard_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        ) {
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
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_view_model_source_for_data_bind(
                    context,
                    data_bind_index,
                    instance_index,
                )
            },
        ) {
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

    /// Rust error projection for C++ pointer paths whose null behavior is not
    /// a safe clear/no-op. The C++-shaped methods below keep `Option` at the
    /// boundary so their intentionally different null branches cannot be
    /// collapsed by a typed convenience API.
    #[doc(hidden)]
    pub(crate) fn bind_data_context(
        &mut self,
        file: &RuntimeFile,
        artboard: &mut ArtboardInstance,
        data_context: Option<&RuntimeStateMachineDataContext>,
    ) -> Result<bool, RuntimeDataContextBindError> {
        let data_context = data_context.ok_or(RuntimeDataContextBindError::NullDataContext)?;
        // Pinned C++: clear the machine registration, register the supplied
        // context, clear/bind the artboard, then bind the machine.
        self.clear_data_context();
        self.primary_data_context = Some(data_context.clone());
        self.record_bind_phase("register-machine");
        data_context.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        let projection = data_context.projection();
        self.record_bind_phase("clear-artboard");
        artboard.clear_data_context_for_state_machine_bind();
        self.record_bind_phase("bind-artboard");
        let mut changed =
            artboard.bind_owned_view_model_artboard_data_context(file, &projection, true, true);
        data_context.add_artboard_rebind_dependent(artboard);
        self.record_bind_phase("bind-machine");
        changed |= self.internal_data_context(Some(&projection))?;
        Ok(changed)
    }

    /// C++ `inheritDataContext`: null is a no-op and, critically, the old
    /// context is not cleared before the new one registers this same sink.
    /// A→B therefore leaves a live weak registration on A while the retained
    /// context pointer and all paths refer to B.
    #[doc(hidden)]
    pub(crate) fn inherit_data_context(
        &mut self,
        data_context: Option<&RuntimeStateMachineDataContext>,
    ) -> Result<bool, RuntimeDataContextBindError> {
        let Some(data_context) = data_context else {
            return Ok(false);
        };
        self.primary_data_context = Some(data_context.clone());
        self.record_bind_phase("register-machine-without-clear");
        data_context.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        self.internal_data_context(Some(&data_context.projection()))
    }

    /// C++ `dataContext(rcp<DataContext>)`: clear only the machine
    /// registration/listener cells, then forward the supplied pointer to the
    /// internal binder without registering it or touching the artboard.
    #[doc(hidden)]
    pub(crate) fn set_data_context(
        &mut self,
        data_context: Option<&RuntimeStateMachineDataContext>,
    ) -> Result<bool, RuntimeDataContextBindError> {
        self.clear_data_context();
        self.primary_data_context = data_context.cloned();
        let projection = data_context.map(RuntimeStateMachineDataContext::projection);
        self.internal_data_context(projection.as_ref())
    }

    /// Borrowed counterpart of C++ `dataContext() const`.
    #[doc(hidden)]
    pub(crate) fn data_context(&self) -> Option<&RuntimeStateMachineDataContext> {
        self.primary_data_context.as_ref()
    }

    /// C++ `setViewModelInstance`: a null pointer is an inert no-op; a live
    /// instance replaces only the main slot and does not bind any path.
    #[doc(hidden)]
    pub(crate) fn set_view_model_instance(
        &mut self,
        view_model_instance: Option<RuntimeOwnedViewModelHandle>,
    ) -> bool {
        let Some(view_model_instance) = view_model_instance else {
            return false;
        };
        let context = self.ensure_primary_data_context();
        context.set_main(view_model_instance);
        true
    }

    /// C++ `setGlobalViewModelInstance`: validate the named file slot, then
    /// replace or empty exactly that slot. The occupying instance may belong
    /// to a different ViewModel; slot identity comes from `name`.
    #[doc(hidden)]
    pub(crate) fn set_global_view_model_instance(
        &mut self,
        file: Option<&RuntimeFile>,
        name: &str,
        view_model_instance: Option<RuntimeOwnedViewModelHandle>,
    ) -> bool {
        let Some(file) = file else {
            return false;
        };
        let mut validated_slot = RuntimeOwnedViewModelContext::default();
        let valid = match view_model_instance.as_ref() {
            Some(instance) => validated_slot.set_global_named_handle(file, name, instance.clone()),
            None => validated_slot.unset_global_named(file, name),
        };
        if !valid {
            return false;
        }
        if view_model_instance.is_none() && self.primary_data_context.is_none() {
            return true;
        }
        let context = self.ensure_primary_data_context();
        let changed = view_model_instance.map_or_else(
            || context.unset_global_named(file, name),
            |instance| context.set_global_named(file, name, instance),
        );
        if !changed {
            return false;
        }
        true
    }

    /// Fill the missing main first, then missing globals in file-global
    /// order. `RuntimeOwnedViewModelContext` stores globals in slot-key order
    /// and treats any existing cross-model occupant as occupied.
    #[doc(hidden)]
    pub(crate) fn complete_view_model_instances(
        &mut self,
        file: Option<&RuntimeFile>,
        artboard: &ArtboardInstance,
    ) -> bool {
        let (Some(file), Some(context)) = (file, self.primary_data_context.clone()) else {
            return false;
        };
        let Some(artboard_index) = file
            .artboards()
            .into_iter()
            .position(|candidate| candidate.id == artboard.graph_global_id)
        else {
            return false;
        };
        if !context.complete_for_artboard(file, artboard_index) {
            return false;
        }
        true
    }

    /// C++ `bind`: create an empty retained context when needed, complete
    /// missing defaults, bind the artboard, then bind this machine.
    #[doc(hidden)]
    pub(crate) fn bind(
        &mut self,
        file: Option<&RuntimeFile>,
        artboard: &mut ArtboardInstance,
    ) -> Result<bool, RuntimeDataContextBindError> {
        self.ensure_primary_data_context();
        self.record_bind_phase("complete-view-models");
        self.complete_view_model_instances(file, artboard);
        let data_context = self
            .primary_data_context
            .clone()
            .expect("checked retained DataContext")
            .projection();
        self.record_bind_phase("bind-artboard");
        let mut changed = file.is_some_and(|file| {
            artboard.bind_owned_view_model_artboard_data_context(file, &data_context, true, true)
        });
        if file.is_some()
            && let Some(context) = self.primary_data_context.as_ref()
        {
            context.add_artboard_rebind_dependent(artboard);
        }
        self.record_bind_phase("bind-machine");
        changed |= self.internal_data_context(Some(&data_context))?;
        Ok(changed)
    }

    /// Convenience C++ member with deliberately asymmetric null behavior.
    /// Null clears only the machine context/listener cells and unbinds the
    /// artboard. It must not explicitly unbind this machine's DataBinds.
    #[doc(hidden)]
    pub(crate) fn bind_view_model_instance(
        &mut self,
        file: Option<&RuntimeFile>,
        artboard: &mut ArtboardInstance,
        view_model_instance: Option<RuntimeOwnedViewModelHandle>,
    ) -> Result<bool, RuntimeDataContextBindError> {
        let Some(view_model_instance) = view_model_instance else {
            self.clear_data_context();
            self.record_bind_phase("unbind-artboard");
            artboard.unbind_for_state_machine_view_model_clear(file);
            return Ok(true);
        };
        self.set_view_model_instance(Some(view_model_instance));
        self.bind(file, artboard)
    }

    /// Pure C++ slot read. Unlike the setter, the lookup intentionally does
    /// not reject a non-global name before consulting that numeric slot.
    #[doc(hidden)]
    pub(crate) fn global_view_model_instance(
        &self,
        file: Option<&RuntimeFile>,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelHandle> {
        let file = file?;
        let slot = file
            .view_models()
            .iter()
            .position(|view_model| view_model.object.string_property("name") == Some(name))?;
        self.primary_data_context.as_ref()?.global_slot_handle(slot)
    }

    /// C++ `rebind`: clear/reapply the artboard first, then reapply the exact
    /// retained machine context. A cleared/null context is still forwarded to
    /// the internal paths and can therefore fail at a ViewModel listener.
    #[doc(hidden)]
    pub(crate) fn rebind(
        &mut self,
        file: &RuntimeFile,
        artboard: &mut ArtboardInstance,
    ) -> Result<bool, RuntimeDataContextBindError> {
        let data_context = self
            .primary_data_context
            .as_ref()
            .map(RuntimeStateMachineDataContext::projection);
        self.record_bind_phase("clear-artboard");
        artboard.clear_data_context_for_state_machine_bind();
        self.record_bind_phase("bind-artboard");
        let mut changed = data_context.as_ref().is_some_and(|data_context| {
            artboard.bind_owned_view_model_artboard_data_context(file, data_context, true, true)
        });
        if data_context.is_some()
            && let Some(context) = self.primary_data_context.as_ref()
        {
            context.add_artboard_rebind_dependent(artboard);
        }
        self.record_bind_phase("bind-machine");
        changed |= self.internal_data_context(data_context.as_ref())?;
        Ok(changed)
    }

    /// C++ `clearDataContext`: unregister/null first, then drop listener
    /// property cells. It does not unbind state-machine DataBinds or touch the
    /// artboard/script occurrences.
    #[doc(hidden)]
    pub(crate) fn clear_data_context(&mut self) {
        self.record_bind_phase("clear-machine");
        self.primary_data_context = None;
        self.owned_data_context = None;
        self.active_owned_view_model_advance_context = None;
        self.active_file_view_model_binding = None;
        self.scripted_data_context_bind_complete = false;
        // Dropping this sink makes all old weak registrations inert, the Rust
        // equivalent of removeDependentContainer.
        self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        self.clear_view_model_listener_cell_bindings();
    }

    /// C++ delegates this member exclusively to the artboard.
    #[doc(hidden)]
    pub(crate) fn relink_data_context(
        &mut self,
        file: &RuntimeFile,
        artboard: &mut ArtboardInstance,
    ) -> bool {
        artboard.relink_data_context_for_state_machine(file)
    }

    /// Rebuild only one context-bind subtype. A plain authored DataBind is
    /// ignored; a null pointer is an error, matching the C++ dereference.
    #[doc(hidden)]
    pub(crate) fn rebuild_data_bind(
        &mut self,
        data_bind_index: Option<usize>,
    ) -> Result<bool, RuntimeDataContextBindError> {
        let data_bind_index = data_bind_index.ok_or(RuntimeDataContextBindError::NullDataBind)?;
        let Some(source_index) = self
            .data_bind_graph
            .default_view_model_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.source.0)
        else {
            return Ok(false);
        };
        if !self
            .data_bind_graph
            .sources
            .get(source_index)
            .is_some_and(|source| source.context_bindable)
        {
            return Ok(false);
        }
        let Some(data_context) = self.owned_data_context.clone() else {
            self.unbind_data_bind_source(source_index);
            return Ok(false);
        };
        let mut changed = self
            .data_bind_graph
            .bind_owned_view_model_data_context_for_data_bind(data_bind_index, &data_context);
        changed |= self
            .data_bind_graph
            .finalize_owned_view_model_data_context_for_data_bind(data_bind_index, &data_context);
        if changed {
            self.needs_advance = true;
        }
        Ok(changed)
    }

    /// C++ `unbind`: context/listener teardown precedes every machine
    /// DataBind source/converter unbind.
    #[doc(hidden)]
    pub(crate) fn unbind(&mut self) {
        self.clear_data_context();
        self.unbind_data_binds();
    }

    /// C++ `internalDataContext` primary machine path: assign, bind ordinary
    /// and keyframe DataBinds, bind listener cells, then hand the new context
    /// to the deferred scripted-object context/init passes.
    #[doc(hidden)]
    pub(crate) fn internal_data_context(
        &mut self,
        data_context: Option<&RuntimeOwnedDataContext>,
    ) -> Result<bool, RuntimeDataContextBindError> {
        self.record_bind_phase("assign-context");
        self.owned_data_context = data_context.cloned();
        let Some(data_context) = data_context else {
            self.record_bind_phase("bind-data-binds");
            self.unbind_data_binds();
            self.record_bind_phase("bind-listener-cells");
            self.clear_view_model_listener_cell_bindings();
            if !self.view_model_listeners.is_empty() {
                return Err(RuntimeDataContextBindError::NullDataContextWithViewModelListeners);
            }
            self.record_bind_phase("script-context-pass");
            self.scripted_data_context_bind_complete = false;
            self.record_bind_phase("script-init-pass");
            self.active_owned_view_model_advance_context = None;
            return Ok(true);
        };

        self.record_bind_phase("bind-data-binds");
        let changed = self.bind_owned_data_binds_from_data_context(data_context);
        self.record_bind_phase("bind-listener-cells");
        self.bind_view_model_listener_cells_for_data_context(data_context);
        // Rust's authenticated scripting facade owns the fallible table
        // context/install and init/hydrate calls. Mark that exact later pass
        // only after every listener cell has been rebound.
        self.record_bind_phase("script-context-pass");
        self.scripted_data_context_bind_complete = false;
        self.record_bind_phase("script-init-pass");
        self.retain_owned_view_model_advance_context(data_context);
        self.needs_advance = true;
        Ok(changed)
    }

    fn ensure_primary_data_context(&mut self) -> RuntimeStateMachineDataContext {
        if let Some(context) = self.primary_data_context.clone() {
            // Reusing an existing DataContext must preserve its registration
            // status. In particular, `dataContext(value)` intentionally
            // installs without `addDependentContainer`.
            return context;
        }
        let context = RuntimeStateMachineDataContext::default();
        self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        context.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        self.primary_data_context = Some(context.clone());
        context
    }

    fn refresh_primary_data_context_projection(&mut self) {
        let Some(context) = self.primary_data_context.as_ref() else {
            self.owned_data_context = None;
            return;
        };
        let data_context = context.projection();
        self.active_file_view_model_binding = None;
        self.owned_data_context = Some(data_context.clone());
        self.retain_owned_view_model_advance_context(&data_context);
    }

    fn unbind_data_bind_source(&mut self, source_index: usize) {
        let Some(source) = self.data_bind_graph.sources.get_mut(source_index) else {
            return;
        };
        source.retained_bind.reset_preserving_notification();
        if let Some(converter) = source.converter.as_ref() {
            source
                .converter_data_binds
                .unbind(converter, &mut source.converter_state);
        }
        source.retained_structural_source = None;
        source.bound = false;
        source.reconcile_pending = false;
    }

    fn unbind_data_binds(&mut self) {
        for source_index in 0..self.data_bind_graph.sources.len() {
            self.unbind_data_bind_source(source_index);
        }
        self.data_bind_graph.context_kind = RuntimeDataBindGraphContextKind::None;
        self.data_bind_graph.imported_view_model_context = None;
        self.data_bind_graph.default_view_model_bindings_dirty = false;
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            for source in &mut graph.sources {
                source.retained_bind.reset_preserving_notification();
                if let Some(converter) = source.converter.as_ref() {
                    source
                        .converter_data_binds
                        .unbind(converter, &mut source.converter_state);
                }
                source.retained_structural_source = None;
                source.bound = false;
                source.reconcile_pending = false;
            }
            graph.context_kind = RuntimeDataBindGraphContextKind::None;
            graph.imported_view_model_context = None;
            graph.default_view_model_bindings_dirty = false;
        }
    }

    /// Machine-only borrow-model adaptation used by the established typed
    /// Rust APIs, which do not own an Artboard borrow in their signatures.
    fn bind_data_context_to_machine(&mut self, data_context: &RuntimeOwnedDataContext) -> bool {
        self.clear_data_context();
        self.primary_data_context = None;
        data_context.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        self.internal_data_context(Some(data_context))
            .unwrap_or(false)
    }

    /// Preserve the established typed Rust context representations while
    /// making their public entry points pure delegating adaptations. The
    /// C++-shaped clear member owns replacement teardown; the closure contains
    /// only the representation-specific graph/listener projection required to
    /// preserve each existing boolean API's signature and behavior.
    fn bind_typed_context_adaptation(
        &mut self,
        bind: impl FnOnce(&mut StateMachineInstance) -> bool,
    ) -> bool {
        self.clear_data_context();
        bind(self)
    }

    pub fn bind_empty_data_context(&mut self) -> bool {
        self.bind_typed_context_adaptation(|machine| {
            if !machine.data_bind_graph.bind_empty_data_context() {
                return false;
            }
            for graph in machine.key_frame_data_bind_graphs.iter_mut().flatten() {
                graph.bind_empty_data_context();
            }
            machine.active_file_view_model_binding = None;
            machine.needs_advance = true;
            true
        })
    }

    pub fn bind_default_view_model_context(&mut self) -> bool {
        self.bind_typed_context_adaptation(|machine| {
            if !machine.data_bind_graph.bind_default_view_model_context() {
                return false;
            }
            if let Some(context) = machine.default_view_model_trigger_instance.as_ref() {
                machine
                    .data_bind_graph
                    .bind_file_view_model_trigger_sources(context);
            }
            for graph in machine.key_frame_data_bind_graphs.iter_mut().flatten() {
                graph.bind_default_view_model_context();
                if let Some(context) = machine.default_view_model_trigger_instance.as_ref() {
                    graph.bind_file_view_model_trigger_sources(context);
                }
            }
            machine.sync_bindable_font_assets_from_default_context();
            machine.active_file_view_model_binding =
                machine.default_view_model_index.map(|index| (index, 0));
            machine.needs_advance = true;
            true
        })
    }

    /// Create and bind the artboard's authored default `DataContext`.
    ///
    /// This is the C++ `createDefaultViewModelInstance(artboard)` followed by
    /// `StateMachineInstance::bindViewModelInstance` path. Unlike the
    /// graph-only compatibility method above, the mutable `DataContext` is
    /// shared with the artboard tree, so nested artboards and the outer state
    /// machine observe the same retained ViewModel cells.
    pub fn bind_default_view_model_context_on_artboard(
        &mut self,
        artboard: &mut ArtboardInstance,
    ) -> bool {
        let Some(file) = artboard.runtime_file_arc() else {
            return false;
        };
        let Some(artboard_index) = file
            .artboards()
            .into_iter()
            .position(|candidate| candidate.id == artboard.graph_global_id)
        else {
            return false;
        };
        let context = RuntimeStateMachineDataContext::default();
        if !context.complete_for_artboard(&file, artboard_index) {
            return false;
        }
        self.bind_data_context(&file, artboard, Some(&context))
            .unwrap_or(false)
    }

    #[cfg(feature = "tools")]
    #[doc(hidden)]
    pub fn debug_set_bound_main_font_bytes_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        font_bytes: Option<std::sync::Arc<[u8]>>,
    ) -> bool {
        let Some(data_context) = self.owned_data_context.as_ref() else {
            return false;
        };
        let Some(main) = data_context.main_context_chain(file).into_iter().next() else {
            return false;
        };
        if !main.scope_path().is_empty() {
            return false;
        }
        let changed = main
            .root_handle()
            .borrow_mut()
            .set_live_font_bytes_by_property_name(property_name, font_bytes);
        if changed {
            self.needs_advance = true;
        }
        changed
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
        self.bind_typed_context_adaptation(|machine| {
            let Some(instance_cells) = machine
                .file_view_model_instances
                .as_ref()
                .and_then(|catalog| catalog.instance(view_model_index, instance_index))
            else {
                return false;
            };
            if !machine.data_bind_graph.bind_view_model_instance_context(
                file,
                view_model_index,
                instance_index,
                &instance_cells,
            ) {
                return false;
            }
            for graph in machine.key_frame_data_bind_graphs.iter_mut().flatten() {
                graph.bind_view_model_instance_context(
                    file,
                    view_model_index,
                    instance_index,
                    &instance_cells,
                );
            }
            machine.sync_bindable_font_assets_from_imported_instance(
                file,
                view_model_index,
                instance_index,
            );
            machine.active_file_view_model_binding = Some((view_model_index, instance_index));
            machine.needs_advance = true;
            true
        })
    }

    pub fn bind_imported_view_model_context(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeImportedViewModelInstanceContext,
    ) -> bool {
        self.bind_typed_context_adaptation(|machine| {
            let Some(instance) = machine
                .file_view_model_instances
                .as_ref()
                .and_then(|catalog| {
                    catalog.instance(context.view_model_index, context.instance_index)
                })
            else {
                return false;
            };
            if !context.adopt_file_trigger_instance(instance) {
                return false;
            }
            if !machine
                .data_bind_graph
                .bind_imported_view_model_context(file, context)
            {
                return false;
            }
            for graph in machine.key_frame_data_bind_graphs.iter_mut().flatten() {
                graph.bind_imported_view_model_context(file, context);
            }
            machine.sync_bindable_font_assets_from_imported_instance(
                file,
                context.view_model_index,
                context.instance_index,
            );
            machine.bind_view_model_listener_cells_for_imported_context(context);
            machine.active_file_view_model_binding =
                Some((context.view_model_index, context.instance_index));
            machine.needs_advance = true;
            true
        })
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
        self.bind_typed_context_adaptation(|machine| {
            machine.bind_owned_view_model_snapshot(context)
        })
    }

    /// Bind and retain a shared owned view-model graph.
    ///
    /// Later mutations through any alias are refreshed at the next data
    /// context advance, so the state machine and host never fork identity.
    pub fn bind_owned_view_model_handle(&mut self, context: &RuntimeOwnedViewModelHandle) -> bool {
        let staged = RuntimeOwnedViewModelContext::from_main_handle(context.clone());
        let context = RuntimeOwnedViewModelContextHandle::root_without_file(context.clone());
        let changed = self.bind_owned_view_model_context_handle(&context);
        let primary = RuntimeStateMachineDataContext::from_owned_context(staged);
        // The immutable adaptation registered this sink directly on the
        // current root. Rotate it before installing the mutable carrier so a
        // later setMain makes that old weak registration inert.
        self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        primary.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        self.primary_data_context = Some(primary);
        changed
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
        self.bind_typed_context_adaptation(|machine| {
            let mut advance_context = RuntimeOwnedViewModelAdvanceContext::default();
            advance_context.extend(context);
            machine.active_owned_view_model_advance_context = Some(advance_context);
            let mut changed = machine
                .data_bind_graph
                .bind_owned_view_model_context(context);
            for graph in machine.key_frame_data_bind_graphs.iter_mut().flatten() {
                changed |= graph.bind_owned_view_model_context(context);
            }
            machine.sync_bindable_font_assets_from_owned_context(context);
            machine.bind_view_model_listener_cells_for_context_chain(context, &[&[]]);
            if changed {
                machine.needs_advance = true;
            }
            changed
        })
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
                                let property_path = listener_property_path_for_resolved_name_path(
                                    context,
                                    file,
                                    context_path,
                                    resolved_name_ids,
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
            listener.report_pending_trigger_bindings(
                &self.reported_listener_view_models,
                listener_index,
            );
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
                                resolved_listener_property_path_for_data_context(
                                    data_context,
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
            listener.report_pending_trigger_bindings(
                &self.reported_listener_view_models,
                listener_index,
            );
        }
    }

    fn bind_view_model_listener_cells_for_imported_context(
        &mut self,
        context: &RuntimeImportedViewModelInstanceContext,
    ) {
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
                    let (view_model_index, property_path) = match path {
                        RuntimeListenerViewModelPath::Absolute {
                            view_model_index,
                            property_path,
                        } => (*view_model_index, property_path.as_slice()),
                        RuntimeListenerViewModelPath::Relative {
                            absolute_fallback: Some((view_model_index, property_path)),
                            ..
                        } => (*view_model_index, property_path.as_slice()),
                        RuntimeListenerViewModelPath::Relative {
                            absolute_fallback: None,
                            ..
                        } => return None,
                    };
                    if view_model_index != context.view_model_index {
                        return None;
                    }
                    let mut source_path = Vec::with_capacity(property_path.len() + 1);
                    source_path.push(u32::try_from(view_model_index).ok()?);
                    source_path.extend(
                        property_path
                            .iter()
                            .copied()
                            .map(u32::try_from)
                            .collect::<Result<Vec<_>, _>>()
                            .ok()?,
                    );
                    context.trigger_cell_for_source_path(&source_path)
                });
                relink_view_model_listener_cell(
                    binding,
                    cell,
                    &self.reported_listener_view_models,
                    listener_index,
                );
            }
            listener.report_pending_trigger_bindings(
                &self.reported_listener_view_models,
                listener_index,
            );
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
                if context
                    .blob_asset_value_by_property_path(property_path)
                    .is_some()
                {
                    let blob_value = asset_value.blob_data_bind_value().unwrap_or_else(|| {
                        RuntimeBlobAssetValue::from_file_asset_index(asset_value.asset_index())
                    });
                    return Some(context.apply_blob_asset_data_bind_value_by_property_path(
                        property_path,
                        &blob_value,
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
        let changed = self.bind_owned_view_model_data_context(
            &RuntimeOwnedDataContext::from_owned_context(context),
        );
        let primary = RuntimeStateMachineDataContext::from_owned_context(context.clone());
        self.owned_view_model_rebind_sink = RuntimeCellDirtSink::new();
        primary.add_rebind_dependent(&self.owned_view_model_rebind_sink);
        self.primary_data_context = Some(primary);
        changed
    }

    pub(crate) fn bind_owned_view_model_context_chain(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> bool {
        self.bind_typed_context_adaptation(|machine| {
            let mut advance_context = RuntimeOwnedViewModelAdvanceContext::default();
            advance_context.extend(context);
            machine.active_owned_view_model_advance_context = Some(advance_context);
            let mut changed = machine.data_bind_graph.bind_owned_view_model_context_chain(
                file,
                context,
                context_chain,
            );
            for graph in machine.key_frame_data_bind_graphs.iter_mut().flatten() {
                changed |= graph.bind_owned_view_model_context_chain(file, context, context_chain);
            }
            machine.sync_bindable_font_assets_from_owned_context_chain(
                file,
                context,
                context_chain,
            );
            machine.bind_view_model_listener_cells_for_context_chain(context, context_chain);
            if changed {
                machine.needs_advance = true;
            }
            changed
        })
    }

    pub(crate) fn bind_owned_view_model_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) -> bool {
        self.bind_data_context_to_machine(data_context)
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
        #[cfg(test)]
        {
            self.data_context_advance_call_count += 1;
        }
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
        let (mut collected, mut schedules_advance) = self
            .data_bind_graph
            .collect_retained_source_dirt_with_schedule();
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            let (graph_collected, graph_schedules_advance) =
                graph.collect_retained_source_dirt_with_schedule();
            collected |= graph_collected;
            schedules_advance |= graph_schedules_advance;
        }
        for binding in &mut self.scripted_object_bindings {
            let binding_collected = binding.collect_source_dirt();
            collected |= binding_collected;
            schedules_advance |= binding_collected;
        }
        if self.owned_data_context.is_none() {
            if schedules_advance {
                self.needs_advance = true;
            }
            return collected;
        }
        let structural_rebind = self
            .owned_view_model_rebind_sink
            .take_dirt()
            .contains(RuntimeCellDirt::BINDINGS);
        if structural_rebind {
            if self.primary_data_context.is_some() {
                self.refresh_primary_data_context_projection();
            }
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
        if schedules_advance || structural_rebind {
            self.needs_advance = true;
        }
        collected || structural_rebind
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
        if self.event_bubble_owner_attached && !self.notifying_event_listeners {
            let bubbled = self
                .bubbled_event_reports
                .len()
                .saturating_sub(self.bubbled_event_report_index);
            if bubbled != 0 {
                return bubbled;
            }
        }
        self.events_applied_during_loop.len()
            + self
                .reported_events
                .len()
                .saturating_sub(self.next_unapplied_reported_event_index())
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
        if self.event_bubble_owner_attached && !self.notifying_event_listeners {
            let bubble_index = self.bubbled_event_report_index.checked_add(index)?;
            if bubble_index < self.bubbled_event_reports.len() {
                if bubble_index + 1 == self.bubbled_event_reports.len() {
                    self.bubbled_event_report_index = self.bubbled_event_reports.len();
                }
                let event = self.bubbled_event_reports.get_mut(bubble_index)?;
                event.refresh_from_live_artboard(artboard);
                return Some(event);
            }
        }
        if index < self.events_applied_during_loop.len() {
            let event = self.events_applied_during_loop.get_mut(index)?;
            event.refresh_from_live_artboard(artboard);
            return Some(event);
        }
        let index = index.checked_sub(self.events_applied_during_loop.len())?;
        let index = self
            .next_unapplied_reported_event_index()
            .checked_add(index)?;
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
        if index < self.events_applied_during_loop.len() {
            return self.events_applied_during_loop.get(index);
        }
        let index = index.checked_sub(self.events_applied_during_loop.len())?;
        self.reported_events.get(
            self.next_unapplied_reported_event_index()
                .checked_add(index)?,
        )
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
        let applied_start = self
            .host_events_applied_during_loop_index
            .min(self.events_applied_during_loop.len());
        let applied_events = &mut self.events_applied_during_loop[applied_start..];
        for event in applied_events.iter_mut() {
            event.refresh_from_live_artboard(artboard);
        }
        let mut output = applied_events.to_vec();
        self.host_events_applied_during_loop_index = self.events_applied_during_loop.len();

        let start = self
            .host_reported_event_index
            .min(self.reported_events.len());
        let events = &mut self.reported_events[start..];
        for event in events.iter_mut() {
            event.refresh_from_live_artboard(artboard);
        }
        output.extend_from_slice(events);
        self.host_reported_event_index = self.reported_events.len();
        output
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

    /// C++-corresponding raw `StateMachineInstance::advance(seconds,
    /// newFrame)`. The Artboard is explicit for Rust's borrow model; elapsed
    /// seconds are forwarded without finite, sign, or zero validation.
    pub(crate) fn advance_on_artboard(
        &mut self,
        artboard: &mut ArtboardInstance,
        elapsed_seconds: f32,
        new_frame: bool,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        let semantic_scroll_changed = self
            .semantic_tree
            .as_mut()
            .and_then(|tree| tree.pending_focus_scroll.take())
            .is_some_and(|route| {
                artboard.scroll_focus_target_into_view(route.owner_identity, route.target_local_id)
            });
        let definitions = artboard.state_machine_definition_owner(self);
        let Some(state_machine) = definitions.get(self.state_machine_index) else {
            return false;
        };
        if new_frame && let Some(context) = owned_context.as_deref_mut() {
            self.bind_owned_view_model_context_mut(context);
        }
        let advanced = self.advance(
            artboard,
            state_machine,
            elapsed_seconds,
            new_frame,
            owned_context,
        );
        #[cfg(test)]
        if !new_frame
            && elapsed_seconds == 0.0
            && let Some((phase, total_order)) = &self.event_settlement_total_order_trace
        {
            total_order.borrow_mut().push(phase);
        }
        advanced | semantic_scroll_changed
    }

    pub(crate) fn advance(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        elapsed_seconds: f32,
        new_frame: bool,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        let result = self.advance_with_report_mode(
            artboard,
            state_machine,
            elapsed_seconds,
            new_frame,
            owned_context,
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    pub(crate) fn drop_hidden_focus_target(&mut self, artboard: &ArtboardInstance) {
        let _ = artboard;
        self.focus.drop_hidden_focus_target();
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
        if self.scripted_data_context_rebind_pending() {
            return Ok(false);
        }
        #[cfg(test)]
        {
            self.transition_probe_count += 1;
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
            }
        }
        self.capture_focus_callbacks();
        if changed_state {
            self.needs_advance = true;
        }
        Ok(changed_state)
    }

    fn advance_with_report_mode(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        elapsed_seconds: f32,
        new_frame: bool,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        if let Some(error) = self.script_error.as_ref() {
            return Err(error.clone());
        }
        if self.scripted_data_context_rebind_pending() {
            return Ok(false);
        }
        #[cfg(test)]
        {
            self.raw_advance_call_count += 1;
        }
        #[cfg(test)]
        self.advance_phase_trace.clear();
        if new_frame {
            for layer in &mut self.layers {
                layer.begin_new_frame();
            }
        }
        self.prepare_advance_event_phase(artboard);
        if new_frame {
            let next_event_index = self.next_unapplied_reported_event_index();
            self.apply_local_event_listeners(
                artboard,
                next_event_index,
                owned_context.as_deref_mut(),
            );
            let applied_report_count = self.next_unapplied_reported_event_index();
            self.discard_reported_event_prefix(applied_report_count);
            // C++ clears m_needsAdvance after focus, semantic, and event
            // processing. Signals queued by those processors can therefore
            // be lost unless later layer/bind work rearms the latch.
            self.record_advance_phase("clear-latch");
            self.needs_advance = false;
        }
        self.finish_advance_after_apply_events(
            artboard,
            state_machine,
            elapsed_seconds,
            owned_context,
            host,
        )
    }

    fn prepare_advance_event_phase(&mut self, artboard: &ArtboardInstance) {
        self.record_advance_phase("draw-sort-check");
        let draw_order_change_counter = artboard.prepared_epoch();
        if self.draw_order_change_counter != draw_order_change_counter {
            self.sort_hit_components(artboard);
            self.draw_order_change_counter = draw_order_change_counter;
        }
        if !self.focus.is_inert() {
            self.focus.drop_hidden_focus_target();
        }
    }

    fn finish_advance_after_apply_events(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        elapsed_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        // A retained context can be mutated through any alias between frames.
        // Collect its pushed source dirt before the pre-layer bind pass.
        self.collect_retained_owned_view_model_dirt();
        // One C++ DataBindContainer pass owns both ordinary and cloned
        // ScriptInput occurrences before layer advancement. Do not run a
        // separate scripted full-list walk here: it loses the container's
        // authored cross-family partition/order.
        self.record_advance_phase("pre-layer-binds");
        self.prepare_key_frame_data_bind_enrollment(
            crate::animation::RuntimeKeyFrameDataBindEnrollment::Initial,
        );
        self.update_data_binds_false(artboard, owned_context.as_deref(), host)?;
        self.prepare_key_frame_data_bind_enrollment(
            crate::animation::RuntimeKeyFrameDataBindEnrollment::Late,
        );
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
        self.record_advance_phase("authored-layers");
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
            keep_going |= layer_result.keep_going;
        }
        // `StateMachineInstance::advance` advances layers first, then every
        // retained DataBind converter. Converter dirt is consumed by the next
        // `updateDataBinds(false)` pass; raw advance does not perform a second
        // bind update after `advanceDataBinds`
        // (`state_machine_instance.cpp:2562-2574`).
        self.record_advance_phase("converter-advance");
        let mut data_bind_advance =
            crate::data_bind_graph::RuntimeDataBindGraphStatefulAdvance::default();
        let mut key_frame_data_bind_keep_going = self.advance_key_frame_data_bind_enrollment(
            crate::animation::RuntimeKeyFrameDataBindEnrollment::Initial,
            elapsed_seconds,
        );
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
        key_frame_data_bind_keep_going |= self.advance_key_frame_data_bind_enrollment(
            crate::animation::RuntimeKeyFrameDataBindEnrollment::Late,
            elapsed_seconds,
        );
        #[cfg(test)]
        if let Some(report) = self.bind_advance_test_report.take() {
            self.reported_events.push(report);
        }
        // A focus action performed by a state/transition entry callback calls
        // FocusListenerGroup immediately in C++. Retain that callback's
        // markNeedsAdvance result after the layer loop; queued actions execute
        // at the next new-frame boundary.
        self.capture_focus_callbacks();
        let focus_needs_advance = self.needs_advance;
        self.record_advance_phase("inputs-advanced");
        for input in &mut self.inputs {
            input.advanced();
        }
        let scheduled = focus_needs_advance
            || data_bind_advance.keep_going
            || key_frame_data_bind_keep_going
            || keep_going;
        self.needs_advance = scheduled;
        let advanced = scheduled
            || self.reported_event_count() != 0
            || self.has_pending_listener_view_model_reports();
        // A fully resolved nested-relative listener is discovered during
        // this raw advance in C++. Publish the report only after computing
        // the return value so it is applied by the next new-frame call
        // without making the discovery frame itself report `true`.
        self.finish_listener_view_model_firing_boundary();
        Ok(advanced)
    }

    fn begin_nested_apply_events(
        &mut self,
        artboard: &mut ArtboardInstance,
    ) -> Option<RuntimeNestedApplyEventsPhase> {
        if self.script_error.is_some() {
            return None;
        }
        #[cfg(test)]
        {
            self.raw_advance_call_count += 1;
            self.advance_phase_trace.clear();
        }
        for layer in &mut self.layers {
            layer.begin_new_frame();
        }
        self.events_applied_during_loop.clear();
        self.host_events_applied_during_loop_index = 0;
        self.prepare_advance_event_phase(artboard);
        let next_event_index = self.next_unapplied_reported_event_index();
        self.process_deferred_listener_group_events(artboard, None);
        self.record_advance_phase("apply-events");
        Some(RuntimeNestedApplyEventsPhase {
            next_event_index,
            event_iterations: 0,
        })
    }

    fn begin_nested_apply_events_batch(
        &mut self,
        artboard: &mut ArtboardInstance,
        phase: &mut RuntimeNestedApplyEventsPhase,
    ) -> Option<RuntimeLocalEventListenerBatch> {
        const MAX_EVENT_ITERATIONS: usize = 100;

        if self.script_error.is_some()
            || (phase.next_event_index >= self.reported_events.len()
                && self.reported_listener_view_models.is_empty())
        {
            return None;
        }
        if phase.event_iterations >= MAX_EVENT_ITERATIONS {
            eprintln!("StateMachine exceeded max event iterations");
            return None;
        }
        phase.event_iterations += 1;
        let batch = self.begin_local_event_listener_batch(
            artboard,
            phase.next_event_index,
            None,
            phase.event_iterations > 1,
            true,
        );
        phase.next_event_index = batch.next_event_index;
        Some(batch)
    }

    fn finish_nested_apply_events_batch(
        &mut self,
        artboard: &mut ArtboardInstance,
        notifier_local: usize,
        mut batch: RuntimeLocalEventListenerBatch,
        ancestor_changed: bool,
    ) -> bool {
        batch.changed |= ancestor_changed;
        for event in &batch.bubbled_events {
            self.flush_deferred_owner_audio_event(event);
            #[cfg(any(test, feature = "tools"))]
            record_runtime_nested_event_chain_step(
                notifier_local,
                RuntimeNestedEventChainPhase::AudioUnwind,
            );
        }
        if !batch.bubbled_events.is_empty() {
            self.flush_deferred_owner_audio_events();
        }
        self.finish_local_event_listener_batch(artboard, batch, None)
            .1
    }

    fn finish_nested_advance_after_apply_events(
        &mut self,
        artboard: &mut ArtboardInstance,
        elapsed_seconds: f32,
        phase: RuntimeNestedApplyEventsPhase,
    ) -> bool {
        self.reported_event_listener_index = phase.next_event_index.min(self.reported_events.len());
        let applied_report_count = self.next_unapplied_reported_event_index();
        self.discard_reported_event_prefix(applied_report_count);
        self.record_advance_phase("clear-latch");
        self.needs_advance = false;

        let definitions = artboard.state_machine_definition_owner(self);
        let Some(state_machine) = definitions.get(self.state_machine_index) else {
            return false;
        };
        let result = self.finish_advance_after_apply_events(
            artboard,
            state_machine,
            elapsed_seconds,
            None,
            &mut NoopScriptHost,
        );
        self.retain_script_result(result)
    }

    /// Advance one authored nested animation and complete every report's full
    /// singleton chain before the next callback, mix, owner, or subtree step.
    pub(crate) fn advance_nested_animation_owner_with(
        parent_artboard: &mut ArtboardInstance,
        nested: &mut crate::artboard::RuntimeNestedArtboardInstance,
        host_local: usize,
        animation_index: usize,
        elapsed_seconds: f32,
        mut nested_events: Option<&mut Vec<(usize, Vec<StateMachineReportedEvent>)>>,
        mut nested_event_dispatch: Option<
            &mut dyn FnMut(&mut ArtboardInstance, usize, &[StateMachineReportedEvent]) -> bool,
        >,
    ) -> Result<bool, ScriptError> {
        let should_deliver = nested_events.is_some() || nested_event_dispatch.is_some();
        let mut deliver_to_ancestor =
            |artboard: &mut ArtboardInstance,
             source_local: usize,
             events: &[StateMachineReportedEvent]| {
                if let Some(dispatch) = nested_event_dispatch.as_mut() {
                    (**dispatch)(artboard, source_local, events)
                } else if let Some(nested_events) = nested_events.as_mut() {
                    nested_events.push((source_local, events.to_vec()));
                    false
                } else {
                    false
                }
            };
        let Some(animation) = nested.animations.get_mut(animation_index) else {
            return Ok(false);
        };
        match animation {
            crate::artboard::RuntimeNestedAnimationInstance::Simple {
                local_id,
                animation,
                is_playing,
                speed,
                mix,
            } => {
                let mut changed = false;
                if *is_playing {
                    if should_deliver {
                        let notifier_local = *local_id;
                        let mut deliver =
                            |_source_artboard: &mut ArtboardInstance,
                             event: Option<StateMachineReportedEvent>| {
                                let Some(mut event) = event else {
                                    return false;
                                };
                                // Pinned C++ NestedSimpleAnimation discards
                                // the computed overshoot at this reporter seam.
                                event.seconds_delay = 0.0;
                                let notified = Self::complete_nested_report_batch(
                                    parent_artboard,
                                    host_local,
                                    notifier_local,
                                    std::slice::from_ref(&event),
                                    &mut deliver_to_ancestor,
                                );
                                #[cfg(any(test, feature = "tools"))]
                                record_runtime_nested_event_chain_step(
                                    notifier_local,
                                    RuntimeNestedEventChainPhase::AudioUnwind,
                                );
                                notified
                            };
                        changed |= nested
                            .child
                            .advance_linear_animation_instance_with_callback_sink(
                                animation,
                                elapsed_seconds * *speed,
                                &mut deliver,
                            );
                    } else {
                        changed |= nested
                            .child
                            .advance_linear_animation_instance(animation, elapsed_seconds * *speed);
                    }
                }
                if *mix != 0.0 {
                    changed |= nested
                        .child
                        .apply_linear_animation_instance(animation, *mix);
                }
                Ok(changed)
            }
            crate::artboard::RuntimeNestedAnimationInstance::Remap { animation, mix, .. } => {
                Ok(*mix != 0.0
                    && nested
                        .child
                        .apply_linear_animation_instance(animation, *mix))
            }
            crate::artboard::RuntimeNestedAnimationInstance::StateMachine(occurrence) => {
                let notifier_local = occurrence.local_id();
                let phase = parent_artboard
                    .active_nested_state_machines
                    .get_mut(&notifier_local)
                    .or_else(|| occurrence.state_machine_mut())
                    .and_then(|state_machine| {
                        state_machine.begin_nested_apply_events(&mut nested.child)
                    });
                let Some(mut phase) = phase else {
                    return Ok(false);
                };
                let mut changed = false;
                loop {
                    let batch = parent_artboard
                        .active_nested_state_machines
                        .get_mut(&notifier_local)
                        .or_else(|| occurrence.state_machine_mut())
                        .and_then(|state_machine| {
                            state_machine
                                .begin_nested_apply_events_batch(&mut nested.child, &mut phase)
                        });
                    let Some(batch) = batch else {
                        break;
                    };
                    let ancestor_changed = should_deliver
                        && !batch.bubbled_events.is_empty()
                        && Self::complete_nested_report_batch(
                            parent_artboard,
                            host_local,
                            notifier_local,
                            &batch.bubbled_events,
                            &mut deliver_to_ancestor,
                        );
                    let batch_changed = parent_artboard
                        .active_nested_state_machines
                        .get_mut(&notifier_local)
                        .or_else(|| occurrence.state_machine_mut())
                        .map_or(ancestor_changed, |state_machine| {
                            state_machine.finish_nested_apply_events_batch(
                                &mut nested.child,
                                notifier_local,
                                batch,
                                ancestor_changed,
                            )
                        });
                    changed |= batch_changed;
                }
                changed |= parent_artboard
                    .active_nested_state_machines
                    .get_mut(&notifier_local)
                    .or_else(|| occurrence.state_machine_mut())
                    .is_some_and(|state_machine| {
                        state_machine.finish_nested_advance_after_apply_events(
                            &mut nested.child,
                            elapsed_seconds,
                            phase,
                        )
                    });
                Ok(changed)
            }
        }
    }

    fn complete_nested_report_batch(
        parent_artboard: &mut ArtboardInstance,
        host_local: usize,
        _notifier_local: usize,
        events: &[StateMachineReportedEvent],
        deliver_to_ancestor: &mut dyn FnMut(
            &mut ArtboardInstance,
            usize,
            &[StateMachineReportedEvent],
        ) -> bool,
    ) -> bool {
        #[cfg(any(test, feature = "tools"))]
        for event in events {
            record_runtime_nested_event_report_step(_notifier_local, event.seconds_delay());
        }
        let notified = deliver_to_ancestor(parent_artboard, host_local, events);
        #[cfg(any(test, feature = "tools"))]
        for _ in events {
            record_runtime_nested_event_chain_step(
                _notifier_local,
                RuntimeNestedEventChainPhase::AncestorDispatch,
            );
        }
        notified
    }

    pub(crate) fn dispatch_nested_events_to_animation_owners(
        parent_artboard: &mut ArtboardInstance,
        parent_host_local: usize,
        animations: &mut [crate::artboard::RuntimeNestedAnimationInstance],
        child: &mut ArtboardInstance,
        source_local_id: usize,
        events: &[StateMachineReportedEvent],
        mut nested_events: Option<&mut Vec<(usize, Vec<StateMachineReportedEvent>)>>,
        mut ancestor_dispatch: Option<
            &mut dyn FnMut(&mut ArtboardInstance, usize, &[StateMachineReportedEvent]) -> bool,
        >,
    ) -> bool {
        let mut changed = false;
        let mut deliver_to_ancestor =
            |artboard: &mut ArtboardInstance,
             source_local: usize,
             events: &[StateMachineReportedEvent]| {
                if let Some(dispatch) = ancestor_dispatch.as_mut() {
                    (**dispatch)(artboard, source_local, events)
                } else if let Some(nested_events) = nested_events.as_mut() {
                    nested_events.push((source_local, events.to_vec()));
                    false
                } else {
                    false
                }
            };
        for animation in animations {
            let crate::artboard::RuntimeNestedAnimationInstance::StateMachine(occurrence) =
                animation
            else {
                continue;
            };
            let notifier_local = occurrence.local_id();
            let (accepts_source, bubbled_events, pending_error) = {
                let state_machine = parent_artboard
                    .active_nested_state_machines
                    .get_mut(&notifier_local)
                    .or_else(|| occurrence.state_machine_mut());
                let Some(state_machine) = state_machine else {
                    continue;
                };
                let accepts_source = !events.is_empty()
                    && state_machine.script_error.is_none()
                    && state_machine.nested_event_source_registered(source_local_id);
                let notification = state_machine.try_notify_events_with_script_host(
                    child,
                    Some(source_local_id),
                    events,
                    &mut NoopScriptHost,
                );
                let (notified, pending_error) = match notification {
                    Ok(notified) => (notified, None),
                    Err(error) => (false, Some(error)),
                };
                changed |= notified;
                (
                    accepts_source,
                    state_machine.drain_bubbled_event_reports(),
                    pending_error,
                )
            };
            if !bubbled_events.is_empty() {
                changed |= Self::complete_nested_report_batch(
                    parent_artboard,
                    parent_host_local,
                    notifier_local,
                    &bubbled_events,
                    &mut deliver_to_ancestor,
                );
                if let Some(state_machine) = parent_artboard
                    .active_nested_state_machines
                    .get_mut(&notifier_local)
                    .or_else(|| occurrence.state_machine_mut())
                {
                    for event in &bubbled_events {
                        state_machine.flush_deferred_owner_audio_event(event);
                        #[cfg(any(test, feature = "tools"))]
                        record_runtime_nested_event_chain_step(
                            notifier_local,
                            RuntimeNestedEventChainPhase::AudioUnwind,
                        );
                    }
                }
            }
            if let Some(error) = pending_error
                && let Some(state_machine) = parent_artboard
                    .active_nested_state_machines
                    .get_mut(&notifier_local)
                    .or_else(|| occurrence.state_machine_mut())
            {
                let _: bool = state_machine.retain_script_result(Err(error));
            }
            if accepts_source
                && let Some(state_machine) = parent_artboard
                    .active_nested_state_machines
                    .get_mut(&notifier_local)
                    .or_else(|| occurrence.state_machine_mut())
            {
                let settlement =
                    state_machine.update_data_binds_false(child, None, &mut NoopScriptHost);
                state_machine.retain_script_result(settlement.map(|()| false));
            }
        }
        changed
    }

    pub(crate) fn dispatch_nested_event_sources_with(
        artboard: &mut ArtboardInstance,
        state_machine: &mut Self,
        advance: impl FnOnce(
            &mut ArtboardInstance,
            &mut dyn FnMut(&mut ArtboardInstance, usize, &[StateMachineReportedEvent]) -> bool,
        ) -> Result<(), ScriptError>,
    ) -> Result<bool, ScriptError> {
        let mut changed = false;
        let mut dispatch_source =
            |artboard: &mut ArtboardInstance,
             host_local: usize,
             events: &[StateMachineReportedEvent]| {
                let accepts_source = !events.is_empty()
                    && state_machine.script_error.is_none()
                    && state_machine.nested_event_source_registered(host_local);
                let source_notified =
                    state_machine.notify_events(artboard, Some(host_local), events);
                if accepts_source {
                    let settlement =
                        state_machine.update_data_binds_false(artboard, None, &mut NoopScriptHost);
                    state_machine.retain_script_result(settlement.map(|()| false));
                }
                changed |= source_notified;
                source_notified
            };
        advance(artboard, &mut dispatch_source)?;
        Ok(changed)
    }

    pub(crate) fn advance_artboard_frame_components(
        artboard: &mut ArtboardInstance,
        state_machines: &mut [Self],
        elapsed_seconds: f32,
    ) -> Result<bool, ScriptError> {
        Self::advance_artboard_frame_components_with(
            artboard,
            state_machines,
            elapsed_seconds,
            None,
            |artboard, elapsed_seconds, nested_event_dispatch| {
                artboard.advance_components_after_root_state_machines(
                    elapsed_seconds,
                    nested_event_dispatch,
                )
            },
        )
    }

    pub(crate) fn advance_artboard_frame_components_with_factory(
        artboard: &mut ArtboardInstance,
        state_machines: &mut [Self],
        elapsed_seconds: f32,
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        Self::advance_artboard_frame_components_with(
            artboard,
            state_machines,
            elapsed_seconds,
            None,
            |artboard, elapsed_seconds, nested_event_dispatch| {
                artboard.advance_components_after_root_state_machines_with_factory(
                    elapsed_seconds,
                    factory,
                    nested_event_dispatch,
                )
            },
        )
    }

    pub(crate) fn advance_artboard_frame_components_with(
        artboard: &mut ArtboardInstance,
        state_machines: &mut [Self],
        elapsed_seconds: f32,
        mut owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        mut advance_artboard: impl FnMut(
            &mut ArtboardInstance,
            f32,
            &mut dyn FnMut(&mut ArtboardInstance, usize, &[StateMachineReportedEvent]) -> bool,
        ) -> Result<bool, ScriptError>,
    ) -> Result<bool, ScriptError> {
        let mut changed = false;
        for state_machine in state_machines.iter_mut() {
            changed |= state_machine.advance_on_artboard(
                artboard,
                elapsed_seconds,
                true,
                owned_context.as_deref_mut(),
            );
            state_machine.drop_hidden_focus_target(artboard);
            changed |= state_machine.sync_text_input_focus(artboard);
        }

        let mut dispatch_nested_source =
            |artboard: &mut ArtboardInstance,
             host_local: usize,
             events: &[StateMachineReportedEvent]| {
                let mut callback_changed = false;
                for state_machine in state_machines.iter_mut() {
                    let accepts_source = !events.is_empty()
                        && state_machine.script_error.is_none()
                        && state_machine.nested_event_source_registered(host_local);
                    let notified = match owned_context.as_deref_mut() {
                        Some(context) => state_machine.notify_events_with_owned_view_model_context(
                            artboard,
                            Some(host_local),
                            events,
                            context,
                        ),
                        None => state_machine.notify_events(artboard, Some(host_local), events),
                    };
                    callback_changed |= notified;
                    if accepts_source {
                        let settlement = state_machine.update_data_binds_false(
                            artboard,
                            owned_context.as_deref(),
                            &mut NoopScriptHost,
                        );
                        state_machine.retain_script_result(settlement.map(|()| false));
                    }
                }
                callback_changed
            };
        changed |= advance_artboard(artboard, elapsed_seconds, &mut dispatch_nested_source)?;
        Ok(changed)
    }

    /// C++-corresponding one-argument `advanceAndApply(seconds)` form.
    ///
    /// The explicit Artboard is Rust's borrow-model adaptation. The method
    /// delegates exactly to the boolean form with ViewModel advancement
    /// enabled.
    pub fn advance_and_apply(
        &mut self,
        artboard: &mut ArtboardInstance,
        elapsed_seconds: f32,
    ) -> Result<bool, ScriptError> {
        self.advance_and_apply_with_view_models(artboard, elapsed_seconds, true)
    }

    /// C++-corresponding `advanceAndApply(seconds, advanceViewModels)` form.
    /// Elapsed seconds, including NaN, infinities, signed zero, and negatives,
    /// are forwarded unchanged through the raw machine and Artboard passes.
    pub fn advance_and_apply_with_view_models(
        &mut self,
        artboard: &mut ArtboardInstance,
        elapsed_seconds: f32,
        advance_view_models: bool,
    ) -> Result<bool, ScriptError> {
        let state_machines = std::slice::from_mut(self);
        Self::advance_and_apply_state_machines_with_view_models(
            artboard,
            state_machines,
            elapsed_seconds,
            advance_view_models,
            || false,
        )
    }

    /// Instance-owned multi-occurrence form used by the Rust facade.
    ///
    /// `advance_detached_view_models` is the borrow-model seam for C++'s
    /// Artboard-owned scripting VM. Rust's authenticated facade retains that
    /// registry on `File`, so the instance owner invokes the supplied
    /// operation at the exact C++ end-of-frame position.
    pub fn advance_and_apply_state_machines_with_view_models(
        artboard: &mut ArtboardInstance,
        state_machines: &mut [Self],
        elapsed_seconds: f32,
        advance_view_models: bool,
        advance_detached_view_models: impl FnOnce() -> bool,
    ) -> Result<bool, ScriptError> {
        let component_result =
            Self::advance_artboard_frame_components(artboard, state_machines, elapsed_seconds);
        let mut changed = component_result.as_ref().copied().unwrap_or(false);
        let settlement_result = if advance_view_models {
            artboard.settle_state_machine_update_passes_after_main_advance_with_script_errors(
                state_machines,
            )
        } else {
            artboard
                .settle_state_machine_update_passes_after_main_advance_without_root_view_model_reset_with_script_errors(
                    state_machines,
                )
        };
        changed |= settlement_result.as_ref().copied().unwrap_or(false);
        if advance_view_models {
            changed |= advance_detached_view_models();
        }
        if let Err(error) = component_result {
            return Err(error);
        }
        if let Err(error) = settlement_result {
            return Err(error);
        }
        Ok(Self::advance_and_apply_return(
            changed,
            elapsed_seconds,
            state_machines,
        ))
    }

    pub fn advance_and_apply_state_machines_with_factory_and_view_models(
        artboard: &mut ArtboardInstance,
        state_machines: &mut [Self],
        elapsed_seconds: f32,
        factory: &mut dyn RenderFactory,
        advance_view_models: bool,
        advance_detached_view_models: impl FnOnce() -> bool,
    ) -> Result<bool, ScriptError> {
        let component_result = Self::advance_artboard_frame_components_with_factory(
            artboard,
            state_machines,
            elapsed_seconds,
            factory,
        );
        let mut changed = component_result.as_ref().copied().unwrap_or(false);
        let settlement_result = if advance_view_models {
            artboard.settle_state_machine_update_passes_after_main_advance_with_factory(
                state_machines,
                factory,
            )
        } else {
            artboard
                .settle_state_machine_update_passes_after_main_advance_without_root_view_model_reset_with_script_errors(
                    state_machines,
                )
        };
        changed |= settlement_result.as_ref().copied().unwrap_or(false);
        if advance_view_models {
            changed |= advance_detached_view_models();
        }
        if let Err(error) = component_result {
            return Err(error);
        }
        if let Err(error) = settlement_result {
            return Err(error);
        }
        Ok(Self::advance_and_apply_return(
            changed,
            elapsed_seconds,
            state_machines,
        ))
    }

    /// Pinned facade return terms. Exact equality deliberately forces both
    /// signs of zero and does not classify or reject any other floating-point
    /// value.
    pub fn advance_and_apply_return(
        changed: bool,
        elapsed_seconds: f32,
        state_machines: &[Self],
    ) -> bool {
        changed
            || elapsed_seconds == 0.0
            || state_machines.iter().any(|instance| {
                instance.reported_event_count() != 0
                    || instance.has_pending_listener_view_model_reports()
            })
    }

    /// Instance-owned five-pass settlement policy for `advanceAndApply`.
    ///
    /// Artboard supplies only its update operation because factory/script
    /// dispatch is Artboard-owned. Transition probing, zero-time follow-up,
    /// optional ViewModel advancement, reset ordering, and the five-pass cap
    /// remain together here.
    pub(crate) fn settle_artboard_update_passes(
        artboard: &mut ArtboardInstance,
        state_machines: &mut [Self],
        reset_root_view_models: bool,
        mut update_pass: impl FnMut(&mut ArtboardInstance) -> bool,
    ) -> bool {
        const MAX_SETTLEMENT_PASSES: usize = 5;

        let mut changed = false;
        for _ in 0..MAX_SETTLEMENT_PASSES {
            changed |= update_pass(artboard);
            for state_machine in state_machines.iter_mut() {
                // Pinned C++ calls tryChangeState on every settlement pass.
                // There is deliberately no capability flag, pending bit, or
                // definition scan guarding this probe.
                if artboard.try_change_state_machine_instance(state_machine) {
                    changed = true;
                    changed |= artboard
                        .advance_state_machine_instance_after_state_probe(state_machine, 0.0);
                }
            }
            changed |= artboard.advance_outer_update_components_for_state_machine_settlement();
            if reset_root_view_models {
                for state_machine in state_machines.iter_mut() {
                    state_machine.reset_advanced_data_context();
                }
            }
            artboard.reset_retained_components_for_state_machine_settlement();
            if !artboard.has_dirt(ComponentDirt::COMPONENTS)
                && !state_machines
                    .iter()
                    .any(StateMachineInstance::has_pending_data_bind_work)
            {
                break;
            }
        }
        changed
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

    #[derive(Default)]
    struct ProfilerListenerCapture {
        tick: u64,
    }

    impl crate::ProfileCapture for ProfilerListenerCapture {
        fn tick(&mut self) -> u64 {
            let tick = self.tick;
            self.tick += 1;
            tick
        }

        fn metadata(&self) -> crate::ProfileCaptureMetadata {
            crate::ProfileCaptureMetadata::default()
        }

        fn current_frame_index(&self) -> u64 {
            0
        }

        fn gpu_frame_delay(&self) -> u64 {
            1
        }

        fn max_frame_history(&self) -> u64 {
            8
        }

        fn captured_frame(&self, _frame_index: u64) -> Option<crate::ProfileCaptureFrame> {
            None
        }
    }

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

    struct ReportingViewModelListenerScript {
        label: &'static str,
        queue: RuntimeCellNotificationQueue,
        listener_index: usize,
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
            _parents: Vec<Option<ScriptViewModel>>,
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

    #[derive(Debug, Clone, PartialEq)]
    struct RecordedDrawablePointerCall {
        method: ScriptMethod,
        pointer_id: i32,
        local_x: f32,
        local_y: f32,
    }

    struct RecordingDrawablePointerScript {
        hit: crate::ScriptedDrawablePointerHit,
        calls: Rc<RefCell<Vec<RecordedDrawablePointerCall>>>,
    }

    struct ResourceFailingDrawablePointerScript;

    impl ScriptInstance for ResourceFailingDrawablePointerScript {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::PointerDown)
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[crate::ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<crate::ScriptValue, ScriptError> {
            unreachable!("typed scripted-drawable pointer dispatch owns this callback")
        }

        fn call_scripted_drawable_pointer(
            &mut self,
            _method: ScriptMethod,
            _pointer_id: i32,
            _local_x: f32,
            _local_y: f32,
            _host: &mut dyn ScriptHost,
        ) -> Result<crate::ScriptedDrawablePointerResult, ScriptError> {
            Err(ScriptError::with_resource_code(
                "terminal pointer resource fence",
                "script.resource.pointer",
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

    impl ScriptInstance for RecordingDrawablePointerScript {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(matches!(
                method,
                ScriptMethod::PointerDown
                    | ScriptMethod::PointerMove
                    | ScriptMethod::PointerUp
                    | ScriptMethod::PointerExit
            ))
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[crate::ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<crate::ScriptValue, ScriptError> {
            unreachable!("typed scripted-drawable pointer dispatch owns this callback")
        }

        fn call_scripted_drawable_pointer(
            &mut self,
            method: ScriptMethod,
            pointer_id: i32,
            local_x: f32,
            local_y: f32,
            _host: &mut dyn ScriptHost,
        ) -> Result<crate::ScriptedDrawablePointerResult, ScriptError> {
            self.calls.borrow_mut().push(RecordedDrawablePointerCall {
                method,
                pointer_id,
                local_x,
                local_y,
            });
            Ok(crate::ScriptedDrawablePointerResult {
                invoked: true,
                hit: self.hit,
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

    impl ScriptInstance for ReportingViewModelListenerScript {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::PerformAction)
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
                state_before_call: 0,
            });
            self.queue.report_data_bind(self.listener_index);
            Ok(())
        }

        fn call_preferred_listener_action(
            &mut self,
            invocation: &ScriptListenerInvocation,
            host: &mut dyn ScriptHost,
        ) -> Result<bool, ScriptError> {
            self.call_listener_action(ScriptListenerActionMethod::PerformAction, invocation, host)?;
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
        let machine = artboard
            .state_machine_instance(0)
            .expect("fixture state machine");
        (artboard, machine)
    }

    fn scripted_listener_machine() -> StateMachineInstance {
        scripted_listener_artboard_and_machine().1
    }

    #[test]
    fn audio_event_seam_plays_the_resolved_sound_fixture_asset() {
        let fixture = PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets/sound.riv");
        let file = read_runtime_file(&std::fs::read(fixture).expect("read sound fixture"))
            .expect("import sound fixture");
        let graph = GraphFile::from_runtime_file(&file).expect("build sound graph");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("sound artboard"),
            &graph.artboards,
        )
        .expect("instantiate sound artboard");
        let owners = crate::RuntimeFileAssetOwners::from_runtime(&file, None);
        artboard.attach_runtime_file_asset_owners(&owners);
        let engine = crate::AudioEngine::new(2, 44_100).expect("headless audio engine");
        artboard.set_audio_engine(Some(engine.clone()));
        let event_local_id = artboard
            .components()
            .iter()
            .find(|component| component.type_name == "AudioEvent")
            .expect("sound AudioEvent")
            .local_id;
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("sound state machine");
        let (event, _) = fl_c5_test_audio_event(event_local_id);

        assert_eq!(engine.playing_sound_count(), 0);
        machine.notify_events(&mut artboard, None, &[event]);
        assert_eq!(engine.playing_sound_count(), 1);
    }

    #[derive(Debug, Clone)]
    struct RecordingHitComponent {
        label: &'static str,
        result: HitResult,
        trace: Rc<RefCell<Vec<String>>>,
        component: Option<ComponentHandle>,
    }

    impl HitComponent for RecordingHitComponent {
        fn clone_box(&self) -> Box<dyn HitComponent> {
            Box::new(self.clone())
        }

        fn component(&self) -> Option<ComponentHandle> {
            self.component
        }

        fn prepare_event(
            &mut self,
            _artboard: &ArtboardInstance,
            _groups: &mut [ListenerGroup],
            _position: (f32, f32),
            _hit_type: RuntimeListenerType,
            _pointer_id: i32,
        ) {
            self.trace
                .borrow_mut()
                .push(format!("prepare:{}", self.label));
        }

        fn process_event(
            &mut self,
            _instance: &mut StateMachineInstance,
            _artboard: &mut ArtboardInstance,
            _groups: &mut [ListenerGroup],
            _position: (f32, f32),
            hit_type: RuntimeListenerType,
            can_hit: bool,
            timestamp_seconds: f32,
            _pointer_id: i32,
            _owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
            _event_context: Option<&StateMachineEventContext>,
            _host: &mut dyn ScriptHost,
        ) -> Result<HitResult, ScriptError> {
            self.trace.borrow_mut().push(format!(
                "process:{}:{can_hit}:{hit_type:?}:{timestamp_seconds:?}",
                self.label
            ));
            Ok(self.result)
        }

        fn hit_test(
            &self,
            _instance: &StateMachineInstance,
            _artboard: &ArtboardInstance,
            _position: (f32, f32),
        ) -> bool {
            self.result.is_hit()
        }

        fn enable_pointer_events(&mut self, _groups: &mut [ListenerGroup], pointer_id: i32) {
            self.trace
                .borrow_mut()
                .push(format!("enable:{}:{pointer_id}", self.label));
        }

        fn disable_pointer_events(&mut self, _groups: &mut [ListenerGroup], pointer_id: i32) {
            self.trace
                .borrow_mut()
                .push(format!("disable:{}:{pointer_id}", self.label));
        }
    }

    #[test]
    fn fl_c5_hit_result_is_tristate_and_aggregates_strongest() {
        assert!(!HitResult::None.is_hit());
        assert!(HitResult::Hit.is_hit());
        assert_eq!(HitResult::None.strongest(HitResult::Hit), HitResult::Hit);
        assert_eq!(
            HitResult::Hit.strongest(HitResult::HitOpaque),
            HitResult::HitOpaque
        );
        assert_eq!(
            HitResult::HitOpaque.strongest(HitResult::None),
            HitResult::HitOpaque
        );
    }

    #[test]
    fn fl_c5_hit_three_passes_continue_after_opaque_with_can_hit_false() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let trace = Rc::new(RefCell::new(Vec::new()));
        machine.listener_groups.clear();
        machine.hit_components = vec![
            Box::new(RecordingHitComponent {
                label: "front",
                result: HitResult::HitOpaque,
                trace: Rc::clone(&trace),
                component: None,
            }),
            Box::new(RecordingHitComponent {
                label: "back",
                result: HitResult::Hit,
                trace: Rc::clone(&trace),
                component: None,
            }),
        ];

        let result = machine
            .update_listeners(
                &mut artboard,
                RuntimeListenerType::Move,
                7.0,
                9.0,
                3,
                -2.5,
                None,
                None,
                &mut NoopScriptHost,
            )
            .expect("hit passes");

        assert_eq!(result, HitResult::HitOpaque);
        assert_eq!(
            trace.borrow().as_slice(),
            [
                "prepare:front",
                "prepare:back",
                "process:front:true:Move:-2.5",
                "process:back:false:Move:-2.5",
            ]
        );
    }

    #[test]
    fn fl_c5_hit_sort_preserves_the_exact_adversarial_swap_order() {
        let (artboard, mut machine) = scripted_listener_artboard_and_machine();
        let artboard_component = artboard
            .components()
            .iter()
            .find(|component| component.type_name == "Artboard")
            .and_then(|component| artboard.component_handle(component.local_id))
            .expect("fixture root artboard component");
        let drawables = artboard
            .runtime_hit_component_order()
            .into_iter()
            .filter(|component| *component != artboard_component)
            .take(3)
            .collect::<Vec<_>>();
        assert_eq!(
            drawables.len(),
            3,
            "the adversarial fixture needs three distinct draw-order identities"
        );
        let trace = Rc::new(RefCell::new(Vec::new()));
        let hit = |label, component| {
            Box::new(RecordingHitComponent {
                label,
                result: HitResult::None,
                trace: Rc::clone(&trace),
                component: Some(component),
            }) as Box<dyn HitComponent>
        };
        machine.hit_components = vec![
            hit("third", drawables[2]),
            hit("root", artboard_component),
            hit("first-a", drawables[0]),
            hit("first-b", drawables[0]),
            hit("second", drawables[1]),
        ];

        machine.sort_hit_components(&artboard);

        assert_eq!(
            machine
                .hit_components
                .iter()
                .map(|hit| hit.component())
                .collect::<Vec<_>>(),
            [
                Some(artboard_component),
                Some(drawables[0]),
                Some(drawables[0]),
                Some(drawables[1]),
                Some(drawables[2]),
            ],
            "the in-place scan must continue after each swap so duplicate identities retain the pinned swap sequence"
        );
    }

    #[test]
    fn fl_c5_pointer_drag_discards_event_timestamps_then_follows_with_move() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let trace = Rc::new(RefCell::new(Vec::new()));
        machine.listener_groups.clear();
        machine.hit_components = vec![Box::new(RecordingHitComponent {
            label: "drag",
            result: HitResult::None,
            trace: Rc::clone(&trace),
            component: None,
        })];

        assert!(!machine.drag_start(&mut artboard, 4.0, 5.0, 9.5, 11));
        assert!(!machine.drag_end(&mut artboard, 6.0, 7.0, -3.25, 11));

        assert_eq!(
            trace.borrow().as_slice(),
            [
                "disable:drag:11",
                "prepare:drag",
                "process:drag:true:DragStart:0.0",
                "enable:drag:11",
                "prepare:drag",
                "process:drag:true:DragEnd:0.0",
                "prepare:drag",
                "process:drag:true:Move:-3.25",
            ]
        );
    }

    #[test]
    fn fl_c5_hit_click_only_duplicate_groups_require_down_and_up() {
        let (artboard, _) = scripted_listener_artboard_and_machine();
        let target = artboard
            .components()
            .iter()
            .find_map(|component| artboard.component_handle(component.local_id))
            .expect("component");
        let listener = RuntimeStateMachineListener {
            name: None,
            target_local_id: artboard.component_at(target).local_id,
            is_single: false,
            listener_types: vec![RuntimeListenerType::Click],
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: Vec::new(),
        };
        let groups = vec![ListenerGroup::authored(0), ListenerGroup::authored(0)];
        let listeners = vec![listener];
        let mut hit = HitDrawable::new(&artboard, Some(target), Some(target), false);

        assert!(hit.add_listener_impl(0, &groups, &listeners));
        assert!(hit.add_listener_impl(1, &groups, &listeners));
        assert_eq!(hit.listeners, [0, 1]);
        assert!(hit.needs_down_listener);
        assert!(hit.needs_up_listener);
        assert!(hit.can_early_out);
    }

    #[test]
    fn fl_c5_pointer_exit_releases_group_history_and_drag_state() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        for group in &mut machine.listener_groups {
            group.reset(-7);
            group.hover(-7);
            group.process(-7, (1.0, 2.0), true, true, false);
            group.record_position(-7, (1.0, 2.0));
        }
        for group in &mut machine.listener_groups {
            group.mark_dragged();
            group.disable(-7);
        }

        let _ = machine.pointer_exit(&mut artboard, 0.0, -0.0, -7);

        assert!(
            machine
                .listener_groups
                .iter()
                .all(|group| group.previous_position(-7).is_none())
        );
        assert!(
            machine
                .listener_groups
                .iter()
                .all(|group| !group.disabled(-7))
        );
    }

    #[test]
    fn fl_c5_pointer_cpp_paths_accept_nonfinite_coordinates_and_timestamps() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        for (x, y, timestamp) in [
            (f32::NAN, f32::INFINITY, f32::NEG_INFINITY),
            (f32::NEG_INFINITY, -0.0, f32::NAN),
            (0.0, f32::NAN, -42.0),
        ] {
            machine
                .update_listeners(
                    &mut artboard,
                    RuntimeListenerType::Move,
                    x,
                    y,
                    77,
                    timestamp,
                    None,
                    None,
                    &mut NoopScriptHost,
                )
                .expect("C++-corresponding path forwards every f32 value");
        }
        let group_index = machine
            .listener_groups
            .iter()
            .position(|group| matches!(group.kind, ListenerGroupKind::Authored { .. }))
            .expect("fixture authored listener group");
        let mut group = machine.listener_groups.remove(group_index);
        group.reset(77);
        group.hover(77);
        machine
            .process_listener_group_event(
                &mut group,
                &mut artboard,
                (0.0, f32::NAN),
                RuntimeListenerType::Move,
                true,
                -42.0,
                77,
                None,
                None,
                &mut NoopScriptHost,
            )
            .expect("StateMachine-to-ListenerGroup integration retains non-finite values");
        let position = group.previous_position(77).expect("group pointer history");
        assert_eq!(position.0.to_bits(), 0.0_f32.to_bits());
        assert!(position.1.is_nan());
        machine.listener_groups.insert(group_index, group);
    }

    #[test]
    fn fl_c5_constructor_order_phase_trace_and_explicit_fields() {
        let (artboard, machine) = scripted_listener_artboard_and_machine();
        assert_eq!(
            machine.constructor_phases,
            [
                RuntimeConstructorPhase::Inputs,
                RuntimeConstructorPhase::LayersAnyEntry,
                RuntimeConstructorPhase::MachineBinds,
                RuntimeConstructorPhase::AuthoredListenerCategories,
                RuntimeConstructorPhase::ComponentProvidedGroups,
                RuntimeConstructorPhase::NestedListTextHits,
                RuntimeConstructorPhase::ScriptedClonesAndFacilities,
                RuntimeConstructorPhase::HitSort,
                RuntimeConstructorPhase::FocusTree,
            ],
            "constructor boundaries follow state_machine_instance.cpp:1711-2127"
        );
        assert_eq!(machine.layer_count, machine.layers.len());
        assert_eq!(
            machine.layer_count,
            artboard.state_machine(0).unwrap().layers.len()
        );
        assert_eq!(machine.draw_order_change_counter, 0);
        assert!(!machine.disposed);
        assert_eq!(machine.has_listeners(), machine.hit_components_count() != 0);
        assert_eq!(
            machine
                .hit_component(machine.hit_components_count())
                .map(HitComponent::component),
            None
        );
    }

    #[test]
    fn fl_c5_constructor_order_retains_unresolved_pointer_group_occurrence() {
        let (mut artboard, _) = scripted_listener_artboard_and_machine();
        let mut definition = reset_input_state_machine(reset_input_actions());
        definition.listeners = Arc::new(vec![RuntimeStateMachineListener {
            name: None,
            target_local_id: usize::MAX,
            is_single: false,
            listener_types: vec![RuntimeListenerType::Down],
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: Vec::new(),
        }]);
        artboard.state_machines = Arc::new(vec![definition]);
        let machine = artboard
            .state_machine_instance(0)
            .expect("state machine with unresolved pointer target");

        assert_eq!(
            machine
                .input(0)
                .and_then(StateMachineInputInstance::bool_value),
            Some(true),
            "entry actions execute during the layer phase"
        );
        assert_eq!(machine.listener_groups.len(), 1);
        assert!(
            machine
                .hit_components
                .iter()
                .all(|owner| owner.component().is_some()),
            "an unresolved target retains its group but creates no hit owner"
        );
    }

    #[test]
    fn fl_c5_hit_component_identity_reuses_owner_but_retains_duplicate_groups() {
        let (mut artboard, _) = scripted_listener_artboard_and_machine();
        let target_local_id = artboard
            .components()
            .iter()
            .find(|component| {
                component.type_name == "Shape"
                    || component.type_name == "TextValueRun"
                    || nuxie_schema::definition_by_name(component.type_name)
                        .is_some_and(|definition| definition.is_a("LayoutComponent"))
            })
            .expect("fixture pointer target")
            .local_id;
        let listener = RuntimeStateMachineListener {
            name: None,
            target_local_id,
            is_single: false,
            listener_types: vec![RuntimeListenerType::Down],
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: Vec::new(),
        };
        let mut definition = reset_input_state_machine(reset_input_actions());
        definition.listeners = Arc::new(vec![listener.clone(), listener]);
        definition.transition_duration_bindings = Arc::new(vec![
            RuntimeTransitionDurationBinding {
                data_bind_index: 3,
                transition_global_id: 44,
            },
            RuntimeTransitionDurationBinding {
                data_bind_index: 7,
                transition_global_id: 44,
            },
        ]);
        artboard.state_machines = Arc::new(vec![definition]);

        let machine = artboard
            .state_machine_instance(0)
            .expect("state machine with duplicate pointer and bind occurrences");

        assert_eq!(machine.listener_groups.len(), 2);
        let target = artboard
            .component_handle(target_local_id)
            .expect("pointer target handle");
        assert_eq!(
            machine
                .hit_components
                .iter()
                .filter(|owner| owner.component() == Some(target))
                .count(),
            1,
            "duplicate groups share one component-identity hit owner"
        );
        assert_eq!(
            machine
                .transition_durations
                .iter()
                .map(|occurrence| occurrence.transition_global_id)
                .collect::<Vec<_>>(),
            [44, 44],
            "duplicate transition-property binds retain distinct authored occurrences"
        );
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

    #[test]
    fn typed_named_inputs_match_type_and_name_in_authored_order() {
        let (_, mut artboard) = fl_c5_bind_file_and_artboard();
        let mut definition = reset_input_state_machine(Vec::new());
        definition.inputs = Arc::new(vec![
            Some(RuntimeStateMachineInput::new_number(
                901,
                Some("x".to_owned()),
                7.0,
            )),
            Some(RuntimeStateMachineInput::new_bool(
                902,
                Some("x".to_owned()),
                true,
            )),
            Some(RuntimeStateMachineInput::new_trigger(
                903,
                Some("x".to_owned()),
            )),
        ]);
        artboard.state_machines = Arc::new(vec![definition]);
        let machine = artboard
            .state_machine_instance(0)
            .expect("typed named-input machine");

        assert_eq!(
            machine
                .input_named("x")
                .and_then(|input| input.number_value()),
            Some(7.0),
            "the untyped Rust convenience keeps first-name semantics"
        );
        assert_eq!(
            machine.get_bool("x").and_then(|input| input.bool_value()),
            Some(true),
            "getBool skips the earlier same-name Number occurrence"
        );
        assert_eq!(
            machine
                .get_number("x")
                .and_then(|input| input.number_value()),
            Some(7.0)
        );
        assert_eq!(
            machine
                .get_trigger("x")
                .and_then(|input| input.trigger_fired()),
            Some(false)
        );
        assert!(machine.get_bool("missing").is_none());
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

    fn fl_c5_state_transition(
        global_id: u32,
        state_to_index: usize,
        conditions: Vec<RuntimeTransitionCondition>,
    ) -> RuntimeStateTransition {
        RuntimeStateTransition {
            global_id,
            state_to_index: Some(state_to_index),
            exit_blend_animation_index: None,
            duration: 0,
            exit_time: 0,
            flags: 0,
            random_weight: 0,
            direct_input_conditions_only: conditions
                .iter()
                .all(RuntimeTransitionCondition::is_direct_input),
            conditions,
            fire_actions: Vec::new(),
            listener_actions: Vec::new(),
            interpolator: None,
            has_unsupported_interpolator: false,
        }
    }

    fn fl_c5_state(
        global_id: u32,
        type_name: &'static str,
        animation: bool,
        transitions: Vec<RuntimeStateTransition>,
    ) -> RuntimeLayerState {
        RuntimeLayerState {
            global_id: Some(global_id),
            type_name: Some(type_name),
            animation: animation.then(RuntimeLinearAnimationHandle::empty),
            blend_state_1d: None,
            blend_state_direct: None,
            speed: 1.0,
            flags: 0,
            fire_actions: Vec::new(),
            listener_actions: Vec::new(),
            transitions,
        }
    }

    fn fl_c5_state_query_machine() -> RuntimeStateMachine {
        let enabled = || {
            RuntimeTransitionCondition::Bool(RuntimeTransitionBoolCondition::new(
                0,
                TransitionConditionOp::Equal,
            ))
        };
        let changing_layer = |layer_global_id, state_global_id| RuntimeStateMachineLayer {
            global_id: layer_global_id,
            name: None,
            states: vec![
                fl_c5_state(
                    state_global_id,
                    "EntryState",
                    false,
                    vec![fl_c5_state_transition(state_global_id + 1, 1, Vec::new())],
                ),
                fl_c5_state(
                    state_global_id + 2,
                    "AnimationState",
                    true,
                    vec![fl_c5_state_transition(
                        state_global_id + 3,
                        2,
                        vec![enabled()],
                    )],
                ),
                fl_c5_state(
                    state_global_id + 4,
                    "AnimationState",
                    true,
                    vec![fl_c5_state_transition(state_global_id + 5, 3, Vec::new())],
                ),
                fl_c5_state(state_global_id + 6, "AnimationState", true, Vec::new()),
            ],
            entry_state_index: Some(0),
            any_state_index: None,
            exit_state_index: None,
        };
        let inert_layer = RuntimeStateMachineLayer {
            global_id: 920,
            name: None,
            states: vec![
                fl_c5_state(
                    921,
                    "EntryState",
                    false,
                    vec![fl_c5_state_transition(922, 1, Vec::new())],
                ),
                fl_c5_state(923, "ExitState", false, Vec::new()),
            ],
            entry_state_index: Some(0),
            any_state_index: None,
            exit_state_index: Some(1),
        };
        let mut machine = reset_input_state_machine(Vec::new());
        machine.layers = Arc::new(vec![
            changing_layer(910, 1_000),
            inert_layer,
            changing_layer(930, 2_000),
        ]);
        machine
    }

    fn fl_c5_advance_fixture() -> (ArtboardInstance, StateMachineInstance) {
        let (mut artboard, _) = scripted_listener_artboard_and_machine();
        artboard.state_machines = Arc::new(vec![reset_input_state_machine(Vec::new())]);
        let machine = artboard
            .state_machine_instance(0)
            .expect("WP7 advance fixture state machine");
        (artboard, machine)
    }

    #[test]
    fn fl_c5_advance_raw_order_and_clean_zero_bookkeeping() {
        let (mut artboard, mut machine) = fl_c5_advance_fixture();

        let _ = artboard.advance_state_machine_instance(&mut machine, 0.25);
        assert!(machine.fire_trigger(2));
        let _ = artboard.advance_state_machine_instance(&mut machine, -0.0);

        assert_eq!(
            machine.advance_phase_trace,
            [
                "draw-sort-check",
                "focus-snapshot",
                "semantic-snapshot",
                "apply-events",
                "clear-latch",
                "pre-layer-binds",
                "authored-layers",
                "converter-advance",
                "inputs-advanced",
            ],
            "raw order matches state_machine_instance.cpp:2546-2585"
        );
        assert_eq!(
            machine.input(2).and_then(|input| input.trigger_fired()),
            Some(false),
            "clean signed-zero advances still run every input advanced()"
        );
    }

    #[test]
    fn fl_c5_advance_new_frame_false_preserves_the_sticky_latch() {
        let (mut artboard, mut machine) = fl_c5_advance_fixture();
        let _ = artboard.advance_state_machine_instance(&mut machine, 0.25);
        assert!(machine.set_bool(0, true));

        let definitions = artboard.state_machine_definition_owner(&machine);
        let definition = definitions.first().expect("advance definition");
        assert!(machine.advance(&mut artboard, definition, 0.0, false, None));
        assert!(machine.needs_advance());
        assert!(
            !machine.advance_phase_trace.contains(&"clear-latch"),
            "newFrame=false must never clear m_needsAdvance"
        );
    }

    #[test]
    fn fl_c5_advance_fp_values_forward_without_validation_and_zero_forces_facade() {
        let (mut artboard, mut machine) = fl_c5_advance_fixture();
        let definitions = artboard.state_machine_definition_owner(&machine);
        let definition = definitions.first().expect("advance definition");

        for seconds in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            -17.25,
            0.0,
            -0.0,
        ] {
            let _ = machine.advance(&mut artboard, definition, seconds, true, None);
            assert_eq!(
                machine.advance_phase_trace.last(),
                Some(&"inputs-advanced"),
                "every f32 value reaches the end of raw bookkeeping"
            );
        }

        assert!(StateMachineInstance::advance_and_apply_return(
            false,
            0.0,
            std::slice::from_ref(&machine),
        ));
        assert!(StateMachineInstance::advance_and_apply_return(
            false,
            -0.0,
            std::slice::from_ref(&machine),
        ));
        assert!(!StateMachineInstance::advance_and_apply_return(
            false,
            f32::NAN,
            &[],
        ));
        assert!(!StateMachineInstance::advance_and_apply_return(
            false,
            -1.0,
            &[],
        ));

        machine.reported_events.push(fl_c5_test_reported_event(7));
        assert!(
            StateMachineInstance::advance_and_apply_return(
                false,
                0.25,
                std::slice::from_ref(&machine),
            ),
            "a pending event keeps the facade going"
        );
        machine.reported_events.clear();
        machine.reported_listener_view_models.report_data_bind(0);
        assert!(
            StateMachineInstance::advance_and_apply_return(
                false,
                0.25,
                std::slice::from_ref(&machine),
            ),
            "a pending listener ViewModel keeps the facade going"
        );
    }

    #[test]
    fn fl_c5_advance_bind_generated_report_is_a_raw_return_term() {
        let (mut artboard, mut machine) = fl_c5_advance_fixture();
        machine.bind_advance_test_report = Some(fl_c5_test_reported_event(17));

        assert!(
            artboard.advance_state_machine_instance(&mut machine, 0.25),
            "a report created during converter/bind advance is a raw return term"
        );
        assert_eq!(
            machine.reported_event_count(),
            1,
            "the bind-generated report remains pending for the next applyEvents snapshot"
        );
    }

    #[test]
    fn fl_c5_advance_and_apply_persistent_dirt_component_stops_after_five_passes() {
        let (mut artboard, mut machine) = fl_c5_advance_fixture();
        artboard.install_persistent_dirt_component_fixture();
        let probe_count = machine.transition_probe_count;

        let advanced = machine
            .advance_and_apply(&mut artboard, 0.25)
            .expect("public advance_and_apply facade");
        let (advance_count, update_count, dirt_remaining) =
            artboard.persistent_dirt_component_fixture_receipt();

        assert_eq!(
            machine.transition_probe_count - probe_count,
            5,
            "persistent sixth-pass dirt is capped after five unconditional probes"
        );
        assert_eq!(
            machine.data_context_advance_call_count, 5,
            "ViewModels advance once per settlement iteration before the Artboard reset"
        );
        assert_eq!(
            (advanced, advance_count, update_count, dirt_remaining),
            (true, 6, 5, true),
            "one main component advance plus five settlement advances leaves sixth-pass dirt pending"
        );
        println!(
            "FL_C5_PERSISTENT_DIRT_RECEIPT advanced={advanced} \
             advance_count={advance_count} update_count={update_count} \
             dirt_remaining={dirt_remaining}"
        );
    }

    #[test]
    fn fl_c5_advance_view_models_false_skips_only_data_context_advancement() {
        let (mut artboard, mut machine) = fl_c5_advance_fixture();
        StateMachineInstance::settle_artboard_update_passes(
            &mut artboard,
            std::slice::from_mut(&mut machine),
            false,
            |artboard| artboard.update_pass(),
        );
        assert_eq!(machine.data_context_advance_call_count, 0);

        StateMachineInstance::settle_artboard_update_passes(
            &mut artboard,
            std::slice::from_mut(&mut machine),
            true,
            |artboard| artboard.update_pass(),
        );
        assert_eq!(machine.data_context_advance_call_count, 1);

        let detached_advance_calls = std::cell::Cell::new(0);
        let (mut artboard, mut machine) = fl_c5_advance_fixture();
        StateMachineInstance::advance_and_apply_state_machines_with_view_models(
            &mut artboard,
            std::slice::from_mut(&mut machine),
            0.25,
            false,
            || {
                detached_advance_calls.set(detached_advance_calls.get() + 1);
                true
            },
        )
        .expect("advanceViewModels=false facade");
        assert_eq!(
            detached_advance_calls.get(),
            0,
            "advanceViewModels=false skips detached scripted ViewModels"
        );

        let (mut artboard, mut machine) = fl_c5_advance_fixture();
        StateMachineInstance::advance_and_apply_state_machines_with_view_models(
            &mut artboard,
            std::slice::from_mut(&mut machine),
            0.25,
            true,
            || {
                detached_advance_calls.set(detached_advance_calls.get() + 1);
                true
            },
        )
        .expect("advanceViewModels=true facade");
        assert_eq!(
            detached_advance_calls.get(),
            1,
            "advanceViewModels=true advances detached scripted ViewModels exactly once"
        );
    }

    #[test]
    fn fl_c5_advance_focus_chaining_and_hidden_target_boundaries() {
        let (mut artboard, mut machine, _) =
            scripted_drawable_subtype_input_artboard_and_machine_with_optional_script(
                "ScriptedDrawable",
                None,
                true,
            );
        machine.focus.take_owner_events();
        machine.queued_focus_events.clear();
        machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
            name: None,
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
            listener_actions: vec![RuntimeScheduledListenerAction::FocusClear(
                RuntimeFocusActionClear::for_test(0),
            )],
        }]);
        machine.focus_listener_groups = vec![
            RuntimeFocusListenerGroup::new(0, 2, &machine.listener_definitions[0])
                .expect("advance focus group"),
        ];
        assert!(machine.focus.clear_focus());
        machine.capture_focus_callbacks();
        machine.queued_focus_events.clear();
        assert!(machine.focus.set_focus_target(1));
        machine.capture_focus_callbacks();

        assert!(
            !artboard.advance_state_machine_instance(&mut machine, 0.25),
            "C++ clears the continuation set by focus generated during the active focus snapshot"
        );
        assert_eq!(
            machine.queued_focus_events,
            [RuntimeQueuedFocusEvent {
                listener_index: 0,
                is_focus: false,
            }],
            "the chained blur callback remains queued despite the pinned lost-latch edge"
        );

        let opacity_key = crate::properties::property_key_for_name("Node", "opacity")
            .expect("Node.opacity property key");
        assert!(artboard.set_double_property(1, opacity_key, 0.0));
        artboard.update_components();
        let _ = machine
            .advance_and_apply(&mut artboard, 0.25)
            .expect("hidden-focus facade advance");
        assert!(
            machine.focus.focused_listener_chain().is_empty(),
            "the facade drops a focus target made ineligible by retained Artboard state"
        );
    }

    #[test]
    fn fl_c5_state_changed_queries_retain_same_frame_flags_in_authored_layer_order() {
        let (mut artboard, _) = scripted_listener_artboard_and_machine();
        artboard.state_machines = Arc::new(vec![fl_c5_state_query_machine()]);
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("state query machine");

        assert!(artboard.advance_state_machine_instance(&mut machine, 0.0));
        assert_eq!(machine.changed_state_count(), 3);
        assert_eq!(
            machine.changed_state(1).and_then(|state| state.global_id),
            Some(923),
            "initial Entry convergence includes the authored non-animation layer"
        );

        assert!(machine.set_bool(0, true));
        assert!(
            artboard.settle_state_machine_update_passes_with_state_machines(std::slice::from_mut(
                &mut machine
            ),)
        );
        assert_eq!(
            machine.changed_state_count(),
            2,
            "several transitions in one layer still count one changed layer"
        );
        assert_eq!(
            machine.changed_state(0).and_then(|state| state.global_id),
            Some(1_006)
        );
        assert_eq!(
            machine.changed_state(1).and_then(|state| state.global_id),
            Some(2_006),
            "the compressed index skips unchanged authored layer 1"
        );
        assert!(machine.changed_state(2).is_none());
        assert_eq!(
            machine.layer_state(0).and_then(|state| state.global_id),
            Some(1_006)
        );
        assert_eq!(
            machine.layer_state(1).and_then(|state| state.global_id),
            Some(923)
        );
        assert_eq!(
            machine.layer_state(2).and_then(|state| state.global_id),
            Some(2_006)
        );
        assert!(machine.layer_state(3).is_none());

        assert!(
            !artboard.settle_state_machine_update_passes_with_state_machines(std::slice::from_mut(
                &mut machine
            ),)
        );
        assert_eq!(
            machine.changed_state_count(),
            0,
            "the next standalone new-frame settlement clears retained flags"
        );
    }

    #[test]
    fn fl_c5_state_changed_current_animation_queries_compress_the_same_authored_layers() {
        let (mut artboard, _) = scripted_listener_artboard_and_machine();
        artboard.state_machines = Arc::new(vec![fl_c5_state_query_machine()]);
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("state query machine");

        assert!(artboard.advance_state_machine_instance(&mut machine, 0.0));
        assert_eq!(machine.current_animation_count(), 2);
        assert!(machine.current_animation(0).is_some());
        assert!(machine.current_animation(1).is_some());
        assert!(machine.current_animation(2).is_none());
        assert_eq!(
            machine.layer_state(1).and_then(|state| state.global_id),
            Some(923),
            "the interleaved non-animation layer remains visible by raw layer index"
        );
    }

    #[test]
    fn fl_c5_state_changed_layer_state_handles_null_current_and_owner_length_disagreement() {
        let (mut artboard, _) = scripted_listener_artboard_and_machine();
        artboard.state_machines = Arc::new(vec![fl_c5_state_query_machine()]);
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("state query machine");

        machine.layers.truncate(2);
        assert!(
            machine.layer_state(2).is_none(),
            "a retained definition without an occurrence is safely absent"
        );

        let extra_layer = machine.layers[0].clone();
        machine.layers.push(extra_layer.clone());
        machine.layers.push(extra_layer);
        assert!(
            machine.layer_state(3).is_none(),
            "the retained machine definition bounds the testing query"
        );

        let null_layer = RuntimeStateMachineLayer {
            global_id: 940,
            name: None,
            states: Vec::new(),
            entry_state_index: None,
            any_state_index: None,
            exit_state_index: None,
        };
        let mut null_machine = reset_input_state_machine(Vec::new());
        null_machine.layers = Arc::new(vec![null_layer]);
        artboard.state_machines = Arc::new(vec![null_machine]);
        let null_instance = artboard
            .state_machine_instance(0)
            .expect("null-current state query machine");
        assert!(
            null_instance.layer_state(0).is_none(),
            "a layer occurrence with no current state projects null"
        );
        assert!(null_instance.layer_state(1).is_none());
    }

    #[test]
    fn fl_c5_state_changed_reset_during_active_transition_keeps_current_state_query_live() {
        let (mut artboard, _) = scripted_listener_artboard_and_machine();
        let mut definition = fl_c5_state_query_machine();
        Arc::make_mut(&mut definition.layers)[0].states[1].transitions[0].duration = 1_000;
        artboard.state_machines = Arc::new(vec![definition]);
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("state query machine");

        assert!(artboard.advance_state_machine_instance(&mut machine, 0.0));
        assert!(machine.set_bool(0, true));
        assert!(artboard.advance_state_machine_instance(&mut machine, 0.0));
        assert_eq!(machine.changed_state_count(), 2);

        machine.reset_state(&mut artboard);
        assert_eq!(
            machine.layer_state(0).and_then(|state| state.type_name),
            Some("EntryState")
        );
        assert_eq!(
            machine.changed_state(0).and_then(|state| state.type_name),
            Some("EntryState"),
            "reset replaces the current occurrence without clearing this frame's flag"
        );
        assert_eq!(
            machine.changed_state_count(),
            2,
            "reset during an active transition does not invent or erase changed layers"
        );
    }

    #[test]
    fn fl_c5_state_changed_random_weight_scratch_is_isolated_between_instances() {
        let (mut artboard, _) = scripted_listener_artboard_and_machine();
        let mut definition = reset_input_state_machine(Vec::new());
        let mut first = fl_c5_state_transition(3_003, 2, Vec::new());
        first.random_weight = 1;
        let mut second = fl_c5_state_transition(3_004, 3, Vec::new());
        second.random_weight = 3;
        definition.layers = Arc::new(vec![RuntimeStateMachineLayer {
            global_id: 3_000,
            name: None,
            states: vec![
                fl_c5_state(
                    3_001,
                    "EntryState",
                    false,
                    vec![fl_c5_state_transition(3_002, 1, Vec::new())],
                ),
                RuntimeLayerState {
                    flags: 1,
                    transitions: vec![first, second],
                    ..fl_c5_state(3_005, "AnimationState", true, Vec::new())
                },
                fl_c5_state(3_006, "AnimationState", true, Vec::new()),
                fl_c5_state(3_007, "AnimationState", true, Vec::new()),
            ],
            entry_state_index: Some(0),
            any_state_index: None,
            exit_state_index: None,
        }]);
        artboard.state_machines = Arc::new(vec![definition]);
        let mut first = artboard
            .state_machine_instance(0)
            .expect("first random state-machine instance");
        let mut second = artboard
            .state_machine_instance(0)
            .expect("second random state-machine instance");
        let _random_values = crate::set_runtime_random_test_values(&[0.0, 0.75]);

        assert!(artboard.advance_state_machine_instance(&mut first, 0.0));
        assert!(artboard.advance_state_machine_instance(&mut second, 0.0));
        assert_eq!(
            first.layer_state(0).and_then(|state| state.global_id),
            Some(3_006)
        );
        assert_eq!(
            second.layer_state(0).and_then(|state| state.global_id),
            Some(3_007)
        );
        let first_scratch = first.layers[0].evaluated_random_weights();
        let second_scratch = second.layers[0].evaluated_random_weights();
        assert_eq!(first_scratch, [1, 3]);
        assert_eq!(second_scratch, [1, 3]);
        assert_ne!(
            first_scratch.as_ptr(),
            second_scratch.as_ptr(),
            "shared definitions never own mutable evaluated-weight scratch"
        );
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
            name: None,
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

    #[test]
    fn scripted_drawable_pointer_hit_flows_through_the_state_machine_hit_aggregate() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let (mut artboard, mut machine, _) = scripted_drawable_input_artboard_and_machine(
            Box::new(RecordingDrawablePointerScript {
                hit: crate::ScriptedDrawablePointerHit::HitOpaque,
                calls: Rc::clone(&calls),
            }),
        );

        assert!(machine.pointer_down(&mut artboard, 11.0, 12.0, 7));
        assert_eq!(
            calls.borrow().as_slice(),
            [RecordedDrawablePointerCall {
                method: ScriptMethod::PointerDown,
                pointer_id: 7,
                local_x: 11.0,
                local_y: 12.0,
            }]
        );
    }

    #[test]
    fn scripted_drawable_pointer_resource_error_restores_hit_ownership_before_returning() {
        let (mut artboard, mut machine, _) = scripted_drawable_input_artboard_and_machine(
            Box::new(ResourceFailingDrawablePointerScript),
        );
        let hit_count = machine.hit_components.len();

        for pointer_id in [1, 2] {
            let error = machine
                .try_pointer_down_with_timestamp_and_script_host(
                    &mut artboard,
                    1.0,
                    2.0,
                    pointer_id,
                    0.0,
                    &mut NoopScriptHost,
                )
                .expect_err("resource-coded callback failure remains terminal");
            assert_eq!(error.resource_code(), Some("script.resource.pointer"));
            assert_eq!(machine.hit_components.len(), hit_count);
        }
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
            name: None,
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

        assert!(machine.key_input(&mut artboard, 259, 0, true, false));
        assert_eq!(artboard.text_input_display_text(1).as_deref(), Some("seed"));
        assert!(!machine.focus.focused_listener_chain().is_empty());
        assert!(!machine.key_input(&mut artboard, 66, 0, true, false));
        assert!(!machine.focus.focused_listener_chain().is_empty());
        assert!(machine.text_input(&mut artboard, "owned"));
        assert_eq!(
            artboard.text_input_display_text(1).as_deref(),
            Some("ownedseed")
        );
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
            name: None,
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

        machine.initialize_authored_listener_categories(&mut artboard);

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

        let pointer = RuntimePointerInput {
            x: 12.0,
            y: 34.0,
            previous_x: 12.0,
            previous_y: 34.0,
            timestamp_seconds: 0.0,
            id: 7,
        };
        assert!(
            !machine
                .dispatch_pointer_listener_type_for_target(
                    &mut artboard,
                    1,
                    pointer,
                    RuntimeListenerType::DragEnd,
                    None,
                    &mut NoopScriptHost,
                    None,
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
            name: None,
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
        machine.initialize_authored_listener_categories(&mut artboard);

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
        #[derive(Debug)]
        struct StubSemanticNodeResolver {
            calls: Rc<RefCell<Vec<u32>>>,
        }

        impl SemanticNodeResolver for StubSemanticNodeResolver {
            fn semantic_data_local_id(&self, semantic_node_id: u32) -> Option<usize> {
                self.calls.borrow_mut().push(semantic_node_id);
                (semantic_node_id == 77).then_some(2)
            }
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

        assert!(machine.enable_semantics());
        assert_eq!(
            machine.semantic_listener_groups[0].semantic_data_local_id,
            2
        );
        let semantic_id = machine
            .drain_semantics_diff(&mut artboard)
            .expect("production retained semantic tree drains")
            .added
            .into_iter()
            .find(|node| node.id != 0)
            .expect("retained SemanticData emits a node")
            .id;
        assert!(machine.fire_semantic_action(semantic_id, 1));
        assert!(
            machine.queued_semantic_events.is_empty(),
            "the retained listener applies its authored tap-only constraint"
        );
        assert!(machine.fire_semantic_action(semantic_id, 0));
        assert_eq!(machine.queued_semantic_events.len(), 1);
        machine.queued_semantic_events.clear();
        assert!(
            !machine.fire_semantic_action(77, 0),
            "W41's recorded-seam contract makes the production-default absent resolver a silent no-op"
        );
        let resolver_calls = Rc::new(RefCell::new(Vec::new()));
        machine.set_semantic_node_resolver(Some(Rc::new(StubSemanticNodeResolver {
            calls: Rc::clone(&resolver_calls),
        })));
        assert!(
            machine.fire_semantic_action(77, 1),
            "increase dispatch reaches SemanticData even though this listener accepts only tap"
        );
        assert!(
            machine.fire_semantic_action(77, 2),
            "decrease dispatch reaches the injected SemanticData resolver seam"
        );
        assert!(
            machine.queued_semantic_events.is_empty(),
            "SemanticData applies the listener's action constraint"
        );
        assert!(
            !machine.fire_semantic_action(77, 3),
            "an out-of-range action is a no-op after resolving a valid node"
        );
        assert!(
            !machine.semantic_action_for_target(1, 1),
            "a nonmatching action is not registered"
        );
        assert!(
            machine.fire_semantic_action(77, 0),
            "tap selects the SemanticData action and queues its listener callback"
        );
        assert_eq!(
            resolver_calls.borrow().as_slice(),
            [77, 77, 77, 77],
            "tap, increase, decrease, and invalid actions all reach the injected node resolver"
        );
        assert_eq!(
            machine.semantic_manager_phase_trace,
            [
                "create-internal-recorded-seam",
                "build-tree-recorded-seam",
                "node-by-id-recorded-seam",
                "semantic-data-recorded-seam",
                "fire-increase-recorded-data-seam",
                "node-by-id-recorded-seam",
                "semantic-data-recorded-seam",
                "fire-decrease-recorded-data-seam",
                "node-by-id-recorded-seam",
                "semantic-data-recorded-seam",
                "node-by-id-recorded-seam",
                "semantic-data-recorded-seam",
                "fire-tap-recorded-data-seam",
            ],
            "the family-owned action switch selects the recorded SemanticData callback"
        );
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

        assert!(machine.fire_semantic_action(77, 0));
        assert!(machine.fire_semantic_action(77, 0));
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
            name: None,
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
                RuntimeQueuedFocusEvent {
                    listener_index: 0,
                    is_focus: false,
                },
                RuntimeQueuedFocusEvent {
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
                RuntimeQueuedFocusEvent {
                    listener_index: 0,
                    is_focus: true,
                },
                RuntimeQueuedFocusEvent {
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
            name: None,
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
            [RuntimeQueuedFocusEvent {
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
            name: None,
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
            [RuntimeQueuedFocusEvent {
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
            name: None,
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
            name: None,
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
    fn fl_c5_event_host_drain_leaves_the_core_queue_for_apply_events() {
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
    fn fl_c5_event_apply_batches_chaining_and_exact_100_cap() {
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
                name: None,
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
            artboard.advance_state_machine_instance(&mut machine, 0.25),
            "events first reported inside applyEvents remain host visible after listener delivery"
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
        assert_eq!(
            machine
                .events_applied_during_loop
                .iter()
                .map(StateMachineReportedEvent::event_local_index)
                .collect::<Vec<_>>(),
            [2, 3]
        );
        assert_eq!(machine.reported_event_count(), 2);
        assert_eq!(machine.next_unapplied_reported_event_index(), 0);
        assert_eq!(
            machine
                .take_reported_events(&artboard)
                .iter()
                .map(StateMachineReportedEvent::event_local_index)
                .collect::<Vec<_>>(),
            [2, 3]
        );
        assert!(machine.take_reported_events(&artboard).is_empty());

        let mut finite_records = vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
        ];
        finite_records
            .extend((0..100).map(|_| record("Event", vec![uint("Event", "parentId", 0)])));
        finite_records.push(record("StateMachine", Vec::new()));
        let finite_file =
            RuntimeFile::from_authoring_records(finite_records).expect("finite chain imports");
        let finite_graph =
            GraphFile::from_runtime_file(&finite_file).expect("finite chain graph builds");
        let mut finite_artboard = ArtboardInstance::from_graph_with_artboards(
            &finite_file,
            finite_graph
                .artboards
                .first()
                .expect("finite chain artboard"),
            &finite_graph.artboards,
        )
        .expect("finite chain artboard instantiates");
        let mut finite_machine = finite_artboard
            .state_machine_instance(0)
            .expect("finite chain state machine");
        finite_machine.listener_definitions = Arc::new(
            (1..=100)
                .map(|event_local_id| {
                    event_listener(
                        event_local_id,
                        (event_local_id < 100).then_some(event_local_id + 1),
                    )
                })
                .collect(),
        );
        finite_machine
            .reported_events
            .push(fl_c5_test_reported_event(1));
        finite_machine.apply_local_event_listeners(&mut finite_artboard, 0, None);
        assert_eq!(
            finite_machine.next_unapplied_reported_event_index(),
            100,
            "a finite chain consumes its hundredth batch"
        );
        assert_eq!(
            finite_machine.reported_event_count(),
            99,
            "events first reported in batches 2 through 100 remain host visible"
        );
        assert_eq!(
            finite_machine
                .reporting_events
                .first()
                .map(StateMachineReportedEvent::event_local_index),
            Some(100)
        );

        let vm_listener_definition = RuntimeStateMachineListener {
            name: None,
            target_local_id: 0,
            is_single: false,
            listener_types: vec![RuntimeListenerType::ViewModel],
            event_local_indices: Vec::new(),
            view_model_path: Some(RuntimeListenerViewModelPath::Absolute {
                view_model_index: 0,
                property_path: vec![0],
            }),
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: vec![RuntimeScheduledListenerAction::FireEvent(
                super::listener_fire_event::RuntimeListenerFireEvent::for_test(0, Some(2)),
            )],
        };
        let mixed_definitions = Arc::new(vec![
            event_listener(1, None),
            vm_listener_definition,
            event_listener(2, None),
        ]);
        finite_machine.listener_definitions = Arc::clone(&mixed_definitions);
        finite_machine.view_model_listeners = vec![
            RuntimeViewModelListenerInstance::new(Arc::clone(&mixed_definitions), 1)
                .expect("mixed ViewModel listener"),
        ];
        finite_machine.reported_events.clear();
        finite_machine.reported_event_listener_index = 0;
        finite_machine
            .reported_events
            .push(fl_c5_test_reported_event(1));
        finite_machine
            .reported_listener_view_models
            .report_data_bind(0);
        finite_machine.apply_local_event_listeners(&mut finite_artboard, 0, None);
        assert_eq!(
            finite_machine
                .reporting_events
                .iter()
                .map(StateMachineReportedEvent::event_local_index)
                .collect::<Vec<_>>(),
            [2],
            "the ViewModel callback fires an event into the next same-call batch after the event phase"
        );
        assert_eq!(
            finite_machine.reported_event_count(),
            1,
            "the event fired by the ViewModel listener remains host visible exactly once"
        );

        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut event_to_vm = scripted_test_listener(
            &mut finite_machine,
            985,
            "event-to-vm",
            ListenerFailure::None,
            vec![RuntimeListenerType::Event],
            &calls,
        );
        event_to_vm.target_local_id = 1;
        event_to_vm.event_local_indices = vec![1];
        let event_to_vm_queue = finite_machine.reported_listener_view_models.clone();
        let event_to_vm_script =
            RuntimeScriptInstanceHandle::new(Box::new(ReportingViewModelListenerScript {
                label: "event-to-vm",
                queue: event_to_vm_queue,
                listener_index: 0,
                calls: Rc::clone(&calls),
            }));
        finite_machine
            .scripted_instances_by_global
            .insert(985, event_to_vm_script.clone());
        finite_machine
            .scripted_listener_action_instances
            .insert(985, event_to_vm_script);
        finite_machine.scripted_object_initialization_complete = true;
        let mut event_after_vm = scripted_test_listener(
            &mut finite_machine,
            986,
            "after-vm",
            ListenerFailure::None,
            vec![RuntimeListenerType::Event],
            &calls,
        );
        event_after_vm.target_local_id = 2;
        event_after_vm.event_local_indices = vec![2];
        let event_to_vm_definitions = Arc::new(vec![
            event_to_vm,
            RuntimeStateMachineListener {
                name: None,
                target_local_id: 0,
                is_single: false,
                listener_types: vec![RuntimeListenerType::ViewModel],
                event_local_indices: Vec::new(),
                view_model_path: Some(RuntimeListenerViewModelPath::Absolute {
                    view_model_index: 0,
                    property_path: vec![0],
                }),
                view_model_input_types: Vec::new(),
                gamepad_input_types: Vec::new(),
                keyboard_input_types: Vec::new(),
                semantic_input_types: Vec::new(),
                hit_paths: Vec::new(),
                listener_actions: vec![RuntimeScheduledListenerAction::FireEvent(
                    super::listener_fire_event::RuntimeListenerFireEvent::for_test(0, Some(2)),
                )],
            },
            event_after_vm,
        ]);
        finite_machine.listener_definitions = Arc::clone(&event_to_vm_definitions);
        finite_machine.view_model_listeners = vec![
            RuntimeViewModelListenerInstance::new(Arc::clone(&event_to_vm_definitions), 1)
                .expect("event-generated ViewModel listener"),
        ];
        finite_machine.reported_events.clear();
        finite_machine.reported_event_listener_index = 0;
        finite_machine
            .reported_events
            .push(fl_c5_test_reported_event(1));
        finite_machine.apply_local_event_listeners(&mut finite_artboard, 0, None);
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["event-to-vm", "after-vm"],
            "event-generated ViewModel work runs in the next batch, then its generated event runs in the following batch"
        );

        machine.listener_definitions = Arc::new(vec![event_listener(1, Some(1))]);
        machine.reported_events.push(StateMachineReportedEvent {
            event_local_index: 1,
            event_core_type: 128,
            name: Some("loop".to_owned()),
            url: None,
            target: None,
            properties: Vec::new(),
            string_properties: Vec::new(),
            seconds_delay: 0.0,
            context: None,
        });
        let start = machine.next_unapplied_reported_event_index();
        machine.apply_local_event_listeners(&mut artboard, start, None);
        assert_eq!(
            machine.next_unapplied_reported_event_index(),
            100,
            "exactly 100 finite callback batches must be consumed"
        );
        assert_eq!(
            machine.reported_event_count(),
            100,
            "batches 2 through 100 remain host visible and the event generated by batch 100 is pending as batch 101"
        );
        assert_eq!(
            machine
                .reported_event_snapshot(0)
                .map(StateMachineReportedEvent::event_local_index),
            Some(1)
        );
    }

    #[test]
    fn fl_c5_event_listener_fire_reports_live_payload_before_advance() {
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

    fn fl_c5_test_reported_event(local_index: usize) -> StateMachineReportedEvent {
        StateMachineReportedEvent {
            event_local_index: local_index,
            event_core_type: 128,
            name: Some(format!("event-{local_index}")),
            url: None,
            target: None,
            properties: Vec::new(),
            string_properties: Vec::new(),
            seconds_delay: 0.0,
            context: None,
        }
    }

    fn fl_c5_test_audio_event(local_index: usize) -> (StateMachineReportedEvent, u32) {
        let audio_file = RuntimeFile::from_authoring_records(vec![
            AuthoringRecord {
                type_key: nuxie_schema::definition_by_name("Backboard")
                    .expect("Backboard schema definition")
                    .type_key
                    .int,
                properties: Vec::new(),
            },
            AuthoringRecord {
                type_key: nuxie_schema::definition_by_name("Artboard")
                    .expect("Artboard schema definition")
                    .type_key
                    .int,
                properties: Vec::new(),
            },
            AuthoringRecord {
                type_key: nuxie_schema::definition_by_name("AudioEvent")
                    .expect("AudioEvent schema definition")
                    .type_key
                    .int,
                properties: vec![AuthoringProperty {
                    key: crate::properties::property_key_for_name("AudioEvent", "parentId")
                        .expect("AudioEvent.parentId"),
                    value: AuthoringValue::Uint(0),
                }],
            },
        ])
        .expect("import live AudioEvent fixture");
        let audio_object = audio_file
            .objects
            .iter()
            .flatten()
            .find(|object| object.type_name == "AudioEvent")
            .expect("live AudioEvent-typed object");
        (
            StateMachineReportedEvent::from_runtime_event(local_index, audio_object),
            u32::from(audio_object.type_key),
        )
    }

    #[test]
    fn fl_c5_event_mid_callback_visibility_excludes_the_reporting_snapshot() {
        let (_artboard, mut machine) = scripted_listener_artboard_and_machine();
        machine.reported_events.push(fl_c5_test_reported_event(7));
        let mut reporting = std::mem::take(&mut machine.reporting_events);
        reporting.clear();
        reporting.extend_from_slice(&machine.reported_events);
        machine.reported_event_listener_index = machine.reported_events.len();

        assert_eq!(machine.reported_event_count(), 0);
        assert!(machine.reported_event_snapshot(0).is_none());
        machine.reported_events.push(fl_c5_test_reported_event(8));
        assert_eq!(machine.reported_event_count(), 1);
        assert_eq!(
            machine
                .reported_event_snapshot(0)
                .map(StateMachineReportedEvent::event_local_index),
            Some(8),
            "callback inspection sees only work appended for a later batch"
        );
        assert_eq!(reporting[0].event_local_index(), 7);
    }

    #[test]
    fn view_model_listener_binding_reports_a_trigger_fired_before_relink() {
        let definitions = Arc::new(vec![RuntimeStateMachineListener {
            name: None,
            target_local_id: 0,
            is_single: false,
            listener_types: vec![RuntimeListenerType::ViewModel],
            event_local_indices: Vec::new(),
            view_model_path: Some(RuntimeListenerViewModelPath::Absolute {
                view_model_index: 0,
                property_path: vec![0],
            }),
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: Vec::new(),
        }]);
        let mut listener = RuntimeViewModelListenerInstance::new(definitions, 0)
            .expect("ViewModel listener instance");
        let queue = RuntimeCellNotificationQueue::default();
        let trigger = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Trigger(1));
        relink_view_model_listener_cell(
            &mut listener.property_bindings[0],
            Some(trigger),
            &queue,
            0,
        );
        assert!(queue.is_empty(), "relink alone does not synthesize dirt");

        listener.report_pending_trigger_bindings(&queue, 0);
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn upstream_view_model_listener_fixture_keeps_loop_fired_event_host_visible_once() {
        let file = read_runtime_file(include_bytes!(
            "../../../../fixtures/sync/vm_listener_fire_event.riv"
        ))
        .expect("upstream ViewModel-listener fixture imports");
        let graphs = GraphFile::from_runtime_file(&file).expect("fixture graph builds");
        let graph = graphs.artboards.first().expect("default artboard graph");
        let mut artboard =
            ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
                .expect("default artboard instantiates");
        let mut machine = artboard
            .state_machine_instance(0)
            .expect("fixture state machine");
        let mut context = artboard
            .imported_view_model_instance_context(0, 0)
            .expect("fixture ViewModel instance");
        assert!(machine.bind_imported_view_model_context(&file, &context));
        artboard.advance_state_machine_instance(&mut machine, 0.0);
        assert_eq!(machine.reported_event_count(), 0);

        assert!(context.set_trigger_by_property_name(&file, "go", 1));
        artboard.advance_state_machine_instance(&mut machine, 0.016);
        assert_eq!(machine.reported_event_count(), 1);
        assert_eq!(
            machine
                .reported_event(&artboard, 0)
                .and_then(|event| event.name()),
            Some("ding")
        );

        artboard.advance_state_machine_instance(&mut machine, 0.016);
        assert_eq!(machine.reported_event_count(), 0);
    }

    #[test]
    fn fl_c5_event_trigger_zero_suppression_and_duplicate_listener_fifo() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Trigger(0));
        let queue = RuntimeCellNotificationQueue::default();
        let first = RuntimeCellDirtSink::reporting_listener(&queue, 3);
        let duplicate = RuntimeCellDirtSink::reporting_listener(&queue, 3);
        cell.add_dependent(&first);
        cell.add_dependent(&duplicate);

        assert!(cell.fire_trigger());
        cell.advanced();
        let mut reporting = Vec::new();
        queue.swap_into(&mut reporting);
        assert_eq!(
            reporting,
            [3, 3],
            "one genuine mutation preserves duplicate dependent registrations"
        );
        assert_eq!(cell.value(), RuntimeViewModelCellValue::Trigger(0));

        let signed_zero = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(-0.0));
        assert_eq!(
            signed_zero.value(),
            RuntimeViewModelCellValue::Number(-0.0),
            "signed zero remains ordinary number payload data"
        );
        let signed_zero_trigger = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Trigger(1));
        let signed_zero_queue = RuntimeCellNotificationQueue::default();
        let signed_zero_sink = RuntimeCellDirtSink::reporting_listener(&signed_zero_queue, 4);
        signed_zero_trigger.add_dependent(&signed_zero_sink);
        assert!(
            signed_zero_trigger.set_value(RuntimeViewModelCellValue::Trigger((-0.0_f32) as u64))
        );
        let mut signed_zero_reports = Vec::new();
        signed_zero_queue.swap_into(&mut signed_zero_reports);
        assert!(
            signed_zero_reports.is_empty(),
            "a trigger reset expressed through signed zero is the same suppressed zero counter"
        );
    }

    #[test]
    fn fl_c5_event_listener_major_event_minor_single_and_multi_order() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut single = scripted_test_listener(
            &mut machine,
            980,
            "single",
            ListenerFailure::None,
            vec![RuntimeListenerType::Event],
            &calls,
        );
        single.target_local_id = 0;
        single.is_single = true;
        single.event_local_indices = vec![7];
        let mut multi = scripted_test_listener(
            &mut machine,
            981,
            "multi",
            ListenerFailure::None,
            vec![RuntimeListenerType::Event],
            &calls,
        );
        multi.target_local_id = 0;
        multi.event_local_indices = vec![7];
        machine.listener_definitions = Arc::new(vec![single, multi]);

        let events = [fl_c5_test_reported_event(7), fl_c5_test_reported_event(7)];
        machine.notify_events(&mut artboard, None, &events);
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["single", "multi", "multi"],
            "listeners are outermost; single breaks at the first [A,A] match while multi scans both"
        );
    }

    #[test]
    fn fl_c5_event_bubbling_precedes_the_recorded_audio_seam_through_two_ancestors() {
        let (mut leaf_artboard, mut leaf) = scripted_listener_artboard_and_machine();
        let (mut parent_artboard, mut parent) = scripted_listener_artboard_and_machine();
        let (mut root_artboard, mut root) = scripted_listener_artboard_and_machine();
        let total_order = Rc::new(RefCell::new(Vec::new()));
        leaf.event_total_order_trace = Some(("leaf-local", "leaf-audio", Rc::clone(&total_order)));
        parent.event_total_order_trace =
            Some(("parent-local", "parent-audio", Rc::clone(&total_order)));
        root.event_total_order_trace = Some(("root-local", "root-audio", Rc::clone(&total_order)));
        leaf.attach_event_bubble_owner();
        parent.attach_event_bubble_owner();
        let ordinary_event = fl_c5_test_reported_event(6);
        let (audio_event, audio_event_core_type) = fl_c5_test_audio_event(7);
        let event = [ordinary_event, audio_event];
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut mismatch = scripted_test_listener(
            &mut parent,
            982,
            "mismatch",
            ListenerFailure::None,
            vec![RuntimeListenerType::Event],
            &calls,
        );
        mismatch.target_local_id = 99;
        mismatch.event_local_indices = vec![7];
        let mut parent_listener = scripted_test_listener(
            &mut parent,
            983,
            "parent",
            ListenerFailure::None,
            vec![RuntimeListenerType::Event],
            &calls,
        );
        parent_listener.target_local_id = 7;
        parent_listener.event_local_indices = vec![7];
        parent.listener_definitions = Arc::new(vec![mismatch, parent_listener]);
        parent
            .nested_event_registrations
            .push(RuntimeNestedEventRegistration {
                source_local_id: 7,
                notifier_local_id: 70,
                kind: RuntimeNestedEventNotifierKind::StateMachine,
            });
        let mut root_listener = scripted_test_listener(
            &mut root,
            984,
            "root",
            ListenerFailure::None,
            vec![RuntimeListenerType::Event],
            &calls,
        );
        root_listener.target_local_id = 8;
        root_listener.event_local_indices = vec![7];
        root.listener_definitions = Arc::new(vec![root_listener]);
        root.nested_event_registrations
            .push(RuntimeNestedEventRegistration {
                source_local_id: 8,
                notifier_local_id: 80,
                kind: RuntimeNestedEventNotifierKind::StateMachine,
            });

        root_artboard.set_frame_origin(false);
        let _ = root_artboard.advance_state_machine_instance(&mut root, 0.0);
        leaf.notify_events(&mut leaf_artboard, None, &event);
        let parent_events = leaf.take_bubbled_event_reports();
        assert_eq!(
            parent_events
                .iter()
                .map(StateMachineReportedEvent::event_local_index)
                .collect::<Vec<_>>(),
            [6, 7],
            "ordinary and audio reports both bubble in authored order"
        );
        parent.notify_events(&mut parent_artboard, Some(7), &parent_events);
        let root_events = parent.take_bubbled_event_reports();
        assert_eq!(
            root_events
                .iter()
                .map(StateMachineReportedEvent::event_local_index)
                .collect::<Vec<_>>(),
            [6, 7]
        );
        root.notify_events(&mut root_artboard, Some(8), &root_events);
        parent.flush_deferred_owner_audio_events();
        leaf.flush_deferred_owner_audio_events();
        assert_eq!(
            *total_order.borrow(),
            [
                "leaf-local",
                "parent-local",
                "root-local",
                "root-audio",
                "parent-audio",
                "leaf-audio",
            ],
            "nested bubbling is synchronous depth-first and audio tails unwind root-first"
        );
        assert!(
            root.take_bubbled_event_reports().is_empty(),
            "root draw state and a post-update probe do not invent an outgoing parent edge"
        );
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["parent", "root"],
            "each owner dispatches the realistic nested source once; the mismatched target stays inert"
        );
        for machine in [&leaf, &parent] {
            assert_eq!(
                machine.event_dispatch_phase_trace,
                ["local-dispatch", "bubble-to-owner", "recorded-audio-seam"]
            );
            assert_eq!(
                machine.audio_event_seam_receipt(),
                (1, Some((7, audio_event_core_type))),
                "only the imported AudioEvent occurrence reaches the production handoff"
            );
        }
        assert_eq!(
            root.event_dispatch_phase_trace,
            ["local-dispatch", "recorded-audio-seam"]
        );
        assert_eq!(
            root.audio_event_seam_receipt(),
            (1, Some((7, audio_event_core_type)))
        );

        root.event_dispatch_phase_trace.clear();
        assert!(
            !root.notify_events(&mut root_artboard, Some(usize::MAX), &event),
            "an unregistered nested source must not dispatch or bubble"
        );
        assert!(root.event_dispatch_phase_trace.is_empty());

        leaf.notify_events(&mut leaf_artboard, None, &event);
        assert_eq!(leaf.reported_event_count(), 2);
        assert!(leaf.reported_event(&leaf_artboard, 1).is_some());
        assert_eq!(leaf.reported_event_count(), 0);
        leaf.notify_events(&mut leaf_artboard, None, &event);
        assert_eq!(
            leaf.bubbled_event_reports.len(),
            2,
            "the next bubble batch reclaims the production cursor's consumed prefix"
        );
    }

    #[test]
    fn fl_c5_event_bubbling_cross_instance_total_order_through_one_ancestor() {
        let (mut leaf_artboard, mut leaf) = scripted_listener_artboard_and_machine();
        let (mut parent_artboard, mut parent) = scripted_listener_artboard_and_machine();
        let total_order = Rc::new(RefCell::new(Vec::new()));
        leaf.event_total_order_trace = Some(("leaf-local", "leaf-audio", Rc::clone(&total_order)));
        parent.event_total_order_trace =
            Some(("parent-local", "parent-audio", Rc::clone(&total_order)));
        leaf.attach_event_bubble_owner();
        parent
            .nested_event_registrations
            .push(RuntimeNestedEventRegistration {
                source_local_id: 7,
                notifier_local_id: 70,
                kind: RuntimeNestedEventNotifierKind::StateMachine,
            });
        let (audio_event, _) = fl_c5_test_audio_event(7);

        leaf.notify_events(&mut leaf_artboard, None, &[audio_event]);
        let events = leaf.take_bubbled_event_reports();
        parent.notify_events(&mut parent_artboard, Some(7), &events);
        leaf.flush_deferred_owner_audio_events();

        assert_eq!(
            *total_order.borrow(),
            ["leaf-local", "parent-local", "parent-audio", "leaf-audio"],
            "the two-level owner seam uses the same depth-first unwind policy"
        );
    }

    #[test]
    fn fl_c5_failing_reporting_owner_completes_deep_bubble_and_audio_before_error_propagation() {
        let (mut child_artboard, mut parent) = scripted_listener_artboard_and_machine();
        let (mut root_artboard, mut root) = scripted_listener_artboard_and_machine();
        let total_order = Rc::new(RefCell::new(Vec::new()));
        parent.event_total_order_trace =
            Some(("parent-local", "parent-audio", Rc::clone(&total_order)));
        root.event_total_order_trace = Some(("root-local", "root-audio", Rc::clone(&total_order)));
        parent.attach_event_bubble_owner();
        root.nested_event_registrations
            .push(RuntimeNestedEventRegistration {
                source_local_id: 8,
                notifier_local_id: 80,
                kind: RuntimeNestedEventNotifierKind::StateMachine,
            });
        parent
            .nested_event_registrations
            .push(RuntimeNestedEventRegistration {
                source_local_id: 7,
                notifier_local_id: 70,
                kind: RuntimeNestedEventNotifierKind::StateMachine,
            });

        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut failing_listener = scripted_test_listener(
            &mut parent,
            985,
            "reporting-owner",
            ListenerFailure::Terminal("script.resource.reporting_owner"),
            vec![RuntimeListenerType::Event],
            &calls,
        );
        failing_listener.target_local_id = 7;
        failing_listener.event_local_indices = vec![7];
        parent.listener_definitions = Arc::new(vec![failing_listener]);
        let (audio_event, audio_event_core_type) = fl_c5_test_audio_event(7);
        let notifier_local = 80;
        let (_, fallback_machine) = scripted_listener_artboard_and_machine();
        let mut animations = vec![
            crate::artboard::RuntimeNestedAnimationInstance::StateMachine(
                RuntimeNestedStateMachineInstance::new(
                    notifier_local,
                    fallback_machine,
                    Vec::new(),
                ),
            ),
        ];
        let mut parent_artboard = scripted_listener_artboard_and_machine().0;
        parent_artboard
            .active_nested_state_machines
            .insert(notifier_local, parent);
        let mid_chain_error_was_none = Rc::new(RefCell::new(None));
        let observed_mid_chain = Rc::clone(&mid_chain_error_was_none);
        let mut ancestor_dispatch =
            |artboard: &mut ArtboardInstance,
             _source_local: usize,
             events: &[StateMachineReportedEvent]| {
                *observed_mid_chain.borrow_mut() = Some(
                    artboard
                        .active_nested_state_machines
                        .get(&notifier_local)
                        .expect("the failing owner remains mounted")
                        .script_error()
                        .is_none(),
                );
                root.notify_events(&mut root_artboard, Some(8), events)
            };

        assert!(
            !StateMachineInstance::dispatch_nested_events_to_animation_owners(
                &mut parent_artboard,
                8,
                &mut animations,
                &mut child_artboard,
                7,
                &[audio_event],
                None,
                Some(&mut ancestor_dispatch),
            )
        );
        assert_eq!(
            *mid_chain_error_was_none.borrow(),
            Some(true),
            "the terminal ScriptError is withheld during ancestor dispatch"
        );
        let parent = parent_artboard
            .active_nested_state_machines
            .get(&notifier_local)
            .expect("the failing owner remains mounted");

        assert_eq!(
            total_order.borrow().as_slice(),
            ["parent-local", "root-local", "root-audio", "parent-audio",],
            "W63 item 3: the failing reporting owner's full-height bubble and audio tail complete before its ScriptError propagates",
        );
        assert!(parent.script_error().is_some());
        assert_eq!(
            parent.audio_event_seam_receipt(),
            (1, Some((7, audio_event_core_type))),
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
            name: None,
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
            RuntimeQueuedFocusEvent {
                listener_index: 0,
                is_focus: true,
            },
            RuntimeQueuedFocusEvent {
                listener_index: 1,
                is_focus: true,
            },
        ];
        machine.queued_semantic_events = vec![RuntimeQueuedSemanticEvent {
            listener_index: Some(2),
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
            RuntimeQueuedFocusEvent {
                listener_index: 0,
                is_focus: true,
            },
            RuntimeQueuedFocusEvent {
                listener_index: 1,
                is_focus: true,
            },
        ];
        machine.queued_semantic_events = vec![RuntimeQueuedSemanticEvent {
            listener_index: Some(2),
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
        machine.queued_focus_events = vec![RuntimeQueuedFocusEvent {
            listener_index: 0,
            is_focus: true,
        }];
        machine.queued_semantic_events = vec![
            RuntimeQueuedSemanticEvent {
                listener_index: Some(1),
                action_type: 1,
            },
            RuntimeQueuedSemanticEvent {
                listener_index: Some(2),
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
        machine.queued_focus_events = vec![RuntimeQueuedFocusEvent {
            listener_index: 0,
            is_focus: true,
        }];
        machine.queued_semantic_events = vec![RuntimeQueuedSemanticEvent {
            listener_index: Some(1),
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
    fn profiler_listener_hook_records_the_runtime_listener_callsite() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        Arc::make_mut(&mut machine.listener_definitions)[0].name =
            Some("Profiler Listener Hook".to_owned());
        let action = machine
            .scripted_listener_actions()
            .first()
            .expect("fixture scripted listener action")
            .clone();
        let calls = Rc::new(RefCell::new(Vec::new()));
        machine
            .set_scripted_listener_action_instance(
                action.action_global_id(),
                script("profiler", true, false, ListenerFailure::None, &calls),
            )
            .expect("attach profiler listener action");

        let records = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        crate::with_rive_profile(|profile| {
            profile.set_capture(Box::new(ProfilerListenerCapture::default()));
            profile.set_listener_perform_change_flush_callback(Some(Box::new({
                let records = std::sync::Arc::clone(&records);
                move |incoming| records.lock().unwrap().extend_from_slice(incoming)
            })));
            profile.start();
        });

        machine
            .try_pointer_down_with_timestamp_and_script_host(
                &mut artboard,
                200.0,
                20.0,
                101,
                1.25,
                &mut NoopScriptHost,
            )
            .expect("profiler pointer down");
        machine
            .try_pointer_up_with_timestamp_and_script_host(
                &mut artboard,
                200.0,
                20.0,
                101,
                1.5,
                &mut NoopScriptHost,
            )
            .expect("profiler pointer up");

        let strings = crate::with_rive_profile(|profile| {
            profile.flush_listener_perform_change_records();
            profile.stop();
            let strings = profile.string_table().to_vec();
            profile.set_listener_perform_change_flush_callback(None);
            strings
        });
        let records = records.lock().unwrap();
        assert!(records.iter().any(|record| {
            strings
                .get(record.listener_name_id as usize)
                .map(String::as_str)
                == Some("Profiler Listener Hook")
                && record.listener_type == RuntimeListenerType::Click.value()
                && record.hit_event == RuntimeListenerType::Up.value()
                && record.pointer_id == 101
        }));
    }

    #[test]
    fn matched_pointer_listener_marks_advance_even_when_actions_are_noops() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let scripted_calls = Rc::new(RefCell::new(Vec::new()));
        let _scripted_object = scripted_test_listener(
            &mut machine,
            98_700,
            "unmounted scripted object",
            ListenerFailure::None,
            Vec::new(),
            &scripted_calls,
        );
        assert!(
            machine.scripted_data_context_prepare_pending(),
            "the exposing fixture must retain one not-yet-mounted scripted object"
        );
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
        let raw_advance_calls_before = machine.raw_advance_call_count;
        let _ = machine
            .advance_and_apply(&mut artboard, 0.25)
            .expect("ordinary bookkeeping is independent of script mount");
        assert!(
            machine.raw_advance_call_count > raw_advance_calls_before,
            "raw advance bookkeeping runs immediately through the public path while the unrelated scripted object remains unavailable"
        );
    }

    #[test]
    fn pointer_history_is_listener_scoped_and_resets_on_first_entry_and_reentry() {
        let mut first = ListenerGroup::authored(0);
        let mut second = ListenerGroup::authored(1);

        first.reset(7);
        first.hover(7);
        let first_entry = first.process(7, (10.0, 20.0), true, false, false);
        assert!(!first_entry.previous_hovered);
        assert_eq!(first_entry.previous_position, (10.0, 20.0));
        first.record_position(7, (10.0, 20.0));

        second.reset(7);
        second.hover(7);
        let overlapping_entry = second.process(7, (100.0, 200.0), true, false, false);
        assert!(!overlapping_entry.previous_hovered);
        assert_eq!(
            overlapping_entry.previous_position,
            (100.0, 200.0),
            "a second listener group must not inherit the first group's history"
        );
        second.record_position(7, (100.0, 200.0));

        first.reset(7);
        first.hover(7);
        let move_inside = first.process(7, (15.0, 25.0), true, false, false);
        assert!(move_inside.previous_hovered);
        assert_eq!(move_inside.previous_position, (10.0, 20.0));
        first.record_position(7, (15.0, 25.0));

        first.reset(7);
        let exit = first.process(7, (30.0, 40.0), true, false, false);
        assert!(exit.previous_hovered);
        assert_eq!(exit.previous_position, (15.0, 25.0));
        first.record_position(7, (30.0, 40.0));

        first.reset(7);
        let outside = first.process(7, (50.0, 60.0), true, false, false);
        assert!(!outside.previous_hovered);
        first.record_position(7, (50.0, 60.0));
        first.reset(7);
        first.hover(7);
        let reentry = first.process(7, (70.0, 80.0), true, false, false);
        assert!(!reentry.previous_hovered);
        assert_eq!(
            reentry.previous_position,
            (70.0, 80.0),
            "reentry resets the prior outside position before dispatch"
        );
    }

    #[test]
    fn pointer_up_position_is_retained_for_exit_then_released() {
        let mut group = ListenerGroup::authored(0);
        group.reset(9);
        group.hover(9);
        group.process(9, (10.0, 20.0), true, true, false);
        group.record_position(9, (10.0, 20.0));
        group.reset(9);
        group.hover(9);
        let up = group.process(9, (15.0, 25.0), true, false, true);
        assert!(up.previous_hovered);
        assert_eq!(up.previous_position, (10.0, 20.0));
        group.record_position(9, (15.0, 25.0));

        group.reset(9);
        let exit = group.process(9, (30.0, 40.0), true, false, false);
        assert!(exit.previous_hovered);
        assert_eq!(exit.previous_position, (15.0, 25.0));
        group.record_position(9, (30.0, 40.0));
        group.release_event(9);

        group.reset(9);
        group.hover(9);
        let next_entry = group.process(9, (50.0, 60.0), true, false, false);
        assert!(!next_entry.previous_hovered);
        assert_eq!(next_entry.previous_position, (50.0, 60.0));
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
    fn fl_c5_clone_teardown_rebuilds_mutable_state_without_aliasing() {
        let mut original = scripted_listener_machine();
        original.reported_events = vec![fl_c5_test_reported_event(10)];
        original.reporting_events = vec![fl_c5_test_reported_event(11)];
        original.bubbled_event_reports = vec![fl_c5_test_reported_event(12)];
        original.reporting_listener_view_models = vec![13];
        original.post_apply_listener_view_models = vec![14];
        original.primary_data_context = Some(RuntimeStateMachineDataContext::default());
        original.queued_focus_events = vec![RuntimeQueuedFocusEvent {
            listener_index: 3,
            is_focus: true,
        }];
        original.queued_semantic_events = vec![RuntimeQueuedSemanticEvent {
            listener_index: Some(4),
            action_type: 2,
        }];
        original.listener_groups.push(ListenerGroup::authored(2));
        let pointer_group = original
            .listener_groups
            .last_mut()
            .expect("pointer listener group");
        pointer_group.reset(5);
        pointer_group.hover(5);
        pointer_group.process(5, (-1.0, -2.0), true, true, false);
        pointer_group.begin_capture(5, None);
        pointer_group.record_position(5, (-1.0, -2.0));
        original
            .nested_event_registrations
            .push(RuntimeNestedEventRegistration {
                source_local_id: 7,
                notifier_local_id: 8,
                kind: RuntimeNestedEventNotifierKind::StateMachine,
            });
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
        assert_eq!(
            cloned
                .reported_events
                .iter()
                .map(StateMachineReportedEvent::event_local_index)
                .collect::<Vec<_>>(),
            [10]
        );
        assert_eq!(
            cloned
                .reporting_events
                .iter()
                .map(StateMachineReportedEvent::event_local_index)
                .collect::<Vec<_>>(),
            [11]
        );
        assert_eq!(
            cloned
                .bubbled_event_reports
                .iter()
                .map(StateMachineReportedEvent::event_local_index)
                .collect::<Vec<_>>(),
            [12]
        );
        assert_ne!(
            cloned.reported_events.as_ptr(),
            original.reported_events.as_ptr(),
            "pending host reports are copied into distinct Vec storage"
        );
        assert_ne!(
            cloned.reporting_events.as_ptr(),
            original.reporting_events.as_ptr(),
            "the active event batch is copied into distinct Vec storage"
        );
        assert_ne!(
            cloned.bubbled_event_reports.as_ptr(),
            original.bubbled_event_reports.as_ptr(),
            "the nested bubbling FIFO is copied into distinct Vec storage"
        );
        assert!(
            cloned.reporting_listener_view_models.is_empty(),
            "an in-flight callback batch cannot be replayed by a snapshot"
        );
        assert_eq!(
            cloned.post_apply_listener_view_models,
            original.post_apply_listener_view_models
        );
        assert_ne!(
            cloned.post_apply_listener_view_models.as_ptr(),
            original.post_apply_listener_view_models.as_ptr(),
            "post-apply listener reports are copied into distinct Vec storage"
        );
        assert_eq!(cloned.queued_focus_events, original.queued_focus_events);
        assert_eq!(
            cloned.queued_semantic_events,
            original.queued_semantic_events
        );
        assert_ne!(
            cloned.queued_focus_events.as_ptr(),
            original.queued_focus_events.as_ptr(),
            "pending focus values are copied into distinct Vec storage"
        );
        assert_ne!(
            cloned.queued_semantic_events.as_ptr(),
            original.queued_semantic_events.as_ptr(),
            "pending semantic values are copied into distinct Vec storage"
        );
        assert_ne!(
            cloned.listener_groups.as_ptr(),
            original.listener_groups.as_ptr(),
            "listener groups and their pointer records use distinct Vec storage"
        );
        let cloned_pointer_group = cloned
            .listener_groups
            .iter_mut()
            .find(|group| group.kind == (ListenerGroupKind::Authored { listener_index: 2 }))
            .expect("cloned pointer group");
        assert_eq!(
            cloned_pointer_group.previous_position(5),
            Some((-1.0, -2.0))
        );
        cloned_pointer_group.record_position(5, (9.0, 9.0));
        assert_eq!(
            original
                .listener_groups
                .iter()
                .find(|group| group.kind == (ListenerGroupKind::Authored { listener_index: 2 }))
                .and_then(|group| group.previous_position(5)),
            Some((-1.0, -2.0)),
            "snapshot pointer records cannot mutate the source group"
        );
        assert_eq!(
            cloned.nested_event_registrations, original.nested_event_registrations,
            "snapshot registration identities are retained"
        );
        assert_ne!(
            cloned.nested_event_registrations.as_ptr(),
            original.nested_event_registrations.as_ptr(),
            "nested registrations are copied into distinct Vec storage"
        );
        assert_ne!(
            cloned.hit_components.as_ptr(),
            original.hit_components.as_ptr(),
            "the polymorphic hit-owner list has distinct Vec storage"
        );
        assert!(
            cloned
                .hit_components
                .iter()
                .zip(&original.hit_components)
                .all(|(clone, source)| !std::ptr::eq(&**clone, &**source)),
            "every polymorphic hit owner is cloned rather than shared"
        );
        assert_ne!(
            cloned.listener_groups.as_ptr(),
            original.listener_groups.as_ptr(),
            "mutable listener-group state has distinct Vec storage"
        );
        let original_context = original
            .primary_data_context
            .as_ref()
            .expect("source primary context");
        let cloned_context = cloned
            .primary_data_context
            .as_ref()
            .expect("snapshot primary context");
        assert!(
            !original_context.shares_state_for_test(&cloned_context),
            "the primary DataContext carrier is rebuilt with detached state"
        );
        original.owned_view_model_rebind_sink.take_dirt();
        cloned.owned_view_model_rebind_sink.take_dirt();
        cloned
            .owned_view_model_rebind_sink
            .add_dirt(RuntimeCellDirt::BINDINGS);
        assert!(
            original.owned_view_model_rebind_sink.peek_dirt().is_empty(),
            "the snapshot callback dirt sink cannot dirty the source occurrence"
        );
        assert!(
            cloned.scripted_listener_action_instances.is_empty()
                && cloned.scripted_instances_by_global.is_empty(),
            "mutable script tables stay cold"
        );
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

        let cold_remount = scripted_listener_machine();
        assert!(
            cold_remount.queued_focus_events.is_empty()
                && cold_remount.queued_semantic_events.is_empty()
                && cold_remount
                    .listener_groups
                    .iter()
                    .all(|group| group.previous_position(5).is_none()),
            "a cold remount starts without the snapshot's pending owned values"
        );
        assert!(
            cold_remount.scripted_listener_action_instances.is_empty()
                && cold_remount.scripted_instances_by_global.is_empty(),
            "a cold remount also starts with cold script occurrence state"
        );
    }

    #[test]
    fn fl_c5_clone_teardown_dispose_is_repeatable_and_drop_order_is_observable() {
        let receipt = Rc::new(RefCell::new(Vec::new()));
        {
            let mut machine = scripted_listener_machine();
            machine.drop_phase_receipt = Some(Rc::clone(&receipt));
            machine
                .nested_event_registrations
                .push(RuntimeNestedEventRegistration {
                    source_local_id: 7,
                    notifier_local_id: 8,
                    kind: RuntimeNestedEventNotifierKind::LinearAnimation,
                });
            machine.dispose();
            machine.dispose();
            assert!(machine.disposed);
            assert!(machine.nested_event_registrations.is_empty());
        }
        assert_eq!(
            receipt.borrow().as_slice(),
            ["nested-detach", "focus", "binds", "layers", "scripts"],
            "manual dispose detaches once; Drop then preserves focus → binds → layers → scripts"
        );

        let implicit_receipt = Rc::new(RefCell::new(Vec::new()));
        {
            let mut machine = scripted_listener_machine();
            machine.drop_phase_receipt = Some(Rc::clone(&implicit_receipt));
            machine
                .nested_event_registrations
                .push(RuntimeNestedEventRegistration {
                    source_local_id: 9,
                    notifier_local_id: 10,
                    kind: RuntimeNestedEventNotifierKind::StateMachine,
                });
        }
        assert_eq!(
            implicit_receipt.borrow().as_slice(),
            ["focus", "nested-detach", "binds", "layers", "scripts"],
            "Drop prevents a stale nested registration when explicit dispose was omitted"
        );

        let event_calls = Rc::new(RefCell::new(Vec::new()));
        let (mut event_artboard, mut event_machine, _) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "dispose event",
                methods: Vec::new(),
                handled: false,
                calls: event_calls,
            }));
        let event_listener = |target_local_id| RuntimeStateMachineListener {
            name: None,
            target_local_id,
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
        };
        let event = StateMachineReportedEvent {
            event_local_index: 7,
            event_core_type: 128,
            name: Some("nested".to_owned()),
            url: None,
            target: None,
            properties: Vec::new(),
            string_properties: Vec::new(),
            seconds_delay: 0.0,
            context: None,
        };
        event_machine.listener_definitions = Arc::new(vec![event_listener(7)]);
        event_machine
            .nested_event_registrations
            .push(RuntimeNestedEventRegistration {
                source_local_id: 7,
                notifier_local_id: 8,
                kind: RuntimeNestedEventNotifierKind::StateMachine,
            });
        assert!(event_machine.notify_events(&mut event_artboard, Some(7), &[event.clone()]));
        assert!(!event_machine.focus.target_has_focus(1));
        assert!(event_machine.focus.set_focus_target(1));
        event_machine.dispose();
        event_machine.dispose();
        assert!(!event_machine.notify_events(&mut event_artboard, Some(7), &[event.clone()]));
        assert!(
            event_machine.focus.target_has_focus(1),
            "a detached child source can no longer clear the parent's focus"
        );
        event_machine.listener_definitions = Arc::new(vec![event_listener(0)]);
        assert!(event_machine.notify_events(&mut event_artboard, None, &[event]));
        assert!(
            !event_machine.focus.target_has_focus(1),
            "dispose detaches nested sources without disabling unrelated local events"
        );

        let focus_calls = Rc::new(RefCell::new(Vec::new()));
        let (_artboard, mut focus_owner, _) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "focus owner",
                methods: Vec::new(),
                handled: false,
                calls: focus_calls,
            }));
        {
            let mut externally_managed = scripted_listener_machine();
            externally_managed.install_external_focus(&focus_owner.focus, 99);
        }
        assert!(
            focus_owner.focus.target_has_focus(1),
            "dropping an external focus projection leaves its owner's tree intact"
        );

        let mut retained_external = scripted_listener_machine();
        {
            let focus_calls = Rc::new(RefCell::new(Vec::new()));
            let (_artboard, internal_owner, _) = scripted_drawable_input_artboard_and_machine(
                Box::new(RecordingDrawableInputScript {
                    label: "internal focus owner",
                    methods: Vec::new(),
                    handled: false,
                    calls: focus_calls,
                }),
            );
            retained_external.install_external_focus(&internal_owner.focus, 101);
            assert!(
                !retained_external.focus.focused_listener_chain().is_empty(),
                "the retained projection observes the internal owner's focus"
            );
        }
        assert!(
            retained_external.focus.focused_listener_chain().is_empty(),
            "dropping the internal owner clears focus before external Rc projections survive"
        );
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

    fn fl_c5_bind_record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn fl_c5_bind_property(
        type_name: &str,
        name: &str,
        value: AuthoringValue,
    ) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
            value,
        }
    }

    fn fl_c5_bind_file_and_artboard() -> (RuntimeFile, ArtboardInstance) {
        let file = RuntimeFile::from_authoring_records(vec![
            fl_c5_bind_record("Backboard", Vec::new()),
            fl_c5_bind_record(
                "ViewModel",
                vec![fl_c5_bind_property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Main".to_owned()),
                )],
            ),
            fl_c5_bind_record(
                "ViewModel",
                vec![
                    fl_c5_bind_property(
                        "ViewModel",
                        "name",
                        AuthoringValue::String("Global A".to_owned()),
                    ),
                    fl_c5_bind_property("ViewModel", "viewModelType", AuthoringValue::Uint(2)),
                ],
            ),
            fl_c5_bind_record(
                "ViewModel",
                vec![
                    fl_c5_bind_property(
                        "ViewModel",
                        "name",
                        AuthoringValue::String("Global B".to_owned()),
                    ),
                    fl_c5_bind_property("ViewModel", "viewModelType", AuthoringValue::Uint(2)),
                ],
            ),
            fl_c5_bind_record(
                "ViewModel",
                vec![fl_c5_bind_property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Standard".to_owned()),
                )],
            ),
            fl_c5_bind_record(
                "Artboard",
                vec![
                    fl_c5_bind_property("Artboard", "width", AuthoringValue::Double(100.0)),
                    fl_c5_bind_property("Artboard", "height", AuthoringValue::Double(100.0)),
                    fl_c5_bind_property("Artboard", "viewModelId", AuthoringValue::Uint(0)),
                ],
            ),
        ])
        .expect("WP5 binding fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("WP5 binding fixture graphs");
        let artboard =
            ArtboardInstance::from_graph(&file, graph.artboards.first().expect("fixture artboard"))
                .expect("WP5 binding fixture artboard");
        (file, artboard)
    }

    fn fl_c5_bind_handle(
        file: &RuntimeFile,
        view_model_index: usize,
    ) -> RuntimeOwnedViewModelHandle {
        RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(file, view_model_index)
                .expect("fixture ViewModel instance"),
        )
    }

    #[test]
    fn fl_c5_bind_staged_main_and_globals_apply_only_through_primary_bind() {
        let (file, mut artboard) = fl_c5_bind_file_and_artboard();
        let mut machine = scripted_listener_machine();
        machine.bind_phase_trace.clear();
        let initial_context_kind = machine.data_bind_graph.context_kind;
        let main = fl_c5_bind_handle(&file, 0);
        let override_a = fl_c5_bind_handle(&file, 2);
        let replacement_a = fl_c5_bind_handle(&file, 3);

        let mut invalid_global_machine = scripted_listener_machine();
        assert!(!invalid_global_machine.set_global_view_model_instance(
            Some(&file),
            "missing",
            Some(override_a.clone()),
        ));
        assert!(!invalid_global_machine.set_global_view_model_instance(
            Some(&file),
            "Standard",
            Some(override_a.clone()),
        ));
        assert!(
            invalid_global_machine.data_context().is_none(),
            "failed global validation must not create or register an empty DataContext"
        );

        assert!(!machine.set_view_model_instance(None));
        assert!(machine.data_context().is_none());
        assert!(machine.set_view_model_instance(Some(main.clone())));
        assert_eq!(machine.data_bind_graph.context_kind, initial_context_kind);
        assert!(
            machine
                .primary_data_context
                .as_ref()
                .map(RuntimeStateMachineDataContext::snapshot)
                .as_ref()
                .and_then(RuntimeOwnedViewModelContext::main_handle)
                .is_some_and(|bound| bound.ptr_eq(&main))
        );

        assert!(machine.set_global_view_model_instance(Some(&file), "Global A", None,));
        assert!(
            machine
                .global_view_model_instance(Some(&file), "Global A")
                .is_none()
        );
        assert!(!machine.set_global_view_model_instance(
            None,
            "Global A",
            Some(override_a.clone()),
        ));
        assert!(!machine.set_global_view_model_instance(
            Some(&file),
            "missing",
            Some(override_a.clone()),
        ));
        assert!(!machine.set_global_view_model_instance(
            Some(&file),
            "Standard",
            Some(override_a.clone()),
        ));
        assert!(machine.set_global_view_model_instance(
            Some(&file),
            "Global A",
            Some(override_a.clone()),
        ));
        assert!(
            machine
                .global_view_model_instance(Some(&file), "Global A")
                .is_some_and(|bound| bound.ptr_eq(&override_a)),
            "slot identity comes from the requested global, not the override's ViewModel"
        );
        assert!(machine.set_global_view_model_instance(
            Some(&file),
            "Global A",
            Some(replacement_a.clone()),
        ));
        assert!(
            machine
                .global_view_model_instance(Some(&file), "Global A")
                .is_some_and(|bound| bound.ptr_eq(&replacement_a))
        );
        assert!(machine.set_global_view_model_instance(Some(&file), "Global A", None));
        assert!(
            machine
                .global_view_model_instance(Some(&file), "Global A")
                .is_none(),
            "a null instance empties the named slot"
        );
        assert!(machine.set_global_view_model_instance(
            Some(&file),
            "Global A",
            Some(replacement_a.clone()),
        ));
        assert!(
            machine
                .global_view_model_instance(Some(&file), "Standard")
                .is_none(),
            "the getter performs a pure numeric-slot read and never creates"
        );
        let unusual_slot = fl_c5_bind_handle(&file, 0);
        machine
            .primary_data_context
            .as_ref()
            .expect("primary context")
            .set_unusual_slot_for_test(3, unusual_slot.clone());
        assert!(
            machine
                .global_view_model_instance(Some(&file), "Standard")
                .is_some_and(|bound| bound.ptr_eq(&unusual_slot)),
            "the pure getter reads an occupied numeric slot even when the named ViewModel is non-global"
        );
        assert_eq!(machine.data_bind_graph.context_kind, initial_context_kind);

        let mut empty_machine = scripted_listener_machine();
        empty_machine.view_model_listeners.clear();
        assert!(empty_machine.data_context().is_none());
        assert!(empty_machine.set_global_view_model_instance(Some(&file), "Global A", None));
        assert!(
            empty_machine.data_context().is_none(),
            "clearing an empty valid slot must not allocate a DataContext"
        );

        let (fresh_file, mut fresh_artboard) = fl_c5_bind_file_and_artboard();
        let mut fresh_machine = scripted_listener_machine();
        fresh_machine.view_model_listeners.clear();
        fresh_machine
            .bind(Some(&fresh_file), &mut fresh_artboard)
            .expect("bind without a prior context completes defaults");
        assert!(fresh_machine.data_context().is_some());
        assert!(
            fresh_machine
                .global_view_model_instance(Some(&fresh_file), "Global A")
                .is_some()
        );

        let mut staged_artboard = fresh_artboard;
        let artboard_global = fl_c5_bind_handle(&fresh_file, 2);
        assert!(staged_artboard.set_global_view_model_instance(
            &fresh_file,
            "Global A",
            Some(artboard_global.clone()),
        ));
        assert!(
            staged_artboard
                .global_view_model_instance(&fresh_file, "Global A")
                .is_some_and(|bound| bound.ptr_eq(&artboard_global))
        );
        assert!(staged_artboard.set_global_view_model_instance(&fresh_file, "Global A", None,));
        assert!(
            staged_artboard
                .global_view_model_instance(&fresh_file, "Global A")
                .is_none()
        );

        machine.bind_phase_trace.clear();
        machine
            .bind(Some(&file), &mut artboard)
            .expect("staged primary bind");
        assert_eq!(
            machine.bind_phase_trace,
            [
                "complete-view-models",
                "bind-artboard",
                "bind-machine",
                "assign-context",
                "bind-data-binds",
                "bind-listener-cells",
                "script-context-pass",
                "script-init-pass",
            ],
            "completion and artboard binding precede the machine's exact internal member order"
        );
        let staged = machine
            .primary_data_context
            .as_ref()
            .expect("completed staged context")
            .snapshot();
        assert!(staged.main_handle().is_some());
        assert!(
            staged
                .global_slot_handle(1)
                .is_some_and(|bound| bound.ptr_eq(&replacement_a))
        );
        assert!(staged.global_slot_handle(2).is_some());
        assert_eq!(
            staged.handles().count(),
            3,
            "completion inserts main first, then both globals, without replacing occupied A"
        );
        let staged_main_view_model = staged
            .main_handle()
            .expect("completed main")
            .borrow()
            .view_model_index();
        let staged_global_a_view_model = staged
            .global_slot_handle(1)
            .expect("occupied global A")
            .borrow()
            .view_model_index();
        println!(
            "FLC5_COMPLETE_DIFF main={staged_main_view_model} global_a={staged_global_a_view_model} global_b={}",
            usize::from(staged.global_slot_handle(2).is_some())
        );

        let bound_before_replacement = machine
            .owned_data_context
            .clone()
            .expect("bound machine projection");
        let replacement_main = fl_c5_bind_handle(&file, 0);
        assert!(machine.set_view_model_instance(Some(replacement_main.clone())));
        let staged_after_replacement = machine
            .primary_data_context
            .as_ref()
            .expect("retained primary context")
            .projection();
        assert!(
            !bound_before_replacement.same_binding(&staged_after_replacement),
            "the shared slot table owns the staged replacement identity"
        );
        assert!(
            machine
                .owned_data_context
                .as_ref()
                .is_some_and(|bound| bound.same_binding(&bound_before_replacement))
                && artboard
                    .artboard_owned_data_context
                    .as_ref()
                    .is_some_and(|bound| bound.same_binding(&bound_before_replacement)),
            "setViewModelInstance leaves machine and artboard paths on the old projection"
        );
        assert!(
            machine.owned_view_model_rebind_sink.peek_dirt().is_empty()
                && artboard
                    .artboard_owned_view_model_rebind_sink
                    .peek_dirt()
                    .is_empty(),
            "staging a replacement does not synthesize structural dirt"
        );
        machine
            .bind(Some(&file), &mut artboard)
            .expect("explicit replacement bind");
        assert!(
            machine
                .owned_data_context
                .as_ref()
                .is_some_and(|bound| bound.same_binding(&staged_after_replacement))
                && artboard
                    .artboard_owned_data_context
                    .as_ref()
                    .is_some_and(|bound| bound.same_binding(&staged_after_replacement)),
            "the explicit bind is the first point where staged paths move"
        );
    }

    #[test]
    fn fl_c5_bind_null_matrix_keeps_every_cpp_branch_distinct() {
        let (file, mut artboard) = fl_c5_bind_file_and_artboard();
        let mut machine = scripted_listener_machine();
        machine.view_model_listeners.clear();
        let main = fl_c5_bind_handle(&file, 0);

        assert!(machine.set_view_model_instance(Some(main.clone())));
        machine
            .bind(Some(&file), &mut artboard)
            .expect("initial primary bind");
        assert_eq!(
            machine.data_bind_graph.context_kind,
            RuntimeDataBindGraphContextKind::OwnedViewModel
        );
        assert!(!machine.set_view_model_instance(None));
        assert!(machine.data_context().is_some());

        machine.bind_phase_trace.clear();
        machine
            .bind_view_model_instance(Some(&file), &mut artboard, None)
            .expect("bindViewModelInstance null is the limited clear branch");
        assert!(machine.data_context().is_none());
        assert!(artboard.artboard_owned_data_context.is_none());
        assert_eq!(
            machine.data_bind_graph.context_kind,
            RuntimeDataBindGraphContextKind::OwnedViewModel,
            "bindViewModelInstance(nullptr) does not explicitly unbind machine DataBinds"
        );
        assert_eq!(
            machine.bind_phase_trace,
            ["clear-machine", "unbind-artboard"]
        );

        assert_eq!(
            machine.bind_data_context(&file, &mut artboard, None),
            Err(RuntimeDataContextBindError::NullDataContext),
            "bindDataContext(nullptr) is not a safe clear"
        );
        assert_eq!(machine.inherit_data_context(None), Ok(false));

        assert_eq!(machine.set_data_context(None), Ok(true));
        assert_eq!(
            machine.data_bind_graph.context_kind,
            RuntimeDataBindGraphContextKind::None,
            "dataContext(nullptr) reaches the internal null bind when no VM listener exists"
        );
        machine
            .view_model_listeners
            .push(RuntimeViewModelListenerInstance {
                listener_definitions: Arc::new(Vec::new()),
                listener_index: 0,
                property_bindings: Vec::new(),
            });
        machine.bind_phase_trace.clear();
        assert_eq!(
            machine.set_data_context(None),
            Err(RuntimeDataContextBindError::NullDataContextWithViewModelListeners),
            "the C++ listener dereference hazard remains distinct"
        );
        assert_eq!(
            machine.bind_phase_trace,
            [
                "clear-machine",
                "assign-context",
                "bind-data-binds",
                "bind-listener-cells",
            ],
            "listener failure prevents both scripted context/init passes"
        );
        assert_eq!(
            machine.rebuild_data_bind(None),
            Err(RuntimeDataContextBindError::NullDataBind)
        );

        let mut differential_machine = scripted_listener_machine();
        differential_machine.view_model_listeners.clear();
        let staged_main = fl_c5_bind_handle(&file, 0);
        assert!(differential_machine.set_view_model_instance(Some(staged_main)));
        let staged_view_model = differential_machine
            .data_context()
            .and_then(|context| context.snapshot().main_handle().cloned())
            .expect("staged differential main")
            .borrow()
            .view_model_index();
        let bound_main = fl_c5_bind_handle(&file, 3);
        differential_machine
            .bind_view_model_instance(Some(&file), &mut artboard, Some(bound_main))
            .expect("non-null differential bind");
        let bound_view_model = differential_machine
            .data_context()
            .and_then(|context| context.snapshot().main_handle().cloned())
            .expect("bound differential main")
            .borrow()
            .view_model_index();
        differential_machine
            .bind_view_model_instance(Some(&file), &mut artboard, None)
            .expect("null differential bind");
        println!(
            "FLC5_BIND_NULL_DIFF staged={staged_view_model} bound={bound_view_model} cleared={}",
            usize::from(differential_machine.data_context().is_none())
        );
    }

    #[test]
    fn fl_c5_bind_data_context_and_rebind_preserve_artboard_machine_order() {
        let (file, mut artboard) = fl_c5_bind_file_and_artboard();
        let mut machine = scripted_listener_machine();
        machine.view_model_listeners.clear();
        let context = RuntimeStateMachineDataContext::from_owned_context(
            RuntimeOwnedViewModelContext::from_main_handle(fl_c5_bind_handle(&file, 0)),
        );

        machine.bind_phase_trace.clear();
        machine
            .bind_data_context(&file, &mut artboard, Some(&context))
            .expect("bindDataContext");
        assert_eq!(
            machine.bind_phase_trace,
            [
                "clear-machine",
                "register-machine",
                "clear-artboard",
                "bind-artboard",
                "bind-machine",
                "assign-context",
                "bind-data-binds",
                "bind-listener-cells",
                "script-context-pass",
                "script-init-pass",
            ]
        );
        assert!(
            machine
                .data_context()
                .is_some_and(|bound| bound.ptr_eq(&context))
        );
        assert!(
            artboard
                .artboard_owned_data_context
                .as_ref()
                .is_some_and(|bound| bound.same_binding(&context.projection()))
        );

        machine.bind_phase_trace.clear();
        machine.rebind(&file, &mut artboard).expect("rebind");
        assert_eq!(
            machine.bind_phase_trace,
            [
                "clear-artboard",
                "bind-artboard",
                "bind-machine",
                "assign-context",
                "bind-data-binds",
                "bind-listener-cells",
                "script-context-pass",
                "script-init-pass",
            ]
        );
        machine.bind_phase_trace.clear();
        let _ = machine.relink_data_context(&file, &mut artboard);
        assert!(
            machine.bind_phase_trace.is_empty(),
            "relinkDataContext delegates to the artboard only"
        );
    }

    #[test]
    fn fl_c5_bind_setters_preserve_an_existing_unregistered_context() {
        let (file, _artboard) = fl_c5_bind_file_and_artboard();
        let mut machine = scripted_listener_machine();
        machine.view_model_listeners.clear();
        let context = RuntimeStateMachineDataContext::from_owned_context(
            RuntimeOwnedViewModelContext::from_main_handle(fl_c5_bind_handle(&file, 0)),
        );

        machine
            .set_data_context(Some(&context))
            .expect("dataContext setter");
        machine.owned_view_model_rebind_sink.take_dirt();
        assert!(machine.set_view_model_instance(Some(fl_c5_bind_handle(&file, 0))));
        assert!(machine.set_global_view_model_instance(
            Some(&file),
            "Global A",
            Some(fl_c5_bind_handle(&file, 1)),
        ));
        context.mark_main_rebind_for_test();
        assert!(
            machine.owned_view_model_rebind_sink.peek_dirt().is_empty(),
            "setters reuse the non-registering dataContext(value) carrier without inventing addDependentContainer"
        );
    }

    #[test]
    fn fl_c5_bind_inherit_a_then_b_retains_the_prior_registration_hazard() {
        let (file, _artboard) = fl_c5_bind_file_and_artboard();
        let mut machine = scripted_listener_machine();
        machine.view_model_listeners.clear();
        let context_a = RuntimeStateMachineDataContext::from_owned_context(
            RuntimeOwnedViewModelContext::from_main_handle(fl_c5_bind_handle(&file, 0)),
        );
        let context_b = RuntimeStateMachineDataContext::from_owned_context(
            RuntimeOwnedViewModelContext::from_main_handle(fl_c5_bind_handle(&file, 3)),
        );

        machine
            .inherit_data_context(Some(&context_a))
            .expect("inherit A");
        let inherited_sink = machine.owned_view_model_rebind_sink.clone();
        machine.bind_phase_trace.clear();
        machine
            .inherit_data_context(Some(&context_b))
            .expect("inherit B");
        inherited_sink.take_dirt();
        context_a.mark_main_rebind_for_test();
        assert!(
            machine
                .owned_view_model_rebind_sink
                .peek_dirt()
                .contains(RuntimeCellDirt::BINDINGS),
            "structural dirt from A after inheriting B still reaches the machine because inherit never clears A"
        );
        assert!(
            machine
                .data_context()
                .is_some_and(|bound| bound.ptr_eq(&context_b))
        );
        assert_eq!(
            machine.bind_phase_trace,
            [
                "register-machine-without-clear",
                "assign-context",
                "bind-data-binds",
                "bind-listener-cells",
                "script-context-pass",
                "script-init-pass",
            ],
            "the A→B path contains no clear phase"
        );
        let a_registered = machine
            .owned_view_model_rebind_sink
            .peek_dirt()
            .contains(RuntimeCellDirt::BINDINGS);
        machine.owned_view_model_rebind_sink.take_dirt();
        context_b.mark_main_rebind_for_test();
        let b_registered = machine
            .owned_view_model_rebind_sink
            .peek_dirt()
            .contains(RuntimeCellDirt::BINDINGS);
        let current_view_model = machine
            .data_context()
            .and_then(|context| context.snapshot().main_handle().cloned())
            .expect("current inherited main")
            .borrow()
            .view_model_index();
        println!(
            "FLC5_INHERIT_DIFF current={current_view_model} a_registered={} b_registered={}",
            usize::from(a_registered),
            usize::from(b_registered)
        );
    }

    #[test]
    fn fl_c5_bind_shared_context_repoints_all_registered_machine_sinks() {
        let (file, mut artboard_a) = fl_c5_bind_file_and_artboard();
        let mut artboard_b = artboard_a.clone();
        let context = RuntimeStateMachineDataContext::default();
        let mut machine_a = scripted_listener_machine();
        let mut machine_b = scripted_listener_machine();
        machine_a.view_model_listeners.clear();
        machine_b.view_model_listeners.clear();

        machine_a
            .bind_data_context(&file, &mut artboard_a, Some(&context))
            .expect("bind shared context to A");
        machine_b
            .bind_data_context(&file, &mut artboard_b, Some(&context))
            .expect("bind shared context to B");
        machine_a.owned_view_model_rebind_sink.take_dirt();
        machine_b.owned_view_model_rebind_sink.take_dirt();
        artboard_a.artboard_owned_view_model_rebind_sink.take_dirt();
        artboard_b.artboard_owned_view_model_rebind_sink.take_dirt();

        let replacement = fl_c5_bind_handle(&file, 0);
        context.set_main(replacement.clone());
        assert!(
            machine_a
                .owned_view_model_rebind_sink
                .peek_dirt()
                .is_empty()
                && machine_b
                    .owned_view_model_rebind_sink
                    .peek_dirt()
                    .is_empty(),
            "slot replacement stages identity without scheduling a bind"
        );
        let detached_relay = context
            .main_rebind_dependent_for_test()
            .expect("replacement relay");
        let final_replacement = fl_c5_bind_handle(&file, 0);
        context.set_main(final_replacement.clone());
        assert!(
            !detached_relay.add_dirt(RuntimeCellDirt::BINDINGS),
            "the replaced handle's relay is dropped, making its weak registration inert"
        );
        context.mark_main_rebind_for_test();
        assert!(
            machine_a
                .owned_view_model_rebind_sink
                .peek_dirt()
                .contains(RuntimeCellDirt::BINDINGS)
        );
        assert!(
            machine_b
                .owned_view_model_rebind_sink
                .peek_dirt()
                .contains(RuntimeCellDirt::BINDINGS)
        );
        assert!(
            artboard_a
                .artboard_owned_view_model_rebind_sink
                .peek_dirt()
                .contains(RuntimeCellDirt::BINDINGS)
                && artboard_b
                    .artboard_owned_view_model_rebind_sink
                    .peek_dirt()
                    .contains(RuntimeCellDirt::BINDINGS),
            "the active relay forwards later structural dirt to every registered artboard"
        );
        assert!(
            context
                .snapshot()
                .main_handle()
                .is_some_and(|bound| bound.ptr_eq(&final_replacement)),
            "one mutable primary context retains the replacement identity for every dependent"
        );

        assert!(machine_a.complete_view_model_instances(Some(&file), &artboard_a));
        assert!(
            machine_b
                .global_view_model_instance(Some(&file), "Global A")
                .is_some(),
            "completion on one registered container mutates the shared slot table"
        );
    }

    #[test]
    fn fl_c5_bind_typed_context_apis_delegate_without_signature_changes() {
        let (file, _artboard) = fl_c5_bind_file_and_artboard();
        let mut machine = scripted_listener_machine();
        machine.view_model_listeners.clear();
        let main = fl_c5_bind_handle(&file, 0);
        let context_handle = RuntimeOwnedViewModelContextHandle::root(&file, main.clone());
        let mut contexts = RuntimeOwnedViewModelContext::from_main_handle(main.clone());
        assert!(contexts.set_global_slot_handle(&file, 1, fl_c5_bind_handle(&file, 2)));

        let _: bool = machine.bind_owned_view_model_handle(&main);
        assert!(machine.data_context().is_some());
        let _: bool = machine.bind_owned_view_model_context_handle(&context_handle);
        assert!(machine.owned_data_context.is_some());
        let _: bool = machine.bind_owned_view_model_contexts(&contexts);
        assert!(
            machine
                .primary_data_context
                .as_ref()
                .map(RuntimeStateMachineDataContext::snapshot)
                .as_ref()
                .and_then(|context| context.global_slot_handle(1))
                .is_some()
        );
        let _: bool = machine
            .bind_script_artboard_data_context(&ScriptArtboardDataContext::root(&context_handle));
        assert!(machine.owned_data_context.is_some());
    }

    #[test]
    fn fl_c5_focus_semantic_focus_state_and_owner_safe_focus_accessors() {
        let (_artboard, mut machine, _) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "focus state",
                methods: Vec::new(),
                handled: false,
                calls: Rc::new(RefCell::new(Vec::new())),
            }));
        machine.keyboard_listener_groups.clear();
        machine.publish_focusable_keyboard_capabilities();

        assert_eq!(
            machine.focus_state(),
            FocusState {
                has_focus: true,
                expects_keyboard_input: false,
            },
            "a focused FocusData without key/text listeners is not a keyboard consumer"
        );
        assert!(machine.internal_focus_manager());
        assert!(!machine.has_external_focus_manager());

        machine.keyboard_listener_groups.push(
            RuntimeKeyboardListenerGroup::scripted(1, 2, 90_901, true, false)
                .expect("keyboard-consuming focus group"),
        );
        machine.publish_focusable_keyboard_capabilities();
        assert_eq!(
            machine.focus_state(),
            FocusState {
                has_focus: true,
                expects_keyboard_input: true,
            }
        );
        let pending_focus_events = machine.queued_focus_events.len();
        assert!(
            !machine.set_focus(Some(1)),
            "setting the already-focused valid FocusData is a no-op"
        );
        assert!(machine.focus_state().has_focus);
        assert_eq!(machine.queued_focus_events.len(), pending_focus_events);

        assert!(machine.set_focus(None));
        assert_eq!(machine.focus_state(), FocusState::default());
        assert!(machine.set_focus(Some(1)));
        assert!(
            machine.set_focus(Some(usize::MAX)),
            "a missing owner-safe retained FocusData/node clears current focus"
        );
        assert_eq!(machine.focus_state(), FocusState::default());
    }

    #[test]
    fn fl_c5_focus_semantic_manager_switch_is_identity_noop_and_restores_internal() {
        let (_artboard, mut machine, _) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "internal",
                methods: Vec::new(),
                handled: false,
                calls: Rc::new(RefCell::new(Vec::new())),
            }));
        let (_parent_artboard, parent, _) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "external",
                methods: Vec::new(),
                handled: false,
                calls: Rc::new(RefCell::new(Vec::new())),
            }));
        machine.focus_manager_phase_trace.clear();
        machine.install_external_focus(&parent.focus, 90_910);
        assert!(machine.has_external_focus_manager());
        assert_eq!(machine.focus.owner_identity(), 90_910);
        assert_eq!(
            machine.focus_manager_phase_trace,
            [
                "clean-retained-tree",
                "assign-external",
                "select-retained-tree",
            ]
        );

        machine.focus_manager_phase_trace.clear();
        let same_manager_other_projection = parent.focus.external_for_owner(90_999);
        machine.install_external_focus(&same_manager_other_projection, 90_911);
        assert_eq!(
            machine.focus.owner_identity(),
            90_910,
            "the same shared manager is a no-op even through a different owner projection"
        );
        assert!(machine.focus_manager_phase_trace.is_empty());

        assert!(machine.clear_external_focus_manager());
        assert!(!machine.has_external_focus_manager());
        assert!(machine.internal_focus_manager());
        assert_eq!(
            machine.focus_manager_phase_trace,
            [
                "clean-retained-tree",
                "assign-internal",
                "select-retained-tree",
            ]
        );
        assert_eq!(
            machine.focus_state(),
            FocusState::default(),
            "cleanup clears old focus before external-to-null restores the retained internal manager"
        );
        assert!(machine.has_focus_nodes());
        assert!(
            !machine.clear_external_focus_manager(),
            "null-to-null is the same-manager no-op"
        );
    }

    #[test]
    fn fl_c5_focus_semantic_batches_snapshot_clear_and_keep_focus_then_semantic_fifo() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let calls = Rc::new(RefCell::new(Vec::new()));
        machine.listener_definitions = Arc::new(vec![
            scripted_test_listener(
                &mut machine,
                90_920,
                "focus",
                ListenerFailure::None,
                vec![RuntimeListenerType::Focus],
                &calls,
            ),
            scripted_test_listener(
                &mut machine,
                90_921,
                "semantic",
                ListenerFailure::None,
                vec![RuntimeListenerType::SemanticAction],
                &calls,
            ),
        ]);

        machine.queue_focus_event(0, true);
        machine.queue_focus_event(0, true);
        machine.queue_semantic_event(None, 0);
        machine.queue_semantic_event(Some(usize::MAX), 0);
        machine.queue_semantic_event(Some(1), 0);
        machine.queue_semantic_event(Some(1), 0);
        assert!(machine.needs_advance);

        assert!(machine.process_focus_events(&mut artboard, None));
        assert!(machine.queued_focus_events.is_empty());
        assert!(
            machine.process_semantic_events(&mut artboard, None),
            "null group/listener records are skipped without suppressing later valid duplicates"
        );
        assert!(machine.queued_semantic_events.is_empty());
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["focus", "focus", "semantic", "semantic"]
        );
    }

    #[test]
    fn fl_c5_focus_semantic_callback_generated_batches_obey_phase_snapshots() {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let calls = Rc::new(RefCell::new(Vec::new()));
        machine.listener_definitions = Arc::new(vec![
            scripted_test_listener(
                &mut machine,
                90_925,
                "focus",
                ListenerFailure::None,
                vec![RuntimeListenerType::Focus],
                &calls,
            ),
            scripted_test_listener(
                &mut machine,
                90_926,
                "semantic",
                ListenerFailure::None,
                vec![RuntimeListenerType::SemanticAction],
                &calls,
            ),
        ]);
        machine.queue_focus_event(0, true);
        machine.deferred_callback_probe = Some(RuntimeDeferredCallbackProbe::FocusQueuesSemantic {
            listener_index: Some(1),
            action_type: 0,
        });

        assert!(machine.process_deferred_listener_group_events(&mut artboard, None));
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|call| call.label)
                .collect::<Vec<_>>(),
            ["focus", "semantic"],
            "semantic work generated by a focus callback joins the later same-frame snapshot"
        );
        assert!(machine.queued_semantic_events.is_empty());

        calls.borrow_mut().clear();
        machine.queue_semantic_event(Some(1), 0);
        machine.deferred_callback_probe =
            Some(RuntimeDeferredCallbackProbe::SemanticQueuesSemantic {
                listener_index: Some(1),
                action_type: 0,
            });
        assert!(machine.process_semantic_events(&mut artboard, None));
        assert_eq!(calls.borrow().len(), 1);
        assert_eq!(
            machine.queued_semantic_events,
            [RuntimeQueuedSemanticEvent {
                listener_index: Some(1),
                action_type: 0,
            }],
            "semantic work generated inside the active semantic batch waits for a later frame"
        );
        assert!(machine.process_semantic_events(&mut artboard, None));
        assert_eq!(calls.borrow().len(), 2);
        assert!(machine.queued_semantic_events.is_empty());
    }

    #[test]
    fn fl_c5_focus_semantic_recorded_semantic_manager_boundaries_keep_call_order() {
        let mut machine = scripted_listener_machine();
        assert_eq!(
            machine.semantic_manager_selection(),
            RuntimeSemanticManagerSelection::None
        );
        assert!(!machine.semantic_manager());
        assert!(machine.enable_semantics());
        assert!(!machine.enable_semantics());
        assert!(machine.semantic_manager());
        assert_eq!(
            machine.semantic_manager_selection(),
            RuntimeSemanticManagerSelection::InternalRecorded
        );
        assert_eq!(
            machine.semantic_manager_phase_trace,
            ["create-internal-recorded-seam", "build-tree-recorded-seam"]
        );
        assert!(
            !machine.fire_semantic_action(77, 0),
            "node lookup and SemanticData callbacks stop at their recorded seams"
        );

        machine.semantic_manager_phase_trace.clear();
        assert!(machine.set_external_semantic_manager(Some(90_930), Some(4)));
        assert_eq!(
            machine.semantic_manager_phase_trace,
            [
                "clean-tree-recorded-seam",
                "assign-external",
                "build-tree-recorded-seam",
            ]
        );
        machine.semantic_manager_phase_trace.clear();
        assert!(
            !machine.set_external_semantic_manager(Some(90_930), Some(9)),
            "same manager identity is a no-op even when the desired parent changes"
        );
        assert!(machine.semantic_manager_phase_trace.is_empty());

        assert!(machine.set_external_semantic_manager(None, None));
        assert_eq!(
            machine.semantic_manager_selection(),
            RuntimeSemanticManagerSelection::InternalRecorded
        );

        let mut without_internal = scripted_listener_machine();
        assert!(without_internal.set_external_semantic_manager(Some(90_931), None));
        without_internal.semantic_manager_phase_trace.clear();
        assert!(
            !without_internal.enable_semantics(),
            "an already-selected external manager suppresses internal creation"
        );
        assert!(
            without_internal.semantic_manager_phase_trace.is_empty(),
            "external-first enable does not create or rebuild an internal manager"
        );
        assert!(without_internal.set_external_semantic_manager(None, None));
        assert_eq!(
            without_internal.semantic_manager_selection(),
            RuntimeSemanticManagerSelection::None
        );
        assert!(!without_internal.semantic_manager());
        assert!(!without_internal.fire_semantic_action(77, 99));
    }
}
