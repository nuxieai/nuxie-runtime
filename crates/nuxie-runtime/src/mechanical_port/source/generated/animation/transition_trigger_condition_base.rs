use crate::mechanical_port::source::{
    animation::transition_input_condition::TransitionInputCondition,
    animation::transition_trigger_condition::TransitionTriggerCondition,
    core::binary_reader::BinaryReader,
};

pub struct TransitionTriggerConditionBase {
    pub base: TransitionInputCondition,
}

impl Default for TransitionTriggerConditionBase {
    fn default() -> Self {
        Self {
            base: TransitionInputCondition::default(),
        }
    }
}

impl TransitionTriggerConditionBase {
    pub const TYPE_KEY: u16 = 68;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 67 | 476)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TransitionTriggerCondition {
        let mut cloned = TransitionTriggerCondition::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for TransitionTriggerConditionBase {
    type Target = TransitionInputCondition;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionTriggerConditionBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
