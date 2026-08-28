use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation,
        state_machine_instance::{
            RuntimeObjectHandle, RuntimeServicesHandle, RuntimeStateMachineInstanceWeakHandle,
        },
    },
    core::CoreHandle,
};

pub struct KeyboardListenerGroup {
    runtime: RuntimeServicesHandle,
    focus_data: CoreHandle,
    listener: Option<CoreHandle>,
    machine: RuntimeStateMachineInstanceWeakHandle,
    keyboard_registration: Option<RuntimeObjectHandle>,
    text_registration: Option<RuntimeObjectHandle>,
}

impl KeyboardListenerGroup {
    pub fn new(
        runtime: RuntimeServicesHandle,
        focus_data: CoreHandle,
        listener: Option<CoreHandle>,
        machine: RuntimeStateMachineInstanceWeakHandle,
    ) -> Box<Self> {
        let keyboard_registration = runtime.borrow_mut().focus_data_add_keyboard_listener(
            &focus_data,
            listener.as_ref(),
            machine.clone(),
        );
        let text_registration = runtime.borrow_mut().focus_data_add_text_listener(
            &focus_data,
            listener.as_ref(),
            machine.clone(),
        );
        Box::new(Self {
            runtime,
            focus_data,
            listener,
            machine,
            keyboard_registration,
            text_registration,
        })
    }

    pub fn listener(&self) -> Option<CoreHandle> {
        self.listener.clone()
    }

    pub fn focus_data(&self) -> CoreHandle {
        self.focus_data.clone()
    }

    pub fn key_input(&mut self, key: u32, modifiers: u32, pressed: bool, repeat: bool) -> bool {
        if let Some(result) = self.runtime.borrow_mut().focus_data_text_input_key(
            &self.focus_data,
            key,
            modifiers,
            pressed,
            repeat,
        ) {
            return result;
        }
        if self.listener.is_none()
            && let Some(result) = self.runtime.borrow_mut().focus_data_scripted_key(
                &self.focus_data,
                key,
                modifiers,
                pressed,
                repeat,
            )
        {
            return result;
        }
        if let Some(listener) = self.listener.as_ref()
            && self
                .runtime
                .borrow()
                .listener_keyboard_constraints_met(listener, key, modifiers, pressed, repeat)
        {
            self.machine.with_instance_mut(|machine| {
                machine.perform_listener_changes(
                    listener,
                    ListenerInvocation::keyboard(key, modifiers, pressed, repeat),
                );
            });
        }
        false
    }

    pub fn text_input(&mut self, text: &str) -> bool {
        if let Some(result) = self
            .runtime
            .borrow_mut()
            .focus_data_text_input_text(&self.focus_data, text)
        {
            return result;
        }
        if self.listener.is_none()
            && let Some(result) = self
                .runtime
                .borrow_mut()
                .focus_data_scripted_text(&self.focus_data, text)
        {
            return result;
        }
        if let Some(listener) = self.listener.as_ref() {
            self.machine.with_instance_mut(|machine| {
                machine.perform_listener_changes(
                    listener,
                    ListenerInvocation::text_input(text.to_owned()),
                );
            });
        }
        false
    }
}

impl Drop for KeyboardListenerGroup {
    fn drop(&mut self) {
        let mut runtime = self.runtime.borrow_mut();
        if let Some(group) = self.keyboard_registration {
            runtime.focus_data_remove_keyboard_listener(&self.focus_data, group);
        }
        if let Some(group) = self.text_registration {
            runtime.focus_data_remove_text_listener(&self.focus_data, group);
        }
    }
}
