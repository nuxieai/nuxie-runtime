use std::any::Any;

use crate::mechanical_port::source::{
    animation::state_machine_layer_component::StateMachineLayerComponent, core::CoreHandle,
};

use super::import_stack::ImportStackObject;

pub fn destroy_state_machine_layer_component(component: &mut StateMachineLayerComponent) {
    // The arena owns each event occurrence; clearing the ordered handle list
    // performs the source destructor's relationship teardown.
    component.events_mut().clear();
}

pub struct StateMachineLayerComponentImporter {
    component: CoreHandle,
}

impl StateMachineLayerComponentImporter {
    pub fn new(component: CoreHandle) -> Self {
        Self { component }
    }
    pub fn add_fire_event(&mut self, fire_event: CoreHandle) {
        self.component
            .with_mut(|component| component.state_machine_layer_component_add_event(fire_event))
            .filter(|added| *added)
            .expect("imported component derives from StateMachineLayerComponent");
    }
    pub fn add_listener_action(&mut self, action: CoreHandle) {
        self.component
            .with_mut(|component| {
                component.state_machine_layer_component_add_listener_action(action)
            })
            .filter(|added| *added)
            .expect("imported component derives from StateMachineLayerComponent");
    }
}

impl ImportStackObject for StateMachineLayerComponentImporter {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
