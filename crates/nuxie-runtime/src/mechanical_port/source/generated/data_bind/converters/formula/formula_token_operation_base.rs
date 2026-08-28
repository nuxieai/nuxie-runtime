use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::formula::formula_token::FormulaToken,
    data_bind::converters::formula::formula_token_operation::FormulaTokenOperation,
};

pub trait FormulaTokenOperationBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn operation_type_changed(&mut self) {}
}

pub struct FormulaTokenOperationBase {
    pub base: FormulaToken,
    operation_type: u32,
}

impl Default for FormulaTokenOperationBase {
    fn default() -> Self {
        Self {
            base: FormulaToken::default(),
            operation_type: 0,
        }
    }
}

impl FormulaTokenOperationBase {
    pub const TYPE_KEY: u16 = 541;
    pub const OPERATION_TYPE_PROPERTY_KEY: u16 = 775;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 537)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn operation_type(&self) -> u32 {
        self.operation_type
    }
    pub fn set_operation_type(
        &mut self,
        value: u32,
        callbacks: &mut impl FormulaTokenOperationBaseCallbacks,
    ) {
        if !self.set_operation_type_value(value) {
            return;
        }
        callbacks.operation_type_changed();
        callbacks.notify_property_changed(Self::OPERATION_TYPE_PROPERTY_KEY);
    }

    pub(crate) fn set_operation_type_value(&mut self, value: u32) -> bool {
        if self.operation_type == value {
            return false;
        }
        self.operation_type = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl FormulaTokenOperationBaseCallbacks,
    ) -> FormulaTokenOperation {
        let mut cloned = FormulaTokenOperation::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl FormulaTokenOperationBaseCallbacks) {
        self.operation_type = object.operation_type;
        self.base.copy(&object.base);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl FormulaTokenOperationBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::OPERATION_TYPE_PROPERTY_KEY => {
                self.operation_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader),
        }
    }
}

impl std::ops::Deref for FormulaTokenOperationBase {
    type Target = FormulaToken;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FormulaTokenOperationBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
