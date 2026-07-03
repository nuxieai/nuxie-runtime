use crate::{
    ArtboardInstance, StateMachineReportedEvent, TransformProperty, color_lerp, mix_value,
};

// Mirrors src/animation/linear_animation.cpp plus keyed object/property keyframe sampling.
#[derive(Debug, Clone)]
pub struct RuntimeLinearAnimation {
    pub global_id: u32,
    pub name: Option<String>,
    pub fps: u64,
    pub duration: u64,
    pub speed: f32,
    pub loop_value: u64,
    pub work_start: u64,
    pub work_end: u64,
    pub enable_work_area: bool,
    pub quantize: bool,
    pub keyed_objects: Vec<RuntimeKeyedObject>,
}

impl RuntimeLinearAnimation {
    pub(crate) fn apply(&self, instance: &mut ArtboardInstance, seconds: f32, mix: f32) -> bool {
        let seconds = if self.quantize && self.fps != 0 {
            let fps = self.fps as f32;
            (seconds * fps).floor() / fps
        } else {
            seconds
        };

        let mut changed = false;
        for keyed_object in &self.keyed_objects {
            for keyed_property in &keyed_object.keyed_properties {
                if let Some(property) = keyed_property.transform_property {
                    let Some(current) =
                        instance.transform_property(keyed_object.target_local_id, property)
                    else {
                        continue;
                    };
                    let Some(value) =
                        keyed_property.double_value_at(seconds, self.fps, current, mix)
                    else {
                        continue;
                    };
                    changed |= instance.set_transform_property(
                        keyed_object.target_local_id,
                        property,
                        value,
                    );
                }
                if keyed_property.transform_property.is_none() && keyed_property.double_property {
                    let current = instance
                        .double_property(keyed_object.target_local_id, keyed_property.property_key)
                        .unwrap_or(keyed_property.double_source_value);
                    let Some(value) =
                        keyed_property.double_value_at(seconds, self.fps, current, mix)
                    else {
                        continue;
                    };
                    changed |= instance.set_double_property(
                        keyed_object.target_local_id,
                        keyed_property.property_key,
                        value,
                    );
                }
                if keyed_property.color_property {
                    let current = instance
                        .color_property(keyed_object.target_local_id, keyed_property.property_key)
                        .unwrap_or(keyed_property.color_source_value);
                    let Some(value) =
                        keyed_property.color_value_at(seconds, self.fps, current, mix)
                    else {
                        continue;
                    };
                    changed |= instance.set_color_property(
                        keyed_object.target_local_id,
                        keyed_property.property_key,
                        value,
                    );
                }
                if keyed_property.bool_property {
                    let Some(value) = keyed_property.bool_value_at(seconds, self.fps) else {
                        continue;
                    };
                    changed |= instance.set_bool_property(
                        keyed_object.target_local_id,
                        keyed_property.property_key,
                        value,
                    );
                }
                if keyed_property.uint_property {
                    let Some(value) = keyed_property.uint_value_at(seconds, self.fps) else {
                        continue;
                    };
                    changed |= instance.set_uint_property(
                        keyed_object.target_local_id,
                        keyed_property.property_key,
                        value,
                    );
                }
                if keyed_property.string_property {
                    let Some(value) = keyed_property.string_value_at(seconds, self.fps) else {
                        continue;
                    };
                    changed |= instance.set_string_property(
                        keyed_object.target_local_id,
                        keyed_property.property_key,
                        value,
                    );
                }
            }
        }
        changed
    }

    fn report_keyed_callbacks(
        &self,
        seconds_from: f32,
        seconds_to: f32,
        speed_direction: f32,
        from_pong: bool,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        let starting_time = self.start_time_with_speed(speed_direction);
        let is_at_start_frame = starting_time == seconds_from;

        if is_at_start_frame && from_pong {
            return;
        }

        for keyed_object in &self.keyed_objects {
            for keyed_property in &keyed_object.keyed_properties {
                keyed_property.report_keyed_callbacks(
                    seconds_from,
                    seconds_to,
                    self.fps,
                    is_at_start_frame,
                    reported_events,
                );
            }
        }
    }

    pub(crate) fn start_seconds(&self) -> f32 {
        self.frame_to_seconds(self.start_frame())
    }

    fn end_seconds(&self) -> f32 {
        self.frame_to_seconds(self.end_frame())
    }

    pub(crate) fn duration_seconds(&self) -> f32 {
        self.frame_to_seconds(self.duration as f32)
    }

    fn start_time_with_speed(&self, speed_multiplier: f32) -> f32 {
        if self.speed * speed_multiplier >= 0.0 {
            self.start_seconds()
        } else {
            self.end_seconds()
        }
    }

    fn fps_as_f32(&self) -> f32 {
        self.fps as f32
    }

    fn start_frame(&self) -> f32 {
        if self.enable_work_area {
            self.work_start as f32
        } else {
            0.0
        }
    }

    fn end_frame(&self) -> f32 {
        if self.enable_work_area {
            self.work_end as f32
        } else {
            self.duration as f32
        }
    }

    fn frame_to_seconds(&self, frame: f32) -> f32 {
        if self.fps == 0 {
            return 0.0;
        }
        frame / self.fps_as_f32()
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyedObject {
    pub global_id: u32,
    pub object_id: usize,
    pub target_local_id: usize,
    pub keyed_properties: Vec<RuntimeKeyedProperty>,
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyedProperty {
    pub global_id: u32,
    pub property_key: u16,
    pub transform_property: Option<TransformProperty>,
    pub double_property: bool,
    pub double_source_value: f32,
    pub color_property: bool,
    pub color_source_value: u32,
    pub bool_property: bool,
    pub bool_source_value: bool,
    pub uint_property: bool,
    pub string_property: bool,
    pub(crate) callback_event: Option<StateMachineReportedEvent>,
    pub key_frames: Vec<RuntimeKeyFrameDouble>,
    pub color_key_frames: Vec<RuntimeKeyFrameColor>,
    pub bool_key_frames: Vec<RuntimeKeyFrameBool>,
    pub uint_key_frames: Vec<RuntimeKeyFrameUint>,
    pub string_key_frames: Vec<RuntimeKeyFrameString>,
    pub(crate) callback_key_frames: Vec<RuntimeKeyFrameCallback>,
}

impl RuntimeKeyedProperty {
    fn double_value_at(&self, seconds: f32, fps: u64, current: f32, mix: f32) -> Option<f32> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds, fps);
        let value = if idx == 0 {
            self.key_frames[0].value
        } else if idx < self.key_frames.len() {
            let from = &self.key_frames[idx - 1];
            let to = &self.key_frames[idx];
            if seconds == to.seconds(fps) {
                to.value
            } else if from.interpolation_type == 0 {
                from.value
            } else if from.interpolator_id.is_some() {
                return None;
            } else {
                let from_seconds = from.seconds(fps);
                let to_seconds = to.seconds(fps);
                let frame_mix = if to_seconds == from_seconds {
                    1.0
                } else {
                    (seconds - from_seconds) / (to_seconds - from_seconds)
                };
                from.value + (to.value - from.value) * frame_mix
            }
        } else {
            self.key_frames.last()?.value
        };

        Some(mix_value(current, value, mix))
    }

    fn color_value_at(&self, seconds: f32, fps: u64, current: u32, mix: f32) -> Option<u32> {
        if self.color_key_frames.is_empty() {
            return None;
        }

        let idx = closest_key_frame_index(&self.color_key_frames, seconds, fps);
        let value = if idx == 0 {
            self.color_key_frames[0].value
        } else if idx < self.color_key_frames.len() {
            let from = &self.color_key_frames[idx - 1];
            let to = &self.color_key_frames[idx];
            if seconds == to.seconds(fps) {
                to.value
            } else if from.interpolation_type == 0 {
                from.value
            } else if from.interpolator_id.is_some() {
                return None;
            } else {
                let from_seconds = from.seconds(fps);
                let to_seconds = to.seconds(fps);
                let frame_mix = if to_seconds == from_seconds {
                    1.0
                } else {
                    (seconds - from_seconds) / (to_seconds - from_seconds)
                };
                color_lerp(from.value, to.value, frame_mix)
            }
        } else {
            self.color_key_frames.last()?.value
        };

        Some(color_lerp(current, value, mix))
    }

    fn bool_value_at(&self, seconds: f32, fps: u64) -> Option<bool> {
        if self.bool_key_frames.is_empty() {
            return None;
        }

        let idx = closest_key_frame_index(&self.bool_key_frames, seconds, fps);
        let value = if idx == 0 {
            self.bool_key_frames[0].value
        } else if idx < self.bool_key_frames.len() {
            let from = &self.bool_key_frames[idx - 1];
            let to = &self.bool_key_frames[idx];
            if seconds == to.seconds(fps) {
                to.value
            } else {
                from.value
            }
        } else {
            self.bool_key_frames.last()?.value
        };

        Some(value)
    }

    fn uint_value_at(&self, seconds: f32, fps: u64) -> Option<u64> {
        if self.uint_key_frames.is_empty() {
            return None;
        }

        let idx = closest_key_frame_index(&self.uint_key_frames, seconds, fps);
        let value = if idx == 0 {
            self.uint_key_frames[0].value
        } else if idx < self.uint_key_frames.len() {
            let from = &self.uint_key_frames[idx - 1];
            let to = &self.uint_key_frames[idx];
            if seconds == to.seconds(fps) {
                to.value
            } else {
                from.value
            }
        } else {
            self.uint_key_frames.last()?.value
        };

        Some(value)
    }

    fn string_value_at(&self, seconds: f32, fps: u64) -> Option<Vec<u8>> {
        if self.string_key_frames.is_empty() {
            return None;
        }

        let idx = closest_key_frame_index(&self.string_key_frames, seconds, fps);
        let value = if idx == 0 {
            &self.string_key_frames[0].value
        } else if idx < self.string_key_frames.len() {
            let from = &self.string_key_frames[idx - 1];
            let to = &self.string_key_frames[idx];
            if seconds == to.seconds(fps) {
                &to.value
            } else {
                &from.value
            }
        } else {
            &self.string_key_frames.last()?.value
        };

        Some(value.clone())
    }

    fn report_keyed_callbacks(
        &self,
        seconds_from: f32,
        seconds_to: f32,
        fps: u64,
        is_at_start_frame: bool,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        if self.callback_key_frames.is_empty() || seconds_from == seconds_to {
            return;
        }
        let Some(event) = self.callback_event.as_ref() else {
            return;
        };

        let is_forward = seconds_from <= seconds_to;
        let mut from_exact_offset = 0;
        let to_exact_offset = usize::from(is_forward);
        if is_forward {
            if !is_at_start_frame {
                from_exact_offset = 1;
            }
        } else if is_at_start_frame {
            from_exact_offset = 1;
        }

        let mut index = closest_key_frame_index_with_exact_offset(
            &self.callback_key_frames,
            seconds_from,
            fps,
            from_exact_offset,
        );
        let mut index_to = closest_key_frame_index_with_exact_offset(
            &self.callback_key_frames,
            seconds_to,
            fps,
            to_exact_offset,
        );
        if index_to < index {
            std::mem::swap(&mut index, &mut index_to);
        }

        while index_to > index {
            let key_frame = &self.callback_key_frames[index];
            let mut reported_event = event.clone();
            reported_event.seconds_delay = seconds_to - key_frame.seconds(fps);
            reported_events.push(reported_event);
            index += 1;
        }
    }

    fn closest_frame_index(&self, seconds: f32, fps: u64) -> usize {
        closest_key_frame_index(&self.key_frames, seconds, fps)
    }
}

trait RuntimeKeyFrameTiming {
    fn seconds(&self, fps: u64) -> f32;
}

fn closest_key_frame_index<T: RuntimeKeyFrameTiming>(
    key_frames: &[T],
    seconds: f32,
    fps: u64,
) -> usize {
    closest_key_frame_index_with_exact_offset(key_frames, seconds, fps, 0)
}

fn closest_key_frame_index_with_exact_offset<T: RuntimeKeyFrameTiming>(
    key_frames: &[T],
    seconds: f32,
    fps: u64,
    exact_offset: usize,
) -> usize {
    let last = key_frames.len() - 1;
    if seconds > key_frames[last].seconds(fps) {
        return key_frames.len();
    }

    let mut start = 0;
    let mut end = last;
    while start <= end {
        let mid = (start + end) >> 1;
        let closest = key_frames[mid].seconds(fps);
        if closest < seconds {
            start = mid + 1;
        } else if closest > seconds {
            if mid == 0 {
                break;
            }
            end = mid - 1;
        } else {
            return mid + exact_offset;
        }
    }
    start
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameDouble {
    pub global_id: u32,
    pub frame: u64,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: f32,
}

impl RuntimeKeyFrameDouble {
    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

impl RuntimeKeyFrameTiming for RuntimeKeyFrameDouble {
    fn seconds(&self, fps: u64) -> f32 {
        self.seconds(fps)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameColor {
    pub global_id: u32,
    pub frame: u64,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: u32,
}

impl RuntimeKeyFrameColor {
    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

impl RuntimeKeyFrameTiming for RuntimeKeyFrameColor {
    fn seconds(&self, fps: u64) -> f32 {
        self.seconds(fps)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameBool {
    pub global_id: u32,
    pub frame: u64,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: bool,
}

impl RuntimeKeyFrameBool {
    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

impl RuntimeKeyFrameTiming for RuntimeKeyFrameBool {
    fn seconds(&self, fps: u64) -> f32 {
        self.seconds(fps)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameUint {
    pub global_id: u32,
    pub frame: u64,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: u64,
}

impl RuntimeKeyFrameUint {
    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

impl RuntimeKeyFrameTiming for RuntimeKeyFrameUint {
    fn seconds(&self, fps: u64) -> f32 {
        self.seconds(fps)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameString {
    pub global_id: u32,
    pub frame: u64,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: Vec<u8>,
}

impl RuntimeKeyFrameString {
    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

impl RuntimeKeyFrameTiming for RuntimeKeyFrameString {
    fn seconds(&self, fps: u64) -> f32 {
        self.seconds(fps)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameCallback {
    pub global_id: u32,
    pub frame: u64,
}

impl RuntimeKeyFrameCallback {
    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

impl RuntimeKeyFrameTiming for RuntimeKeyFrameCallback {
    fn seconds(&self, fps: u64) -> f32 {
        self.seconds(fps)
    }
}

// Mirrors src/animation/linear_animation_instance.cpp and include/rive/animation/loop.hpp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnimationLoop {
    OneShot,
    Loop,
    PingPong,
}

impl AnimationLoop {
    pub(crate) fn from_loop_value(value: u64) -> Self {
        match value {
            1 => Self::Loop,
            2 => Self::PingPong,
            _ => Self::OneShot,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinearAnimationInstance {
    pub(crate) animation_index: usize,
    pub(crate) time: f32,
    pub(crate) speed_direction: f32,
    pub(crate) total_time: f32,
    pub(crate) last_total_time: f32,
    pub(crate) spilled_time: f32,
    pub(crate) direction: f32,
    pub(crate) did_loop: bool,
    pub(crate) loop_value: Option<u64>,
}

impl LinearAnimationInstance {
    pub(crate) fn new(
        animation_index: usize,
        animation: &RuntimeLinearAnimation,
        speed_multiplier: f32,
    ) -> Self {
        Self {
            animation_index,
            time: animation.start_time_with_speed(speed_multiplier),
            speed_direction: if speed_multiplier >= 0.0 { 1.0 } else { -1.0 },
            total_time: 0.0,
            last_total_time: 0.0,
            spilled_time: 0.0,
            direction: 1.0,
            did_loop: false,
            loop_value: None,
        }
    }

    pub fn animation_index(&self) -> usize {
        self.animation_index
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

    pub fn did_loop(&self) -> bool {
        self.did_loop
    }

    pub fn loop_value(&self) -> Option<u64> {
        self.loop_value
    }

    pub(crate) fn set_time(&mut self, animation: &RuntimeLinearAnimation, value: f32) {
        if self.time == value {
            return;
        }
        self.time = value;
        let diff = self.total_time - self.last_total_time;
        self.total_time = value - animation.start_seconds();
        self.last_total_time = self.total_time - diff;
        self.direction = 1.0;
    }

    pub fn directed_speed(&self, animation: &RuntimeLinearAnimation) -> f32 {
        self.direction * animation.speed
    }

    pub(crate) fn resolved_loop_kind(&self, animation: &RuntimeLinearAnimation) -> AnimationLoop {
        AnimationLoop::from_loop_value(self.loop_value.unwrap_or(animation.loop_value))
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
        self.advance_and_report(animation, elapsed_seconds, None)
    }

    pub(crate) fn advance_with_events(
        &mut self,
        animation: &RuntimeLinearAnimation,
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        self.advance_and_report(animation, elapsed_seconds, Some(reported_events))
    }

    fn advance_and_report(
        &mut self,
        animation: &RuntimeLinearAnimation,
        elapsed_seconds: f32,
        mut reported_events: Option<&mut Vec<StateMachineReportedEvent>>,
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
        if let Some(events) = reported_events.as_mut() {
            animation.report_keyed_callbacks(
                last_time,
                self.time,
                self.speed_direction,
                false,
                *events,
            );
        }
        let fps = animation.fps_as_f32();
        if fps == 0.0 {
            self.did_loop = false;
            return self.keep_going_with_speed_multiplier(animation, elapsed_seconds);
        }

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
                if range != 0.0 && direction == 1 && frames >= end {
                    let delta_frames = delta_seconds * fps;
                    let remainder = (frames - start) % range;
                    let spilled_frames_ratio = remainder / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = start + remainder;
                    self.time = frames / fps;
                    did_loop = true;
                    if let Some(events) = reported_events.as_mut() {
                        animation.report_keyed_callbacks(
                            0.0,
                            self.time,
                            self.speed_direction,
                            false,
                            *events,
                        );
                    }
                } else if range != 0.0 && direction == -1 && frames <= start {
                    let delta_frames = delta_seconds * fps;
                    let remainder = ((start - frames) % range).abs();
                    let spilled_frames_ratio = (remainder / delta_frames).abs();
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = end - remainder;
                    self.time = frames / fps;
                    did_loop = true;
                    if let Some(events) = reported_events.as_mut() {
                        animation.report_keyed_callbacks(
                            end / fps,
                            self.time,
                            self.speed_direction,
                            false,
                            *events,
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
                    if let Some(events) = reported_events.as_mut() {
                        animation.report_keyed_callbacks(
                            last_time,
                            self.time,
                            self.speed_direction,
                            from_pong,
                            *events,
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
