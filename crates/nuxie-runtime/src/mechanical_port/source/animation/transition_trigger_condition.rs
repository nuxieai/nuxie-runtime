use crate::mechanical_port::source::animation::state_machine_instance::RuntimeStateMachineLayerInstanceWeakHandle;
use crate::mechanical_port::source::{
    generated::animation::{
        state_machine_trigger_base::StateMachineTriggerBase,
        transition_trigger_condition_base::TransitionTriggerConditionBase,
    },
    importers::import_stack::ImportStack,
    status_code::StatusCode,
};
pub trait TriggerConditionStateMachine {
    fn trigger_fired_and_unused_in_layer(
        &self,
        id: u32,
        layer: &RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> Option<bool>;
    fn use_trigger_in_layer(&mut self, id: u32, layer: RuntimeStateMachineLayerInstanceWeakHandle);
}
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
        machine: &dyn TriggerConditionStateMachine,
        layer: &RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> bool {
        let Some(fired_and_unused) =
            machine.trigger_fired_and_unused_in_layer(self.base.base.input_id(), layer)
        else {
            return true;
        };
        fired_and_unused
    }
    pub fn use_in_layer(
        &self,
        machine: &mut dyn TriggerConditionStateMachine,
        layer: RuntimeStateMachineLayerInstanceWeakHandle,
    ) {
        machine.use_trigger_in_layer(self.base.base.input_id(), layer);
    }
}
