// Runtime instance orchestration for the C++ state machine path.
// Mirrors /Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp.
use super::*;

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
pub(super) struct AudioEventOccurrence {
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
pub(super) trait AudioEventSeam: std::fmt::Debug {
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

    pub(super) fn strongest(self, other: Self) -> Self {
        self.max(other)
    }
}

// Internal shorthand: the FL-ported listener pipeline reads like the pinned
// C++ when the local name matches C++'s `HitResult`.
pub(super) use self::RuntimeHitResult as HitResult;

pub(super) trait HitComponent: std::fmt::Debug {
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
pub(super) struct HitDrawable {
    component: Option<ComponentHandle>,
    drawable: Option<ComponentHandle>,
    pub(super) listeners: Vec<usize>,
    is_hovered: bool,
    pub(super) can_early_out: bool,
    pub(super) needs_down_listener: bool,
    pub(super) needs_up_listener: bool,
    is_opaque: bool,
}

impl HitDrawable {
    pub(super) fn new(
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

    pub(super) fn add_listener_impl(
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
pub(super) enum RuntimeNestedEventNotifierKind {
    StateMachine,
    LinearAnimation,
}

/// One value-owned registration corresponding to one C++ nested animation
/// notifier. Rust polls nested reports rather than storing a raw listener
/// back-pointer in the child, so retaining the exact source/notifier identity
/// here is the ownership-safe adaptation. `dispose` explicitly removes every
/// occurrence before this owner can receive another nested report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RuntimeNestedEventRegistration {
    pub(super) source_local_id: usize,
    pub(super) notifier_local_id: usize,
    pub(super) kind: RuntimeNestedEventNotifierKind,
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
pub(super) enum RuntimeSemanticManagerSelection {
    None,
    InternalRecorded,
    ExternalRecorded(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RuntimeSemanticOccurrenceKey {
    pub(crate) owner_identity: u64,
    pub(crate) data_local_id: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeSemanticRoute {
    pub(crate) owner_identity: u64,
    pub(crate) target_local_id: usize,
    pub(crate) data_local_id: usize,
}

pub(crate) fn closest_semantic_node(
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
pub(super) struct RuntimeQueuedFocusEvent {
    pub(super) listener_index: usize,
    pub(super) is_focus: bool,
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
pub(super) struct RuntimeQueuedSemanticEvent {
    pub(super) listener_index: Option<usize>,
    pub(super) action_type: u32,
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
pub(super) enum RuntimeDeferredCallbackProbe {
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
pub(super) enum RuntimeConstructorPhase {
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

pub(super) fn listener_property_path_for_resolved_name_path(
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

pub(super) fn resolved_listener_property_path_for_data_context(
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RuntimeStateMachineDataBindOccurrence {
    Ordinary {
        data_bind_index: usize,
    },
    ScriptedObject {
        action_binding_index: usize,
        input_index: usize,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimePointerInput {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) previous_x: f32,
    pub(super) previous_y: f32,
    pub(super) timestamp_seconds: f32,
    pub(super) id: i32,
}

impl RuntimeStateMachineListenerActionExecutor<'_> {}

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

pub(super) fn listener_types_use_report_queue(listener_types: &[RuntimeListenerType]) -> bool {
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
pub(super) struct RuntimeViewModelListenerInstance {
    /// Stable authored listener-definition arena plus index, matching C++'s
    /// retained `const StateMachineListener*`.
    pub(super) listener_definitions: Arc<Vec<RuntimeStateMachineListener>>,
    pub(super) listener_index: usize,
    pub(super) property_bindings: Vec<RuntimeViewModelListenerPropertyBinding>,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum RuntimeViewModelListenerSource {
    Single,
    Input(usize),
}

#[derive(Debug)]
pub(super) struct RuntimeViewModelListenerPropertyBinding {
    pub(super) source: RuntimeViewModelListenerSource,
    /// The retained scalar cell this listener's condition currently reads,
    /// with this listener's dirt sink
    /// registered as a dependent (C++ `ListenerViewModelPropertyBinding`,
    /// src/animation/state_machine_instance.cpp:1331-1407 at pin d788e8ec).
    /// `None` for list/view-model conditions and unresolved paths.
    pub(super) cell_binding: Option<RuntimeViewModelListenerCellBinding>,
}

impl RuntimeViewModelListenerInstance {
    pub(super) fn new(
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

    pub(super) fn listener(&self) -> &RuntimeStateMachineListener {
        &self.listener_definitions[self.listener_index]
    }

    fn actions(&self) -> &[RuntimeScheduledListenerAction] {
        &self.listener().listener_actions
    }

    pub(super) fn report_pending_trigger_bindings(
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
pub(super) struct RuntimeViewModelListenerCellBinding {
    pub(super) cell: RuntimeViewModelCell,
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

pub(super) fn relink_view_model_listener_cell(
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

pub(super) fn apply_scripted_input_update(
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

    pub(super) fn publish_focusable_keyboard_capabilities(&self) {
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

    pub(super) fn sort_hit_components(&mut self, artboard: &ArtboardInstance) {
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

    pub(super) fn initialize_authored_listener_categories(
        &mut self,
        artboard: &mut ArtboardInstance,
    ) {
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

    pub(super) fn ensure_scripted_input_groups_current(&mut self, artboard: &ArtboardInstance) {
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
    pub(in crate::state_machine) fn retain_protected_script_result<T>(
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

    /// Return the authored layer index corresponding to the Nth changed state.
    /// The ordering matches [`Self::changed_state`] and pinned C++
    /// `stateChangedByIndex`.
    pub fn changed_state_layer_index(&self, index: usize) -> Option<usize> {
        self.layers
            .iter()
            .enumerate()
            .filter(|(_, layer)| layer.state_changed_on_advance())
            .nth(index)
            .map(|(layer_index, _)| layer_index)
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
    pub(super) fn hit_components_count(&self) -> usize {
        self.hit_components.len()
    }

    #[cfg(test)]
    pub(super) fn hit_component(&self, index: usize) -> Option<&dyn HitComponent> {
        self.hit_components.get(index).map(Box::as_ref)
    }

    /// C++ exposes this only under TESTING. Bound against the retained machine
    /// definition before reading the occurrence, matching `layerState`.
    #[cfg(test)]
    pub(super) fn layer_state(&self, index: usize) -> Option<&RuntimeLayerState> {
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

    /// Pinned `FocusManager::{focusNext,focusPrevious,...}` evaluate
    /// eligibility live at every traversal query
    /// (`src/input/focus_manager.cpp:20-46`; `src/focus_data.cpp:511-560`),
    /// while the retained Rust tree bakes collapse/hidden/renderOpacity into
    /// its nodes. These artboard-bearing entry points resynchronize the bake
    /// before traversing, like `RuntimeFocusActionTraversal::perform`;
    /// otherwise a mount recorded before the first update pass would pin its
    /// subtree ineligible forever.
    pub fn focus_next(&mut self, artboard: &ArtboardInstance) -> bool {
        self.focus.refresh_visibility_change(artboard);
        self.change_focus(|focus| focus.traverse(0))
    }

    pub fn focus_previous(&mut self, artboard: &ArtboardInstance) -> bool {
        self.focus.refresh_visibility_change(artboard);
        self.change_focus(|focus| focus.traverse(1))
    }

    pub fn focus_up(&mut self, artboard: &ArtboardInstance) -> bool {
        self.focus.refresh_visibility_change(artboard);
        self.change_focus(|focus| focus.traverse(2))
    }

    pub fn focus_down(&mut self, artboard: &ArtboardInstance) -> bool {
        self.focus.refresh_visibility_change(artboard);
        self.change_focus(|focus| focus.traverse(3))
    }

    pub fn focus_left(&mut self, artboard: &ArtboardInstance) -> bool {
        self.focus.refresh_visibility_change(artboard);
        self.change_focus(|focus| focus.traverse(4))
    }

    pub fn focus_right(&mut self, artboard: &ArtboardInstance) -> bool {
        self.focus.refresh_visibility_change(artboard);
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

    pub(super) fn capture_focus_callbacks(&mut self) {
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

    pub(super) fn queue_focus_event(&mut self, listener_index: usize, is_focus: bool) {
        self.queued_focus_events.push(RuntimeQueuedFocusEvent {
            listener_index,
            is_focus,
        });
        self.needs_advance = true;
    }

    pub(super) fn queue_semantic_event(&mut self, listener_index: Option<usize>, action_type: u32) {
        self.queued_semantic_events
            .push(RuntimeQueuedSemanticEvent {
                listener_index,
                action_type,
            });
        self.needs_advance = true;
    }

    pub(super) fn semantic_manager_selection(&self) -> RuntimeSemanticManagerSelection {
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
    /// absent-manifest-row B6-0329 boundary against upstream
    /// `src/semantic/semantic_manager.cpp`.
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
        // Semantic focus requests route through `FocusManager::setFocus`,
        // whose eligibility gate re-walks live collapse/hidden/renderOpacity
        // state on every call (`src/input/focus_manager.cpp:118-141`;
        // `src/focus_data.cpp:511-560`). `request_semantic_focus` carries no
        // Artboard borrow, so the retained eligibility bake is resynchronized
        // here — the drain that constructs the semantic routes is its
        // artboard-bearing precondition.
        self.focus.refresh_visibility_change(artboard);
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
    pub(super) fn set_semantic_node_resolver(
        &mut self,
        resolver: Option<Rc<dyn SemanticNodeResolver>>,
    ) {
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
    pub(super) fn process_listener_group_event(
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
    pub(super) fn update_listeners(
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

    pub(super) fn dispatch_pointer_listener_type_for_target(
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

    /// Timestamp-bearing, fallible tri-state pointer-down seam for atomic
    /// foreign-function adapters. It preserves the exact C++ HitResult while
    /// keeping script failures explicit until the operation commits.
    pub fn try_pointer_down_hit_result_with_timestamp_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        host: &mut dyn ScriptHost,
    ) -> Result<RuntimeHitResult, ScriptError> {
        self.update_listeners(
            artboard,
            RuntimeListenerType::Down,
            x,
            y,
            pointer_id,
            timestamp_seconds,
            None,
            None,
            host,
        )
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

    /// Timestamp-bearing, fallible tri-state pointer-move seam.
    pub fn try_pointer_move_hit_result_with_timestamp_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        host: &mut dyn ScriptHost,
    ) -> Result<RuntimeHitResult, ScriptError> {
        self.update_listeners(
            artboard,
            RuntimeListenerType::Move,
            x,
            y,
            pointer_id,
            timestamp_seconds,
            None,
            None,
            host,
        )
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

    /// Timestamp-bearing, fallible tri-state pointer-up seam.
    pub fn try_pointer_up_hit_result_with_timestamp_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        host: &mut dyn ScriptHost,
    ) -> Result<RuntimeHitResult, ScriptError> {
        self.update_listeners(
            artboard,
            RuntimeListenerType::Up,
            x,
            y,
            pointer_id,
            timestamp_seconds,
            None,
            None,
            host,
        )
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

    /// Timestamp-bearing, fallible tri-state pointer-exit seam.
    pub fn try_pointer_exit_hit_result_with_timestamp_and_script_host(
        &mut self,
        artboard: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
        timestamp_seconds: f32,
        host: &mut dyn ScriptHost,
    ) -> Result<RuntimeHitResult, ScriptError> {
        self.update_listeners(
            artboard,
            RuntimeListenerType::Exit,
            x,
            y,
            pointer_id,
            timestamp_seconds,
            None,
            None,
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

    pub(super) fn process_deferred_listener_group_events(
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

    pub(super) fn process_focus_events(
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

    pub(super) fn process_semantic_events(
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

    pub(in crate::state_machine) fn perform_listener_actions(
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

    pub(in crate::state_machine) fn perform_listener_actions_with_event_context(
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
        changed |= artboard.sync_stateful_nested_view_model_contexts();

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
            // Pinned C++ resets the DataContext and Artboard after every pass,
            // then continues solely on Component dirt. DataBind bookkeeping is
            // not a second outer-loop continuation term
            // (`state_machine_instance.cpp:2689-2703`).
            if !artboard.has_dirt(ComponentDirt::COMPONENTS) {
                break;
            }
        }
        changed
    }
}

pub(super) fn runtime_owned_font_asset_value_for_state_machine_source(
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
