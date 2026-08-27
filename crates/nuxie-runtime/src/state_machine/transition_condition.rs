//! Base transition-condition owner and dispatch.
//!
//! Mirrors pinned C++ `src/animation/transition_condition.cpp`: one retained
//! base definition dispatches to the concrete condition subclass, while each
//! subclass keeps its filename-corresponding Rust implementation.

use super::transition_viewmodel_condition::RuntimeTransitionViewModelCondition;
use super::{
    RuntimeScheduledListenerActionExecutor, RuntimeScriptedTransitionCondition,
    RuntimeTransitionBoolCondition, RuntimeTransitionFocusCondition,
    RuntimeTransitionNumberCondition, RuntimeTransitionTriggerCondition, StateMachineInputInstance,
    TransitionEvaluationContext,
};
use crate::ArtboardInstance;
use nuxie_binary::{RuntimeFile, RuntimeObject};
use nuxie_graph::ArtboardGraph;

#[derive(Debug, Clone)]
pub(super) enum RuntimeTransitionCondition {
    Focus(RuntimeTransitionFocusCondition),
    Scripted(RuntimeScriptedTransitionCondition),
    Bool(RuntimeTransitionBoolCondition),
    Number(RuntimeTransitionNumberCondition),
    Trigger(RuntimeTransitionTriggerCondition),
    ViewModel(RuntimeTransitionViewModelCondition),
}

impl RuntimeTransitionCondition {
    /// Mechanical translation of `TransitionCondition::onAddedDirty`.
    ///
    /// The pinned callback unconditionally returns `StatusCode::Ok`; Rust's
    /// infallible equivalent is `()`.
    fn on_added_dirty(&self) {}

    /// Mechanical translation of `TransitionCondition::onAddedClean`.
    ///
    /// The pinned callback unconditionally returns `StatusCode::Ok`; Rust's
    /// infallible equivalent is `()`.
    fn on_added_clean(&self) {}

    pub(super) fn from_object(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        state_machine_inputs: &[Option<&RuntimeObject>],
        object: &RuntimeObject,
    ) -> Option<Self> {
        // The binary import-stack pass has already performed
        // `TransitionCondition::import`: it requires the latest retained
        // StateTransitionImporter, appends this exact object to that
        // transition, and then delegates to Core's infallible import.
        let condition = match object.type_name {
            "TransitionFocusCondition" => {
                Self::Focus(RuntimeTransitionFocusCondition::from_object(file, object))
            }
            "ScriptedTransitionCondition" => {
                Self::Scripted(RuntimeScriptedTransitionCondition::from_object(object))
            }
            "TransitionBoolCondition" => Self::Bool(RuntimeTransitionBoolCondition::from_object(
                state_machine_inputs,
                object,
            )?),
            "TransitionNumberCondition" => Self::Number(
                RuntimeTransitionNumberCondition::from_object(state_machine_inputs, object)?,
            ),
            "TransitionTriggerCondition" => Self::Trigger(
                RuntimeTransitionTriggerCondition::from_object(state_machine_inputs, object)?,
            ),
            "TransitionViewModelCondition" | "TransitionArtboardCondition" => {
                // Pinned C++ retains the condition occurrence even when
                // initialize() cannot form a compatible comparison. In that
                // case it owns ConditionComparisonNone and evaluates false.
                Self::ViewModel(
                    RuntimeTransitionViewModelCondition::from_object(file, graph, object)
                        .unwrap_or(RuntimeTransitionViewModelCondition::NoComparison),
                )
            }
            _ => return None,
        };

        condition.on_added_dirty();
        condition.on_added_clean();
        Some(condition)
    }

    pub(super) fn evaluate(
        &self,
        context: &TransitionEvaluationContext<'_>,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        executor: &dyn RuntimeScheduledListenerActionExecutor,
    ) -> bool {
        // The pinned base implementation returns true, but
        // TransitionCondition is abstract and every constructible condition
        // represented below overrides evaluate().
        match self {
            Self::Focus(condition) => condition.evaluate(executor),
            Self::Scripted(condition) => condition.evaluate(executor),
            Self::Bool(condition) => condition.evaluate(inputs),
            Self::Number(condition) => condition.evaluate(inputs),
            Self::Trigger(condition) => condition.evaluate(inputs, context.layer_index),
            Self::ViewModel(condition) => condition.evaluate(context, artboard, inputs, executor),
        }
    }

    /// Mechanical dispatch for `TransitionCondition::useInLayer`.
    pub(super) fn use_in_layer(
        &self,
        executor: &dyn RuntimeScheduledListenerActionExecutor,
        inputs: &mut [StateMachineInputInstance],
        layer_index: usize,
        view_model_trigger_layer_id: u64,
    ) {
        match self {
            Self::Trigger(condition) => condition.use_input(inputs, layer_index),
            Self::ViewModel(condition) => {
                condition.use_input(executor, inputs, layer_index, view_model_trigger_layer_id);
            }
            // Focus, scripted, bool, and number conditions inherit the pinned
            // base implementation, whose body is intentionally empty.
            _ => {}
        }
    }

    // The pinned base `validateInputType` returns true. There is no callable
    // Rust path to translate: TransitionCondition and TransitionInputCondition
    // are abstract, while every constructible input condition overrides the
    // validator and is checked by the binary import-stack pass.

    pub(super) fn is_direct_input(&self) -> bool {
        matches!(self, Self::Bool(_) | Self::Number(_) | Self::Trigger(_))
    }

    pub(super) fn evaluate_direct_input(
        &self,
        inputs: &[StateMachineInputInstance],
        layer_index: usize,
    ) -> Option<bool> {
        match self {
            Self::Bool(condition) => Some(condition.evaluate(inputs)),
            Self::Number(condition) => Some(condition.evaluate(inputs)),
            Self::Trigger(condition) => Some(condition.evaluate(inputs, layer_index)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::TransitionConditionOp;
    use super::*;

    #[test]
    fn base_dispatch_classifies_direct_input_conditions() {
        assert!(
            RuntimeTransitionCondition::Number(RuntimeTransitionNumberCondition::new(
                0,
                TransitionConditionOp::Equal,
                1.0,
            ))
            .is_direct_input()
        );
        assert!(
            RuntimeTransitionCondition::Bool(RuntimeTransitionBoolCondition::new(
                0,
                TransitionConditionOp::Equal,
            ))
            .is_direct_input()
        );
        assert!(
            !RuntimeTransitionCondition::Focus(RuntimeTransitionFocusCondition::new(
                0,
                TransitionConditionOp::Equal,
            ))
            .is_direct_input()
        );
        assert!(
            !RuntimeTransitionCondition::Scripted(RuntimeScriptedTransitionCondition::new(7))
                .is_direct_input()
        );
    }
}
