//! Host ownership and naming adapters for the translated animation instance.

use crate::mechanical_port::source::{
    animation::linear_animation_instance::LinearAnimationInstance as NativeAnimation,
    artboard::RuntimeArtboardInstanceHandle, file::RuntimeFileHandle,
};

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
}
