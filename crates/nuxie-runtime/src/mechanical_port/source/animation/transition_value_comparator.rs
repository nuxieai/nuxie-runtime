use crate::mechanical_port::source::generated::animation::transition_value_comparator_base::TransitionValueComparatorBase;

#[derive(Default)]
pub struct TransitionValueComparator {
    pub base: TransitionValueComparatorBase,
}
impl std::ops::Deref for TransitionValueComparator {
    type Target = TransitionValueComparatorBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TransitionValueComparator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
