use crate::mechanical_port::source::{component::Component, core::binary_reader::BinaryReader};

pub trait AxisBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
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
        if !self.set_offset_value(value) {
            return;
        }
        callbacks.offset_changed();
        callbacks.notify_property_changed(Self::OFFSET_PROPERTY_KEY);
    }

    pub(crate) fn set_offset_value(&mut self, value: f32) -> bool {
        if self.offset == value {
            return false;
        }
        self.offset = value;
        true
    }
    pub fn normalized(&self) -> bool {
        self.normalized
    }
    pub fn set_normalized(&mut self, value: bool, callbacks: &mut impl AxisBaseCallbacks) {
        if !self.set_normalized_value(value) {
            return;
        }
        callbacks.normalized_changed();
        callbacks.notify_property_changed(Self::NORMALIZED_PROPERTY_KEY);
    }

    pub(crate) fn set_normalized_value(&mut self, value: bool) -> bool {
        if self.normalized == value {
            return false;
        }
        self.normalized = value;
        true
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

impl std::ops::Deref for AxisBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for AxisBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
