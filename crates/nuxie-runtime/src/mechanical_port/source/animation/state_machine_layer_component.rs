use crate::mechanical_port::source::{
    animation::{
        listener_action::ListenerAction, state_machine_fire_action::StateMachineFireAction,
    },
    generated::animation::state_machine_layer_component_base::StateMachineLayerComponentBase,
};

#[derive(Default)]
pub struct StateMachineLayerComponent {
    pub base: StateMachineLayerComponentBase,
    events: Vec<Box<StateMachineFireAction>>,
    listener_actions: Vec<Box<ListenerAction>>,
}

impl StateMachineLayerComponent {
    pub fn events(&self) -> &[Box<StateMachineFireAction>] {
        &self.events
    }

    pub fn listener_actions(&self) -> &[Box<ListenerAction>] {
        &self.listener_actions
    }

    pub(crate) fn events_mut(&mut self) -> &mut Vec<Box<StateMachineFireAction>> {
        &mut self.events
    }

    pub(crate) fn listener_actions_mut(&mut self) -> &mut Vec<Box<ListenerAction>> {
        &mut self.listener_actions
    }
}

impl Drop for StateMachineLayerComponent {
    fn drop(&mut self) {
        for event in self.events.drain(..) {
            drop(event);
        }
    }
}
