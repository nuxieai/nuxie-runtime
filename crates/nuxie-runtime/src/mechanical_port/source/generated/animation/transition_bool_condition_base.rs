use crate::mechanical_port::source::{
    animation::transition_bool_condition::TransitionBoolCondition,
    animation::transition_value_condition::TransitionValueCondition,
    core::binary_reader::BinaryReader,
};

pub struct TransitionBoolConditionBase {
    pub base: TransitionValueCondition,
}

impl Default for TransitionBoolConditionBase {
    fn default() -> Self {
        Self {
            base: TransitionValueCondition::default(),
        }
    }
}

impl TransitionBoolConditionBase {
    pub const TYPE_KEY: u16 = 71;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 69 | 67 | 476)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TransitionBoolCondition {
        let mut cloned = TransitionBoolCondition::default();
        cloned.base.copy(self);
        cloned
    }
}
