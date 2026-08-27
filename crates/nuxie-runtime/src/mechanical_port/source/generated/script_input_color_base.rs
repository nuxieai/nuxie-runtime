use crate::mechanical_port::source::{
    custom_property_color::CustomPropertyColor, script_input_color::ScriptInputColor,
};

pub struct ScriptInputColorBase {
    pub base: CustomPropertyColor,
}
impl Default for ScriptInputColorBase {
    fn default() -> Self {
        Self {
            base: CustomPropertyColor::default(),
        }
    }
}
impl ScriptInputColorBase {
    pub const TYPE_KEY: u16 = 626;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 592 | 167 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy(&mut self, object: &Self) {
        self.base.base.copy(&object.base.base);
    }
    pub fn clone_into(&self) -> ScriptInputColor {
        let mut cloned = ScriptInputColor::default();
        cloned.base.copy(self);
        cloned
    }
}
