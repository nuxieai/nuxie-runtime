use crate::mechanical_port::source::{
    animation::state_machine_input_instance::StateMachineInputDefinition,
    generated::animation::state_machine_bool_base::StateMachineBoolBase,
};

#[derive(Default)]
pub struct StateMachineBool {
    pub base: StateMachineBoolBase,
}

impl StateMachineInputDefinition for StateMachineBool {
    fn core_type(&self) -> u16 {
        StateMachineBoolBase::TYPE_KEY
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn bool_value(&self) -> bool {
        self.base.value()
    }
}
