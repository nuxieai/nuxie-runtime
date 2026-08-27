use crate::mechanical_port::source::{
    animation::nested_input::NestedInput, animation::nested_number::NestedNumber,
    core::binary_reader::BinaryReader,
};

pub trait NestedNumberBaseCallbacks {
    fn nested_value_changed(&mut self) {}
    fn nested_value_f32(&mut self, value: f32);
}

pub struct NestedNumberBase {
    pub base: NestedInput,
    nested_value: f32,
}

impl Default for NestedNumberBase {
    fn default() -> Self {
        Self {
            base: NestedInput::default(),
            nested_value: 0.0,
        }
    }
}

impl NestedNumberBase {
    pub const TYPE_KEY: u16 = 124;
    pub const NESTED_VALUE_PROPERTY_KEY: u16 = 239;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 121 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn nested_value(&self) -> f32 {
        self.nested_value
    }
    pub fn clone_into(&self, callbacks: &mut impl NestedNumberBaseCallbacks) -> NestedNumber {
        let mut cloned = NestedNumber::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NestedNumberBaseCallbacks) {
        self.nested_value = object.nested_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NestedNumberBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::NESTED_VALUE_PROPERTY_KEY => {
                self.nested_value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
