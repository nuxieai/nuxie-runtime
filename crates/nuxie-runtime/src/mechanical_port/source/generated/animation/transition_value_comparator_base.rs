use crate::mechanical_port::source::{
    animation::transition_comparator::TransitionComparator, core::binary_reader::BinaryReader,
};

pub struct TransitionValueComparatorBase {
    pub base: TransitionComparator,
}

impl Default for TransitionValueComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionComparator::default(),
        }
    }
}

impl TransitionValueComparatorBase {
    pub const TYPE_KEY: u16 = 480;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 477)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for TransitionValueComparatorBase {
    type Target = TransitionComparator;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionValueComparatorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
