use super::focused_input_dispatch::RuntimeInputDispatchOutcome;
use super::instance::StateMachineInstance;
use super::listener_types::{RuntimeGamepadInputEvent, RuntimeListenerType};
use super::{
    RuntimeStateMachineListener, ScriptGamepadInputChange, ScriptGamepadSnapshot,
    ScriptListenerInvocation,
};
use crate::{ArtboardInstance, NoopScriptHost, ScriptedDrawableInputResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeGamepadScriptedDrawable {
    pub(crate) global_id: u32,
    wants_connected: bool,
    wants_event: bool,
    wants_disconnected: bool,
}

impl RuntimeGamepadScriptedDrawable {
    pub(crate) fn new(
        global_id: u32,
        wants_connected: bool,
        wants_event: bool,
        wants_disconnected: bool,
    ) -> Option<Self> {
        (wants_connected || wants_event || wants_disconnected).then_some(Self {
            global_id,
            wants_connected,
            wants_event,
            wants_disconnected,
        })
    }

    pub(crate) fn accepts(&self, invocation: &ScriptListenerInvocation) -> bool {
        match invocation {
            ScriptListenerInvocation::GamepadConnected { .. } => self.wants_connected,
            ScriptListenerInvocation::GamepadEvent { .. } => self.wants_event,
            ScriptListenerInvocation::GamepadDisconnected { .. } => self.wants_disconnected,
            _ => false,
        }
    }
}

/// One occurrence of pinned C++ `GamepadListenerGroup`.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeGamepadListenerGroup {
    pub(crate) listener_index: usize,
    pub(crate) target_local_id: usize,
    pub(crate) focus_data_local_id: usize,
}

impl RuntimeGamepadListenerGroup {
    pub(crate) fn new(
        listener_index: usize,
        focus_data_local_id: usize,
        listener: &RuntimeStateMachineListener,
    ) -> Option<Self> {
        listener
            .has_listener(RuntimeListenerType::Gamepad)
            .then_some(Self {
                listener_index,
                target_local_id: listener.target_local_id,
                focus_data_local_id,
            })
    }

    pub(crate) fn constrained_invocation<'a>(
        &self,
        listener: &RuntimeStateMachineListener,
        invocation: &'a ScriptListenerInvocation,
    ) -> Option<&'a ScriptListenerInvocation> {
        let event = match invocation {
            ScriptListenerInvocation::GamepadConnected { .. } => {
                RuntimeGamepadInputEvent::Connected
            }
            ScriptListenerInvocation::GamepadDisconnected { .. } => {
                RuntimeGamepadInputEvent::Disconnected
            }
            ScriptListenerInvocation::GamepadEvent {
                change: ScriptGamepadInputChange::Button { index, value },
                standard_button_intent,
                ..
            } => RuntimeGamepadInputEvent::Button {
                index: u32::from(*index),
                value: *value,
                standard_intent: *standard_button_intent,
            },
            ScriptListenerInvocation::GamepadEvent {
                change: ScriptGamepadInputChange::Axis { index, .. },
                standard_axis_intent,
                ..
            } => RuntimeGamepadInputEvent::Axis {
                index: u32::from(*index),
                standard_intent: *standard_axis_intent,
            },
            _ => return None,
        };
        listener
            .gamepad_constraints_met(event)
            .then_some(invocation)
    }

    pub(crate) fn connected(snapshot: ScriptGamepadSnapshot) -> ScriptListenerInvocation {
        ScriptListenerInvocation::GamepadConnected { snapshot }
    }

    pub(crate) fn disconnected(device_id: i32) -> ScriptListenerInvocation {
        ScriptListenerInvocation::GamepadDisconnected { device_id }
    }

    /// Execute one pinned C++ `GamepadListenerGroup::gamepadDispatch`.
    ///
    /// A ScriptedDrawable parent has exclusive precedence under
    /// `WITH_RIVE_SCRIPTING`, including null-VM and missing-method false
    /// results. C++ selects this branch from the concrete parent type before
    /// asking the drawable to dispatch (`gamepad_listener_group.cpp:25-49`;
    /// `scripted_drawable.cpp:86-121`).
    pub(crate) fn gamepad_dispatch(
        &self,
        machine: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        invocation: &ScriptListenerInvocation,
    ) -> (RuntimeInputDispatchOutcome, Option<(u64, u32)>) {
        let scripted_global_id = artboard
            .component(self.target_local_id)
            .filter(|component| {
                nuxie_schema::definition_by_name(component.type_name)
                    .is_some_and(|definition| definition.is_a("ScriptedDrawable"))
            })
            .map(|component| component.global_id);
        if let Some(scripted_global_id) = scripted_global_id {
            let dispatched = Some((artboard.instance_identity(), scripted_global_id));
            let Some(script) = artboard.script_instance_for_global(scripted_global_id) else {
                return (RuntimeInputDispatchOutcome::default(), dispatched);
            };
            let result = script
                .borrow_mut()
                .call_scripted_drawable_input(invocation, &mut NoopScriptHost);
            let outcome = machine.retain_protected_script_result(
                result,
                ScriptedDrawableInputResult {
                    invoked: true,
                    handled: true,
                },
            );
            if machine.script_error.is_some() {
                return (RuntimeInputDispatchOutcome::terminal(), dispatched);
            }
            if outcome.invoked {
                artboard.wake_script_advance_for_global(scripted_global_id);
            }
            return (
                RuntimeInputDispatchOutcome::handled(outcome.handled),
                dispatched,
            );
        }

        let Some(listener) = machine
            .listener_definitions
            .get(self.listener_index)
            .cloned()
        else {
            return (RuntimeInputDispatchOutcome::default(), None);
        };
        if self.constrained_invocation(&listener, invocation).is_none() {
            return (RuntimeInputDispatchOutcome::default(), None);
        }
        let result = listener.perform_changes(
            machine,
            artboard,
            None,
            invocation,
            &mut NoopScriptHost,
            None,
        );
        let _: bool = machine.retain_script_result(result);
        if machine.script_error.is_some() {
            return (RuntimeInputDispatchOutcome::terminal(), None);
        }
        machine.needs_advance = true;
        (RuntimeInputDispatchOutcome::default(), None)
    }
}
