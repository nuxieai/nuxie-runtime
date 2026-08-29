use crate::mechanical_port::source::{
    animation::state_machine_input::StateMachineInput,
    animation::state_machine_trigger::StateMachineTrigger, core::binary_reader::BinaryReader,
};

pub struct StateMachineTriggerBase {
    pub base: StateMachineInput,
}

impl Default for StateMachineTriggerBase {
    fn default() -> Self {
        Self {
            base: StateMachineInput::default(),
        }
    }
}

impl StateMachineTriggerBase {
    pub const TYPE_KEY: u16 = 58;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 55 | 54)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> StateMachineTrigger {
        let mut cloned = StateMachineTrigger::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for StateMachineTriggerBase {
    type Target = StateMachineInput;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StateMachineTriggerBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
