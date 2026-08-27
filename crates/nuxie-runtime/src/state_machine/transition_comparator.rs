//! Transition-comparator attachment.
//!
//! Mirrors pinned C++ `src/animation/transition_comparator.cpp`: comparator
//! records attach to their containing ViewModel condition in authored order.
//! The primary header adds no retained fields and gives the base comparator a
//! no-op `useInLayer` implementation.

use nuxie_binary::{RuntimeFile, RuntimeObject, RuntimeTransitionViewModelConditionComparators};

pub(super) fn runtime_transition_comparators<'a>(
    file: &'a RuntimeFile,
    condition: &RuntimeObject,
) -> Option<RuntimeTransitionViewModelConditionComparators<'a>> {
    file.transition_view_model_condition_comparators(condition)
}

/// Mechanical translation of `TransitionComparator::useInLayer`.
pub(super) fn use_in_layer() {}
