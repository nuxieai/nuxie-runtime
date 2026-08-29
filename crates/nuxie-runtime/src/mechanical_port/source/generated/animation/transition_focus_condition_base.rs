use crate::mechanical_port::source::{
    animation::transition_focus_condition::TransitionFocusCondition,
    animation::transition_viewmodel_condition::TransitionViewModelCondition,
    core::binary_reader::BinaryReader,
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
        matches!(type_key, Self::TYPE_KEY | 482 | 476)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TransitionFocusCondition {
        let mut cloned = TransitionFocusCondition::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for TransitionFocusConditionBase {
    type Target = TransitionViewModelCondition;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionFocusConditionBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
