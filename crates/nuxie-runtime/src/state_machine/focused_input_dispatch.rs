use super::ScriptListenerInvocation;
use crate::ArtboardInstance;
use crate::artboard::{RuntimeNestedAnimationInstance, RuntimeNestedArtboardInstance};

/// Result of dispatching one focused-input batch through an occurrence.
///
/// Ordinary protected-call failures are consumed by the scripting owner like
/// C++. A typed resource-limit failure is Rust's binding safety fence and
/// stops every later callback in the current cross-artboard batch.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct RuntimeInputDispatchOutcome {
    pub(crate) handled: bool,
    pub(crate) terminal_resource_failure: bool,
}

impl RuntimeInputDispatchOutcome {
    pub(crate) const fn handled(handled: bool) -> Self {
        Self {
            handled,
            terminal_resource_failure: false,
        }
    }

    pub(crate) const fn terminal() -> Self {
        Self {
            handled: false,
            terminal_resource_failure: true,
        }
    }
}

/// Route one focus-node input callback to the nested state-machine occurrence
/// that registered it on the shared C++ FocusManager.
///
/// Pinned C++ stores listener-group pointers directly on each FocusData, so a
/// root manager naturally invokes groups owned by nested state machines. Rust
/// retains those groups on their owning machine and uses the shared focus
/// node's Artboard-occurrence identity to reach that exact owner.
impl ArtboardInstance {
    pub(crate) fn dispatch_nested_key_input_at_focus(
        &mut self,
        owner_identity: u64,
        focus_data_local_id: usize,
        key: u32,
        modifiers: u32,
        is_pressed: bool,
        is_repeat: bool,
    ) -> RuntimeInputDispatchOutcome {
        for nested in self.nested_artboards.values_mut() {
            if nested.child.instance_identity() == owner_identity {
                let RuntimeNestedArtboardInstance {
                    animations, child, ..
                } = nested;
                for animation in animations {
                    let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                        continue;
                    };
                    let Some(machine) = occurrence.state_machine_mut() else {
                        continue;
                    };
                    let outcome = machine.key_input_at_focus_data(
                        child,
                        focus_data_local_id,
                        key,
                        modifiers,
                        is_pressed,
                        is_repeat,
                    );
                    if outcome.handled || outcome.terminal_resource_failure {
                        return outcome;
                    }
                }
                return RuntimeInputDispatchOutcome::default();
            }
            let outcome = nested.child.dispatch_nested_key_input_at_focus(
                owner_identity,
                focus_data_local_id,
                key,
                modifiers,
                is_pressed,
                is_repeat,
            );
            if outcome.handled || outcome.terminal_resource_failure {
                return outcome;
            }
        }

        let list_locals = self.component_list_locals().collect::<Vec<_>>();
        for list_local_id in list_locals {
            let Some(items) = self.component_list_items_mut(list_local_id) else {
                continue;
            };
            for item in items {
                if item.child.instance_identity() == owner_identity {
                    for machine in &mut item.state_machines {
                        let outcome = machine.key_input_at_focus_data(
                            &mut item.child,
                            focus_data_local_id,
                            key,
                            modifiers,
                            is_pressed,
                            is_repeat,
                        );
                        if outcome.handled || outcome.terminal_resource_failure {
                            return outcome;
                        }
                    }
                    return RuntimeInputDispatchOutcome::default();
                }
                let outcome = item.child.dispatch_nested_key_input_at_focus(
                    owner_identity,
                    focus_data_local_id,
                    key,
                    modifiers,
                    is_pressed,
                    is_repeat,
                );
                if outcome.handled || outcome.terminal_resource_failure {
                    return outcome;
                }
            }
        }
        RuntimeInputDispatchOutcome::default()
    }

    pub(crate) fn dispatch_nested_text_input_at_focus(
        &mut self,
        owner_identity: u64,
        focus_data_local_id: usize,
        text: &str,
    ) -> RuntimeInputDispatchOutcome {
        for nested in self.nested_artboards.values_mut() {
            if nested.child.instance_identity() == owner_identity {
                let RuntimeNestedArtboardInstance {
                    animations, child, ..
                } = nested;
                for animation in animations {
                    let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                        continue;
                    };
                    let Some(machine) = occurrence.state_machine_mut() else {
                        continue;
                    };
                    let outcome =
                        machine.text_input_at_focus_data(child, focus_data_local_id, text);
                    if outcome.handled || outcome.terminal_resource_failure {
                        return outcome;
                    }
                }
                return RuntimeInputDispatchOutcome::default();
            }
            let outcome = nested.child.dispatch_nested_text_input_at_focus(
                owner_identity,
                focus_data_local_id,
                text,
            );
            if outcome.handled || outcome.terminal_resource_failure {
                return outcome;
            }
        }

        let list_locals = self.component_list_locals().collect::<Vec<_>>();
        for list_local_id in list_locals {
            let Some(items) = self.component_list_items_mut(list_local_id) else {
                continue;
            };
            for item in items {
                if item.child.instance_identity() == owner_identity {
                    for machine in &mut item.state_machines {
                        let outcome = machine.text_input_at_focus_data(
                            &mut item.child,
                            focus_data_local_id,
                            text,
                        );
                        if outcome.handled || outcome.terminal_resource_failure {
                            return outcome;
                        }
                    }
                    return RuntimeInputDispatchOutcome::default();
                }
                let outcome = item.child.dispatch_nested_text_input_at_focus(
                    owner_identity,
                    focus_data_local_id,
                    text,
                );
                if outcome.handled || outcome.terminal_resource_failure {
                    return outcome;
                }
            }
        }
        RuntimeInputDispatchOutcome::default()
    }

    pub(crate) fn dispatch_nested_gamepad_at_focus(
        &mut self,
        owner_identity: u64,
        focus_data_local_id: usize,
        invocation: &ScriptListenerInvocation,
    ) -> (RuntimeInputDispatchOutcome, Option<(u64, u32)>) {
        for nested in self.nested_artboards.values_mut() {
            if nested.child.instance_identity() == owner_identity {
                let RuntimeNestedArtboardInstance {
                    animations, child, ..
                } = nested;
                let mut already_dispatched = None;
                for animation in animations {
                    let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                        continue;
                    };
                    let Some(machine) = occurrence.state_machine_mut() else {
                        continue;
                    };
                    let (outcome, dispatched) = machine.gamepad_dispatch_at_focus_data(
                        child,
                        focus_data_local_id,
                        invocation,
                    );
                    if dispatched.is_some() {
                        already_dispatched = dispatched;
                    }
                    if outcome.handled || outcome.terminal_resource_failure {
                        return (outcome, already_dispatched);
                    }
                }
                return (RuntimeInputDispatchOutcome::default(), already_dispatched);
            }
            let (outcome, dispatched) = nested.child.dispatch_nested_gamepad_at_focus(
                owner_identity,
                focus_data_local_id,
                invocation,
            );
            if outcome.handled || outcome.terminal_resource_failure || dispatched.is_some() {
                return (outcome, dispatched);
            }
        }

        let list_locals = self.component_list_locals().collect::<Vec<_>>();
        for list_local_id in list_locals {
            let Some(items) = self.component_list_items_mut(list_local_id) else {
                continue;
            };
            for item in items {
                if item.child.instance_identity() == owner_identity {
                    let mut already_dispatched = None;
                    for machine in &mut item.state_machines {
                        let (outcome, dispatched) = machine.gamepad_dispatch_at_focus_data(
                            &mut item.child,
                            focus_data_local_id,
                            invocation,
                        );
                        if dispatched.is_some() {
                            already_dispatched = dispatched;
                        }
                        if outcome.handled || outcome.terminal_resource_failure {
                            return (outcome, already_dispatched);
                        }
                    }
                    return (RuntimeInputDispatchOutcome::default(), already_dispatched);
                }
                let (outcome, dispatched) = item.child.dispatch_nested_gamepad_at_focus(
                    owner_identity,
                    focus_data_local_id,
                    invocation,
                );
                if outcome.handled || outcome.terminal_resource_failure || dispatched.is_some() {
                    return (outcome, dispatched);
                }
            }
        }
        (RuntimeInputDispatchOutcome::default(), None)
    }

    /// Mirror the nested-artboard portion of
    /// `StateMachineInstance::broadcastGamepadToScriptedDrawables`.
    ///
    /// C++ walks every nested state-machine occurrence and asks that
    /// occurrence to broadcast to its own retained scripted-drawable list
    /// (`state_machine_instance.cpp:942-959`;
    /// `gamepad_batch.cpp:298-362`). The focused drawable is identified by
    /// occurrence pointer in C++; Rust's equivalent key is the artboard
    /// occurrence identity plus the authored global id.
    pub(crate) fn broadcast_nested_gamepad_to_scripted_drawables(
        &mut self,
        invocation: &ScriptListenerInvocation,
        already_dispatched: Option<(u64, u32)>,
    ) -> RuntimeInputDispatchOutcome {
        // `StateMachineInstance::sortHitComponents` first publishes owners
        // found in the retained live drawable order, then leaves the
        // non-drawable remainder in its construction order: nested artboards
        // followed by component lists (`state_machine_instance.cpp:
        // 2017-2054,2255-2301`). Preserve that one interleaved owner walk
        // instead of batching every nested artboard before every list.
        let mut owner_order = self
            .runtime_hit_component_order()
            .into_iter()
            .filter_map(|handle| self.component_local_id(handle))
            .filter(|local_id| {
                self.nested_artboards.get(local_id).is_some()
                    || self.component_list_state(*local_id).is_some()
            })
            .collect::<Vec<_>>();
        for local_id in self.nested_artboards.keys().copied() {
            if !owner_order.contains(&local_id) {
                owner_order.push(local_id);
            }
        }
        for local_id in self.component_list_locals() {
            if !owner_order.contains(&local_id) {
                owner_order.push(local_id);
            }
        }

        let mut handled = false;
        for owner_local_id in owner_order {
            if let Some(nested) = self.nested_artboards.get_mut(&owner_local_id) {
                let RuntimeNestedArtboardInstance {
                    animations, child, ..
                } = nested;
                for animation in animations {
                    let RuntimeNestedAnimationInstance::StateMachine(occurrence) = animation else {
                        continue;
                    };
                    if let Some(machine) = occurrence.state_machine_mut() {
                        let outcome = machine.broadcast_gamepad_to_scripted_drawables(
                            child,
                            invocation,
                            already_dispatched,
                        );
                        if outcome.terminal_resource_failure {
                            return RuntimeInputDispatchOutcome::terminal();
                        }
                    }
                }
                // `HitNestedArtboard::processGamepadInvocation` invokes every
                // child but always returns `HitResult::none`; a nested
                // scripted drawable therefore cannot make its parent batch
                // handled (`state_machine_instance.cpp:942-961`).
                continue;
            }

            if self
                .component(owner_local_id)
                .is_none_or(|component| component.is_collapsed())
            {
                continue;
            }
            let Some(runtime) = self.runtime_file_arc() else {
                continue;
            };
            let Some(list) = self.component_list_state(owner_local_id) else {
                continue;
            };
            let order =
                crate::artboard_component_list_order::runtime_component_list_order(&runtime, list)
                    .indices
                    .clone();
            let Some(items) = self.component_list_items_mut(owner_local_id) else {
                continue;
            };
            // C++ walks `orderedListIndices()` back-to-front
            // (`state_machine_instance.cpp:1227-1268`). Retained Rust list
            // items use the same concrete-owner cache as drawing.
            for item_index in order.into_iter().rev() {
                let Some(item) = items.get_mut(item_index) else {
                    continue;
                };
                for machine in &mut item.state_machines {
                    let outcome = machine.broadcast_gamepad_to_scripted_drawables(
                        &mut item.child,
                        invocation,
                        already_dispatched,
                    );
                    handled |= outcome.handled;
                    if outcome.terminal_resource_failure {
                        return RuntimeInputDispatchOutcome::terminal();
                    }
                }
            }
        }
        RuntimeInputDispatchOutcome::handled(handled)
    }
}
