//! Runtime LinearAnimation owner translated against pinned Rive 160085c654874d35.

use crate::mechanical_port::source::{
    animation::{
        interpolating_keyframe::KeyFrameValueContext,
        keyed_callback_reporter::KeyedCallbackReporter,
        keyed_object::{KeyedObject, KeyedObjectContext},
        r#loop::Loop,
    },
    core::CoreHandle,
    generated::{
        animation::linear_animation_base::LinearAnimationBase, artboard_base::ArtboardBase,
    },
    importers::{artboard_importer::ArtboardImporter, import_stack::ImportStack},
    lazy_vector::LazyVector,
    status_code::StatusCode,
};
#[cfg(test)]
use std::sync::atomic::{AtomicI32, Ordering};

use std::{cell::RefCell, rc::Rc};

fn positive_mod(value: f32, range: f32) -> f32 {
    debug_assert!(range > 0.0);
    crate::mechanical_port::source::math::math_types::positive_mod(value, range)
}

#[cfg(test)]
static DELETE_COUNT: AtomicI32 = AtomicI32::new(0);
pub trait LinearAnimationArtboard {
    fn apply_keyed_object(
        &mut self,
        object: CoreHandle,
        time: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    );

    /// Apply an occurrence whose identity is retained by the calling traversal.
    /// Existing custom implementations keep their owned-hook contract.
    fn apply_keyed_object_borrowed(
        &mut self,
        object: &CoreHandle,
        time: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        self.apply_keyed_object(object.clone(), time, mix, context);
    }
}

/// The two existing Rust representations of C++'s retained animation pointer.
/// Access guards are local to a read; reporting never retains them through a
/// callback, which may edit the animation definition synchronously.
#[derive(Clone)]
pub(crate) enum LinearAnimationOwner {
    Authored(CoreHandle),
    Runtime(Rc<RefCell<LinearAnimation>>),
}

impl LinearAnimationOwner {
    pub(crate) fn with<R>(&self, f: impl FnOnce(&LinearAnimation) -> R) -> R {
        match self {
            Self::Authored(animation) => animation
                .with_downcast::<LinearAnimation, _>(f)
                .expect("LinearAnimationInstance retains a LinearAnimation"),
            Self::Runtime(animation) => f(&animation.borrow()),
        }
    }

    pub(crate) fn with_mut<R>(&self, f: impl FnOnce(&mut LinearAnimation) -> R) -> R {
        match self {
            Self::Authored(animation) => animation
                .with_downcast_mut::<LinearAnimation, _>(f)
                .expect("LinearAnimationInstance retains a LinearAnimation"),
            Self::Runtime(animation) => f(&mut animation.borrow_mut()),
        }
    }

    pub(crate) fn apply(
        &self,
        artboard: &mut dyn LinearAnimationArtboard,
        mut time: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        let objects = self.with(|animation| {
            if animation.base.quantize() {
                let fps = animation.base.fps() as f32;
                time = (time * fps).floor() / fps;
            }
            animation.keyed_objects.snapshot()
        });
        for object in objects.iter() {
            artboard.apply_keyed_object_borrowed(object, time, mix, context);
        }
    }

    pub(crate) fn report_keyed_callbacks(
        &self,
        reporter: &mut dyn KeyedCallbackReporter,
        from: f32,
        to: f32,
        speed_direction: f32,
        from_pong: bool,
    ) {
        let (starting_time, objects) = self.with(|animation| {
            (
                animation.start_time_with_multiplier(speed_direction),
                animation.keyed_objects.snapshot(),
            )
        });
        let at_start = starting_time == from;
        if !at_start || !from_pong {
            for object in objects.iter() {
                KeyedObject::report_keyed_callbacks_occurrence(
                    object, reporter, from, to, at_start,
                );
            }
        }
    }
}

#[derive(Default)]
pub struct LinearAnimation {
    pub base: LinearAnimationBase,
    keyed_objects: LazyVector<CoreHandle>,
}
impl LinearAnimation {
    #[cfg(test)]
    pub fn delete_count() -> i32 {
        DELETE_COUNT.load(Ordering::Relaxed)
    }
    pub fn add_keyed_object(&mut self, v: CoreHandle) {
        self.keyed_objects.push_back(v)
    }
    pub fn keyed_objects(&self) -> &[CoreHandle] {
        self.keyed_objects.view()
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn KeyedObjectContext) -> StatusCode {
        let mut status = StatusCode::Ok;
        let mut failed = Vec::new();
        for i in 0..self.keyed_objects.size() {
            let code = self.keyed_objects.view()[i]
                .with_downcast_mut::<KeyedObject, _>(|object| object.on_added_dirty(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                if status == StatusCode::Ok || status == StatusCode::MissingObject {
                    status = code;
                }
                failed.push(i);
            }
        }
        for i in failed.into_iter().rev() {
            self.keyed_objects.remove(i);
        }
        if status == StatusCode::MissingObject {
            StatusCode::Ok
        } else {
            status
        }
    }
    pub fn on_added_clean(&mut self, context: &mut dyn KeyedObjectContext) -> StatusCode {
        for object in self.keyed_objects.iter() {
            let code = object
                .with_downcast_mut::<KeyedObject, _>(|object| object.on_added_clean(context))
                .unwrap_or(StatusCode::MissingObject);
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
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        if self.base.quantize() {
            let fps = self.base.fps() as f32;
            time = (time * fps).floor() / fps;
        }
        for object in self.keyed_objects.iter() {
            artboard.apply_keyed_object_borrowed(object, time, mix, context)
        }
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(i) = stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY) else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        i.add_animation(this);
        self.base.base.base.base.import(stack)
    }
    pub fn loop_kind(&self) -> Loop {
        Loop::from(self.base.loop_value())
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
            Loop::Loop => positive_mod(seconds, self.duration_seconds()) + self.start_time(),
            Loop::PingPong => {
                let local = positive_mod(seconds, self.duration_seconds());
                let direction = (seconds / self.duration_seconds()) as i32 % 2;
                if direction == 0 {
                    local + self.start_time()
                } else {
                    self.end_time() - local
                }
            }
            _ => unreachable!("source globalToLocalSeconds requires a named loop value"),
        }
    }
    pub fn get_object(&self, i: usize) -> Option<CoreHandle> {
        self.keyed_objects.view().get(i).cloned()
    }
    pub fn num_keyed_objects(&self) -> usize {
        self.keyed_objects.size()
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
            for object in self.keyed_objects.iter() {
                KeyedObject::report_keyed_callbacks_occurrence(object, r, from, to, at_start);
            }
        }
    }
}

impl Drop for LinearAnimation {
    fn drop(&mut self) {
        #[cfg(test)]
        DELETE_COUNT.fetch_add(1, Ordering::Relaxed);
    }
}

impl std::ops::Deref for LinearAnimation {
    type Target = LinearAnimationBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for LinearAnimation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
