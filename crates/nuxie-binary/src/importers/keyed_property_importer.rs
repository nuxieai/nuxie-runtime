//! Mechanical translation of pinned `KeyedPropertyImporter`.

use super::*;

pub(super) fn dispatch_imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    definition
        .is_a("KeyFrame")
        .then(|| context.latest(ImportStackKey::KeyedProperty))
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    (definition.name == "KeyedProperty")
        .then(|| context.latest(ImportStackKey::KeyedObject))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "KeyedProperty" {
        context.make_latest(ImportStackKey::KeyedProperty);
    }
}

/// Pinned `readNullObject` consumes the null without appending a keyframe or
/// allowing it to propagate to an older importer.
pub(super) fn read_null_object() -> bool {
    true
}

/// Occurrence coordinates retain the exact KeyedProperty relationship. A
/// missing location is an imported property excluded by later owner/property
/// validation; it remains the current importer sink until another
/// KeyedProperty replaces it, just as the C++ pointer does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct KeyedPropertyImporter {
    animation_fps: u32,
    animation_index: Option<usize>,
    keyed_object_index: Option<usize>,
    keyed_property_index: Option<usize>,
}

impl KeyedPropertyImporter {
    /// Mechanical translation of the constructor: retain the independently
    /// latest LinearAnimation and the newly imported KeyedProperty.
    pub(super) fn new(animation_fps: u32, owner: Option<(usize, usize, usize)>) -> Self {
        let (animation_index, keyed_object_index, keyed_property_index) = owner
            .map(|(animation, keyed_object, keyed_property)| {
                (Some(animation), Some(keyed_object), Some(keyed_property))
            })
            .unwrap_or((None, None, None));
        Self {
            animation_fps,
            animation_index,
            keyed_object_index,
            keyed_property_index,
        }
    }

    /// Mechanical translation of `addKeyFrame`: compute seconds from the
    /// retained animation first, then append to the retained property.
    pub(super) fn add_key_frame<'a>(
        &self,
        animations: &mut [RuntimeLinearAnimation<'a>],
        key_frame: &'a RuntimeObject,
    ) {
        let frame = key_frame.uint_property("frame").unwrap_or(0) as u32;
        let seconds = frame as f32 / self.animation_fps as f32;
        let (Some(animation_index), Some(keyed_object_index), Some(keyed_property_index)) = (
            self.animation_index,
            self.keyed_object_index,
            self.keyed_property_index,
        ) else {
            return;
        };
        let property = &mut animations[animation_index].keyed_objects[keyed_object_index]
            .keyed_properties[keyed_property_index];
        property.key_frames.push(RuntimeImportedKeyFrame {
            object: key_frame,
            seconds,
        });
        if property.first_key_frame.is_none() {
            property.first_key_frame = Some(key_frame);
        }
    }
}
