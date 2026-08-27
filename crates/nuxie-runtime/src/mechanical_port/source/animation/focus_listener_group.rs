use crate::mechanical_port::source::listener_type::ListenerType;
use std::ptr::NonNull;
pub trait FocusListenerGroupHost {
    fn listener_has(&self, listener: *const (), listener_type: ListenerType) -> bool;
    fn add_focus_listener(&mut self, listener: NonNull<FocusListenerGroup>);
    fn remove_focus_listener(&mut self, listener: NonNull<FocusListenerGroup>);
    fn queue_focus_event(
        &mut self,
        machine: *mut (),
        group: NonNull<FocusListenerGroup>,
        focused: bool,
    );
}
pub struct FocusListenerGroup {
    focus_data: NonNull<dyn FocusListenerGroupHost>,
    listener: *const (),
    state_machine_instance: *mut (),
    is_focus_listener: bool,
    is_blur_listener: bool,
}
impl FocusListenerGroup {
    pub fn new(
        mut focus_data: NonNull<dyn FocusListenerGroupHost>,
        listener: *const (),
        state_machine_instance: *mut (),
    ) -> Box<Self> {
        let is_focus = unsafe {
            focus_data
                .as_ref()
                .listener_has(listener, ListenerType::Focus)
        };
        let is_blur = unsafe {
            focus_data
                .as_ref()
                .listener_has(listener, ListenerType::Blur)
        };
        let mut value = Box::new(Self {
            focus_data,
            listener,
            state_machine_instance,
            is_focus_listener: is_focus,
            is_blur_listener: is_blur,
        });
        unsafe {
            focus_data
                .as_mut()
                .add_focus_listener(NonNull::from(value.as_mut()))
        };
        value
    }
    pub fn listener(&self) -> *const () {
        self.listener
    }
    pub fn focus_data(&self) -> NonNull<dyn FocusListenerGroupHost> {
        self.focus_data
    }
    pub fn is_focus_listener(&self) -> bool {
        self.is_focus_listener
    }
    pub fn is_blur_listener(&self) -> bool {
        self.is_blur_listener
    }
    pub fn on_focused(&mut self) {
        if self.is_focus_listener {
            let this = NonNull::from(&mut *self);
            unsafe {
                self.focus_data
                    .as_mut()
                    .queue_focus_event(self.state_machine_instance, this, true)
            };
        }
    }
    pub fn on_blurred(&mut self) {
        if self.is_blur_listener {
            let this = NonNull::from(&mut *self);
            unsafe {
                self.focus_data
                    .as_mut()
                    .queue_focus_event(self.state_machine_instance, this, false)
            };
        }
    }
}
impl Drop for FocusListenerGroup {
    fn drop(&mut self) {
        let this = NonNull::from(&mut *self);
        unsafe { self.focus_data.as_mut().remove_focus_listener(this) };
    }
}
