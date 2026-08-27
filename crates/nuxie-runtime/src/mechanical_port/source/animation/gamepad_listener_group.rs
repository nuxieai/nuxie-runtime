use crate::mechanical_port::source::animation::listener_invocation::ListenerInvocation;
use std::ptr::NonNull;
pub trait GamepadListenerGroupHost {
    fn add_gamepad_listener(&mut self, listener: NonNull<GamepadListenerGroup>);
    fn remove_gamepad_listener(&mut self, listener: NonNull<GamepadListenerGroup>);
    #[cfg(feature = "rive_scripting")]
    fn dispatch_to_parent_scripted_drawable(
        &mut self,
        invocation: &ListenerInvocation,
    ) -> Option<(*mut (), bool)>;
    fn gamepad_constraints_met(&self, listener: *const (), invocation: &ListenerInvocation)
    -> bool;
    fn perform_changes(
        &mut self,
        machine: *mut (),
        listener: *const (),
        invocation: &ListenerInvocation,
    );
    fn mark_needs_advance(&mut self, machine: *mut ());
}
pub struct GamepadListenerGroup {
    focus_data: NonNull<dyn GamepadListenerGroupHost>,
    listener: *const (),
    state_machine_instance: *mut (),
}
impl GamepadListenerGroup {
    pub fn new(
        mut focus_data: NonNull<dyn GamepadListenerGroupHost>,
        listener: *const (),
        state_machine_instance: *mut (),
    ) -> Box<Self> {
        let mut value = Box::new(Self {
            focus_data,
            listener,
            state_machine_instance,
        });
        unsafe {
            focus_data
                .as_mut()
                .add_gamepad_listener(NonNull::from(value.as_mut()))
        };
        value
    }
    pub fn listener(&self) -> *const () {
        self.listener
    }
    pub fn focus_data(&self) -> NonNull<dyn GamepadListenerGroupHost> {
        self.focus_data
    }
    pub fn gamepad_dispatch(
        &mut self,
        invocation: &ListenerInvocation,
        out_scripted_drawable: Option<&mut *mut ()>,
    ) -> bool {
        #[cfg(feature = "rive_scripting")]
        if let Some((drawable, handled)) = unsafe {
            self.focus_data
                .as_mut()
                .dispatch_to_parent_scripted_drawable(invocation)
        } {
            if let Some(output) = out_scripted_drawable {
                *output = drawable;
            }
            return handled;
        }
        if self.listener.is_null()
            || !unsafe {
                self.focus_data
                    .as_ref()
                    .gamepad_constraints_met(self.listener, invocation)
            }
        {
            return false;
        }
        unsafe {
            self.focus_data.as_mut().perform_changes(
                self.state_machine_instance,
                self.listener,
                invocation,
            );
            self.focus_data
                .as_mut()
                .mark_needs_advance(self.state_machine_instance);
        }
        false
    }
}
impl Drop for GamepadListenerGroup {
    fn drop(&mut self) {
        let this = NonNull::from(&mut *self);
        unsafe { self.focus_data.as_mut().remove_gamepad_listener(this) };
    }
}
