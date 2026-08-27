use crate::mechanical_port::source::{
    animation::nested_bool::NestedBool, animation::nested_input::NestedInput,
    core::binary_reader::BinaryReader,
};

pub trait NestedBoolBaseCallbacks {
    fn nested_value_changed(&mut self) {}
    fn nested_value_bool(&mut self, value: bool);
}

pub struct NestedBoolBase {
    pub base: NestedInput,
    nested_value: bool,
}

impl Default for NestedBoolBase {
    fn default() -> Self {
        Self {
            base: NestedInput::default(),
            nested_value: false,
        }
    }
}

impl NestedBoolBase {
    pub const TYPE_KEY: u16 = 123;
    pub const NESTED_VALUE_PROPERTY_KEY: u16 = 238;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 121 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn nested_value(&self) -> bool {
        self.nested_value
    }
    pub fn clone_into(&self, callbacks: &mut impl NestedBoolBaseCallbacks) -> NestedBool {
        let mut cloned = NestedBool::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NestedBoolBaseCallbacks) {
        self.nested_value = object.nested_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NestedBoolBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::NESTED_VALUE_PROPERTY_KEY => {
                self.nested_value = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
