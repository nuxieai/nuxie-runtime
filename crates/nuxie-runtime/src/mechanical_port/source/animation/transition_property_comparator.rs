use crate::mechanical_port::source::generated::animation::transition_property_comparator_base::TransitionPropertyComparatorBase;

#[derive(Default)]
pub struct TransitionPropertyComparator {
    pub base: TransitionPropertyComparatorBase,
}
impl std::ops::Deref for TransitionPropertyComparator {
    type Target = TransitionPropertyComparatorBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TransitionPropertyComparator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
