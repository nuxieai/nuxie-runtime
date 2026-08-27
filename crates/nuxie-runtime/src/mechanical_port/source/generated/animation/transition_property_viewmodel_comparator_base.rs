use crate::mechanical_port::source::animation::transition_property_viewmodel_comparator::TransitionPropertyViewModelComparator;

use crate::mechanical_port::source::{
    animation::transition_property_comparator::TransitionPropertyComparator,
    core::binary_reader::BinaryReader,
};

pub struct TransitionPropertyViewModelComparatorBase {
    pub base: TransitionPropertyComparator,
}

impl Default for TransitionPropertyViewModelComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionPropertyComparator::default(),
        }
    }
}

impl TransitionPropertyViewModelComparatorBase {
    pub const TYPE_KEY: u16 = 479;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 478 | 477)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TransitionPropertyViewModelComparator {
        let mut cloned = TransitionPropertyViewModelComparator::default();
        cloned.base.copy(self);
        cloned
    }
}
