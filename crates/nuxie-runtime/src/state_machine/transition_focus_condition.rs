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
    // C++ casts the retained uint without normalizing it and then compares it
    // only with `TransitionConditionOp::equal`. Every other value, including
    // an unknown future value, therefore takes the negating branch.
    op_value: u32,
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
                        nuxie_schema::definition_by_name(comparator.type_name).is_some_and(
                            |definition| definition.is_a("TransitionPropertyComponentComparator"),
                        )
                    })
                    .or_else(|| {
                        comparators.left.filter(|comparator| {
                            nuxie_schema::definition_by_name(comparator.type_name).is_some_and(
                                |definition| {
                                    definition.is_a("TransitionPropertyComponentComparator")
                                },
                            )
                        })
                    })
            })
            .and_then(RuntimeTransitionPropertyComponentComparator::from_object)
            .map(RuntimeTransitionPropertyComponentComparator::local_id);
        Self {
            target_local_id,
            op_value: object
                .uint_property("opValue")
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(0),
        }
    }

    pub(super) fn new(target_local_id: usize, op: TransitionConditionOp) -> Self {
        Self {
            target_local_id: Some(target_local_id),
            op_value: match op {
                TransitionConditionOp::Equal => 0,
                TransitionConditionOp::NotEqual => 1,
                TransitionConditionOp::LessThanOrEqual => 2,
                TransitionConditionOp::GreaterThanOrEqual => 3,
                TransitionConditionOp::LessThan => 4,
                TransitionConditionOp::GreaterThan => 5,
            },
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
        if self.op_value == 0 {
            focused
        } else {
            !focused
        }
    }
}
