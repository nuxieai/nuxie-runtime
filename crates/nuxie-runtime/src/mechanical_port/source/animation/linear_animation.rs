use crate::mechanical_port::source::{
    animation::{
        keyed_callback_reporter::KeyedCallbackReporter, keyed_object::KeyedObject, r#loop::Loop,
    },
    generated::{
        animation::linear_animation_base::LinearAnimationBase, artboard_base::ArtboardBase,
    },
    importers::{artboard_importer::ArtboardImporter, import_stack::ImportStack},
    status_code::StatusCode,
};
use std::ptr::NonNull;
pub trait LinearAnimationArtboard {
    fn apply_keyed_object(
        &mut self,
        object: &mut KeyedObject,
        time: f32,
        mix: f32,
        context: *const (),
    );
}
#[derive(Default)]
pub struct LinearAnimation {
    pub base: LinearAnimationBase,
    keyed_objects: Vec<Box<KeyedObject>>,
}
impl LinearAnimation {
    pub fn add_keyed_object(&mut self, v: Box<KeyedObject>) {
        self.keyed_objects.push(v)
    }
    pub fn on_added_dirty(&mut self, c: *mut ()) -> StatusCode {
        let mut status = StatusCode::Ok;
        let mut i = 0;
        while i < self.keyed_objects.len() {
            let code = self.keyed_objects[i].on_added_dirty_raw(c);
            if code != StatusCode::Ok {
                if status == StatusCode::Ok || status == StatusCode::MissingObject {
                    status = code;
                }
                self.keyed_objects.remove(i);
            } else {
                i += 1
            }
        }
        if status == StatusCode::MissingObject {
            StatusCode::Ok
        } else {
            status
        }
    }
    pub fn on_added_clean(&mut self, c: *mut ()) -> StatusCode {
        for o in &mut self.keyed_objects {
            let code = o.on_added_clean_raw(c);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn apply(
        &mut self,
        artboard: &mut dyn LinearAnimationArtboard,
        mut time: f32,
        mix: f32,
        context: *const (),
    ) {
        if self.base.quantize() {
            let fps = self.base.fps() as f32;
            time = (time * fps).floor() / fps;
        }
        for object in &mut self.keyed_objects {
            artboard.apply_keyed_object(object, time, mix, context)
        }
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(i) = stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY) else {
            return StatusCode::MissingObject;
        };
        i.add_animation(NonNull::from(&mut *self));
        self.base.base.import(stack)
    }
    pub fn loop_kind(&self) -> Loop {
        match self.base.loop_value() {
            0 => Loop::OneShot,
            1 => Loop::Loop,
            _ => Loop::PingPong,
        }
    }
    pub fn start_seconds(&self) -> f32 {
        (if self.base.enable_work_area() {
            self.base.work_start() as f32
        } else {
            0.0
        }) / self.base.fps() as f32
    }
    pub fn end_seconds(&self) -> f32 {
        (if self.base.enable_work_area() {
            self.base.work_end()
        } else {
            self.base.duration()
        }) as f32
            / self.base.fps() as f32
    }
    pub fn start_time(&self) -> f32 {
        if self.base.speed() >= 0.0 {
            self.start_seconds()
        } else {
            self.end_seconds()
        }
    }
    pub fn start_time_with_multiplier(&self, m: f32) -> f32 {
        if self.base.speed() * m >= 0.0 {
            self.start_seconds()
        } else {
            self.end_seconds()
        }
    }
    pub fn end_time(&self) -> f32 {
        if self.base.speed() >= 0.0 {
            self.end_seconds()
        } else {
            self.start_seconds()
        }
    }
    pub fn duration_seconds(&self) -> f32 {
        (self.end_seconds() - self.start_seconds()).abs()
    }
    pub fn global_to_local_seconds(&self, seconds: f32) -> f32 {
        match self.loop_kind() {
            Loop::OneShot => seconds + self.start_time(),
            Loop::Loop => seconds.rem_euclid(self.duration_seconds()) + self.start_time(),
            Loop::PingPong => {
                let local = seconds.rem_euclid(self.duration_seconds());
                let direction = (seconds / self.duration_seconds()) as i32 % 2;
                if direction == 0 {
                    local + self.start_time()
                } else {
                    self.end_time() - local
                }
            }
        }
    }
    pub fn get_object(&self, i: usize) -> Option<&KeyedObject> {
        self.keyed_objects.get(i).map(Box::as_ref)
    }
    pub fn num_keyed_objects(&self) -> usize {
        self.keyed_objects.len()
    }
    pub fn report_keyed_callbacks(
        &self,
        r: &mut dyn KeyedCallbackReporter,
        from: f32,
        to: f32,
        speed_direction: f32,
        from_pong: bool,
    ) {
        let start = self.start_time_with_multiplier(speed_direction);
        let at_start = start == from;
        if !at_start || !from_pong {
            for o in &self.keyed_objects {
                o.report_keyed_callbacks(r, from, to, at_start)
            }
        }
    }
}
