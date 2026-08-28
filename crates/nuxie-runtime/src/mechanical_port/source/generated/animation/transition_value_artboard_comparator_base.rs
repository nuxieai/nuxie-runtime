use crate::mechanical_port::source::{
    animation::transition_value_artboard_comparator::TransitionValueArtboardComparator,
    animation::transition_value_id_comparator::TransitionValueIdComparator,
    core::binary_reader::BinaryReader,
};

pub struct TransitionValueArtboardComparatorBase {
    pub base: TransitionValueIdComparator,
}

impl Default for TransitionValueArtboardComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionValueIdComparator::default(),
        }
    }
}

impl TransitionValueArtboardComparatorBase {
    pub const TYPE_KEY: u16 = 630;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 601 | 480 | 477)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TransitionValueArtboardComparator {
        let mut cloned = TransitionValueArtboardComparator::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for TransitionValueArtboardComparatorBase {
    type Target = TransitionValueIdComparator;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionValueArtboardComparatorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
