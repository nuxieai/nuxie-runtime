use crate::mechanical_port::source::{component::Component, core::binary_reader::BinaryReader};

pub trait AxisBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn offset_changed(&mut self) {}
    fn normalized_changed(&mut self) {}
}

pub struct AxisBase {
    pub base: Component,
    offset: f32,
    normalized: bool,
}

impl Default for AxisBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            offset: 0.0,
            normalized: false,
        }
    }
}

impl AxisBase {
    pub const TYPE_KEY: u16 = 492;
    pub const OFFSET_PROPERTY_KEY: u16 = 675;
    pub const NORMALIZED_PROPERTY_KEY: u16 = 676;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn offset(&self) -> f32 {
        self.offset
    }
    pub fn set_offset(&mut self, value: f32, callbacks: &mut impl AxisBaseCallbacks) {
        if self.offset == value {
            return;
        }
        self.offset = value;
        callbacks.offset_changed();
        callbacks.notify_property_changed(Self::OFFSET_PROPERTY_KEY);
    }
    pub fn normalized(&self) -> bool {
        self.normalized
    }
    pub fn set_normalized(&mut self, value: bool, callbacks: &mut impl AxisBaseCallbacks) {
        if self.normalized == value {
            return;
        }
        self.normalized = value;
        callbacks.normalized_changed();
        callbacks.notify_property_changed(Self::NORMALIZED_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl AxisBaseCallbacks) {
        self.offset = object.offset;
        self.normalized = object.normalized;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl AxisBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::OFFSET_PROPERTY_KEY => {
                self.offset = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::NORMALIZED_PROPERTY_KEY => {
                self.normalized = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
