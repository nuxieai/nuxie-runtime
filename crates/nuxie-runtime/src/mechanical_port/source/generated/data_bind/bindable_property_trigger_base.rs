use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader,
    data_bind::bindable_property_integer::BindablePropertyInteger,
    data_bind::bindable_property_trigger::BindablePropertyTrigger,
};

pub struct BindablePropertyTriggerBase {
    pub base: BindablePropertyInteger,
}

impl Default for BindablePropertyTriggerBase {
    fn default() -> Self {
        Self {
            base: BindablePropertyInteger::default(),
        }
    }
}

impl BindablePropertyTriggerBase {
    pub const TYPE_KEY: u16 = 503;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 567 | 9)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> BindablePropertyTrigger {
        let mut cloned = BindablePropertyTrigger::default();
        cloned.base.copy(self);
        cloned
    }
}
