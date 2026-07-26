use super::{StateMachineInputKind, StateMachineInstance};
use crate::RuntimeOwnedViewModelInstance;
use crate::artboard_data_bind::RuntimeOwnedDataContext;
use crate::focus::RuntimeFocusTree;
use crate::{ArtboardInstance, StateMachineReportedEvent};
use nuxie_binary::RuntimeFile;
use nuxie_graph::ArtboardGraph;

/// One authored nested-input occurrence retained in insertion order.
#[derive(Debug, Clone)]
struct RuntimeNestedInput {
    input_id: usize,
    name: Option<String>,
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

/// Mutable occurrence corresponding to pinned C++ `NestedStateMachine`.
///
/// The child state machine is uniquely owned by this occurrence. Authored
/// nested inputs remain in file order and are not rediscovered by scanning the
/// parent artboard when values are applied.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeNestedStateMachineInstance {
    local_id: usize,
    state_machine: StateMachineInstance,
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
                |(input_id, name, bool_value, number_value)| RuntimeNestedInput {
                    input_id,
                    name,
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
            state_machine,
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
    ) -> Option<Self> {
        let state_machine_index = usize::try_from(object.uint_property("animationId")?).ok()?;
        let mut state_machine = child.state_machine_instance(state_machine_index)?;
        state_machine.schedule_post_update_probe();
        state_machine.bind_default_view_model_context();
        state_machine.advance_data_context();

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
                    name: object.string_property("name").map(ToOwned::to_owned),
                    authored_value: value,
                    value_applied: false,
                })
            })
            .collect::<Vec<_>>();

        let mut occurrence = Self {
            local_id,
            state_machine,
            nested_inputs,
        };
        occurrence.apply_authored_values();
        Some(occurrence)
    }

    pub(crate) fn cold_clone(&self, child: &mut ArtboardInstance) -> Option<Self> {
        let state_machine_index = self.state_machine.state_machine_index();
        let mut state_machine = child.state_machine_instance(state_machine_index)?;
        state_machine.schedule_post_update_probe();
        state_machine.bind_default_view_model_context();
        state_machine.advance_data_context();
        let nested_inputs = self
            .nested_inputs
            .iter()
            .map(|input| RuntimeNestedInput {
                input_id: input.input_id,
                name: input.name.clone(),
                authored_value: input.authored_value,
                value_applied: false,
            })
            .collect();
        let mut occurrence = Self {
            local_id: self.local_id,
            state_machine,
            nested_inputs,
        };
        occurrence.apply_authored_values();
        Some(occurrence)
    }

    pub(crate) fn local_id(&self) -> usize {
        self.local_id
    }

    pub(crate) fn state_machine(&self) -> &StateMachineInstance {
        &self.state_machine
    }

    pub(crate) fn state_machine_mut(&mut self) -> &mut StateMachineInstance {
        &mut self.state_machine
    }

    pub(crate) fn install_external_focus(
        &mut self,
        parent_focus: &RuntimeFocusTree,
        child_identity: u64,
    ) {
        self.state_machine
            .install_external_focus(parent_focus, child_identity);
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
            .find(|input| input.name.as_deref() == Some(name))
            .map(|input| input.input_id)
    }

    fn apply_authored_values(&mut self) -> bool {
        let mut changed = false;
        for input in &mut self.nested_inputs {
            if input.value_applied {
                continue;
            }
            input.value_applied = true;
            match input.authored_value {
                RuntimeNestedInputValue::Bool(value)
                    if self
                        .state_machine
                        .input(input.input_id)
                        .is_some_and(|input| input.kind() == StateMachineInputKind::Bool) =>
                {
                    changed |= self.state_machine.set_bool(input.input_id, value);
                }
                RuntimeNestedInputValue::Number(value)
                    if self
                        .state_machine
                        .input(input.input_id)
                        .is_some_and(|input| input.kind() == StateMachineInputKind::Number) =>
                {
                    changed |= self.state_machine.set_number(input.input_id, value);
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

    pub(crate) fn advance(
        &mut self,
        child: &mut ArtboardInstance,
        elapsed_seconds: f32,
        mut reported_events: Option<&mut Vec<StateMachineReportedEvent>>,
    ) -> bool {
        let changed =
            child.advance_state_machine_instance(&mut self.state_machine, elapsed_seconds);
        if let Some(reported_events) = reported_events.as_mut() {
            for index in 0..self.state_machine.reported_event_count() {
                if let Some(event) = self.state_machine.reported_event(index) {
                    (**reported_events).push(event.clone());
                }
            }
        }
        changed
    }

    pub(crate) fn hit_test(&self, child: &ArtboardInstance, x: f32, y: f32) -> bool {
        self.state_machine.hit_test(child, x, y)
    }

    pub(crate) fn pointer_down(
        &mut self,
        child: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        self.state_machine.pointer_down(child, x, y, pointer_id)
    }

    pub(crate) fn pointer_move(
        &mut self,
        child: &mut ArtboardInstance,
        x: f32,
        y: f32,
        timestamp_seconds: f32,
        pointer_id: i32,
    ) -> bool {
        self.state_machine
            .pointer_move(child, x, y, timestamp_seconds, pointer_id)
    }

    pub(crate) fn pointer_up(
        &mut self,
        child: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        self.state_machine.pointer_up(child, x, y, pointer_id)
    }

    pub(crate) fn pointer_exit(
        &mut self,
        child: &mut ArtboardInstance,
        x: f32,
        y: f32,
        pointer_id: i32,
    ) -> bool {
        self.state_machine.pointer_exit(child, x, y, pointer_id)
    }

    pub(crate) fn drag_start(
        &mut self,
        child: &mut ArtboardInstance,
        x: f32,
        y: f32,
        timestamp_seconds: f32,
        pointer_id: i32,
    ) -> bool {
        self.state_machine
            .drag_start(child, x, y, timestamp_seconds, pointer_id)
    }

    pub(crate) fn drag_end(
        &mut self,
        child: &mut ArtboardInstance,
        x: f32,
        y: f32,
        timestamp_seconds: f32,
        pointer_id: i32,
    ) -> bool {
        self.state_machine
            .drag_end(child, x, y, timestamp_seconds, pointer_id)
    }

    pub(crate) fn bind_owned_data_context(
        &mut self,
        data_context: &RuntimeOwnedDataContext,
    ) -> bool {
        let changed = self
            .state_machine
            .bind_owned_view_model_data_context(data_context);
        changed | self.state_machine.advance_data_context()
    }

    pub(crate) fn bind_owned_view_model_context_chain(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> bool {
        if !self
            .state_machine
            .bind_owned_view_model_context_chain(file, context, context_chain)
        {
            return false;
        }
        let _ = self.state_machine.advance_data_context();
        true
    }

    pub(crate) fn clear_data_context(&mut self) -> bool {
        let changed = self.state_machine.bind_empty_data_context();
        changed | self.state_machine.advance_data_context()
    }

    pub(crate) fn try_change_state(&mut self, child: &mut ArtboardInstance) -> bool {
        child.try_change_state_machine_instance(&mut self.state_machine)
    }
}
