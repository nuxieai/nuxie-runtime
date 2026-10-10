//! A KeyFrame's time and interpolation type, which a KeyedProperty's frame
//! search and selection read for every applied frame. The keyframe and its
//! property share them, so that search runs without borrowing the keyframes.

use std::cell::Cell;

pub(crate) struct KeyFrameCells {
    // frame / fps, computed when the keyframe is imported.
    seconds: Cell<f32>,
    // InterpolatingKeyFrame's interpolationType; 0 (hold) for any other frame.
    interpolation_type: Cell<u32>,
}

impl Default for KeyFrameCells {
    fn default() -> Self {
        Self {
            seconds: Cell::new(0.0),
            interpolation_type: Cell::new(0),
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
}
