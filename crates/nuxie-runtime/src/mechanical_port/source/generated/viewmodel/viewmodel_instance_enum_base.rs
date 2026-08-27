use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_instance_value::ViewModelInstanceValue,
};

pub trait ViewModelInstanceEnumBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn property_value_changed(&mut self) {}
}

pub struct ViewModelInstanceEnumBase {
    pub base: ViewModelInstanceValue,
    property_value: u32,
}

impl Default for ViewModelInstanceEnumBase {
    fn default() -> Self {
        Self {
            base: ViewModelInstanceValue::default(),
            property_value: 0,
        }
    }
}

impl ViewModelInstanceEnumBase {
    pub const TYPE_KEY: u16 = 432;
    pub const PROPERTY_VALUE_PROPERTY_KEY: u16 = 560;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 0 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_value(&self) -> u32 {
        self.property_value
    }
    pub fn set_property_value(
        &mut self,
        value: u32,
        callbacks: &mut impl ViewModelInstanceEnumBaseCallbacks,
    ) {
        if self.property_value == value {
            return;
        }
        self.property_value = value;
        callbacks.property_value_changed();
        callbacks.notify_property_changed(Self::PROPERTY_VALUE_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ViewModelInstanceEnumBaseCallbacks) {
        self.property_value = object.property_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelInstanceEnumBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::PROPERTY_VALUE_PROPERTY_KEY => {
                self.property_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
