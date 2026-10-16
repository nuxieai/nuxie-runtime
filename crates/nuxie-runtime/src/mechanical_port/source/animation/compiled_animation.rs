//! A LinearAnimation compiled for one LinearAnimationInstance: its keyed
//! objects and properties as flat tracks, each property's keyframe times,
//! values and interpolation kinds in contiguous arrays, and where each
//! property's target stores it.
//!
//! LinearAnimation::apply walks the animation's keyed objects through their
//! shared cells on every apply: it snapshots the object and property lists,
//! resolves each target by id, then for each property gathers the frame list,
//! searches the frame times and reads the selected keyframes' cells before
//! setting the value through CoreRegistry, which looks up where the target's
//! type stores that key. A CompiledAnimation reads all of that once. It is
//! rebuilt whenever keyed animation data changes anywhere (see
//! `keyed_data_changed`), so its arrays always equal what those cells hold;
//! targets are still resolved on every apply, and each property's sink is
//! re-resolved when its target changes.
//!
//! An apply computes each value from the arrays exactly as the cells path
//! would, and writes it to the same place in the same order. Whatever the
//! arrays do not cover (keyframes other than doubles, scripted
//! interpolators, bound keyframe values, blend accumulators) takes the cells
//! path for that property.

use std::{cell::Cell, rc::Rc};

use crate::mechanical_port::source::{
    animation::{
        interpolating_keyframe::KeyFrameValueContext,
        keyed_object::{KeyedObject, KeyedObjectContext},
        keyed_property::{select_frames, KeyedProperty},
        keyframe_cells::{closest_time_index, KeyFrameEntry, KeyedObjectCells, KeyedPropertyCells},
        keyframe_double::{interpolated_value, DoubleTarget, KeyFrameDouble, RegistryDouble, SinkDouble},
        keyframe_interpolator::KeyFrameInterpolator,
        linear_animation::LinearAnimationArtboard,
    },
    component_graph::PlainInputSink,
    core::CoreHandle,
    generated::core_registry::CoreRegistry,
    lazy_vector::LazyVectorSnapshot,
};

thread_local! {
    // Counts changes to keyed animation data on this thread.
    static KEYED_DATA_EPOCH: Cell<u64> = const { Cell::new(0) };
}

/// Keyed animation data changed: a keyframe's time, value, interpolation type
/// or interpolator presence, a keyed property's key or frames, a keyed
/// object's target id or properties, or an animation's keyed objects. Every
/// compiled animation rebuilds on its next apply.
#[inline]
pub(crate) fn keyed_data_changed() {
    KEYED_DATA_EPOCH.with(|epoch| epoch.set(epoch.get().wrapping_add(1)));
}

#[inline]
pub(crate) fn keyed_data_epoch() -> u64 {
    KEYED_DATA_EPOCH.with(Cell::get)
}

#[derive(Default)]
pub(crate) struct CompiledAnimation {
    // The epoch `objects` was built at; None before the first build.
    epoch: Option<u64>,
    objects: Vec<CompiledEntry>,
}

enum CompiledEntry {
    /// A keyed object recorded without cells, applied by its handle.
    Borrowed(CoreHandle),
    Object(CompiledObject),
}

struct CompiledObject {
    handle: CoreHandle,
    cells: Rc<KeyedObjectCells>,
    object_id: u32,
    tracks: Vec<Track>,
    // The target the tracks' sinks were resolved for.
    target: Option<CoreHandle>,
}

struct Track {
    property: Rc<KeyedPropertyCells>,
    property_key: u32,
    callback: bool,
    frames: Rc<[KeyFrameEntry]>,
    times: Box<[f32]>,
    keys: Box<[FrameKey]>,
    // Where the target stores this property, when it is a plain transform
    // input; resolved with the target.
    sink: Option<PlainInputSink>,
}

/// What application reads from one keyframe besides its time.
struct FrameKey {
    // A KeyFrameDouble's value; None for any other keyframe.
    value: Option<f32>,
    // Interpolation type hold (0).
    holds: bool,
    interpolator: Interpolation,
}

/// How a keyframe interpolates to the next.
enum Interpolation {
    /// No interpolator object: linear.
    Linear,
    /// A built-in interpolator, which transforms under one loan.
    BuiltIn(CoreHandle),
    /// A scripted interpolator, or one that is not live: the cells path.
    Other,
}

impl CompiledAnimation {
    /// LinearAnimation::apply for the animation whose keyed objects and
    /// cells `snapshot` reads (called only when the compiled form is stale),
    /// at `time` already quantized as LinearAnimation::apply quantizes it.
    pub(crate) fn apply(
        &mut self,
        snapshot: impl FnOnce() -> (LazyVectorSnapshot<CoreHandle>, LazyVectorSnapshot<Option<Rc<KeyedObjectCells>>>),
        artboard: &mut dyn LinearAnimationArtboard,
        time: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        let epoch = keyed_data_epoch();
        if self.epoch != Some(epoch) {
            let (objects, cells) = snapshot();
            self.build(objects.iter(), cells.iter());
            self.epoch = Some(epoch);
        }
        match artboard.as_keyed_object_context() {
            Some(objects) => {
                for entry in &mut self.objects {
                    match entry {
                        CompiledEntry::Borrowed(object) => {
                            KeyedObject::apply_occurrence(object, objects, time, mix, context)
                        }
                        CompiledEntry::Object(object) => object.apply(objects, time, mix, context),
                    }
                }
            }
            None => {
                for entry in &self.objects {
                    match entry {
                        CompiledEntry::Borrowed(object) => {
                            artboard.apply_keyed_object_borrowed(object, time, mix, context)
                        }
                        CompiledEntry::Object(object) => artboard.apply_keyed_object_cells(
                            &object.handle,
                            &object.cells,
                            time,
                            mix,
                            context,
                        ),
                    }
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn build<'a>(
        &mut self,
        objects: impl Iterator<Item = &'a CoreHandle>,
        cells: impl Iterator<Item = &'a Option<Rc<KeyedObjectCells>>>,
    ) {
        self.objects = objects
            .zip(cells)
            .map(|(object, cells)| match cells {
                None => CompiledEntry::Borrowed(object.clone()),
                Some(cells) => CompiledEntry::Object(CompiledObject {
                    handle: object.clone(),
                    cells: Rc::clone(cells),
                    object_id: cells.object_id(),
                    tracks: cells
                        .properties()
                        .iter()
                        .map(|property| Track::build(&property.cells))
                        .collect(),
                    target: None,
                }),
            })
            .collect();
    }
}

impl CompiledObject {
    /// KeyedObject::apply_cells over the compiled tracks.
    #[inline]
    fn apply(
        &mut self,
        artboard: &mut dyn KeyedObjectContext,
        time: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        let Some(object) = artboard.resolve_object(self.object_id) else {
            return;
        };
        if self.target.as_ref() != Some(&object) {
            self.resolve_sinks(&object);
        }
        // InterpolatorHost::from, once for every property of this target.
        let interpolator_host = KeyFrameInterpolator::host_of(&object);
        for track in &self.tracks {
            if track.callback {
                continue;
            }
            // KeyedProperty::apply_cells under a host: the host's hook,
            // asked per property in order, may take the whole value. The
            // hook reads only the host's layout animation and the key, so
            // the frames it leaves are the ones compiled.
            let mix = match &interpolator_host {
                Some(host) if artboard.overrides_keyed_interpolation(host, track.property_key) => 1.0,
                _ => mix,
            };
            track.apply(&object, time, mix, context);
        }
    }

    #[cold]
    #[inline(never)]
    fn resolve_sinks(&mut self, target: &CoreHandle) {
        for track in &mut self.tracks {
            track.sink = PlainInputSink::resolve(target, track.property_key);
        }
        self.target = Some(target.clone());
    }
}

impl Track {
    fn build(property: &Rc<KeyedPropertyCells>) -> Self {
        let frames = property.frames();
        let property_key = property.property_key();
        Self {
            property: Rc::clone(property),
            property_key,
            callback: CoreRegistry::is_callback(property_key),
            times: frames.iter().map(|frame| frame.cells.seconds()).collect(),
            keys: frames
                .iter()
                .map(|frame| FrameKey {
                    value: frame.cells.double_value(),
                    holds: frame.cells.interpolation_type() == 0,
                    interpolator: match frame.cells.interpolator() {
                        None => Interpolation::Linear,
                        Some(interpolator) => {
                            let scripted = interpolator.with_mut(|interpolator| {
                                interpolator.keyframe_interpolator_is_scripted().unwrap_or(false)
                            });
                            match scripted {
                                Some(false) => Interpolation::BuiltIn(interpolator),
                                _ => Interpolation::Other,
                            }
                        }
                    },
                })
                .collect(),
            frames,
            sink: None,
        }
    }

    /// KeyedProperty::apply_cells from the arrays, at `mix` after any
    /// interpolation host's override.
    #[inline]
    fn apply(&self, object: &CoreHandle, seconds: f32, mix: f32, context: Option<&dyn KeyFrameValueContext>) {
        assert!(!self.times.is_empty());
        let index = closest_time_index(&self.times, seconds, 0);
        let (frame, next) = select_frames(index, self.times.len(), seconds, |i| self.times[i], |i| {
            self.keys[i].holds
        });
        let key = self.property_key as i32;
        match &self.sink {
            Some(sink) if sink.serves(object, self.property_key) => {
                self.apply_selected(frame, next, index, &SinkDouble { object, key, sink }, seconds, mix, context)
            }
            _ => self.apply_selected(frame, next, index, &RegistryDouble { object, key }, seconds, mix, context),
        }
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn apply_selected(
        &self,
        frame: usize,
        next: Option<usize>,
        index: i32,
        target: &impl DoubleTarget,
        seconds: f32,
        mix: f32,
        context: Option<&dyn KeyFrameValueContext>,
    ) {
        // A plain context changes nothing a double keyframe reads.
        if context.is_none_or(|context| context.plain_numbers()) {
            let from = &self.keys[frame];
            match (next, from.value) {
                (None, Some(value)) => {
                    KeyFrameDouble::apply_value_to(target, mix, value);
                    return;
                }
                (Some(next), Some(from_value)) => {
                    if let Some(to_value) = self.keys[next].value {
                        let from_seconds = self.times[frame];
                        let factor = (seconds - from_seconds) / (self.times[next] - from_seconds);
                        match &from.interpolator {
                            Interpolation::Linear => {
                                KeyFrameDouble::apply_value_to(
                                    target,
                                    mix,
                                    interpolated_value(from_value, to_value, factor),
                                );
                                return;
                            }
                            // KeyFrameDouble::apply_interpolation with a
                            // built-in interpolator: its transform under one
                            // loan, linear where that loan fails.
                            Interpolation::BuiltIn(interpolator) => {
                                let value = interpolator
                                    .with_mut(|interpolator| {
                                        interpolator.keyframe_interpolator_transform_value(from_value, to_value, factor)
                                    })
                                    .flatten()
                                    .unwrap_or_else(|| interpolated_value(from_value, to_value, factor));
                                KeyFrameDouble::apply_value_to(target, mix, value);
                                return;
                            }
                            Interpolation::Other => {}
                        }
                    }
                }
                _ => {}
            }
        }
        KeyedProperty::apply_frames(
            &self.frames,
            self.frames.len(),
            index,
            target,
            seconds,
            mix,
            context,
            context,
        );
    }
}
