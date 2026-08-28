use crate::mechanical_port::source::{
    custom_property_boolean::CustomPropertyBoolean, script_input_boolean::ScriptInputBoolean,
};

pub struct ScriptInputBooleanBase {
    pub base: CustomPropertyBoolean,
}
impl Default for ScriptInputBooleanBase {
    fn default() -> Self {
        Self {
            base: CustomPropertyBoolean::default(),
        }
    }
}
impl ScriptInputBooleanBase {
    pub const TYPE_KEY: u16 = 631;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 129 | 167 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy(&mut self, object: &Self) {
        self.base.base.copy(&object.base.base);
    }
    pub fn clone_into(&self) -> ScriptInputBoolean {
        let mut cloned = ScriptInputBoolean::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for ScriptInputBooleanBase {
    type Target = CustomPropertyBoolean;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScriptInputBooleanBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
