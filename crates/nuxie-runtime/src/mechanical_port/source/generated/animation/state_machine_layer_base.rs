use crate::mechanical_port::source::{
    animation::state_machine_component::StateMachineComponent,
    animation::state_machine_layer::StateMachineLayer, core::binary_reader::BinaryReader,
};

pub struct StateMachineLayerBase {
    pub base: StateMachineComponent,
}

impl Default for StateMachineLayerBase {
    fn default() -> Self {
        Self {
            base: StateMachineComponent::default(),
        }
    }
}

impl StateMachineLayerBase {
    pub const TYPE_KEY: u16 = 57;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 54)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> StateMachineLayer {
        let mut cloned = StateMachineLayer::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for StateMachineLayerBase {
    type Target = StateMachineComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StateMachineLayerBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
