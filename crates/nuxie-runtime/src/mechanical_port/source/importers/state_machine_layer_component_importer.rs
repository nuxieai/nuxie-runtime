use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::animation::{
    listener_action::ListenerAction, state_machine_fire_action::StateMachineFireAction,
    state_machine_layer_component::StateMachineLayerComponent,
};

use super::import_stack::ImportStackObject;

pub fn destroy_state_machine_layer_component(component: &mut StateMachineLayerComponent) {
    // Vec<Box<_>> performs the source destructor's ordered deletion.
    component.events_mut().clear();
}

pub struct StateMachineLayerComponentImporter {
    component: NonNull<StateMachineLayerComponent>,
}

impl StateMachineLayerComponentImporter {
    pub fn new(component: NonNull<StateMachineLayerComponent>) -> Self {
        Self { component }
    }
    pub fn add_fire_event(&mut self, fire_event: Box<StateMachineFireAction>) {
        unsafe { self.component.as_mut().events_mut().push(fire_event) };
    }
    pub fn add_listener_action(&mut self, action: Box<ListenerAction>) {
        unsafe { self.component.as_mut().listener_actions_mut().push(action) };
    }
}

impl ImportStackObject for StateMachineLayerComponentImporter {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
