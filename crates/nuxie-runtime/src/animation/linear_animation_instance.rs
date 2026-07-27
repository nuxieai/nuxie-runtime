//! Direct Rust owner for pinned C++
//! `include/rive/animation/linear_animation_instance.hpp` and
//! `src/animation/linear_animation_instance.cpp`.

use super::{
    AnimationLoop, RuntimeKeyFrameValue, RuntimeKeyFrameValueContext, RuntimeKeyedCallback,
    RuntimeLinearAnimation,
};
use crate::data_bind_graph::{
    RuntimeDataBindGraph, RuntimeDataBindGraphApplyPhase, RuntimeDataBindGraphTarget,
};
use crate::{ArtboardInstance, StateMachineReportedEvent};
use std::collections::HashMap;
use std::sync::Arc;

/// Stable typed identity for one definition in an Artboard's immutable
/// LinearAnimation arena. C++ occurrences retain `const LinearAnimation*`;
/// Rust retains this non-dereferenceable handle and resolves it only through
/// the owning Artboard arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeLinearAnimationHandle(Option<usize>);

impl RuntimeLinearAnimationHandle {
    pub(crate) fn new(index: usize) -> Self {
        Self(Some(index))
    }

    pub(crate) fn empty() -> Self {
        Self(None)
    }

    pub(crate) fn resolve<'a>(
        self,
        definitions: &'a [RuntimeLinearAnimation],
        empty: &'a RuntimeLinearAnimation,
    ) -> Option<&'a RuntimeLinearAnimation> {
        match self.0 {
            Some(index) => definitions.get(index),
            None => Some(empty),
        }
    }

    pub fn index(self) -> usize {
        self.0.unwrap_or(usize::MAX)
    }

    pub(crate) fn definition_index(self) -> Option<usize> {
        self.0
    }
}

#[derive(Debug)]
pub struct LinearAnimationInstance {
    pub(super) animation: RuntimeLinearAnimationHandle,
    pub(crate) time: f32,
    pub(crate) speed_direction: f32,
    pub(crate) total_time: f32,
    pub(crate) last_total_time: f32,
    pub(crate) spilled_time: f32,
    pub(crate) direction: f32,
    pub(crate) did_loop: bool,
    /// C++ `m_loopValue`: `-1` means use the definition value.
    pub(crate) loop_value_override: i32,
    pub(super) key_frame_value_holders: Option<Box<HashMap<u32, RuntimeKeyFrameValue>>>,
    pub(super) key_frame_data_bind_graph: Option<Box<RuntimeDataBindGraph>>,
    key_frame_prototype_revision: u64,
}

impl Clone for LinearAnimationInstance {
    fn clone(&self) -> Self {
        Self {
            animation: self.animation,
            time: self.time,
            speed_direction: self.speed_direction,
            total_time: self.total_time,
            last_total_time: self.last_total_time,
            spilled_time: self.spilled_time,
            direction: self.direction,
            did_loop: self.did_loop,
            loop_value_override: self.loop_value_override,
            // Keyframe holders model C++'s per-LAI runtime-owned bind targets.
            // A copied LAI starts unbound; state transitions move the outgoing
            // instance when they need to preserve its concrete binding identity.
            key_frame_value_holders: None,
            key_frame_data_bind_graph: None,
            key_frame_prototype_revision: 0,
        }
    }
}

impl LinearAnimationInstance {
    pub(crate) fn new(
        animation: RuntimeLinearAnimationHandle,
        definition: &RuntimeLinearAnimation,
        speed_multiplier: f32,
    ) -> Self {
        Self {
            animation,
            time: definition.start_time_with_speed(speed_multiplier),
            speed_direction: if speed_multiplier >= 0.0 { 1.0 } else { -1.0 },
            total_time: 0.0,
            last_total_time: 0.0,
            spilled_time: 0.0,
            direction: 1.0,
            did_loop: false,
            loop_value_override: -1,
            key_frame_value_holders: None,
            key_frame_data_bind_graph: None,
            key_frame_prototype_revision: 0,
        }
    }

    fn initialize_key_frame_data_bind_graph(&mut self, prototype: &RuntimeDataBindGraph) {
        if self.key_frame_data_bind_graph.is_some() {
            return;
        }
        for target in &prototype.targets {
            let (global_id, value) = match target.target {
                RuntimeDataBindGraphTarget::KeyFrameNumber { global_id } => {
                    (global_id, RuntimeKeyFrameValue::Number(0.0))
                }
                RuntimeDataBindGraphTarget::KeyFrameColor { global_id } => {
                    (global_id, RuntimeKeyFrameValue::Color(0xFF1D1D1D))
                }
                RuntimeDataBindGraphTarget::KeyFrameBoolean { global_id } => {
                    (global_id, RuntimeKeyFrameValue::Boolean(false))
                }
                RuntimeDataBindGraphTarget::KeyFrameString { global_id } => {
                    (global_id, RuntimeKeyFrameValue::String(Vec::new()))
                }
                _ => continue,
            };
            self.add_key_frame_value_holder(global_id, value);
        }
        self.key_frame_data_bind_graph = Some(Box::new(prototype.clone_for_key_frame_instance()));
        self.key_frame_prototype_revision = prototype.key_frame_source_revision();
    }

    fn sync_key_frame_data_bind_graph(&mut self, prototype: &RuntimeDataBindGraph) {
        self.initialize_key_frame_data_bind_graph(prototype);
        if self.key_frame_prototype_revision == prototype.key_frame_source_revision() {
            return;
        }
        if let Some(graph) = self.key_frame_data_bind_graph.as_deref_mut() {
            graph.sync_key_frame_sources_from(prototype);
        }
        self.key_frame_prototype_revision = prototype.key_frame_source_revision();
    }

    fn apply_key_frame_data_bind_updates(
        &mut self,
        updates: Vec<(RuntimeDataBindGraphTarget, crate::RuntimeDataBindGraphValue)>,
    ) -> bool {
        let mut changed = false;
        for (target, value) in updates {
            let (global_id, value) = match (target, value) {
                (
                    RuntimeDataBindGraphTarget::KeyFrameNumber { global_id },
                    crate::RuntimeDataBindGraphValue::Number(value),
                ) => (global_id, RuntimeKeyFrameValue::Number(value)),
                (
                    RuntimeDataBindGraphTarget::KeyFrameColor { global_id },
                    crate::RuntimeDataBindGraphValue::Color(value),
                ) => (global_id, RuntimeKeyFrameValue::Color(value)),
                (
                    RuntimeDataBindGraphTarget::KeyFrameBoolean { global_id },
                    crate::RuntimeDataBindGraphValue::Boolean(value),
                ) => (global_id, RuntimeKeyFrameValue::Boolean(value)),
                (
                    RuntimeDataBindGraphTarget::KeyFrameString { global_id },
                    crate::RuntimeDataBindGraphValue::String(value),
                ) => (global_id, RuntimeKeyFrameValue::String(value)),
                _ => continue,
            };
            let Some(holder) = self.key_frame_value_holder_mut(global_id) else {
                continue;
            };
            if *holder != value {
                *holder = value;
                changed = true;
            }
        }
        changed
    }

    pub(crate) fn prepare_key_frame_data_binds(
        &mut self,
        prototype: Option<&RuntimeDataBindGraph>,
    ) -> bool {
        let Some(prototype) = prototype else {
            return false;
        };
        self.sync_key_frame_data_bind_graph(prototype);
        let updates = self
            .key_frame_data_bind_graph
            .as_deref_mut()
            .map(|graph| {
                graph.take_key_frame_binding_updates(
                    RuntimeDataBindGraphApplyPhase::BeforeStatefulAdvance,
                )
            })
            .unwrap_or_default();
        self.apply_key_frame_data_bind_updates(updates)
    }

    pub(crate) fn advance_key_frame_data_binds(
        &mut self,
        prototype: Option<&RuntimeDataBindGraph>,
        elapsed_seconds: f32,
    ) -> bool {
        let Some(prototype) = prototype else {
            return false;
        };
        let mut keep_going = false;
        let mut changed = self.prepare_key_frame_data_binds(Some(prototype));
        if let Some(graph) = self.key_frame_data_bind_graph.as_deref_mut() {
            let advance = graph.advance_stateful_converters(elapsed_seconds);
            changed |= advance.changed;
            keep_going |= advance.keep_going;
        }
        let updates = self
            .key_frame_data_bind_graph
            .as_deref_mut()
            .map(|graph| {
                graph.take_key_frame_binding_updates(
                    RuntimeDataBindGraphApplyPhase::AfterStatefulAdvance,
                )
            })
            .unwrap_or_default();
        changed |= self.apply_key_frame_data_bind_updates(updates);
        changed || keep_going
    }

    pub(crate) fn add_key_frame_value_holder(
        &mut self,
        key_frame_global_id: u32,
        value: RuntimeKeyFrameValue,
    ) {
        self.key_frame_value_holders
            .get_or_insert_with(|| Box::new(HashMap::new()))
            .insert(key_frame_global_id, value);
    }

    pub(crate) fn key_frame_value_holder(
        &self,
        key_frame_global_id: u32,
    ) -> Option<&RuntimeKeyFrameValue> {
        self.key_frame_value_holders
            .as_deref()?
            .get(&key_frame_global_id)
    }

    pub(crate) fn key_frame_value_holder_mut(
        &mut self,
        key_frame_global_id: u32,
    ) -> Option<&mut RuntimeKeyFrameValue> {
        self.key_frame_value_holders
            .as_deref_mut()?
            .get_mut(&key_frame_global_id)
    }

    pub(super) fn key_frame_value_context(&self) -> RuntimeKeyFrameValueContext<'_> {
        RuntimeKeyFrameValueContext {
            holders: self.key_frame_value_holders.as_deref(),
        }
    }

    pub(crate) fn apply(&self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        let Some(index) = self.animation.definition_index() else {
            // C++'s shared empty animation owns no KeyedObjects.
            return false;
        };
        let definitions = Arc::clone(&artboard.linear_animations);
        let Some(definition) = definitions.get(index) else {
            return false;
        };
        definition.apply_with_key_frame_values(
            artboard,
            self.time,
            mix,
            self.key_frame_value_context(),
        )
    }

    pub fn animation_index(&self) -> usize {
        self.animation.index()
    }

    pub(crate) fn animation_handle(&self) -> RuntimeLinearAnimationHandle {
        self.animation
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn speed_direction(&self) -> f32 {
        self.speed_direction
    }

    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    pub fn last_total_time(&self) -> f32 {
        self.last_total_time
    }

    pub fn spilled_time(&self) -> f32 {
        self.spilled_time
    }

    pub fn direction(&self) -> f32 {
        self.direction
    }

    pub fn set_direction(&mut self, direction: i32) {
        self.direction = if direction > 0 { 1.0 } else { -1.0 };
    }

    pub fn did_loop(&self) -> bool {
        self.did_loop
    }

    pub fn clear_spilled_time(&mut self) {
        self.spilled_time = 0.0;
    }

    pub fn loop_value(&self) -> Option<u64> {
        u64::try_from(self.loop_value_override).ok()
    }

    pub(crate) fn set_loop_value(&mut self, definition: &RuntimeLinearAnimation, value: i32) {
        if self.loop_value_override == value
            || (self.loop_value_override == -1 && definition.loop_value as i32 == value)
        {
            return;
        }
        self.loop_value_override = value;
    }

    pub(crate) fn set_time(&mut self, animation: &RuntimeLinearAnimation, value: f32) {
        if self.time == value {
            return;
        }
        self.time = value;
        let diff = self.total_time - self.last_total_time;
        let start = if animation.enable_work_area {
            animation.work_start as f32
        } else {
            0.0
        } * animation.fps_as_f32();
        self.total_time = value - start;
        self.last_total_time = self.total_time - diff;
        self.direction = 1.0;
    }

    pub(crate) fn reset(&mut self, animation: &RuntimeLinearAnimation, speed_multiplier: f32) {
        self.time = animation.start_time_with_speed(speed_multiplier);
    }

    pub fn directed_speed(&self, animation: &RuntimeLinearAnimation) -> f32 {
        self.direction * animation.speed
    }

    pub(crate) fn resolved_loop_kind(&self, animation: &RuntimeLinearAnimation) -> AnimationLoop {
        AnimationLoop::from_loop_value(if self.loop_value_override != -1 {
            self.loop_value_override
        } else {
            animation.loop_value as i32
        })
    }

    pub(crate) fn keep_going(&self, animation: &RuntimeLinearAnimation) -> bool {
        self.resolved_loop_kind(animation) != AnimationLoop::OneShot
            || (self.directed_speed(animation) > 0.0 && self.time < animation.end_seconds())
            || (self.directed_speed(animation) < 0.0 && self.time > animation.start_seconds())
    }

    pub(crate) fn keep_going_with_speed_multiplier(
        &self,
        animation: &RuntimeLinearAnimation,
        speed_multiplier: f32,
    ) -> bool {
        self.resolved_loop_kind(animation) != AnimationLoop::OneShot
            || (self.directed_speed(animation) * speed_multiplier > 0.0
                && self.time < animation.end_seconds())
            || (self.directed_speed(animation) * speed_multiplier < 0.0
                && self.time > animation.start_seconds())
    }

    pub(crate) fn advance(
        &mut self,
        animation: &RuntimeLinearAnimation,
        elapsed_seconds: f32,
    ) -> bool {
        self.advance_and_report(animation, elapsed_seconds, None, None)
    }

    pub(crate) fn advance_with_events(
        &mut self,
        animation: &RuntimeLinearAnimation,
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
        keyed_callbacks: &mut Vec<RuntimeKeyedCallback>,
    ) -> bool {
        self.advance_and_report(
            animation,
            elapsed_seconds,
            Some(reported_events),
            Some(keyed_callbacks),
        )
    }

    fn advance_and_report(
        &mut self,
        animation: &RuntimeLinearAnimation,
        elapsed_seconds: f32,
        mut reported_events: Option<&mut Vec<StateMachineReportedEvent>>,
        mut keyed_callbacks: Option<&mut Vec<RuntimeKeyedCallback>>,
    ) -> bool {
        let delta_seconds = elapsed_seconds * animation.speed * self.direction;
        self.spilled_time = 0.0;
        if delta_seconds == 0.0 {
            self.did_loop = false;
            return false;
        }

        self.last_total_time = self.total_time;
        self.total_time += delta_seconds.abs();
        let kill_spilled_time = !self.keep_going_with_speed_multiplier(animation, elapsed_seconds);

        let mut last_time = self.time;
        self.time += delta_seconds;
        if let (Some(events), Some(callbacks)) = (
            reported_events.as_deref_mut(),
            keyed_callbacks.as_deref_mut(),
        ) {
            animation.report_keyed_callbacks(
                last_time,
                self.time,
                self.speed_direction,
                false,
                events,
                callbacks,
            );
        }
        let fps = animation.fps_as_f32();
        let mut frames = self.time * fps;
        let start = animation.start_frame();
        let end = animation.end_frame();
        let range = end - start;
        let mut did_loop = false;
        let mut direction = if delta_seconds < 0.0 { -1 } else { 1 };

        match self.resolved_loop_kind(animation) {
            AnimationLoop::OneShot => {
                if direction == 1 && frames > end {
                    let delta_frames = delta_seconds * fps;
                    let spilled_frames_ratio = (frames - end) / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = end;
                    self.time = frames / fps;
                    did_loop = true;
                } else if direction == -1 && frames < start {
                    let delta_frames = (delta_seconds * fps).abs();
                    let spilled_frames_ratio = (start - frames) / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = start;
                    self.time = frames / fps;
                    did_loop = true;
                }
            }
            AnimationLoop::Loop => {
                if direction == 1 && frames >= end {
                    let delta_frames = delta_seconds * fps;
                    let remainder = (frames - start) % range;
                    let spilled_frames_ratio = remainder / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = start + remainder;
                    self.time = frames / fps;
                    did_loop = true;
                    if let (Some(events), Some(callbacks)) = (
                        reported_events.as_deref_mut(),
                        keyed_callbacks.as_deref_mut(),
                    ) {
                        animation.report_keyed_callbacks(
                            0.0,
                            self.time,
                            self.speed_direction,
                            false,
                            events,
                            callbacks,
                        );
                    }
                } else if direction == -1 && frames <= start {
                    let delta_frames = delta_seconds * fps;
                    let remainder = ((start - frames) % range).abs();
                    let spilled_frames_ratio = (remainder / delta_frames).abs();
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = end - remainder;
                    self.time = frames / fps;
                    did_loop = true;
                    if let (Some(events), Some(callbacks)) = (
                        reported_events.as_deref_mut(),
                        keyed_callbacks.as_deref_mut(),
                    ) {
                        animation.report_keyed_callbacks(
                            end / fps,
                            self.time,
                            self.speed_direction,
                            false,
                            events,
                            callbacks,
                        );
                    }
                }
            }
            AnimationLoop::PingPong => {
                let mut from_pong = true;
                loop {
                    if direction == 1 && frames >= end {
                        self.spilled_time = (frames - end) / fps;
                        frames = end + (end - frames);
                        last_time = end / fps;
                    } else if direction == -1 && frames < start {
                        self.spilled_time = (start - frames) / fps;
                        frames = start + (start - frames);
                        last_time = start / fps;
                    } else {
                        break;
                    }
                    self.time = frames / fps;
                    self.direction *= -1.0;
                    direction *= -1;
                    did_loop = true;
                    if let (Some(events), Some(callbacks)) = (
                        reported_events.as_deref_mut(),
                        keyed_callbacks.as_deref_mut(),
                    ) {
                        animation.report_keyed_callbacks(
                            last_time,
                            self.time,
                            self.speed_direction,
                            from_pong,
                            events,
                            callbacks,
                        );
                    }
                    from_pong = !from_pong;
                }
            }
        }

        if kill_spilled_time {
            self.spilled_time = 0.0;
        }
        self.did_loop = did_loop;
        self.keep_going_with_speed_multiplier(animation, elapsed_seconds)
    }
}

impl ArtboardInstance {
    pub fn linear_animation_instance(&self, index: usize) -> Option<LinearAnimationInstance> {
        self.linear_animation_instance_with_speed(index, 1.0)
    }

    pub fn linear_animation_instance_with_speed(
        &self,
        index: usize,
        speed_multiplier: f32,
    ) -> Option<LinearAnimationInstance> {
        let animation = self.linear_animation(index)?;
        Some(LinearAnimationInstance::new(
            RuntimeLinearAnimationHandle::new(index),
            animation,
            speed_multiplier,
        ))
    }

    pub fn advance_linear_animation_instance(
        &self,
        instance: &mut LinearAnimationInstance,
        elapsed_seconds: f32,
    ) -> bool {
        let Some(animation) = instance
            .animation_handle()
            .resolve(&self.linear_animations, &self.empty_linear_animation)
        else {
            return false;
        };
        instance.advance(animation, elapsed_seconds)
    }

    pub fn advance_linear_animation_instance_with_events(
        &mut self,
        instance: &mut LinearAnimationInstance,
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        let (mut changed, keyed_callbacks) = {
            let Some(animation) = instance
                .animation_handle()
                .resolve(&self.linear_animations, &self.empty_linear_animation)
            else {
                return false;
            };
            if !animation.has_keyed_callbacks {
                return instance.advance(animation, elapsed_seconds);
            }
            let mut keyed_callbacks = Vec::new();
            let changed = instance.advance_with_events(
                animation,
                elapsed_seconds,
                reported_events,
                &mut keyed_callbacks,
            );
            (changed, keyed_callbacks)
        };
        for callback in keyed_callbacks {
            changed |= self.apply_keyed_callback(callback);
        }
        changed
    }

    pub fn apply_linear_animation_instance(
        &mut self,
        instance: &LinearAnimationInstance,
        mix: f32,
    ) -> bool {
        instance.apply(self, mix)
    }

    pub fn linear_animation_instance_keep_going(&self, instance: &LinearAnimationInstance) -> bool {
        let Some(animation) = instance
            .animation_handle()
            .resolve(&self.linear_animations, &self.empty_linear_animation)
        else {
            return false;
        };
        instance.keep_going(animation)
    }

    pub(crate) fn linear_animation_instance_definition(
        &self,
        instance: &LinearAnimationInstance,
    ) -> Option<&RuntimeLinearAnimation> {
        instance
            .animation_handle()
            .resolve(&self.linear_animations, &self.empty_linear_animation)
    }

    pub(crate) fn linear_animation_definition(
        &self,
        handle: RuntimeLinearAnimationHandle,
    ) -> Option<&RuntimeLinearAnimation> {
        handle.resolve(&self.linear_animations, &self.empty_linear_animation)
    }
}
