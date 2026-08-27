use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, field_types::core_color_type::CoreColorType},
    custom_property::CustomProperty,
    custom_property_color::CustomPropertyColor,
};

pub trait CustomPropertyColorBaseCallbacks {
    fn property_value_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}
pub struct CustomPropertyColorBase {
    pub base: CustomProperty,
    property_value: i32,
}
impl Default for CustomPropertyColorBase {
    fn default() -> Self {
        Self {
            base: CustomProperty::default(),
            property_value: 0xFF1D1D1Du32 as i32,
        }
    }
}
impl CustomPropertyColorBase {
    pub const TYPE_KEY: u16 = 592;
    pub const PROPERTY_VALUE_PROPERTY_KEY: u16 = 836;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 167 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_value(&self) -> i32 {
        self.property_value
    }
    pub fn set_property_value<C: CustomPropertyColorBaseCallbacks>(
        &mut self,
        value: i32,
        c: &mut C,
    ) {
        if self.property_value == value {
            return;
        }
        self.property_value = value;
        c.property_value_changed();
        c.notify_property_changed(Self::PROPERTY_VALUE_PROPERTY_KEY);
    }
    pub fn copy<C: CustomPropertyColorBaseCallbacks>(&mut self, object: &Self, c: &mut C) {
        self.property_value = object.property_value;
        self.base.base.copy(&object.base.base, c);
    }
    pub fn deserialize<C: CustomPropertyColorBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        c: &mut C,
    ) -> bool {
        match key {
            Self::PROPERTY_VALUE_PROPERTY_KEY => {
                self.property_value = CoreColorType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(key, reader, c),
        }
    }
    pub fn clone_into<C: CustomPropertyColorBaseCallbacks>(
        &self,
        c: &mut C,
    ) -> CustomPropertyColor {
        let mut cloned = CustomPropertyColor::default();
        cloned.base.copy(self, c);
        cloned
    }
}
