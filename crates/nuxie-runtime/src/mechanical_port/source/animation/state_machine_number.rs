use crate::mechanical_port::source::{
    animation::state_machine_input_instance::StateMachineInputDefinition,
    generated::animation::state_machine_number_base::StateMachineNumberBase,
};

#[derive(Default)]
pub struct StateMachineNumber {
    pub base: StateMachineNumberBase,
}

impl StateMachineInputDefinition for StateMachineNumber {
    fn core_type(&self) -> u16 {
        StateMachineNumberBase::TYPE_KEY
    }

    fn name(&self) -> &str {
        self.base.name()
    }

    fn number_value(&self) -> f32 {
        self.base.value()
    }
}
