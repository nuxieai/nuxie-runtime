use crate::mechanical_port::source::{
    custom_property_trigger::CustomPropertyTrigger, script_input_trigger::ScriptInputTrigger,
};

pub struct ScriptInputTriggerBase {
    pub base: CustomPropertyTrigger,
}
impl Default for ScriptInputTriggerBase {
    fn default() -> Self {
        Self {
            base: CustomPropertyTrigger::default(),
        }
    }
}
impl ScriptInputTriggerBase {
    pub const TYPE_KEY: u16 = 618;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 613 | 167 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy(&mut self, object: &Self) {
        let mut base = std::mem::take(&mut self.base.base);
        base.copy(&object.base.base, &mut self.base);
        self.base.base = base;
    }
    pub fn clone_into(&self) -> ScriptInputTrigger {
        let mut cloned = ScriptInputTrigger::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for ScriptInputTriggerBase {
    type Target = CustomPropertyTrigger;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScriptInputTriggerBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
