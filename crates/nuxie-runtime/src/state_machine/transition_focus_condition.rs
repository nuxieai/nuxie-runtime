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
    target_local_id: Option<usize>,
    op: TransitionConditionOp,
}

impl RuntimeTransitionFocusCondition {
    pub(super) fn from_object(file: &RuntimeFile, object: &RuntimeObject) -> Self {
        // Pinned C++ accepts the component comparator from either side because
        // runtime import can place the editor's right comparator in the left
        // slot (`transition_focus_condition.cpp:28-41`).
        let target_local_id = runtime_transition_comparators(file, object)
            .and_then(|comparators| {
                comparators
                    .right
                    .filter(|comparator| {
                        comparator.type_name == "TransitionPropertyComponentComparator"
                    })
                    .or_else(|| {
                        comparators.left.filter(|comparator| {
                            comparator.type_name == "TransitionPropertyComponentComparator"
                        })
                    })
            })
            .and_then(RuntimeTransitionPropertyComponentComparator::from_object)
            .map(RuntimeTransitionPropertyComponentComparator::local_id);
        Self {
            target_local_id,
            op: TransitionConditionOp::from_value(object.uint_property("opValue").unwrap_or(0)),
        }
    }

    pub(super) fn new(target_local_id: usize, op: TransitionConditionOp) -> Self {
        Self {
            target_local_id: Some(target_local_id),
            op,
        }
    }

    pub(super) fn evaluate(&self, executor: &dyn RuntimeScheduledListenerActionExecutor) -> bool {
        // Pinned C++ retains an authored focus condition even when neither
        // comparator is a component comparator. That occurrence evaluates
        // false before applying the authored operator
        // (`transition_focus_condition.cpp:30-39`).
        let Some(target_local_id) = self.target_local_id else {
            return false;
        };
        let focused = executor.target_has_focus(target_local_id);
        if self.op == TransitionConditionOp::Equal {
            focused
        } else {
            !focused
        }
    }
}
