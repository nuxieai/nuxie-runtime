use crate::mechanical_port::source::{
    animation::transition_value_asset_comparator::TransitionValueAssetComparator,
    animation::transition_value_id_comparator::TransitionValueIdComparator,
    core::binary_reader::BinaryReader,
};

pub struct TransitionValueAssetComparatorBase {
    pub base: TransitionValueIdComparator,
}

impl Default for TransitionValueAssetComparatorBase {
    fn default() -> Self {
        Self {
            base: TransitionValueIdComparator::default(),
        }
    }
}

impl TransitionValueAssetComparatorBase {
    pub const TYPE_KEY: u16 = 602;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 601 | 480 | 477)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> TransitionValueAssetComparator {
        let mut cloned = TransitionValueAssetComparator::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for TransitionValueAssetComparatorBase {
    type Target = TransitionValueIdComparator;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransitionValueAssetComparatorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
