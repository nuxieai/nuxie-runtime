use crate::mechanical_port::source::{
    custom_property_string::CustomPropertyString, script_input_string::ScriptInputString,
};

pub struct ScriptInputStringBase {
    pub base: CustomPropertyString,
}
impl Default for ScriptInputStringBase {
    fn default() -> Self {
        Self {
            base: CustomPropertyString::default(),
        }
    }
}
impl ScriptInputStringBase {
    pub const TYPE_KEY: u16 = 627;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 130 | 167 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy(&mut self, object: &Self) {
        self.base.base.copy(&object.base.base);
    }
    pub fn clone_into(&self) -> ScriptInputString {
        let mut cloned = ScriptInputString::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for ScriptInputStringBase {
    type Target = CustomPropertyString;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScriptInputStringBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
