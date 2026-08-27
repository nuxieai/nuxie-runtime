use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader,
    shapes::paint::gradient_stop::GradientStop,
};

pub trait GradientStopBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn color_value_changed(&mut self) {}
    fn position_changed(&mut self) {}
}

pub struct GradientStopBase {
    pub base: Component,
    color_value: i32,
    position: f32,
}

impl Default for GradientStopBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            color_value: 0xFFFFFFFFu32 as i32,
            position: 0.0,
        }
    }
}

impl GradientStopBase {
    pub const TYPE_KEY: u16 = 19;
    pub const COLOR_VALUE_PROPERTY_KEY: u16 = 38;
    pub const POSITION_PROPERTY_KEY: u16 = 39;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn color_value(&self) -> i32 {
        self.color_value
    }
    pub fn set_color_value(&mut self, value: i32, callbacks: &mut impl GradientStopBaseCallbacks) {
        if self.color_value == value {
            return;
        }
        self.color_value = value;
        callbacks.color_value_changed();
        callbacks.notify_property_changed(Self::COLOR_VALUE_PROPERTY_KEY);
    }
    pub fn position(&self) -> f32 {
        self.position
    }
    pub fn set_position(&mut self, value: f32, callbacks: &mut impl GradientStopBaseCallbacks) {
        if self.position == value {
            return;
        }
        self.position = value;
        callbacks.position_changed();
        callbacks.notify_property_changed(Self::POSITION_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl GradientStopBaseCallbacks) -> GradientStop {
        let mut cloned = GradientStop::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl GradientStopBaseCallbacks) {
        self.color_value = object.color_value;
        self.position = object.position;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl GradientStopBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::COLOR_VALUE_PROPERTY_KEY => {
                self.color_value = crate::mechanical_port::source::core::field_types::core_color_type::CoreColorType::deserialize(reader);
                true
            }
            Self::POSITION_PROPERTY_KEY => {
                self.position = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
