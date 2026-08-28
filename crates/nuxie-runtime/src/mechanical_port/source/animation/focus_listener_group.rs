use crate::mechanical_port::source::{
    animation::state_machine_instance::{
        RuntimeObjectHandle, RuntimeServicesHandle, RuntimeStateMachineInstanceWeakHandle,
    },
    core::CoreHandle,
    listener_type::ListenerType,
};

pub struct FocusListenerGroup {
    runtime: RuntimeServicesHandle,
    focus_data: CoreHandle,
    listener: CoreHandle,
    state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
    group: RuntimeObjectHandle,
    is_focus_listener: bool,
    is_blur_listener: bool,
}

impl FocusListenerGroup {
    pub fn new(
        runtime: RuntimeServicesHandle,
        focus_data: CoreHandle,
        listener: CoreHandle,
        state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
    ) -> Box<Self> {
        let is_focus_listener = runtime
            .borrow()
            .listener_has(&listener, ListenerType::Focus);
        let is_blur_listener = runtime.borrow().listener_has(&listener, ListenerType::Blur);
        let group = runtime.borrow_mut().focus_data_add_focus_listener(
            &focus_data,
            &listener,
            state_machine_instance.clone(),
        );
        Box::new(Self {
            runtime,
            focus_data,
            listener,
            state_machine_instance,
            group,
            is_focus_listener,
            is_blur_listener,
        })
    }

    pub fn listener(&self) -> CoreHandle {
        self.listener.clone()
    }

    pub fn focus_data(&self) -> CoreHandle {
        self.focus_data.clone()
    }

    pub fn is_focus_listener(&self) -> bool {
        self.is_focus_listener
    }

    pub fn is_blur_listener(&self) -> bool {
        self.is_blur_listener
    }

    pub fn on_focused(&mut self) {
        if self.is_focus_listener {
            self.state_machine_instance
                .with_instance_mut(|machine| machine.queue_focus_event(self.group, true));
        }
    }

    pub fn on_blurred(&mut self) {
        if self.is_blur_listener {
            self.state_machine_instance
                .with_instance_mut(|machine| machine.queue_focus_event(self.group, false));
        }
    }
}

impl Drop for FocusListenerGroup {
    fn drop(&mut self) {
        self.runtime
            .borrow_mut()
            .focus_data_remove_focus_listener(&self.focus_data, self.group);
    }
}
