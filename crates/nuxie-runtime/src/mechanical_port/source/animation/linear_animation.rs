//! Runtime LinearAnimation owner translated against pinned Rive 160085c654874d35.

use crate::mechanical_port::source::{
    animation::{
        interpolating_keyframe::KeyFrameValueContext,
        keyed_callback_reporter::KeyedCallbackReporter,
        keyed_object::{KeyedObject, KeyedObjectContext},
        keyed_property::KeyedProperty,
        keyframe_cells::KeyedObjectCells,
        r#loop::Loop,
    },
    core::CoreHandle,
    generated::{
        animation::{keyframe_base::KeyFrameBase, linear_animation_base::LinearAnimationBase},
        artboard_base::ArtboardBase,
        core_registry::CoreRegistry,
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

    /// Apply a keyed object through the shared cells its animation recorded
    /// for it. A context without a cells path applies it by its handle.
    fn apply_keyed_object_cells(
        &mut self,
        object: &CoreHandle,
        cells: &KeyedObjectCells,
        time: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        let _ = cells;
        self.apply_keyed_object_borrowed(object, time, mix, context);
    }
}

/// One keyed object of an animation, through its cells when it has them.
fn apply_keyed_object_entry(
    artboard: &mut dyn LinearAnimationArtboard,
    object: &CoreHandle,
    cells: &Option<Rc<KeyedObjectCells>>,
    time: f32,
    mix: f32,
    context: Option<&dyn KeyFrameValueContext>,
) {
    match cells {
        Some(cells) => artboard.apply_keyed_object_cells(object, cells, time, mix, context),
        None => artboard.apply_keyed_object_borrowed(object, time, mix, context),
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
        let (objects, cells) = self.with(|animation| {
            if animation.base.quantize() {
                let fps = animation.base.fps() as f32;
                time = (time * fps).floor() / fps;
            }
            (
                animation.keyed_objects.snapshot(),
                animation.keyed_object_cells.snapshot(),
            )
        });
        for (object, cells) in objects.iter().zip(cells.iter()) {
            apply_keyed_object_entry(artboard, object, cells, time, mix, context);
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
        #[cfg(not(feature = "editor"))]
        if !self.with(|animation| animation.has_keyed_callbacks) {
            return;
        }
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

pub struct LinearAnimation {
    pub base: LinearAnimationBase,
    has_keyed_callbacks: bool,
    is_static_pose: bool,
    first_key_frame: u16,
    last_key_frame: u16,
    keyed_objects: LazyVector<CoreHandle>,
    // Each keyed object's shared cells, in step with `keyed_objects`; None
    // for one added by handle alone, which applies through that handle.
    keyed_object_cells: LazyVector<Option<Rc<KeyedObjectCells>>>,
}
impl Default for LinearAnimation {
    fn default() -> Self {
        Self {
            base: LinearAnimationBase::default(),
            has_keyed_callbacks: true,
            is_static_pose: false,
            first_key_frame: 0,
            last_key_frame: u16::MAX,
            keyed_objects: LazyVector::default(),
            keyed_object_cells: LazyVector::default(),
        }
    }
}
impl LinearAnimation {
    pub fn set_has_keyed_callbacks(&mut self, value: bool) {
        self.has_keyed_callbacks = value;
    }

    pub fn applies_key_frame_values_at(&self, mut seconds: f32) -> bool {
        if self.is_static_pose {
            return true;
        }
        let fps = self.base.fps() as f32;
        if self.base.quantize() {
            seconds = (seconds * fps).floor() / fps;
        }
        seconds <= self.first_key_frame as f32 / fps
            || (self.last_key_frame != u16::MAX && seconds >= self.last_key_frame as f32 / fps)
    }
    #[cfg(test)]
    pub fn delete_count() -> i32 {
        DELETE_COUNT.load(Ordering::Relaxed)
    }
    pub fn add_keyed_object(&mut self, v: CoreHandle) {
        self.add_keyed_object_with_cells(v, None);
    }
    /// Add a keyed object with the shared cells its application reads.
    pub(crate) fn add_keyed_object_with_cells(
        &mut self,
        v: CoreHandle,
        cells: Option<Rc<KeyedObjectCells>>,
    ) {
        self.keyed_objects.push_back(v);
        self.keyed_object_cells.push_back(cells);
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
            self.keyed_object_cells.remove(i);
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
        #[cfg(not(feature = "editor"))]
        {
            let mut first = u32::MAX;
            let mut last = 0;
            for object in self.keyed_objects.iter() {
                object
                    .with_downcast::<KeyedObject, _>(|object| {
                        for property in object.keyed_properties() {
                            property
                                .with_downcast::<KeyedProperty, _>(|property| {
                                    let frames = property.keyframes();
                                    if frames.len() < 2 {
                                        return;
                                    }
                                    let frame_number = |frame: &CoreHandle| {
                                        CoreRegistry::get_uint_handle(
                                            frame,
                                            KeyFrameBase::FRAME_PROPERTY_KEY as i32,
                                        )
                                        .expect("KeyedProperty retains a KeyFrame occurrence")
                                    };
                                    first = first.min(frame_number(&frames[0]));
                                    last = last.max(frame_number(&frames[frames.len() - 1]));
                                })
                                .expect("KeyedObject retains a KeyedProperty occurrence");
                        }
                    })
                    .expect("LinearAnimation retains a KeyedObject occurrence");
            }
            self.is_static_pose = self.keyed_objects.size() != 0 && first == u32::MAX;
            if first != u32::MAX {
                self.first_key_frame = first.min(u16::MAX as u32) as u16;
                self.last_key_frame = if last < u16::MAX as u32 {
                    last as u16
                } else {
                    u16::MAX
                };
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
        for (object, cells) in self.keyed_objects.iter().zip(self.keyed_object_cells.iter()) {
            apply_keyed_object_entry(artboard, object, cells, time, mix, context);
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
        #[cfg(not(feature = "editor"))]
        if !self.has_keyed_callbacks {
            return;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{
        animation::keyframe_double::KeyFrameDouble,
        artboard::Artboard,
        core::CoreArena,
        generated::animation::{
            keyed_object_base::KeyedObjectBase, keyed_property_base::KeyedPropertyBase,
        },
        importers::{
            keyed_object_importer::KeyedObjectImporter,
            linear_animation_importer::LinearAnimationImporter,
        },
    };

    #[test]
    fn imported_callback_presence_is_rearmed_only_by_callback_properties() {
        let arena = CoreArena::default();
        let animation = arena.insert(LinearAnimation::default());
        assert!(
            animation
                .with_downcast::<LinearAnimation, _>(|v| v.has_keyed_callbacks)
                .unwrap()
        );
        let importer = LinearAnimationImporter::new(animation.clone());
        assert!(
            !animation
                .with_downcast::<LinearAnimation, _>(|v| v.has_keyed_callbacks)
                .unwrap()
        );
        let mut stack = ImportStack::default();
        assert_eq!(
            stack.make_latest(LinearAnimationBase::TYPE_KEY, Some(Box::new(importer))),
            StatusCode::Ok
        );
        let object = arena.insert(KeyedObject::default());
        assert_eq!(
            stack.make_latest(
                KeyedObjectBase::TYPE_KEY,
                Some(Box::new(KeyedObjectImporter::new(object)))
            ),
            StatusCode::Ok
        );
        for (key, expected) in [(13, false), (395, true)] {
            let property = arena.insert(KeyedProperty::default());
            CoreRegistry::set_uint_handle(
                &property,
                KeyedPropertyBase::PROPERTY_KEY_PROPERTY_KEY as i32,
                key,
            );
            assert_eq!(
                property.with_downcast_mut::<KeyedProperty, _>(|v| v.import(&mut stack)),
                Some(StatusCode::Ok)
            );
            assert_eq!(
                animation.with_downcast::<LinearAnimation, _>(|v| v.has_keyed_callbacks),
                Some(expected)
            );
        }
    }

    #[test]
    fn keyframe_value_bounds_keep_default_sentinel_and_quantize_like_apply() {
        let mut animation = LinearAnimation::default();
        animation.base.set_fps_value(10);
        assert!(animation.applies_key_frame_values_at(0.0));
        assert!(!animation.applies_key_frame_values_at(1.0));
        assert!(!animation.applies_key_frame_values_at(f32::INFINITY));
        animation.first_key_frame = 10;
        animation.last_key_frame = 20;
        assert!(animation.applies_key_frame_values_at(1.0));
        assert!(!animation.applies_key_frame_values_at(1.05));
        assert!(animation.applies_key_frame_values_at(2.0));
        animation.base.set_quantize_value(true);
        assert!(animation.applies_key_frame_values_at(1.05));
        assert!(!animation.applies_key_frame_values_at(1.95));
        animation.is_static_pose = true;
        assert!(animation.applies_key_frame_values_at(1.5));
    }

    #[cfg(not(feature = "editor"))]
    fn animation_with_frame_ranges(arena: &CoreArena, ranges: &[&[u32]]) -> LinearAnimation {
        let mut object = KeyedObject::default();
        for range in ranges {
            let mut property = KeyedProperty::default();
            for number in *range {
                let frame = arena.insert(KeyFrameDouble::default());
                CoreRegistry::set_uint_handle(
                    &frame,
                    KeyFrameBase::FRAME_PROPERTY_KEY as i32,
                    *number,
                );
                property.add_key_frame(frame);
            }
            object.add_keyed_property(arena.insert(property));
        }
        let mut animation = LinearAnimation::default();
        animation.add_keyed_object(arena.insert(object));
        assert_eq!(
            animation.on_added_clean(&mut Artboard::default()),
            StatusCode::Ok
        );
        animation
    }

    #[cfg(not(feature = "editor"))]
    #[test]
    fn clean_collects_only_multi_frame_bounds_and_preserves_overflow_sentinel() {
        let arena = CoreArena::default();
        let animation = animation_with_frame_ranges(&arena, &[&[0], &[12, 24], &[6, 30]]);
        assert!(!animation.is_static_pose);
        assert_eq!(
            (animation.first_key_frame, animation.last_key_frame),
            (6, 30)
        );
        let animation = animation_with_frame_ranges(&arena, &[&[70_000, 80_000]]);
        assert_eq!(
            (animation.first_key_frame, animation.last_key_frame),
            (u16::MAX, u16::MAX)
        );
        let animation = animation_with_frame_ranges(&arena, &[&[9], &[90]]);
        assert!(animation.is_static_pose);
        assert!(animation.applies_key_frame_values_at(1000.0));
        let mut empty = LinearAnimation::default();
        assert_eq!(
            empty.on_added_clean(&mut Artboard::default()),
            StatusCode::Ok
        );
        assert!(!empty.is_static_pose);
    }
}
