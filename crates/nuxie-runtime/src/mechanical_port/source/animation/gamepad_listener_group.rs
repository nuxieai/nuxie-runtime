use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation,
        state_machine_instance::{
            RuntimeObjectHandle, RuntimeServicesHandle, RuntimeStateMachineInstanceWeakHandle,
        },
    },
    core::CoreHandle,
};

pub struct GamepadListenerGroup {
    runtime: RuntimeServicesHandle,
    focus_data: CoreHandle,
    listener: Option<CoreHandle>,
    state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
    group: RuntimeObjectHandle,
}

impl GamepadListenerGroup {
    pub fn new(
        runtime: RuntimeServicesHandle,
        focus_data: CoreHandle,
        listener: Option<CoreHandle>,
        state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
    ) -> Box<Self> {
        let group = runtime.borrow_mut().focus_data_add_gamepad_listener(
            &focus_data,
            listener.as_ref(),
            state_machine_instance.clone(),
        );
        Box::new(Self {
            runtime,
            focus_data,
            listener,
            state_machine_instance,
            group,
        })
    }

    pub fn listener(&self) -> Option<CoreHandle> {
        self.listener.clone()
    }

    pub fn focus_data(&self) -> CoreHandle {
        self.focus_data.clone()
    }

    pub fn gamepad_dispatch(
        &mut self,
        invocation: &ListenerInvocation,
        out_scripted_drawable: Option<&mut CoreHandle>,
    ) -> bool {
        if let Some((drawable, handled)) = self
            .runtime
            .borrow_mut()
            .focus_data_dispatch_scripted_gamepad(&self.focus_data, invocation)
        {
            if let Some(output) = out_scripted_drawable {
                *output = drawable;
            }
            return handled;
        }
        let Some(listener) = self.listener.as_ref() else {
            return false;
        };
        if !self
            .runtime
            .borrow()
            .listener_gamepad_constraints_met(listener, invocation)
        {
            return false;
        }
        self.state_machine_instance.with_instance_mut(|machine| {
            machine.perform_listener_changes(listener, invocation.clone());
            machine.mark_needs_advance();
        });
        false
    }
}

impl Drop for GamepadListenerGroup {
    fn drop(&mut self) {
        self.runtime
            .borrow_mut()
            .focus_data_remove_gamepad_listener(&self.focus_data, self.group);
    }
}
