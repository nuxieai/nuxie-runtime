use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation,
        listener_types::listener_input_type_gamepad::ListenerInputTypeGamepad,
        state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
        state_machine_listener::StateMachineListener,
    },
    core::CoreHandle,
    focus_data::{FocusData, RuntimeGamepadListenerHandle},
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Clone)]
pub struct RuntimeGamepadListenerGroupHandle(Rc<RefCell<GamepadListenerGroup>>);

#[derive(Clone, Default)]
pub struct RuntimeGamepadListenerGroupWeakHandle(Weak<RefCell<GamepadListenerGroup>>);

impl RuntimeGamepadListenerGroupHandle {
    pub fn new(
        focus_data: CoreHandle,
        listener: Option<CoreHandle>,
        state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
    ) -> Self {
        let handle = Self(Rc::new(RefCell::new(GamepadListenerGroup {
            occurrence: RuntimeGamepadListenerGroupWeakHandle::default(),
            focus_data,
            listener,
            state_machine_instance,
        })));
        let occurrence = handle.downgrade();
        handle.0.borrow_mut().occurrence = occurrence.clone();
        let focus_data = handle.0.borrow().focus_data.clone();
        focus_data.with_downcast_mut::<FocusData, _>(|focus_data| {
            focus_data.add_gamepad_listener(RuntimeGamepadListenerHandle::new(occurrence));
        });
        handle
    }

    pub fn downgrade(&self) -> RuntimeGamepadListenerGroupWeakHandle {
        RuntimeGamepadListenerGroupWeakHandle(Rc::downgrade(&self.0))
    }

    pub fn with_group_mut<R>(&self, use_group: impl FnOnce(&mut GamepadListenerGroup) -> R) -> R {
        use_group(&mut self.0.borrow_mut())
    }
}

impl RuntimeGamepadListenerGroupWeakHandle {
    pub fn upgrade(&self) -> Option<RuntimeGamepadListenerGroupHandle> {
        self.0.upgrade().map(RuntimeGamepadListenerGroupHandle)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

fn focus_parent(focus_data: &CoreHandle) -> Option<CoreHandle> {
    focus_data
        .with(|focus_data| focus_data.as_component()?.parent_handle())
        .flatten()
}

pub struct GamepadListenerGroup {
    occurrence: RuntimeGamepadListenerGroupWeakHandle,
    focus_data: CoreHandle,
    listener: Option<CoreHandle>,
    state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
}

impl GamepadListenerGroup {
    pub fn listener(&self) -> Option<CoreHandle> {
        self.listener.clone()
    }

    pub fn focus_data(&self) -> CoreHandle {
        self.focus_data.clone()
    }

    pub fn gamepad_dispatch(
        &mut self,
        invocation: &ListenerInvocation,
        out_scripted_drawable: Option<&mut Option<CoreHandle>>,
    ) -> bool {
        if let Some(parent) = focus_parent(&self.focus_data) {
            let scripted_result = parent.with_mut(|parent| {
                parent
                    .as_scripted_drawable_mut()
                    .map(|scripted| scripted.gamepad_dispatch(invocation))
            });
            if let Some(Some(handled)) = scripted_result {
                if let Some(output) = out_scripted_drawable {
                    *output = Some(parent);
                }
                return handled;
            }
        }
        let Some(listener) = self.listener.as_ref() else {
            return false;
        };
        let constraints_met = listener
            .with(|listener| {
                listener.as_state_machine_listener().map(|listener| {
                    ListenerInputTypeGamepad::gamepad_listener_constraints_met(
                        Some(listener),
                        invocation,
                    )
                })
            })
            .flatten()
            .unwrap_or(false);
        if !constraints_met {
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
        self.focus_data
            .with_downcast_mut::<FocusData, _>(|focus_data| {
                focus_data.remove_gamepad_listener(RuntimeGamepadListenerHandle::new(
                    self.occurrence.clone(),
                ));
            });
    }
}
