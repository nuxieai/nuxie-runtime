//! Host ownership and naming adapters for the translated animation instance.

use crate::host_state_machine::StateMachineReportedEvent;
use crate::mechanical_port::source::{
    animation::{
        keyed_callback_reporter::KeyedCallbackReporter,
        linear_animation_instance::LinearAnimationInstance as NativeAnimation,
        state_machine_instance::EventReport,
    },
    artboard::RuntimeArtboardInstanceHandle,
    file::RuntimeFileHandle,
    generated::event_base::EventBase,
};

#[derive(Clone, Debug)]
pub struct RuntimeLinearAnimationAdvanceResult {
    pub changed: bool,
    pub keep_going: bool,
    pub reported_events: Vec<StateMachineReportedEvent>,
}

#[derive(Default)]
struct PendingKeyedCallbacks(Vec<(u32, u32, f32)>);

impl KeyedCallbackReporter for PendingKeyedCallbacks {
    fn report_keyed_callback(&mut self, object_id: u32, property_key: u32, elapsed_seconds: f32) {
        self.0.push((object_id, property_key, elapsed_seconds));
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct AnimationStateObservation {
    time: u32,
    direction: u32,
    total_time: u32,
    last_total_time: u32,
    spilled_time: u32,
    did_loop: bool,
}

pub struct LinearAnimationInstance {
    native: Box<NativeAnimation>,
    artboard: RuntimeArtboardInstanceHandle,
    file: RuntimeFileHandle,
    index: usize,
}

impl std::fmt::Debug for LinearAnimationInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinearAnimationInstance")
            .field("index", &self.index)
            .field("name", &self.name())
            .field("time", &self.time())
            .finish()
    }
}

impl LinearAnimationInstance {
    pub(crate) fn from_native(
        file: RuntimeFileHandle,
        artboard: RuntimeArtboardInstanceHandle,
        index: usize,
        native: Box<NativeAnimation>,
    ) -> Self {
        Self {
            native,
            artboard,
            file,
            index,
        }
    }

    pub fn native(&self) -> &NativeAnimation {
        &self.native
    }
    pub fn native_mut(&mut self) -> &mut NativeAnimation {
        &mut self.native
    }
    pub fn native_artboard(&self) -> RuntimeArtboardInstanceHandle {
        self.artboard.clone()
    }
    pub fn native_file(&self) -> RuntimeFileHandle {
        self.file.clone()
    }
    pub fn animation_index(&self) -> usize {
        self.index
    }
    pub fn name(&self) -> String {
        self.native.name()
    }
    pub fn time(&self) -> f32 {
        self.native.time()
    }
    pub fn set_time(&mut self, value: f32) {
        self.native.set_time(value);
    }
    pub fn total_time(&self) -> f32 {
        self.native.total_time()
    }
    pub fn last_total_time(&self) -> f32 {
        self.native.last_total_time()
    }
    pub fn spilled_time(&self) -> f32 {
        self.native.spilled_time()
    }
    pub fn clear_spilled_time(&mut self) {
        self.native.clear_spilled_time();
    }
    pub fn direction(&self) -> f32 {
        self.native.direction()
    }
    pub fn set_direction(&mut self, direction: i32) {
        self.native.set_direction(direction);
    }
    pub fn did_loop(&self) -> bool {
        self.native.did_loop()
    }
    pub fn loop_value(&self) -> i32 {
        self.native.loop_value()
    }
    pub fn set_loop_value(&mut self, value: i32) {
        self.native.set_loop_value(value);
    }
    pub fn duration_seconds(&self) -> f32 {
        self.native.duration_seconds()
    }
    pub fn duration(&self) -> u32 {
        self.native.duration()
    }
    pub fn fps(&self) -> u32 {
        self.native.fps()
    }
    pub fn speed(&self) -> f32 {
        self.native.speed()
    }
    pub fn directed_speed(&self) -> f32 {
        self.native.directed_speed()
    }
    pub fn start_time(&self) -> f32 {
        self.native.start_time()
    }
    pub fn global_to_local_seconds(&self, seconds: f32) -> f32 {
        self.native.global_to_local_seconds(seconds)
    }
    pub fn keep_going(&self) -> bool {
        self.native.keep_going()
    }
    pub fn reset(&mut self, speed_multiplier: f32) {
        self.native.reset(speed_multiplier);
    }
    pub fn advance(&mut self, seconds: f32) -> bool {
        self.native.advance_and_report_to_self(seconds)
    }
    pub fn apply(&self, mix: f32) {
        self.native.apply(mix);
    }
    pub fn advance_and_apply(&mut self, seconds: f32) -> bool {
        self.native.advance_and_apply(seconds)
    }

    fn state_observation(&self) -> AnimationStateObservation {
        AnimationStateObservation {
            time: self.native.time().to_bits(),
            direction: self.native.direction().to_bits(),
            total_time: self.native.total_time().to_bits(),
            last_total_time: self.native.last_total_time().to_bits(),
            spilled_time: self.native.spilled_time().to_bits(),
            did_loop: self.native.did_loop(),
        }
    }

    /// Host adaptation of pinned `LinearAnimationInstance::advanceAndApply`.
    ///
    /// The translated owner cannot borrow itself as the keyed callback reporter
    /// while it advances, so callbacks are retained in source order and replayed
    /// immediately afterward. Event observations are taken from that exact
    /// reporter stream before the callback is replayed through the native
    /// instance, which preserves nested event notification and other callback
    /// side effects.
    pub(crate) fn advance_and_apply_with_observed_events(
        &mut self,
        seconds: f32,
    ) -> RuntimeLinearAnimationAdvanceResult {
        let before = self.state_observation();
        let mut callbacks = PendingKeyedCallbacks::default();
        let animation_more = self.native.advance(seconds, Some(&mut callbacks));
        let mut reported_events = Vec::new();

        for (object_id, property_key, elapsed_seconds) in callbacks.0 {
            let target = self
                .artboard
                .with_artboard(|artboard| artboard.base.resolve_handle(object_id));
            if property_key == u32::from(EventBase::TRIGGER_PROPERTY_KEY)
                && let Some(event) = target.as_ref()
                && event.is_type_of(EventBase::TYPE_KEY)
                && let Some(report) = StateMachineReportedEvent::from_native(
                    EventReport {
                        event: Some(event.clone()),
                        seconds_delay: elapsed_seconds,
                    },
                    &self.artboard,
                    None,
                )
            {
                reported_events.push(report);
            }

            KeyedCallbackReporter::report_keyed_callback(
                &mut *self.native,
                object_id,
                property_key,
                elapsed_seconds,
            );
        }

        self.native.apply(1.0);
        let artboard_changed = self.artboard.advance_default(seconds);
        let keep_going = animation_more || artboard_changed || self.native.keep_going();
        let changed =
            before != self.state_observation() || artboard_changed || !reported_events.is_empty();

        RuntimeLinearAnimationAdvanceResult {
            changed,
            keep_going,
            reported_events,
        }
    }

    pub(crate) fn apply_at_and_settle(&mut self, time: f32, mix: f32) -> bool {
        let before = self.state_observation();
        self.native.set_time(time);
        self.native.apply(mix);
        let animation_changed = before != self.state_observation();
        let components_changed = self.artboard.update_pass(true);
        animation_changed || components_changed
    }
}
