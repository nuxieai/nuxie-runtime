use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader,
    data_bind::converters::formula::formula_token_function::FormulaTokenFunction,
    data_bind::converters::formula::formula_token_parenthesis::FormulaTokenParenthesis,
};

pub trait FormulaTokenFunctionBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn function_type_changed(&mut self) {}
}

pub struct FormulaTokenFunctionBase {
    pub base: FormulaTokenParenthesis,
    function_type: u32,
}

impl Default for FormulaTokenFunctionBase {
    fn default() -> Self {
        Self {
            base: FormulaTokenParenthesis::default(),
            function_type: 0,
        }
    }
}

impl FormulaTokenFunctionBase {
    pub const TYPE_KEY: u16 = 542;
    pub const FUNCTION_TYPE_PROPERTY_KEY: u16 = 776;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 539 | 537)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn function_type(&self) -> u32 {
        self.function_type
    }
    pub fn set_function_type(
        &mut self,
        value: u32,
        callbacks: &mut impl FormulaTokenFunctionBaseCallbacks,
    ) {
        if self.function_type == value {
            return;
        }
        self.function_type = value;
        callbacks.function_type_changed();
        callbacks.notify_property_changed(Self::FUNCTION_TYPE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl FormulaTokenFunctionBaseCallbacks,
    ) -> FormulaTokenFunction {
        let mut cloned = FormulaTokenFunction::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl FormulaTokenFunctionBaseCallbacks) {
        self.function_type = object.function_type;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl FormulaTokenFunctionBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::FUNCTION_TYPE_PROPERTY_KEY => {
                self.function_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
