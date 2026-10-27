use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation,
        listener_types::listener_input_type_keyboard::ListenerInputTypeKeyboard,
        state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
        state_machine_listener::StateMachineListener,
    },
    core::CoreHandle,
    focus_data::{FocusData, RuntimeKeyboardListenerHandle},
    input::focusable::{Key, KeyModifiers},
    listener_type::ListenerType,
    scripted::scripted_drawable::ScriptedDrawable,
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Clone)]
pub struct RuntimeKeyboardListenerGroupHandle(Rc<RefCell<KeyboardListenerGroup>>);

#[derive(Clone, Default)]
pub struct RuntimeKeyboardListenerGroupWeakHandle(Weak<RefCell<KeyboardListenerGroup>>);

impl RuntimeKeyboardListenerGroupHandle {
    pub fn new(
        focus_data: CoreHandle,
        listener: Option<CoreHandle>,
        machine: RuntimeStateMachineInstanceWeakHandle,
    ) -> Self {
        let (register_keyboard, register_text) = listener.as_ref().map_or_else(
            || scripted_registration(&focus_data),
            |listener| {
                (
                    listener_has(listener, ListenerType::Keyboard),
                    listener_has(listener, ListenerType::TextInput),
                )
            },
        );
        // Retain registration phase even when the editor later changes it.
        let captures_keys = listener
            .as_ref()
            .and_then(|listener| {
                listener
                    .with(|listener| {
                        listener
                            .as_state_machine_listener()
                            .map(StateMachineListener::is_capture)
                    })
                    .flatten()
            })
            .unwrap_or(false);
        let handle = Self(Rc::new(RefCell::new(KeyboardListenerGroup {
            occurrence: RuntimeKeyboardListenerGroupWeakHandle::default(),
            focus_data,
            listener,
            machine,
            captures_keys,
            registered_keyboard: register_keyboard,
            registered_text: register_text,
        })));
        let occurrence = handle.downgrade();
        handle.0.borrow_mut().occurrence = occurrence.clone();
        let focus_data = handle.0.borrow().focus_data.clone();
        focus_data.with_downcast_mut::<FocusData, _>(|focus_data| {
            if register_keyboard {
                let registration = RuntimeKeyboardListenerHandle::new(occurrence.clone());
                if captures_keys {
                    focus_data.add_capture_keyboard_listener(registration);
                } else {
                    focus_data.add_keyboard_listener(registration);
                }
            }
            if register_text {
                focus_data.add_text_input_listener(RuntimeKeyboardListenerHandle::new(occurrence));
            }
        });
        handle
    }

    pub fn downgrade(&self) -> RuntimeKeyboardListenerGroupWeakHandle {
        RuntimeKeyboardListenerGroupWeakHandle(Rc::downgrade(&self.0))
    }

    pub fn with_group_mut<R>(&self, use_group: impl FnOnce(&mut KeyboardListenerGroup) -> R) -> R {
        use_group(&mut self.0.borrow_mut())
    }
}

impl RuntimeKeyboardListenerGroupWeakHandle {
    pub fn upgrade(&self) -> Option<RuntimeKeyboardListenerGroupHandle> {
        self.0.upgrade().map(RuntimeKeyboardListenerGroupHandle)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

fn listener_has(listener: &CoreHandle, kind: ListenerType) -> bool {
    listener
        .with(|listener| listener.state_machine_listener_has(kind))
        .flatten()
        .unwrap_or(false)
}

fn focus_parent(focus_data: &CoreHandle) -> Option<CoreHandle> {
    focus_data
        .with(|focus_data| focus_data.as_component()?.parent_handle())
        .flatten()
}

fn scripted_registration(focus_data: &CoreHandle) -> (bool, bool) {
    focus_parent(focus_data)
        .and_then(|parent| {
            parent.with(|parent| {
                parent.as_scripted_drawable().map(|scripted| {
                    (
                        scripted.scripted.wants_keyboard_input(),
                        scripted.scripted.wants_text_input(),
                    )
                })
            })
        })
        .flatten()
        .unwrap_or((false, false))
}

pub struct KeyboardListenerGroup {
    occurrence: RuntimeKeyboardListenerGroupWeakHandle,
    focus_data: CoreHandle,
    listener: Option<CoreHandle>,
    machine: RuntimeStateMachineInstanceWeakHandle,
    captures_keys: bool,
    registered_keyboard: bool,
    registered_text: bool,
}

impl KeyboardListenerGroup {
    pub fn listener(&self) -> Option<CoreHandle> {
        self.listener.clone()
    }

    pub fn focus_data(&self) -> CoreHandle {
        self.focus_data.clone()
    }

    pub fn key_input(
        &mut self,
        key: Key,
        modifiers: KeyModifiers,
        pressed: bool,
        repeat: bool,
    ) -> bool {
        if let Some(parent) = focus_parent(&self.focus_data) {
            if self.listener.is_none()
                && parent
                    .with(|parent| parent.as_scripted_drawable().is_some())
                    .unwrap_or(false)
            {
                return ScriptedDrawable::key_input_occurrence(
                    &parent, key, modifiers, pressed, repeat,
                );
            }
        }
        let Some(listener) = self.listener.as_ref() else {
            return false;
        };
        let constraints_met = listener
            .with(|listener| {
                listener.as_state_machine_listener().map(|listener| {
                    ListenerInputTypeKeyboard::keyboard_listener_constraints_met(
                        Some(listener),
                        key.raw(),
                        modifiers.bits(),
                        pressed,
                        repeat,
                    )
                })
            })
            .flatten()
            .unwrap_or(false);
        if constraints_met {
            self.machine.with_instance_mut(|machine| {
                machine.perform_listener_changes(
                    listener,
                    ListenerInvocation::keyboard(key.raw(), modifiers.bits(), pressed, repeat),
                );
            });
            let text_input = focus_parent(&self.focus_data).is_some_and(|parent| {
                parent
                    .with(|parent| parent.as_text_input().is_some())
                    .unwrap_or(false)
            });
            if text_input && !Self::owns_key(listener, key, modifiers) {
                return false;
            }
            // The matching listener consumes this key after its changes run.
            // FocusData still offers it to peers before reporting the claim up.
            return true;
        }
        false
    }

    fn owns_key(listener: &CoreHandle, key: Key, modifiers: KeyModifiers) -> bool {
        listener
            .with(|listener| {
                listener.as_state_machine_listener().map(|listener| {
                    ListenerInputTypeKeyboard::keyboard_listener_owns_key(
                        Some(listener),
                        key.raw(),
                        modifiers.bits(),
                    )
                })
            })
            .flatten()
            .unwrap_or(false)
    }
    pub fn capture_key_input(
        &mut self,
        key: Key,
        modifiers: KeyModifiers,
        pressed: bool,
        repeat: bool,
    ) -> bool {
        if !self.captures_keys {
            return false;
        }
        let Some(listener) = self.listener.as_ref() else {
            return false;
        };
        let constraints_met = listener
            .with(|listener| {
                listener.as_state_machine_listener().map(|listener| {
                    ListenerInputTypeKeyboard::keyboard_listener_constraints_met(
                        Some(listener),
                        key.raw(),
                        modifiers.bits(),
                        pressed,
                        repeat,
                    )
                })
            })
            .flatten()
            .unwrap_or(false);
        if constraints_met {
            self.machine.with_instance_mut(|machine| {
                machine.perform_listener_changes(
                    listener,
                    ListenerInvocation::keyboard(key.raw(), modifiers.bits(), pressed, repeat),
                )
            });
        }
        Self::owns_key(listener, key, modifiers)
    }

    pub fn text_input(&mut self, text: &str) -> bool {
        if let Some(parent) = focus_parent(&self.focus_data) {
            let result = parent.with_mut(|parent| {
                if let Some(text_input) = parent.as_text_input_mut() {
                    return Some(text_input.text_input(text));
                }
                None
            });
            if let Some(Some(result)) = result {
                return result;
            }
            if self.listener.is_none()
                && parent
                    .with(|parent| parent.as_scripted_drawable().is_some())
                    .unwrap_or(false)
            {
                return ScriptedDrawable::text_input_occurrence(&parent, text);
            }
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
        self.focus_data
            .with_downcast_mut::<FocusData, _>(|focus_data| {
                let registration = RuntimeKeyboardListenerHandle::new(self.occurrence.clone());
                if self.registered_keyboard {
                    if self.captures_keys {
                        focus_data.remove_capture_keyboard_listener(registration.clone());
                    } else {
                        focus_data.remove_keyboard_listener(registration.clone());
                    }
                }
                if self.registered_text {
                    focus_data.remove_text_input_listener(registration);
                }
            });
    }
}
