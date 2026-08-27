use crate::mechanical_port::source::{
    animation::transition_comparator::TransitionComparator, core::binary_reader::BinaryReader,
};

pub struct TransitionPropertyComparatorBase {
    pub base: TransitionComparator,
}

impl Default for TransitionPropertyComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionComparator::default(),
        }
    }
}

impl TransitionPropertyComparatorBase {
    pub const TYPE_KEY: u16 = 478;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 477)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
