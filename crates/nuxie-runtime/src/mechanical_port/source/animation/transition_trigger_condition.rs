use crate::mechanical_port::source::animation::state_machine_instance::{
    RuntimeStateMachineLayerInstanceWeakHandle, StateMachineInstance,
};
use crate::mechanical_port::source::{
    generated::animation::{
        state_machine_trigger_base::StateMachineTriggerBase,
        transition_trigger_condition_base::TransitionTriggerConditionBase,
    },
    importers::import_stack::ImportStack,
    status_code::StatusCode,
};
pub trait TriggerInputKind {
    fn is_state_machine_trigger(&self) -> bool;
}
#[derive(Default)]
pub struct TransitionTriggerCondition {
    pub base: TransitionTriggerConditionBase,
}
impl TransitionTriggerCondition {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.base.import_with(import_stack, |input| {
            input.is_type_of(StateMachineTriggerBase::TYPE_KEY)
        })
    }

    pub fn validate_input_type(&self, input: Option<&dyn TriggerInputKind>) -> bool {
        input.is_none() || input.is_some_and(TriggerInputKind::is_state_machine_trigger)
    }
    pub fn evaluate(
        &self,
        machine: &StateMachineInstance,
        layer: &RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> bool {
        let Some(trigger) = machine.trigger_input(self.base.base.input_id()) else {
            return true;
        };
        trigger.fired() && !trigger.triggerable.is_used_in_layer(layer)
    }
    pub fn use_in_layer(
        &self,
        machine: &mut StateMachineInstance,
        layer: RuntimeStateMachineLayerInstanceWeakHandle,
    ) {
        if let Some(trigger) = machine.trigger_input_mut(self.base.base.input_id()) {
            trigger.triggerable.use_in_layer(layer);
        }
    }
}
