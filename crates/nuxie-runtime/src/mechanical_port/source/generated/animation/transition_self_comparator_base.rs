use crate::mechanical_port::source::{
    animation::transition_comparator::TransitionComparator,
    animation::transition_self_comparator::TransitionSelfComparator,
    core::binary_reader::BinaryReader,
};

pub struct TransitionSelfComparatorBase {
    pub base: TransitionComparator,
}

impl Default for TransitionSelfComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionComparator::default(),
        }
    }
}

impl TransitionSelfComparatorBase {
    pub const TYPE_KEY: u16 = 593;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 477)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TransitionSelfComparator {
        let mut cloned = TransitionSelfComparator::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for TransitionSelfComparatorBase {
    type Target = TransitionComparator;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionSelfComparatorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
