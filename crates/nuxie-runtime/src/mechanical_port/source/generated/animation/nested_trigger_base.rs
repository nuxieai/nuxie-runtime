use crate::mechanical_port::source::{
    animation::nested_input::NestedInput, animation::nested_trigger::NestedTrigger,
    core::binary_reader::BinaryReader,
};

pub trait NestedTriggerBaseCallbacks:
    crate::mechanical_port::source::generated::animation::nested_input_base::NestedInputBaseCallbacks
{
    fn fire(&mut self, value: &mut CallbackData<'_>);
}

pub struct NestedTriggerBase {
    pub base: NestedInput,
}

impl Default for NestedTriggerBase {
    fn default() -> Self {
        Self {
            base: NestedInput::default(),
        }
    }
}

impl NestedTriggerBase {
    pub const TYPE_KEY: u16 = 122;
    pub const FIRE_PROPERTY_KEY: u16 = 401;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 121 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self, callbacks: &mut impl NestedTriggerBaseCallbacks) -> NestedTrigger {
        let mut cloned = NestedTrigger::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
}

impl std::ops::Deref for NestedTriggerBase {
    type Target = NestedInput;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NestedTriggerBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
