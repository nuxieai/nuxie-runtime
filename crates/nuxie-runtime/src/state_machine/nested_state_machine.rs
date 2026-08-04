use super::{StateMachineInputKind, StateMachineInstance};
use crate::ArtboardInstance;
use crate::RuntimeOwnedViewModelInstance;
use crate::artboard_data_bind::RuntimeOwnedDataContext;
use crate::focus::RuntimeFocusTree;
use nuxie_binary::RuntimeFile;
use nuxie_graph::ArtboardGraph;

/// One authored nested-input occurrence retained in insertion order.
#[derive(Debug, Clone)]
struct RuntimeNestedInput {
    input_id: usize,
    /// Retain the authored definition value so a public Artboard clone can
    /// rebuild a cold NestedStateMachine occurrence exactly like generated
    /// C++ clone + `initializeAnimation`. `value_applied` belongs only to this
    /// live occurrence.
    authored_value: RuntimeNestedInputValue,
    value_applied: bool,
}

#[derive(Debug, Clone, Copy)]
enum RuntimeNestedInputValue {
    Bool(bool),
    Number(f32),
    Trigger,
}

/// Read-only differential report for one imported nested-state-machine owner.
///
/// This narrow oracle surface exposes the ownership, authored input order,
/// nullable child, and empty forwarding contract without exposing the mutable
/// occurrence itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeNestedStateMachineReport {
    pub local_id: usize,
    pub animation_id: usize,
    pub has_instance: bool,
    pub input_ids: Vec<usize>,
    pub input_names: Vec<String>,
    pub empty_advance: bool,
    pub empty_hit_test: bool,
    pub empty_pointer_down: bool,
    pub empty_pointer_move: bool,
    pub empty_pointer_up: bool,
    pub empty_pointer_exit: bool,
    pub empty_drag_start: bool,
    pub empty_drag_end: bool,
    pub empty_try_change_state: bool,
    pub empty_context_forwarding_completed: bool,
}

/// Mutable occurrence corresponding to pinned C++ `NestedStateMachine`.
///
/// The child state machine is uniquely owned by this occurrence. Authored
/// nested inputs remain in file order and are not rediscovered by scanning the
/// parent artboard when values are applied.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeNestedStateMachineInstance {
    local_id: usize,
    animation_id: usize,
    state_machine: Option<StateMachineInstance>,
    nested_inputs: Vec<RuntimeNestedInput>,
}

impl RuntimeNestedStateMachineInstance {
    pub(crate) fn new(
        local_id: usize,
        state_machine: StateMachineInstance,
        nested_inputs: Vec<(usize, Option<String>, Option<bool>, Option<f32>)>,
    ) -> Self {
        let nested_inputs = nested_inputs
            .into_iter()
            .map(
                |(input_id, _name, bool_value, number_value)| RuntimeNestedInput {
                    input_id,
                    authored_value: bool_value
                        .map(RuntimeNestedInputValue::Bool)
                        .or_else(|| number_value.map(RuntimeNestedInputValue::Number))
                        .unwrap_or(RuntimeNestedInputValue::Trigger),
                    value_applied: false,
                },
            )
            .collect();
        let mut occurrence = Self {
            local_id,
            animation_id: state_machine.state_machine_index(),
            state_machine: Some(state_machine),
            nested_inputs,
        };
        occurrence.apply_authored_values();
        occurrence
    }

    pub(crate) fn from_imported(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        local_id: usize,
        object: &nuxie_binary::RuntimeObject,
        child: &mut ArtboardInstance,
    ) -> Self {
        let animation_id = object
            .uint_property("animationId")
            .and_then(|value| usize::try_from(value).ok())
            .unwrap_or(u32::MAX as usize);
        let mut state_machine = child.state_machine_instance(animation_id);
        if let Some(state_machine) = state_machine.as_mut() {
            state_machine.bind_default_view_model_context();
            state_machine.advance_data_context();
        }

        let nested_inputs = graph
            .local_objects
            .iter()
            .filter_map(|local_object| {
                let object = file.object(local_object.global_id as usize)?;
                if object.uint_property("parentId") != Some(local_id as u64) {
                    return None;
                }
                let input_id = usize::try_from(object.uint_property("inputId")?).ok()?;
                let value = match object.type_name {
                    "NestedBool" => RuntimeNestedInputValue::Bool(
                        object.bool_property("nestedValue").unwrap_or(false),
                    ),
                    "NestedNumber" => RuntimeNestedInputValue::Number(
                        object.double_property("nestedValue").unwrap_or(0.0),
                    ),
                    "NestedTrigger" => RuntimeNestedInputValue::Trigger,
                    _ => return None,
                };
                Some(RuntimeNestedInput {
                    input_id,
                    authored_value: value,
                    value_applied: false,
                })
            })
            .collect::<Vec<_>>();

        let mut occurrence = Self {
            local_id,
            animation_id,
            state_machine,
            nested_inputs,
        };
        occurrence.apply_authored_values();
        occurrence
    }

    pub(crate) fn cold_clone(&self, child: &mut ArtboardInstance) -> Self {
        let mut state_machine = child.state_machine_instance(self.animation_id);
        if let Some(state_machine) = state_machine.as_mut() {
            state_machine.bind_default_view_model_context();
            state_machine.advance_data_context();
        }
        let nested_inputs = self
            .nested_inputs
            .iter()
            .map(|input| RuntimeNestedInput {
                input_id: input.input_id,
                authored_value: input.authored_value,
                value_applied: false,
            })
            .collect();
        let mut occurrence = Self {
            local_id: self.local_id,
            animation_id: self.animation_id,
            state_machine,
            nested_inputs,
        };
        occurrence.apply_authored_values();
        occurrence
    }

    pub(crate) fn local_id(&self) -> usize {
        self.local_id
    }

    pub(crate) fn animation_id(&self) -> usize {
        self.animation_id
    }

    pub(crate) fn has_state_machine(&self) -> bool {
        self.state_machine.is_some()
    }

    pub(crate) fn state_machine(&self) -> Option<&StateMachineInstance> {
        self.state_machine.as_ref()
    }

    pub(crate) fn state_machine_mut(&mut self) -> Option<&mut StateMachineInstance> {
        self.state_machine.as_mut()
    }

    pub(crate) fn take_state_machine(&mut self) -> Option<StateMachineInstance> {
        self.state_machine.take()
    }

    pub(crate) fn restore_state_machine(&mut self, state_machine: StateMachineInstance) {
        debug_assert!(self.state_machine.is_none());
        self.state_machine = Some(state_machine);
    }

    pub(crate) fn install_external_focus(
        &mut self,
        parent_focus: &RuntimeFocusTree,
        child_identity: u64,
    ) {
        if let Some(state_machine) = self.state_machine.as_mut() {
            state_machine.install_external_focus(parent_focus, child_identity);
        }
    }

    pub(crate) fn input_count(&self) -> usize {
        self.nested_inputs.len()
    }

    pub(crate) fn input_id_at(&self, index: usize) -> Option<usize> {
        self.nested_inputs.get(index).map(|input| input.input_id)
    }

    pub(crate) fn input_id_named(&self, name: &str) -> Option<usize> {
        self.nested_inputs
            .iter()
            .find(|input| {
                self.state_machine.as_ref().map_or_else(
                    || name.is_empty(),
                    |state_machine| {
                        // `NestedInput::name()` returns an empty std::string
                        // when the child input id is absent or its authored
                        // name is empty (`nested_input.hpp:41-52`).
                        state_machine
                            .input(input.input_id)
                            .and_then(|input| input.name())
                            .unwrap_or_default()
                            == name
                    },
                )
            })
            .map(|input| input.input_id)
    }

    fn input_name_at(&self, index: usize) -> Option<String> {
        let input = self.nested_inputs.get(index)?;
        Some(
            self.state_machine
                .as_ref()
                .map_or_else(String::new, |state_machine| {
                    state_machine
                        .input(input.input_id)
                        .and_then(|input| input.name())
                        .unwrap_or_default()
                        .to_owned()
                }),
        )
    }

    pub(crate) fn empty_contract_report(
        &self,
        child: &ArtboardInstance,
    ) -> RuntimeNestedStateMachineReport {
        let mut child = child.clone();
        let mut occurrence = self.cold_clone(&mut child);
        let input_ids = (0..self.input_count())
            .filter_map(|index| self.input_id_at(index))
            .collect();
        let input_names = (0..self.input_count())
            .filter_map(|index| self.input_name_at(index))
            .collect();
        let empty_advance = occurrence.advance(&mut child, 0.0);
        let empty_hit_test = occurrence.hit_test(&child, 0.0, 0.0);
        let empty_pointer_down = !occurrence.pointer_down(&mut child, 0.0, 0.0, 1);
        let empty_pointer_move = !occurrence.pointer_move(&mut child, 0.0, 0.0, 0.25, 1);
        let empty_pointer_up = !occurrence.pointer_up(&mut child, 0.0, 0.0, 1);
        let empty_pointer_exit = !occurrence.pointer_exit(&mut child, 0.0, 0.0, 1);
        let empty_drag_start = !occurrence.drag_start(&mut child, 0.0, 0.0, 0.25, 1);
        let empty_drag_end = !occurrence.drag_end(&mut child, 0.0, 0.0, 0.5, 1);
        let empty_try_change_state = occurrence.try_change_state(&mut child);
        let data_context = RuntimeOwnedDataContext::default();
        let empty_context_forwarding_completed =
            !occurrence.bind_owned_data_context(&data_context) && !occurrence.clear_data_context();

        RuntimeNestedStateMachineReport {
            local_id: self.local_id,
            animation_id: self.animation_id,
            has_instance: self.has_state_machine(),
            input_ids,
            input_names,
            empty_advance,
            empty_hit_test,
            empty_pointer_down,
            empty_pointer_move,
            empty_pointer_up,
            empty_pointer_exit,
            empty_drag_start,
            empty_drag_end,
            empty_try_change_state,
            empty_context_forwarding_completed,
        }
    }

    fn apply_authored_values(&mut self) -> bool {
        let mut changed = false;
        for input in &mut self.nested_inputs {
            if input.value_applied {
                continue;
            }
            input.value_applied = true;
            let Some(state_machine) = self.state_machine.as_mut() else {
                continue;
            };
            match input.authored_value {
                RuntimeNestedInputValue::Bool(value)
                    if state_machine
                        .input(input.input_id)
                        .is_some_and(|input| input.kind() == StateMachineInputKind::Bool) =>
                {
                    changed |= state_machine.set_bool(input.input_id, value);
                }
                RuntimeNestedInputValue::Number(value)
                    if state_machine
                        .input(input.input_id)
                        .is_some_and(|input| input.kind() == StateMachineInputKind::Number) =>
                {
                    changed |= state_machine.set_number(input.input_id, value);
                }
                // Pinned C++ deliberately does not apply NestedTrigger during
                // `initializeAnimation`.
                RuntimeNestedInputValue::Bool(_)
                | RuntimeNestedInputValue::Number(_)
                | RuntimeNestedInputValue::Trigger => {}
            }
        }
        changed
    }

    pub(crate) fn advance(&mut self, child: &mut ArtboardInstance, elapsed_seconds: f32) -> bool {
        let Some(state_machine) = self.state_machine.as_mut() else {
            return false;
        };
        child.advance_state_machine_instance(state_machine, elapsed_seconds)
    }

    pub(crate) fn hit_test(&self, child: &ArtboardInstance, x: f32, y: f32) -> bool {
        self.state_machine
            .as_ref()
            .is_some_and(|state_machine| state_machine.hit_test(child, x, y))
    }

    pub(crate) fn pointer_down(
        &mut self,
        child: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        self.state_machine
            .as_mut()
            .is_some_and(|state_machine| state_machine.pointer_down(child, x, y, pointer_id))
    }

    pub(crate) fn pointer_move(
        &mut self,
        child: &mut ArtboardInstance,
        x: f32,
        y: f32,
        timestamp_seconds: f32,
        pointer_id: i32,
    ) -> bool {
        self.state_machine.as_mut().is_some_and(|state_machine| {
            state_machine.pointer_move(child, x, y, timestamp_seconds, pointer_id)
        })
    }

    pub(crate) fn pointer_up(
        &mut self,
        child: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        self.state_machine
            .as_mut()
            .is_some_and(|state_machine| state_machine.pointer_up(child, x, y, pointer_id))
    }

    pub(crate) fn pointer_exit(
        &mut self,
        child: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        self.state_machine
            .as_mut()
            .is_some_and(|state_machine| state_machine.pointer_exit(child, x, y, pointer_id))
    }

    pub(crate) fn drag_start(
        &mut self,
        child: &mut ArtboardInstance,
        x: f32,
        y: f32,
        timestamp_seconds: f32,
        pointer_id: i32,
    ) -> bool {
        self.state_machine.as_mut().is_some_and(|state_machine| {
            state_machine.drag_start(child, x, y, timestamp_seconds, pointer_id)
        })
    }

    pub(crate) fn drag_end(
        &mut self,
        child: &mut ArtboardInstance,
        x: f32,
        y: f32,
        timestamp_seconds: f32,
        pointer_id: i32,
    ) -> bool {
        self.state_machine.as_mut().is_some_and(|state_machine| {
            state_machine.drag_end(child, x, y, timestamp_seconds, pointer_id)
        })
    }

    pub(crate) fn bind_owned_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) -> bool {
        let Some(state_machine) = self.state_machine.as_mut() else {
            return false;
        };
        // C++ `NestedStateMachine::dataContext` forwards to
        // `StateMachineInstance::dataContext`, which clears and rebinds but
        // does not call `DataContext::advanced`. The outer settlement reset
        // owns consumption after every nested transition probe.
        state_machine.bind_owned_view_model_data_context(data_context)
    }

    pub(crate) fn bind_owned_view_model_context_chain(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> bool {
        let Some(state_machine) = self.state_machine.as_mut() else {
            return false;
        };
        if !state_machine.bind_owned_view_model_context_chain(file, context, context_chain) {
            return false;
        }
        let _ = state_machine.advance_data_context();
        true
    }

    pub(crate) fn clear_data_context(&mut self) -> bool {
        let Some(state_machine) = self.state_machine.as_mut() else {
            return false;
        };
        let changed = state_machine.bind_empty_data_context();
        changed | state_machine.advance_data_context()
    }

    pub(crate) fn try_change_state(&mut self, child: &mut ArtboardInstance) -> bool {
        self.state_machine
            .as_mut()
            .is_some_and(|state_machine| child.try_change_state_machine_instance(state_machine))
    }
}
