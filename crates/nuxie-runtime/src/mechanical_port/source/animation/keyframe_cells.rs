//! A KeyFrame's time and interpolation type, which a KeyedProperty's frame
//! search and selection read for every applied frame. The keyframe and its
//! property share them, so that search runs without borrowing the keyframes.

use std::cell::Cell;

pub(crate) struct KeyFrameCells {
    // frame / fps, computed when the keyframe is imported.
    seconds: Cell<f32>,
    // InterpolatingKeyFrame's interpolationType; 0 (hold) for any other frame.
    interpolation_type: Cell<u32>,
    // A KeyFrameDouble's value (its storage); None for any other frame.
    double_value: Cell<Option<f32>>,
}

impl Default for KeyFrameCells {
    fn default() -> Self {
        Self {
            seconds: Cell::new(0.0),
            interpolation_type: Cell::new(0),
            double_value: Cell::new(None),
        }
    }
}

impl KeyFrameCells {
    #[inline]
    pub(crate) fn seconds(&self) -> f32 {
        self.seconds.get()
    }

    #[inline]
    pub(crate) fn set_seconds(&self, seconds: f32) {
        self.seconds.set(seconds);
    }

    #[inline]
    pub(crate) fn interpolation_type(&self) -> u32 {
        self.interpolation_type.get()
    }

    #[inline]
    pub(crate) fn set_interpolation_type(&self, value: u32) {
        self.interpolation_type.set(value);
    }

    /// The frame's value when it is a KeyFrameDouble.
    #[inline]
    pub(crate) fn double_value(&self) -> Option<f32> {
        self.double_value.get()
    }

    #[inline]
    pub(crate) fn set_double_value(&self, value: f32) {
        self.double_value.set(Some(value));
    }
}

/// One of a KeyedProperty's keyframes: its occurrence (for application) and
/// the cells its frame search reads.
#[derive(Clone)]
pub(crate) struct KeyFrameEntry {
    pub(crate) handle: crate::mechanical_port::source::core::CoreHandle,
    pub(crate) cells: std::rc::Rc<KeyFrameCells>,
}

/// A KeyedProperty's property key and keyframes, shared with its KeyedObject
/// so animation application reads them without borrowing the property.
pub(crate) struct KeyedPropertyCells {
    property_key: Cell<u32>,
    // The owning KeyedProperty's whole keyframe list, replaced (never
    // edited) on every add. A base moved into a fresh property keeps its old
    // list until the first add, so such a rebuild must add at least one key,
    // as upstream apply requires a non-empty list anyway.
    frames: std::cell::RefCell<std::rc::Rc<[KeyFrameEntry]>>,
}

impl KeyedPropertyCells {
    pub(crate) fn new(property_key: u32) -> Self {
        Self {
            property_key: Cell::new(property_key),
            frames: std::cell::RefCell::new(std::rc::Rc::new([])),
        }
    }

    #[inline]
    pub(crate) fn property_key(&self) -> u32 {
        self.property_key.get()
    }

    #[inline]
    pub(crate) fn set_property_key(&self, value: u32) {
        self.property_key.set(value);
    }

    /// The current keyframes, as a list that outlives callbacks.
    #[inline]
    pub(crate) fn frames(&self) -> std::rc::Rc<[KeyFrameEntry]> {
        std::rc::Rc::clone(&self.frames.borrow())
    }

    /// Read the current keyframes for work that runs no callback.
    #[inline]
    pub(crate) fn with_frames<R>(&self, read: impl FnOnce(&[KeyFrameEntry]) -> R) -> R {
        read(&self.frames.borrow())
    }

    /// Replace the keyframes with the owning KeyedProperty's whole list.
    pub(crate) fn set_frames(&self, frames: std::rc::Rc<[KeyFrameEntry]>) {
        *self.frames.borrow_mut() = frames;
    }
}

/// KeyedProperty::closestFrameIndex over a keyframe list.
pub(crate) fn closest_frame_index(frames: &[KeyFrameEntry], seconds: f32, offset: i32) -> i32 {
    let mut start = 0;
    let mut end = frames.len() as i32 - 1;
    if seconds > frames[end as usize].cells.seconds() {
        return end + 1;
    }
    while start <= end {
        let mid = (start + end) >> 1;
        let value = frames[mid as usize].cells.seconds();
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

/// One of a KeyedObject's keyed properties and its cells.
#[derive(Clone)]
pub(crate) struct PropertyEntry {
    pub(crate) handle: crate::mechanical_port::source::core::CoreHandle,
    pub(crate) cells: std::rc::Rc<KeyedPropertyCells>,
}

/// A KeyedObject's target id and keyed properties, read by every application.
pub(crate) struct KeyedObjectCells {
    object_id: Cell<u32>,
    // Replaced, never edited, when a property is added or removed.
    properties: std::cell::RefCell<std::rc::Rc<[PropertyEntry]>>,
}

impl Default for KeyedObjectCells {
    fn default() -> Self {
        Self {
            object_id: Cell::new(0),
            properties: std::cell::RefCell::new(std::rc::Rc::new([])),
        }
    }
}

impl KeyedObjectCells {
    #[inline]
    pub(crate) fn object_id(&self) -> u32 {
        self.object_id.get()
    }

    #[inline]
    pub(crate) fn set_object_id(&self, value: u32) {
        self.object_id.set(value);
    }

    #[inline]
    pub(crate) fn properties(&self) -> std::rc::Rc<[PropertyEntry]> {
        std::rc::Rc::clone(&self.properties.borrow())
    }

    pub(crate) fn push_property(&self, property: PropertyEntry) {
        let properties: std::rc::Rc<[PropertyEntry]> = self
            .properties
            .borrow()
            .iter()
            .cloned()
            .chain(std::iter::once(property))
            .collect();
        *self.properties.borrow_mut() = properties;
    }

    pub(crate) fn remove_property(&self, index: usize) {
        let properties: std::rc::Rc<[PropertyEntry]> = self
            .properties
            .borrow()
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .map(|(_, property)| property.clone())
            .collect();
        *self.properties.borrow_mut() = properties;
    }
}
