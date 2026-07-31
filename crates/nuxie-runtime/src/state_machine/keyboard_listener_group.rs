use super::focused_input_dispatch::RuntimeInputDispatchOutcome;
use super::instance::StateMachineInstance;
use super::listener_types::RuntimeListenerType;
use super::{RuntimeStateMachineListener, ScriptListenerInvocation};
use crate::{ArtboardInstance, NoopScriptHost, ScriptedDrawableInputResult};

/// One occurrence of pinned C++ `KeyboardListenerGroup`.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeKeyboardListenerGroup {
    pub(crate) listener_index: Option<usize>,
    pub(crate) scripted_global_id: Option<u32>,
    pub(crate) target_local_id: usize,
    pub(crate) focus_data_local_id: usize,
    listens_keyboard: bool,
    listens_text: bool,
}

impl RuntimeKeyboardListenerGroup {
    pub(crate) fn new(
        listener_index: usize,
        focus_data_local_id: usize,
        listener: &RuntimeStateMachineListener,
    ) -> Option<Self> {
        let listens_keyboard = listener.has_listener(RuntimeListenerType::Keyboard);
        let listens_text = listener.has_listener(RuntimeListenerType::TextInput);
        (listens_keyboard || listens_text).then_some(Self {
            listener_index: Some(listener_index),
            scripted_global_id: None,
            target_local_id: listener.target_local_id,
            focus_data_local_id,
            listens_keyboard,
            listens_text,
        })
    }

    pub(crate) fn scripted(
        target_local_id: usize,
        focus_data_local_id: usize,
        scripted_global_id: u32,
        listens_keyboard: bool,
        listens_text: bool,
    ) -> Option<Self> {
        (listens_keyboard || listens_text).then_some(Self {
            listener_index: None,
            scripted_global_id: Some(scripted_global_id),
            target_local_id,
            focus_data_local_id,
            listens_keyboard,
            listens_text,
        })
    }

    fn text_input_parent_local(&self, artboard: &ArtboardInstance) -> Option<usize> {
        let parent_local = artboard.component_parent_local(self.focus_data_local_id)?;
        (artboard.runtime_object_type_name(parent_local) == Some("TextInput"))
            .then_some(parent_local)
    }

    /// Preserve C++ `KeyboardListenerGroup`'s TextInput-first dispatch
    /// boundary. Returning `Some` means the call belongs exclusively to the
    /// TextInput owner and must not fall through to a scripted drawable or
    /// authored listener.
    ///
    /// The editable `RawTextInput` implementation belongs to the later text
    /// owner family. Until that owner supplies its delegate, use the exact
    /// no-text-feature return from pinned C++ (`keyInput` is false) rather
    /// than approximating key editing in this listener owner.
    pub(crate) fn text_input_key_result(
        &self,
        artboard: &mut ArtboardInstance,
        key: u32,
        modifiers: u32,
        is_pressed: bool,
        is_repeat: bool,
    ) -> Option<bool> {
        let text_input_local = self.text_input_parent_local(artboard)?;
        Some(artboard.text_input_key_input(text_input_local, key, modifiers, is_pressed, is_repeat))
    }

    /// Pinned C++ returns true even without the text feature after routing the
    /// call to TextInput, so retain that boundary result until the text owner
    /// adds the live insertion delegate.
    pub(crate) fn text_input_text_result(
        &self,
        artboard: &mut ArtboardInstance,
        text: &str,
    ) -> Option<bool> {
        let text_input_local = self.text_input_parent_local(artboard)?;
        Some(artboard.text_input_text_input(text_input_local, text))
    }

    pub(crate) fn keyboard_invocation(
        &self,
        listener: &RuntimeStateMachineListener,
        key: u32,
        modifiers: u32,
        is_pressed: bool,
        is_repeat: bool,
    ) -> Option<ScriptListenerInvocation> {
        (self.listens_keyboard
            && listener.keyboard_constraints_met(key, modifiers, is_pressed, is_repeat))
        .then_some(ScriptListenerInvocation::Keyboard {
            key,
            modifiers,
            is_pressed,
            is_repeat,
        })
    }

    pub(crate) fn text_invocation(&self, text: &str) -> Option<ScriptListenerInvocation> {
        self.listens_text
            .then_some(ScriptListenerInvocation::TextInput {
                text: text.to_owned(),
            })
    }

    /// Execute one C++ `KeyboardListenerGroup::keyInput` occurrence.
    ///
    /// TextInput owns the call first, then an attached scripted occurrence
    /// has exclusive precedence, and only then does the authored listener
    /// constraint/action path run (`keyboard_listener_group.cpp:92-136`).
    pub(crate) fn key_input(
        &self,
        machine: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        key: u32,
        modifiers: u32,
        is_pressed: bool,
        is_repeat: bool,
    ) -> RuntimeInputDispatchOutcome {
        if self.listens_keyboard
            && let Some(handled) =
                self.text_input_key_result(artboard, key, modifiers, is_pressed, is_repeat)
        {
            return RuntimeInputDispatchOutcome::handled(handled);
        }

        let invocation = ScriptListenerInvocation::Keyboard {
            key,
            modifiers,
            is_pressed,
            is_repeat,
        };
        if let Some(global_id) = self.scripted_global_id {
            if !self.listens_keyboard {
                return RuntimeInputDispatchOutcome::default();
            }
            let Some(script) = artboard.script_instance_for_global(global_id) else {
                return RuntimeInputDispatchOutcome::default();
            };
            let result = script
                .borrow_mut()
                .call_scripted_drawable_input(&invocation, &mut NoopScriptHost);
            let outcome = machine.retain_protected_script_result(
                result,
                ScriptedDrawableInputResult {
                    invoked: true,
                    handled: false,
                },
            );
            if machine.script_error.is_some() {
                return RuntimeInputDispatchOutcome::terminal();
            }
            if outcome.invoked {
                artboard.wake_script_advance_for_global(global_id);
            }
            return RuntimeInputDispatchOutcome::handled(outcome.handled);
        }

        let Some(listener_index) = self.listener_index else {
            return RuntimeInputDispatchOutcome::default();
        };
        let Some(listener) = machine.listener_definitions.get(listener_index).cloned() else {
            return RuntimeInputDispatchOutcome::default();
        };
        let Some(invocation) =
            self.keyboard_invocation(&listener, key, modifiers, is_pressed, is_repeat)
        else {
            return RuntimeInputDispatchOutcome::default();
        };
        let result = listener.perform_changes(
            machine,
            artboard,
            None,
            &invocation,
            &mut NoopScriptHost,
            None,
        );
        let _: bool = machine.retain_script_result(result);
        if machine.script_error.is_some() {
            RuntimeInputDispatchOutcome::terminal()
        } else {
            RuntimeInputDispatchOutcome::default()
        }
    }

    /// Execute one C++ `KeyboardListenerGroup::textInput` occurrence with the
    /// same TextInput/script/listener precedence as `key_input`
    /// (`keyboard_listener_group.cpp:138-176`).
    pub(crate) fn text_input(
        &self,
        machine: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        text: &str,
    ) -> RuntimeInputDispatchOutcome {
        if self.listens_text
            && let Some(handled) = self.text_input_text_result(artboard, text)
        {
            return RuntimeInputDispatchOutcome::handled(handled);
        }

        let invocation = ScriptListenerInvocation::TextInput {
            text: text.to_owned(),
        };
        if let Some(global_id) = self.scripted_global_id {
            if !self.listens_text {
                return RuntimeInputDispatchOutcome::default();
            }
            let Some(script) = artboard.script_instance_for_global(global_id) else {
                return RuntimeInputDispatchOutcome::default();
            };
            let result = script
                .borrow_mut()
                .call_scripted_drawable_input(&invocation, &mut NoopScriptHost);
            let outcome = machine.retain_protected_script_result(
                result,
                ScriptedDrawableInputResult {
                    invoked: true,
                    handled: false,
                },
            );
            if machine.script_error.is_some() {
                return RuntimeInputDispatchOutcome::terminal();
            }
            if outcome.invoked {
                artboard.wake_script_advance_for_global(global_id);
            }
            return RuntimeInputDispatchOutcome::handled(outcome.handled);
        }

        let Some(listener_index) = self.listener_index else {
            return RuntimeInputDispatchOutcome::default();
        };
        let Some(listener) = machine.listener_definitions.get(listener_index).cloned() else {
            return RuntimeInputDispatchOutcome::default();
        };
        let Some(invocation) = self.text_invocation(text) else {
            return RuntimeInputDispatchOutcome::default();
        };
        let result = listener.perform_changes(
            machine,
            artboard,
            None,
            &invocation,
            &mut NoopScriptHost,
            None,
        );
        let _: bool = machine.retain_script_result(result);
        if machine.script_error.is_some() {
            RuntimeInputDispatchOutcome::terminal()
        } else {
            RuntimeInputDispatchOutcome::default()
        }
    }
}
