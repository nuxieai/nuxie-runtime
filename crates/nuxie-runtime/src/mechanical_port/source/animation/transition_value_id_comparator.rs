use crate::mechanical_port::source::generated::animation::transition_value_id_comparator_base::TransitionValueIdComparatorBase;

#[derive(Default)]
pub struct TransitionValueIdComparator {
    pub base: TransitionValueIdComparatorBase,
}
impl std::ops::Deref for TransitionValueIdComparator {
    type Target = TransitionValueIdComparatorBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TransitionValueIdComparator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::transition_value_id_comparator_base::TransitionValueIdComparatorBaseCallbacks for TransitionValueIdComparator { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
