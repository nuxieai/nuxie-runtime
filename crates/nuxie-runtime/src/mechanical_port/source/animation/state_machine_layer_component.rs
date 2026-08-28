use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::animation::state_machine_layer_component_base::StateMachineLayerComponentBase,
};

#[derive(Default)]
pub struct StateMachineLayerComponent {
    pub base: StateMachineLayerComponentBase,
    events: Vec<CoreHandle>,
    listener_actions: Vec<CoreHandle>,
}

impl StateMachineLayerComponent {
    pub fn events(&self) -> &[CoreHandle] {
        &self.events
    }

    pub fn listener_actions(&self) -> &[CoreHandle] {
        &self.listener_actions
    }

    pub(crate) fn events_mut(&mut self) -> &mut Vec<CoreHandle> {
        &mut self.events
    }

    pub(crate) fn listener_actions_mut(&mut self) -> &mut Vec<CoreHandle> {
        &mut self.listener_actions
    }
}
impl std::ops::Deref for StateMachineLayerComponent {
    type Target = StateMachineLayerComponentBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for StateMachineLayerComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
