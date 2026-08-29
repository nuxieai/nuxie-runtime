use crate::mechanical_port::source::{
    animation::keyframe_interpolator::KeyFrameInterpolator, core::binary_reader::BinaryReader,
};

pub trait CubicInterpolatorBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn x1_changed(&mut self) {}
    fn y1_changed(&mut self) {}
    fn x2_changed(&mut self) {}
    fn y2_changed(&mut self) {}
}

pub struct CubicInterpolatorBase {
    pub base: KeyFrameInterpolator,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl Default for CubicInterpolatorBase {
    fn default() -> Self {
        Self {
            base: KeyFrameInterpolator::default(),
            x1: 0.42,
            y1: 0.0,
            x2: 0.58,
            y2: 1.0,
        }
    }
}

impl CubicInterpolatorBase {
    pub const TYPE_KEY: u16 = 139;
    pub const X1_PROPERTY_KEY: u16 = 63;
    pub const Y1_PROPERTY_KEY: u16 = 64;
    pub const X2_PROPERTY_KEY: u16 = 65;
    pub const Y2_PROPERTY_KEY: u16 = 66;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 175)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn x1(&self) -> f32 {
        self.x1
    }
    pub fn set_x1(&mut self, value: f32, callbacks: &mut impl CubicInterpolatorBaseCallbacks) {
        if !self.set_x1_value(value) {
            return;
        }
        callbacks.x1_changed();
        callbacks.notify_property_changed(Self::X1_PROPERTY_KEY);
    }

    pub(crate) fn set_x1_value(&mut self, value: f32) -> bool {
        if self.x1 == value {
            return false;
        }
        self.x1 = value;
        true
    }
    pub fn y1(&self) -> f32 {
        self.y1
    }
    pub fn set_y1(&mut self, value: f32, callbacks: &mut impl CubicInterpolatorBaseCallbacks) {
        if !self.set_y1_value(value) {
            return;
        }
        callbacks.y1_changed();
        callbacks.notify_property_changed(Self::Y1_PROPERTY_KEY);
    }

    pub(crate) fn set_y1_value(&mut self, value: f32) -> bool {
        if self.y1 == value {
            return false;
        }
        self.y1 = value;
        true
    }
    pub fn x2(&self) -> f32 {
        self.x2
    }
    pub fn set_x2(&mut self, value: f32, callbacks: &mut impl CubicInterpolatorBaseCallbacks) {
        if !self.set_x2_value(value) {
            return;
        }
        callbacks.x2_changed();
        callbacks.notify_property_changed(Self::X2_PROPERTY_KEY);
    }

    pub(crate) fn set_x2_value(&mut self, value: f32) -> bool {
        if self.x2 == value {
            return false;
        }
        self.x2 = value;
        true
    }
    pub fn y2(&self) -> f32 {
        self.y2
    }
    pub fn set_y2(&mut self, value: f32, callbacks: &mut impl CubicInterpolatorBaseCallbacks) {
        if !self.set_y2_value(value) {
            return;
        }
        callbacks.y2_changed();
        callbacks.notify_property_changed(Self::Y2_PROPERTY_KEY);
    }

    pub(crate) fn set_y2_value(&mut self, value: f32) -> bool {
        if self.y2 == value {
            return false;
        }
        self.y2 = value;
        true
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl CubicInterpolatorBaseCallbacks) {
        self.x1 = object.x1;
        self.y1 = object.y1;
        self.x2 = object.x2;
        self.y2 = object.y2;
        self.base.copy(&object.base);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl CubicInterpolatorBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::X1_PROPERTY_KEY => {
                self.x1 = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::Y1_PROPERTY_KEY => {
                self.y1 = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::X2_PROPERTY_KEY => {
                self.x2 = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::Y2_PROPERTY_KEY => {
                self.y2 = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader),
        }
    }
}

impl std::ops::Deref for CubicInterpolatorBase {
    type Target = KeyFrameInterpolator;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CubicInterpolatorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
