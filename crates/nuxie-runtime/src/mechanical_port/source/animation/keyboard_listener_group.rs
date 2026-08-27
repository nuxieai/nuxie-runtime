use crate::mechanical_port::source::animation::listener_invocation::ListenerInvocation;
use std::ptr::NonNull;
pub trait KeyboardListenerHost {
    fn add_keyboard(&mut self, group: NonNull<KeyboardListenerGroup>);
    fn remove_keyboard(&mut self, group: NonNull<KeyboardListenerGroup>);
    fn add_text(&mut self, group: NonNull<KeyboardListenerGroup>);
    fn remove_text(&mut self, group: NonNull<KeyboardListenerGroup>);
    fn listener_wants_keyboard(&self, listener: *const ()) -> bool;
    fn listener_wants_text(&self, listener: *const ()) -> bool;
    fn scripted_wants_keyboard(&self) -> bool;
    fn scripted_wants_text(&self) -> bool;
    fn text_input_key(
        &mut self,
        key: u32,
        modifiers: u32,
        pressed: bool,
        repeat: bool,
    ) -> Option<bool>;
    fn text_input_text(&mut self, text: &str) -> Option<bool>;
    #[cfg(feature = "rive_scripting")]
    fn scripted_key(
        &mut self,
        key: u32,
        modifiers: u32,
        pressed: bool,
        repeat: bool,
    ) -> Option<bool>;
    #[cfg(feature = "rive_scripting")]
    fn scripted_text(&mut self, text: &str) -> Option<bool>;
    fn keyboard_constraints_met(
        &self,
        listener: *const (),
        key: u32,
        modifiers: u32,
        pressed: bool,
        repeat: bool,
    ) -> bool;
    fn perform_changes(
        &mut self,
        machine: *mut (),
        listener: *const (),
        invocation: ListenerInvocation,
    );
}
pub struct KeyboardListenerGroup {
    focus_data: NonNull<dyn KeyboardListenerHost>,
    listener: *const (),
    machine: *mut (),
}
impl KeyboardListenerGroup {
    pub fn new(
        mut focus: NonNull<dyn KeyboardListenerHost>,
        listener: *const (),
        machine: *mut (),
    ) -> Box<Self> {
        let keyboard = if listener.is_null() {
            unsafe { focus.as_ref().scripted_wants_keyboard() }
        } else {
            unsafe { focus.as_ref().listener_wants_keyboard(listener) }
        };
        let text = if listener.is_null() {
            unsafe { focus.as_ref().scripted_wants_text() }
        } else {
            unsafe { focus.as_ref().listener_wants_text(listener) }
        };
        let mut value = Box::new(Self {
            focus_data: focus,
            listener,
            machine,
        });
        let this = NonNull::from(value.as_mut());
        unsafe {
            if keyboard {
                focus.as_mut().add_keyboard(this);
            }
            if text {
                focus.as_mut().add_text(this);
            }
        }
        value
    }
    pub fn key_input(&mut self, key: u32, modifiers: u32, pressed: bool, repeat: bool) -> bool {
        if let Some(result) = unsafe {
            self.focus_data
                .as_mut()
                .text_input_key(key, modifiers, pressed, repeat)
        } {
            return result;
        }
        #[cfg(feature = "rive_scripting")]
        if self.listener.is_null() {
            if let Some(result) = unsafe {
                self.focus_data
                    .as_mut()
                    .scripted_key(key, modifiers, pressed, repeat)
            } {
                return result;
            }
        }
        if !self.listener.is_null()
            && unsafe {
                self.focus_data.as_ref().keyboard_constraints_met(
                    self.listener,
                    key,
                    modifiers,
                    pressed,
                    repeat,
                )
            }
        {
            unsafe {
                self.focus_data.as_mut().perform_changes(
                    self.machine,
                    self.listener,
                    ListenerInvocation::keyboard(key, modifiers, pressed, repeat),
                );
            }
        }
        false
    }
    pub fn text_input(&mut self, text: &str) -> bool {
        if let Some(result) = unsafe { self.focus_data.as_mut().text_input_text(text) } {
            return result;
        }
        #[cfg(feature = "rive_scripting")]
        if self.listener.is_null() {
            if let Some(result) = unsafe { self.focus_data.as_mut().scripted_text(text) } {
                return result;
            }
        }
        if !self.listener.is_null() {
            unsafe {
                self.focus_data.as_mut().perform_changes(
                    self.machine,
                    self.listener,
                    ListenerInvocation::text_input(text.to_owned()),
                );
            }
        }
        false
    }
}
impl Drop for KeyboardListenerGroup {
    fn drop(&mut self) {
        let this = NonNull::from(&mut *self);
        let keyboard = if self.listener.is_null() {
            unsafe { self.focus_data.as_ref().scripted_wants_keyboard() }
        } else {
            unsafe {
                self.focus_data
                    .as_ref()
                    .listener_wants_keyboard(self.listener)
            }
        };
        let text = if self.listener.is_null() {
            unsafe { self.focus_data.as_ref().scripted_wants_text() }
        } else {
            unsafe { self.focus_data.as_ref().listener_wants_text(self.listener) }
        };
        unsafe {
            if keyboard {
                self.focus_data.as_mut().remove_keyboard(this);
            }
            if text {
                self.focus_data.as_mut().remove_text(this);
            }
        }
    }
}
