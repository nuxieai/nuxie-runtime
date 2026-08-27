use crate::mechanical_port::source::{
    animation::transition_focus_condition::TransitionFocusCondition,
    core::binary_reader::BinaryReader,
    transition_view_model_condition::TransitionViewModelCondition,
};

pub struct TransitionFocusConditionBase {
    pub base: TransitionViewModelCondition,
}

impl Default for TransitionFocusConditionBase {
    fn default() -> Self {
        Self {
            base: TransitionViewModelCondition::default(),
        }
    }
}

impl TransitionFocusConditionBase {
    pub const TYPE_KEY: u16 = 1038;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 0 | 476)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TransitionFocusCondition {
        let mut cloned = TransitionFocusCondition::default();
        cloned.base.copy(self);
        cloned
    }
}
