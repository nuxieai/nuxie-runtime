//! Runtime KeyedProperty owner translated against pinned Rive 160085c654874d35.

use std::rc::Rc;

use crate::mechanical_port::source::{
    animation::{
        interpolating_keyframe::KeyFrameValueContext,
        keyframe_cells::{KeyFrameCells, KeyFrameEntry, KeyedPropertyCells},
        keyframe_double::KeyFrameDouble,
        keyed_callback_reporter::KeyedCallbackReporter, keyed_object::KeyedObjectContext,
        keyframe_interpolator::KeyFrameInterpolator, linear_animation::LinearAnimation,
    },
    core::CoreHandle,
    generated::animation::{
        keyed_object_base::KeyedObjectBase, keyed_property_base::KeyedPropertyBase,
        linear_animation_base::LinearAnimationBase,
    },
    generated::core_registry::CoreRegistry,
    importers::{
        import_stack::ImportStack, keyed_object_importer::KeyedObjectImporter,
        linear_animation_importer::LinearAnimationImporter,
    },
    status_code::StatusCode,
};
#[derive(Default)]
pub struct KeyedProperty {
    pub base: KeyedPropertyBase,
    keyframes: Vec<CoreHandle>,
    // Each keyframe's seconds and interpolation type, by keyframe index.
    frame_cells: Vec<Rc<KeyFrameCells>>,
}
impl KeyedProperty {
    pub fn add_key_frame(&mut self, value: CoreHandle) {
        let cells = value
            .with(|frame| frame.as_key_frame().map(|frame| Rc::clone(frame.cells())))
            .flatten()
            .expect("KeyedProperty retains only KeyFrame-derived occurrences");
        self.add_key_frame_with_cells(value, cells);
    }

    /// Add a keyframe the caller already borrows, with its shared cells.
    pub(crate) fn add_key_frame_with_cells(&mut self, value: CoreHandle, cells: Rc<KeyFrameCells>) {
        self.keyframes.push(value);
        self.frame_cells.push(cells);
        // Upstream keeps this list on KeyedProperty itself (m_keyFrames), so
        // the property's own list is the authority and the shared cells only
        // mirror all of it. A base moved in from another property, as an
        // in-place track rebuild does, must not keep that property's frames.
        self.base.cells().set_frames(
            self.keyframes
                .iter()
                .zip(&self.frame_cells)
                .map(|(handle, cells)| KeyFrameEntry {
                    handle: handle.clone(),
                    cells: Rc::clone(cells),
                })
                .collect(),
        );
    }
    pub fn keyframes(&self) -> &[CoreHandle] {
        &self.keyframes
    }
    #[inline]
    fn keyframe_seconds(&self, index: usize) -> f32 {
        self.frame_cells[index].seconds()
    }
    fn closest_frame_index(&self, seconds: f32, offset: i32) -> i32 {
        let mut start = 0;
        let mut end = self.keyframes.len() as i32 - 1;
        if seconds > self.keyframe_seconds(end as usize) {
            return end + 1;
        }
        while start <= end {
            let mid = (start + end) >> 1;
            let value = self.keyframe_seconds(mid as usize);
            if value < seconds {
                start = mid + 1
            } else if value > seconds {
                end = mid - 1
            } else {
                return mid + offset;
            }
        }
        start
    }
    pub fn report_keyed_callbacks(
        &self,
        reporter: &mut dyn KeyedCallbackReporter,
        object: u32,
        from: f32,
        to: f32,
        at_start: bool,
    ) {
        if from == to {
            return;
        }
        let forward = from <= to;
        let from_offset = if (forward && !at_start) || (!forward && at_start) {
            1
        } else {
            0
        };
        let mut index = self.closest_frame_index(from, from_offset);
        let mut end = self.closest_frame_index(to, if forward { 1 } else { 0 });
        if end < index {
            std::mem::swap(&mut index, &mut end)
        }
        while end > index {
            let frame_seconds = self.keyframe_seconds(index as usize);
            reporter.report_keyed_callback(object, self.base.property_key(), to - frame_seconds);
            index += 1;
        }
    }
    /// The search bounds are computed once by the source routine. Each event
    /// then reads its current property key and keyframe time before dispatch.
    pub(crate) fn report_keyed_callbacks_occurrence(
        owner: &CoreHandle,
        reporter: &mut dyn KeyedCallbackReporter,
        object_id: u32,
        from: f32,
        to: f32,
        at_start: bool,
    ) {
        if from == to {
            return;
        }
        let forward = from <= to;
        let from_offset = if (forward && !at_start) || (!forward && at_start) {
            1
        } else {
            0
        };
        let (mut index, mut end) = owner
            .with_downcast::<Self, _>(|property| {
                (
                    property.closest_frame_index(from, from_offset),
                    property.closest_frame_index(to, if forward { 1 } else { 0 }),
                )
            })
            .expect("callback traversal retains its KeyedProperty");
        if end < index {
            std::mem::swap(&mut index, &mut end);
        }
        while end > index {
            let (property_key, frame_seconds) = owner
                .with_downcast::<Self, _>(|property| {
                    (
                        property.base.property_key(),
                        property.keyframe_seconds(index as usize),
                    )
                })
                .expect("callback traversal retains its KeyedProperty");
            reporter.report_keyed_callback(object_id, property_key, to - frame_seconds);
            index += 1;
        }
    }

    fn apply_selection(
        &self,
        seconds: f32,
        index: i32,
        frame_count: usize,
    ) -> (i32, CoreHandle, Option<CoreHandle>) {
        let property_key = self.base.property_key() as i32;
        if index == 0 {
            (property_key, self.keyframes[0].clone(), None)
        } else if index < frame_count as i32 {
            let from = &self.keyframes[index as usize - 1];
            let to = &self.keyframes[index as usize];
            if seconds == self.keyframe_seconds(index as usize) {
                (property_key, to.clone(), None)
            } else if self.frame_cells[index as usize - 1].interpolation_type() == 0 {
                (property_key, from.clone(), None)
            } else {
                (property_key, from.clone(), Some(to.clone()))
            }
        } else {
            (
                property_key,
                self.keyframes[index as usize - 1].clone(),
                None,
            )
        }
    }

    fn apply_selected(
        selection: (i32, CoreHandle, Option<CoreHandle>),
        object: CoreHandle,
        seconds: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        let (property_key, frame, next) = selection;
        frame.with(|keyframe| {
            if let Some(next) = next {
                keyframe.keyframe_interpolate(object, property_key, seconds, next, mix, context)
            } else {
                keyframe.keyframe_apply(object, property_key, mix, context)
            }
        });
    }

    /// Borrowed compatibility entry for callers that already resolved mixing.
    pub fn apply(
        &self,
        object: CoreHandle,
        seconds: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
        override_mix: bool,
    ) {
        assert!(!self.keyframes.is_empty());
        let frame_count = self.keyframes.len();
        let index = self.closest_frame_index(seconds, 0);
        let mix = if override_mix { 1.0 } else { mix };
        Self::apply_selected(
            self.apply_selection(seconds, index, frame_count),
            object,
            seconds,
            mix,
            context,
        );
    }

    /// apply_occurrence for a property entered through its shared cells: the
    /// same frame search before the host override and fresh key and frame
    /// reads after it, without borrowing the property.
    pub(crate) fn apply_cells(
        property: &KeyedPropertyCells,
        object: &CoreHandle,
        seconds: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
        artboard: &dyn KeyedObjectContext,
        interpolator_host: Option<&CoreHandle>,
    ) {
        // The frame lists are replaced, never edited, so one snapshot serves
        // both the search and the selection unless the host hook runs in
        // between; then the selection reads the current list.
        let searched = property.frames();
        assert!(!searched.is_empty());
        let frame_count = searched.len();
        let index = property.closest_frame_index(seconds, 0);
        let (frames, mix) = match interpolator_host {
            None => (searched, mix),
            Some(host) => {
                let mix = if artboard.overrides_keyed_interpolation(host, property.property_key()) {
                    1.0
                } else {
                    mix
                };
                drop(searched);
                (property.frames(), mix)
            }
        };
        let property_key = property.property_key() as i32;
        let (frame, next) = if index == 0 {
            (&frames[0], None)
        } else if index < frame_count as i32 {
            let from = &frames[index as usize - 1];
            let to = &frames[index as usize];
            if seconds == to.cells.seconds() {
                (to, None)
            } else if from.cells.interpolation_type() == 0 {
                (from, None)
            } else {
                (from, Some(to))
            }
        } else {
            (&frames[index as usize - 1], None)
        };
        match (next, frame.cells.double_value()) {
            // A double keyframe applies from its cells.
            (None, Some(stored)) => {
                KeyFrameDouble::apply_stored(&frame.handle, stored, object, property_key, mix, context);
            }
            (None, None) => {
                frame.handle.with(|keyframe| {
                    keyframe.keyframe_apply(object.clone(), property_key, mix, context)
                });
            }
            // A linear pair of double keyframes interpolates from its cells.
            (Some(next), Some(_))
                if !frame.cells.has_interpolator() && next.cells.double_value().is_some() =>
            {
                KeyFrameDouble::interpolate_stored(frame, next, object, property_key, seconds, mix, context);
            }
            (Some(next), _) => {
                let next = next.handle.clone();
                frame.handle.with(|keyframe| {
                    keyframe.keyframe_interpolate(object.clone(), property_key, seconds, next, mix, context)
                });
            }
        }
    }

    /// The source searches frames before the host override, then reads the
    /// current property key and selected frames after that synchronous call.
    /// Neither the host hook nor keyframe application retains a property loan.
    pub(crate) fn apply_occurrence(
        owner: &CoreHandle,
        object: CoreHandle,
        seconds: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
        artboard: &dyn KeyedObjectContext,
    ) {
        let interpolator_host = KeyFrameInterpolator::host_from(object.clone());
        Self::apply_occurrence_with_interpolator_host(
            owner,
            object,
            seconds,
            mix,
            context,
            artboard,
            interpolator_host.as_ref(),
        );
    }

    /// KeyedObject resolves the host once for all properties of its target.
    /// Keep the virtual override call per property, outside the property loan.
    pub(crate) fn apply_occurrence_with_interpolator_host(
        owner: &CoreHandle,
        object: CoreHandle,
        seconds: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
        artboard: &dyn KeyedObjectContext,
        interpolator_host: Option<&CoreHandle>,
    ) {
        let (frame_count, index, override_key) = owner
            .with_downcast::<Self, _>(|property| {
                assert!(!property.keyframes.is_empty());
                (
                    property.keyframes.len(),
                    property.closest_frame_index(seconds, 0),
                    property.base.property_key(),
                )
            })
            .expect("animation application retains its KeyedProperty");
        let mix = if interpolator_host
            .is_some_and(|host| artboard.overrides_keyed_interpolation(host, override_key))
        {
            1.0
        } else {
            mix
        };
        let selection = owner
            .with_downcast::<Self, _>(|property| {
                property.apply_selection(seconds, index, frame_count)
            })
            .expect("animation application retains its KeyedProperty");
        Self::apply_selected(selection, object, seconds, mix, context);
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn KeyedObjectContext) -> StatusCode {
        for frame in &self.keyframes {
            let s = frame
                .with_mut(|frame| frame.on_added_dirty(context))
                .unwrap_or(StatusCode::MissingObject);
            if s != StatusCode::Ok {
                return s;
            }
        }
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, context: &mut dyn KeyedObjectContext) -> StatusCode {
        for frame in &self.keyframes {
            let s = frame
                .with_mut(|frame| frame.on_added_clean(context))
                .unwrap_or(StatusCode::MissingObject);
            if s != StatusCode::Ok {
                return s;
            }
        }
        StatusCode::Ok
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(i) = stack.latest::<KeyedObjectImporter>(KeyedObjectBase::TYPE_KEY) else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        i.add_keyed_property(this, Rc::clone(self.base.cells()));
        if CoreRegistry::is_callback(self.base.property_key()) {
            if let Some(importer) =
                stack.latest::<LinearAnimationImporter>(LinearAnimationBase::TYPE_KEY)
            {
                importer
                    .animation()
                    .with_downcast_mut::<LinearAnimation, _>(|animation| {
                        animation.set_has_keyed_callbacks(true);
                    })
                    .expect("LinearAnimationImporter retains a LinearAnimation");
            }
        }
        self.base.base.import(stack)
    }
    pub fn first(&self) -> Option<CoreHandle> {
        self.keyframes.first().cloned()
    }
    pub fn num_key_frames(&self) -> usize {
        self.keyframes.len()
    }
    pub fn get_key_frame(&self, i: usize) -> Option<CoreHandle> {
        self.keyframes.get(i).cloned()
    }
    pub fn is_callback(&self) -> bool {
        CoreRegistry::is_callback(self.base.property_key())
    }
}

impl std::ops::Deref for KeyedProperty {
    type Target = KeyedPropertyBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for KeyedProperty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
