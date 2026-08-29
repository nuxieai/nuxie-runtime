use crate::mechanical_port::source::{
    animation::state_machine_input_instance::StateMachineInputDefinition,
    generated::animation::state_machine_trigger_base::StateMachineTriggerBase,
};

#[derive(Default)]
pub struct StateMachineTrigger {
    pub base: StateMachineTriggerBase,
}

impl StateMachineInputDefinition for StateMachineTrigger {
    fn core_type(&self) -> u16 {
        StateMachineTriggerBase::TYPE_KEY
    }

    fn name(&self) -> &str {
        self.base.name()
    }
}
