use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader, text::text_style_axis::TextStyleAxis,
};

pub trait TextStyleAxisBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn tag_changed(&mut self) {}
    fn axis_value_changed(&mut self) {}
}

pub struct TextStyleAxisBase {
    pub base: Component,
    tag: u32,
    axis_value: f32,
}

impl Default for TextStyleAxisBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            tag: 0,
            axis_value: 0.0,
        }
    }
}

impl TextStyleAxisBase {
    pub const TYPE_KEY: u16 = 144;
    pub const TAG_PROPERTY_KEY: u16 = 289;
    pub const AXIS_VALUE_PROPERTY_KEY: u16 = 288;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn tag(&self) -> u32 {
        self.tag
    }
    pub fn set_tag(&mut self, value: u32, callbacks: &mut impl TextStyleAxisBaseCallbacks) {
        if self.tag == value {
            return;
        }
        self.tag = value;
        callbacks.tag_changed();
        callbacks.notify_property_changed(Self::TAG_PROPERTY_KEY);
    }
    pub fn axis_value(&self) -> f32 {
        self.axis_value
    }
    pub fn set_axis_value(&mut self, value: f32, callbacks: &mut impl TextStyleAxisBaseCallbacks) {
        if self.axis_value == value {
            return;
        }
        self.axis_value = value;
        callbacks.axis_value_changed();
        callbacks.notify_property_changed(Self::AXIS_VALUE_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl TextStyleAxisBaseCallbacks) -> TextStyleAxis {
        let mut cloned = TextStyleAxis::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TextStyleAxisBaseCallbacks) {
        self.tag = object.tag;
        self.axis_value = object.axis_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TextStyleAxisBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::TAG_PROPERTY_KEY => {
                self.tag = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::AXIS_VALUE_PROPERTY_KEY => {
                self.axis_value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
