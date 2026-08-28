use crate::mechanical_port::source::{
    animation::nested_state_machine::NestedStateMachine, core::binary_reader::BinaryReader,
    animation::nested_animation::NestedAnimation,
};

pub struct NestedStateMachineBase {
    pub base: NestedAnimation,
}

impl Default for NestedStateMachineBase {
    fn default() -> Self {
        Self {
            base: NestedAnimation::default(),
        }
    }
}

impl NestedStateMachineBase {
    pub const TYPE_KEY: u16 = 95;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 93 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> NestedStateMachine {
        let mut cloned = NestedStateMachine::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for NestedStateMachineBase {
    type Target = NestedAnimation;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NestedStateMachineBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
