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
        Self::TYPE_KEY == type_key
            || crate::mechanical_port::source::generated::core_type_tree::has_ancestor(
                Self::TYPE_KEY,
                type_key,
            )
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for TransitionPropertyComparatorBase {
    type Target = TransitionComparator;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionPropertyComparatorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
