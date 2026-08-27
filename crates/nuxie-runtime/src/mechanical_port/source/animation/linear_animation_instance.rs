use crate::mechanical_port::source::animation::{
    keyed_callback_reporter::KeyedCallbackReporter, r#loop::Loop,
};
use std::collections::HashMap;
pub trait LinearAnimationInstanceDefinition {
    fn speed(&self) -> f32;
    fn fps(&self) -> u32;
    fn duration(&self) -> u32;
    fn duration_seconds(&self) -> f32;
    fn start_seconds(&self) -> f32;
    fn end_seconds(&self) -> f32;
    fn start_time(&self) -> f32;
    fn end_time(&self) -> f32;
    fn enable_work_area(&self) -> bool;
    fn work_start(&self) -> u32;
    fn work_end(&self) -> u32;
    fn loop_value(&self) -> i32;
    fn name(&self) -> &str;
    fn apply(&self, artboard: *mut (), time: f32, mix: f32, context: &LinearAnimationInstance);
    fn report_keyed_callbacks(
        &self,
        reporter: &mut dyn KeyedCallbackReporter,
        from: f32,
        to: f32,
        speed_direction: f32,
        from_pong: bool,
    );
}
pub trait LinearAnimationInstanceArtboard {
    fn advance(&mut self, seconds: f32) -> bool;
    fn is_translucent(&self, instance: &LinearAnimationInstance) -> bool;
    fn remove_and_delete_data_bind(&mut self, bind: *mut ());
    fn notify_event(&mut self, event: *mut (), delay: f32);
}
pub struct LinearAnimationInstance {
    animation: *const dyn LinearAnimationInstanceDefinition,
    artboard: *mut dyn LinearAnimationInstanceArtboard,
    time: f32,
    speed_direction: f32,
    total_time: f32,
    last_total_time: f32,
    spilled_time: f32,
    direction: f32,
    did_loop: bool,
    loop_value: i32,
    scripted_interpolators: Option<HashMap<usize, Box<dyn std::any::Any>>>,
    cloned_artboard_data_binds: Vec<*mut ()>,
    keyframe_value_holders: Option<HashMap<usize, Box<dyn std::any::Any>>>,
}
impl LinearAnimationInstance {
    pub fn new(
        animation: &dyn LinearAnimationInstanceDefinition,
        artboard: &mut dyn LinearAnimationInstanceArtboard,
        speed_multiplier: f32,
    ) -> Self {
        Self {
            animation,
            artboard,
            time: if speed_multiplier >= 0.0 {
                animation.start_time()
            } else {
                animation.end_time()
            },
            speed_direction: if speed_multiplier >= 0.0 { 1.0 } else { -1.0 },
            total_time: 0.0,
            last_total_time: 0.0,
            spilled_time: 0.0,
            direction: 1.0,
            did_loop: false,
            loop_value: -1,
            scripted_interpolators: None,
            cloned_artboard_data_binds: Vec::new(),
            keyframe_value_holders: None,
        }
    }
    fn animation(&self) -> &dyn LinearAnimationInstanceDefinition {
        unsafe { &*self.animation }
    }
    pub fn clear_spilled_time(&mut self) {
        self.spilled_time = 0.0
    }
    pub fn time(&self) -> f32 {
        self.time
    }
    pub fn direction(&self) -> f32 {
        self.direction
    }
    pub fn directed_speed(&self) -> f32 {
        self.direction * self.speed()
    }
    pub fn set_direction(&mut self, value: i32) {
        self.direction = if value > 0 { 1.0 } else { -1.0 }
    }
    pub fn set_time(&mut self, value: f32) {
        if self.time == value {
            return;
        }
        self.time = value;
        let difference = self.total_time - self.last_total_time;
        let start = (if self.animation().enable_work_area() {
            self.animation().work_start() as f32
        } else {
            0.0
        }) * self.animation().fps() as f32;
        self.total_time = value - start;
        self.last_total_time = self.total_time - difference;
        self.direction = 1.0
    }
    pub fn apply(&self, mix: f32) {
        self.animation()
            .apply(self.artboard.cast(), self.time, mix, self)
    }
    pub fn did_loop(&self) -> bool {
        self.did_loop
    }
    pub fn keep_going(&self) -> bool {
        self.keep_going_with_multiplier(1.0)
    }
    pub fn keep_going_with_multiplier(&self, m: f32) -> bool {
        self.loop_value() != Loop::OneShot as i32
            || (self.directed_speed() * m > 0.0 && self.time < self.animation().end_seconds())
            || (self.directed_speed() * m < 0.0 && self.time > self.animation().start_seconds())
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
    pub fn duration_seconds(&self) -> f32 {
        self.animation().duration_seconds()
    }
    pub fn fps(&self) -> u32 {
        self.animation().fps()
    }
    pub fn duration(&self) -> u32 {
        self.animation().duration()
    }
    pub fn speed(&self) -> f32 {
        self.animation().speed()
    }
    pub fn start_time(&self) -> f32 {
        self.animation().start_time()
    }
    pub fn name(&self) -> String {
        self.animation().name().to_owned()
    }
    pub fn loop_value(&self) -> i32 {
        if self.loop_value != -1 {
            self.loop_value
        } else {
            self.animation().loop_value()
        }
    }
    pub fn set_loop_value(&mut self, value: i32) {
        if self.loop_value == value
            || (self.loop_value == -1 && self.animation().loop_value() == value)
        {
            return;
        }
        self.loop_value = value
    }
    pub fn reset(&mut self, m: f32) {
        self.time = if m >= 0.0 {
            self.animation().start_time()
        } else {
            self.animation().end_time()
        }
    }
    pub fn add_keyframe_value_holder(&mut self, key: *const (), holder: Box<dyn std::any::Any>) {
        self.keyframe_value_holders
            .get_or_insert_with(HashMap::new)
            .insert(key as usize, holder);
    }
    pub fn keyframe_value_holder(&self, key: *const ()) -> Option<&dyn std::any::Any> {
        self.keyframe_value_holders
            .as_ref()?
            .get(&(key as usize))
            .map(Box::as_ref)
    }
    pub fn cache_scripted_interpolator(
        &mut self,
        key: *const (),
        value: Box<dyn std::any::Any>,
        binds: Vec<*mut ()>,
    ) {
        self.scripted_interpolators
            .get_or_insert_with(HashMap::new)
            .insert(key as usize, value);
        self.cloned_artboard_data_binds.extend(binds)
    }
    pub fn advance_and_apply(&mut self, seconds: f32) -> bool {
        let mut more = self.advance(seconds, None);
        self.apply(1.0);
        if unsafe { (&mut *self.artboard).advance(seconds) } {
            more = true
        }
        more || self.keep_going()
    }
    pub fn advance(
        &mut self,
        elapsed: f32,
        mut reporter: Option<&mut dyn KeyedCallbackReporter>,
    ) -> bool {
        let speed = self.animation().speed();
        let fps = self.animation().fps() as f32;
        let delta = elapsed * speed * self.direction;
        self.spilled_time = 0.0;
        if delta == 0.0 {
            self.did_loop = false;
            return false;
        }
        self.last_total_time = self.total_time;
        self.total_time += delta.abs();
        let kill = !self.keep_going_with_multiplier(elapsed);
        let mut last = self.time;
        self.time += delta;
        if let Some(r) = reporter.as_deref_mut() {
            self.animation()
                .report_keyed_callbacks(r, last, self.time, self.speed_direction, false)
        }
        let mut frames = self.time * fps;
        let start = if self.animation().enable_work_area() {
            self.animation().work_start() as f32
        } else {
            0.0
        };
        let end = if self.animation().enable_work_area() {
            self.animation().work_end() as f32
        } else {
            self.animation().duration() as f32
        };
        let range = end - start;
        let mut looped = false;
        let mut direction = if delta < 0.0 { -1 } else { 1 };
        match self.loop_value() {
            x if x == Loop::OneShot as i32 => {
                if direction == 1 && frames > end {
                    self.spilled_time = (frames - end) / (delta * fps) * elapsed;
                    frames = end;
                    self.time = frames / fps;
                    looped = true
                } else if direction == -1 && frames < start {
                    self.spilled_time = (start - frames) / (delta * fps).abs() * elapsed;
                    frames = start;
                    self.time = frames / fps;
                    looped = true
                }
            }
            x if x == Loop::Loop as i32 => {
                if direction == 1 && frames >= end {
                    let remainder = (frames - start) % range;
                    self.spilled_time = remainder / (delta * fps) * elapsed;
                    frames = start + remainder;
                    self.time = frames / fps;
                    looped = true;
                    if let Some(r) = reporter.as_deref_mut() {
                        self.animation().report_keyed_callbacks(
                            r,
                            0.0,
                            self.time,
                            self.speed_direction,
                            false,
                        )
                    }
                } else if direction == -1 && frames <= start {
                    let remainder = ((start - frames) % range).abs();
                    self.spilled_time = (remainder / (delta * fps)).abs() * elapsed;
                    frames = end - remainder;
                    self.time = frames / fps;
                    looped = true;
                    if let Some(r) = reporter.as_deref_mut() {
                        self.animation().report_keyed_callbacks(
                            r,
                            end / fps,
                            self.time,
                            self.speed_direction,
                            false,
                        )
                    }
                }
            }
            _ => {
                let mut from_pong = true;
                loop {
                    if direction == 1 && frames >= end {
                        self.spilled_time = (frames - end) / fps;
                        frames = end + (end - frames);
                        last = end / fps
                    } else if direction == -1 && frames < start {
                        self.spilled_time = (start - frames) / fps;
                        frames = start + (start - frames);
                        last = start / fps
                    } else {
                        break;
                    }
                    self.time = frames / fps;
                    self.direction *= -1.0;
                    direction *= -1;
                    looped = true;
                    if let Some(r) = reporter.as_deref_mut() {
                        self.animation().report_keyed_callbacks(
                            r,
                            last,
                            self.time,
                            self.speed_direction,
                            from_pong,
                        )
                    }
                    from_pong = !from_pong
                }
            }
        }
        if kill {
            self.spilled_time = 0.0
        }
        self.did_loop = looped;
        self.keep_going_with_multiplier(elapsed)
    }
    pub fn is_translucent(&self) -> bool {
        unsafe { (&*self.artboard).is_translucent(self) }
    }
    pub fn report_event(&mut self, event: *mut (), delay: f32) {
        unsafe { (&mut *self.artboard).notify_event(event, delay) }
    }
}
impl Clone for LinearAnimationInstance {
    fn clone(&self) -> Self {
        Self {
            animation: self.animation,
            artboard: self.artboard,
            time: self.time,
            speed_direction: self.speed_direction,
            total_time: self.total_time,
            last_total_time: self.last_total_time,
            spilled_time: self.spilled_time,
            direction: self.direction,
            did_loop: self.did_loop,
            loop_value: self.loop_value,
            scripted_interpolators: None,
            cloned_artboard_data_binds: Vec::new(),
            keyframe_value_holders: None,
        }
    }
}
impl Drop for LinearAnimationInstance {
    fn drop(&mut self) {
        for bind in self.cloned_artboard_data_binds.drain(..) {
            unsafe { (&mut *self.artboard).remove_and_delete_data_bind(bind) }
        }
        self.scripted_interpolators.take();
        self.keyframe_value_holders.take();
    }
}
