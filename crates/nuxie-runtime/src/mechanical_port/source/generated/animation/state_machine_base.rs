use crate::mechanical_port::source::{
    animation::animation::Animation, animation::state_machine::StateMachine,
    core::binary_reader::BinaryReader,
};

pub struct StateMachineBase {
    pub base: Animation,
}

impl Default for StateMachineBase {
    fn default() -> Self {
        Self {
            base: Animation::default(),
        }
    }
}

impl StateMachineBase {
    pub const TYPE_KEY: u16 = 53;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 27)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> StateMachine {
        let mut cloned = StateMachine::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for StateMachineBase {
    type Target = Animation;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StateMachineBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
