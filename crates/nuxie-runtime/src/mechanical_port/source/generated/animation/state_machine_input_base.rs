use crate::mechanical_port::source::{
    animation::state_machine_component::StateMachineComponent, core::binary_reader::BinaryReader,
};

pub struct StateMachineInputBase {
    pub base: StateMachineComponent,
}

impl Default for StateMachineInputBase {
    fn default() -> Self {
        Self {
            base: StateMachineComponent::default(),
        }
    }
}

impl StateMachineInputBase {
    pub const TYPE_KEY: u16 = 55;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 54)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}

impl std::ops::Deref for StateMachineInputBase {
    type Target = StateMachineComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StateMachineInputBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
