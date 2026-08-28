use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::formula::formula_token::FormulaToken,
    data_bind::converters::formula::formula_token_value::FormulaTokenValue,
};

pub trait FormulaTokenValueBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn operation_value_changed(&mut self) {}
}

pub struct FormulaTokenValueBase {
    pub base: FormulaToken,
    operation_value: f32,
}

impl Default for FormulaTokenValueBase {
    fn default() -> Self {
        Self {
            base: FormulaToken::default(),
            operation_value: 1.0,
        }
    }
}

impl FormulaTokenValueBase {
    pub const TYPE_KEY: u16 = 543;
    pub const OPERATION_VALUE_PROPERTY_KEY: u16 = 777;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 537)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn operation_value(&self) -> f32 {
        self.operation_value
    }
    pub fn set_operation_value(
        &mut self,
        value: f32,
        callbacks: &mut impl FormulaTokenValueBaseCallbacks,
    ) {
        if !self.set_operation_value_value(value) {
            return;
        }
        callbacks.operation_value_changed();
        callbacks.notify_property_changed(Self::OPERATION_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_operation_value_value(&mut self, value: f32) -> bool {
        if self.operation_value == value {
            return false;
        }
        self.operation_value = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl FormulaTokenValueBaseCallbacks,
    ) -> FormulaTokenValue {
        let mut cloned = FormulaTokenValue::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl FormulaTokenValueBaseCallbacks) {
        self.operation_value = object.operation_value;
        self.base.copy(&object.base);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl FormulaTokenValueBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::OPERATION_VALUE_PROPERTY_KEY => {
                self.operation_value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader),
        }
    }
}

impl std::ops::Deref for FormulaTokenValueBase {
    type Target = FormulaToken;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FormulaTokenValueBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
