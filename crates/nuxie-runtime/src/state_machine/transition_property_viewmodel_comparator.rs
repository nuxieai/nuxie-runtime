//! ViewModel-property transition comparator ownership.
//!
//! Mirrors pinned C++
//! `src/animation/transition_property_viewmodel_comparator.cpp`.

use nuxie_binary::{RuntimeFile, RuntimeObject};

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionPropertyViewModelComparator<'a> {
    comparator: &'a RuntimeObject,
    bindable: &'a RuntimeObject,
}

impl<'a> RuntimeTransitionPropertyViewModelComparator<'a> {
    pub(super) fn from_object(
        file: &'a RuntimeFile,
        comparator: &'a RuntimeObject,
    ) -> Option<Self> {
        if comparator.type_name != "TransitionPropertyViewModelComparator" {
            return None;
        }
        Some(Self {
            comparator,
            bindable: file.latest_bindable_property_for_object(comparator)?,
        })
    }

    pub(super) fn bindable(self) -> &'a RuntimeObject {
        debug_assert_eq!(
            self.comparator.type_name,
            "TransitionPropertyViewModelComparator"
        );
        self.bindable
    }
}
