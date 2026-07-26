//! Focus transition-condition definition and evaluation.
//!
//! Mirrors pinned C++ `src/animation/transition_focus_condition.cpp`.

use super::{
    RuntimeScheduledListenerActionExecutor, RuntimeTransitionPropertyComponentComparator,
    TransitionConditionOp, runtime_transition_comparators,
};
use nuxie_binary::{RuntimeFile, RuntimeObject};

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionFocusCondition {
    target_local_id: usize,
    op: TransitionConditionOp,
}

impl RuntimeTransitionFocusCondition {
    pub(super) fn from_object(file: &RuntimeFile, object: &RuntimeObject) -> Option<Self> {
        let comparators = runtime_transition_comparators(file, object)?;
        // Pinned C++ accepts the component comparator from either side because
        // runtime import can place the editor's right comparator in the left
        // slot (`transition_focus_condition.cpp:28-41`).
        let comparator = comparators
            .right
            .filter(|comparator| comparator.type_name == "TransitionPropertyComponentComparator")
            .or_else(|| {
                comparators.left.filter(|comparator| {
                    comparator.type_name == "TransitionPropertyComponentComparator"
                })
            })?;
        let comparator = RuntimeTransitionPropertyComponentComparator::from_object(comparator)?;
        Some(Self::new(
            comparator.local_id(),
            TransitionConditionOp::from_value(object.uint_property("opValue").unwrap_or(0)),
        ))
    }

    pub(super) fn new(target_local_id: usize, op: TransitionConditionOp) -> Self {
        Self {
            target_local_id,
            op,
        }
    }

    pub(super) fn evaluate(&self, executor: &dyn RuntimeScheduledListenerActionExecutor) -> bool {
        let focused = executor.target_has_focus(self.target_local_id);
        if self.op == TransitionConditionOp::Equal {
            focused
        } else {
            !focused
        }
    }
}
