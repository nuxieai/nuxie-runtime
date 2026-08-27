use crate::mechanical_port::source::{
    animation::transition_artboard_condition::TransitionArtboardCondition,
    core::binary_reader::BinaryReader,
    transition_view_model_condition::TransitionViewModelCondition,
};

pub struct TransitionArtboardConditionBase {
    pub base: TransitionViewModelCondition,
}

impl Default for TransitionArtboardConditionBase {
    fn default() -> Self {
        Self {
            base: TransitionViewModelCondition::default(),
        }
    }
}

impl TransitionArtboardConditionBase {
    pub const TYPE_KEY: u16 = 497;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 482 | 476)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TransitionArtboardCondition {
        let mut cloned = TransitionArtboardCondition::default();
        cloned.base.copy(self);
        cloned
    }
}
