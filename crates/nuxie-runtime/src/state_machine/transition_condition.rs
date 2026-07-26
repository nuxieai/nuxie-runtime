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
    pub(super) fn is_direct_input(&self) -> bool {
        matches!(self, Self::Bool(_) | Self::Number(_) | Self::Trigger(_))
    }

    pub(super) fn can_change_during_artboard_update(&self) -> bool {
        !self.is_direct_input()
    }

    pub(super) fn from_object(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        state_machine_inputs: &[Option<&RuntimeObject>],
        object: &RuntimeObject,
    ) -> Option<Self> {
        match object.type_name {
            "TransitionFocusCondition" => Some(Self::Focus(
                RuntimeTransitionFocusCondition::from_object(file, object),
            )),
            "ScriptedTransitionCondition" => Some(Self::Scripted(
                RuntimeScriptedTransitionCondition::from_object(object),
            )),
            "TransitionBoolCondition" => {
                RuntimeTransitionBoolCondition::from_object(state_machine_inputs, object)
                    .map(Self::Bool)
            }
            "TransitionNumberCondition" => {
                RuntimeTransitionNumberCondition::from_object(state_machine_inputs, object)
                    .map(Self::Number)
            }
            "TransitionTriggerCondition" => {
                RuntimeTransitionTriggerCondition::from_object(state_machine_inputs, object)
                    .map(Self::Trigger)
            }
            "TransitionViewModelCondition" | "TransitionArtboardCondition" => {
                // Pinned C++ retains the condition occurrence even when
                // initialize() cannot form a compatible comparison. In that
                // case it owns ConditionComparisonNone and evaluates false.
                Some(Self::ViewModel(
                    RuntimeTransitionViewModelCondition::from_object(file, graph, object)
                        .unwrap_or(RuntimeTransitionViewModelCondition::NoComparison),
                ))
            }
            _ => None,
        }
    }

    pub(super) fn evaluate(
        &self,
        context: &TransitionEvaluationContext<'_>,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        executor: &dyn RuntimeScheduledListenerActionExecutor,
    ) -> bool {
        match self {
            Self::Focus(condition) => condition.evaluate(executor),
            Self::Scripted(condition) => condition.evaluate(executor),
            Self::Bool(condition) => condition.evaluate(inputs),
            Self::Number(condition) => condition.evaluate(inputs),
            Self::Trigger(condition) => condition.evaluate(inputs, context.layer_index),
            Self::ViewModel(condition) => condition.evaluate(context, artboard, inputs, executor),
        }
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

    pub(super) fn use_input(
        &self,
        executor: &dyn RuntimeScheduledListenerActionExecutor,
        inputs: &mut [StateMachineInputInstance],
        layer_index: usize,
        view_model_trigger_layer_id: u64,
    ) {
        if let Self::Trigger(condition) = self {
            condition.use_input(inputs, layer_index);
        }
        if let Self::ViewModel(condition) = self {
            condition.use_input(executor, inputs, layer_index, view_model_trigger_layer_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::TransitionConditionOp;
    use super::*;

    #[test]
    fn base_dispatch_classifies_direct_and_live_conditions() {
        assert!(
            !RuntimeTransitionCondition::Number(RuntimeTransitionNumberCondition::new(
                0,
                TransitionConditionOp::Equal,
                1.0,
            ))
            .can_change_during_artboard_update()
        );
        assert!(
            !RuntimeTransitionCondition::Bool(RuntimeTransitionBoolCondition::new(
                0,
                TransitionConditionOp::Equal,
            ))
            .can_change_during_artboard_update()
        );
        assert!(
            RuntimeTransitionCondition::Focus(RuntimeTransitionFocusCondition::new(
                0,
                TransitionConditionOp::Equal,
            ))
            .can_change_during_artboard_update()
        );
        assert!(
            RuntimeTransitionCondition::Scripted(RuntimeScriptedTransitionCondition::new(7))
                .can_change_during_artboard_update()
        );
    }
}
