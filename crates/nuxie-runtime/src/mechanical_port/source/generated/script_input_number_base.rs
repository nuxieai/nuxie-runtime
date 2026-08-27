use crate::mechanical_port::source::{
    custom_property_number::CustomPropertyNumber, script_input_number::ScriptInputNumber,
};

pub struct ScriptInputNumberBase {
    pub base: CustomPropertyNumber,
}
impl Default for ScriptInputNumberBase {
    fn default() -> Self {
        Self {
            base: CustomPropertyNumber::default(),
        }
    }
}
impl ScriptInputNumberBase {
    pub const TYPE_KEY: u16 = 611;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 127 | 167 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy(&mut self, object: &Self) {
        self.base.base.copy(&object.base.base);
    }
    pub fn clone_into(&self) -> ScriptInputNumber {
        let mut cloned = ScriptInputNumber::default();
        cloned.base.copy(self);
        cloned
    }
}
