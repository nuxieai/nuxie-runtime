use super::*;

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "StateMachineLayer" {
        return Some(
            imports_successfully(object, definition, context)
                .expect("StateMachineLayer is owned by StateMachineLayerImporter"),
        );
    }
    None
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition.name == "StateMachineLayer" {
        update_context(definition, context);
    }
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    (definition.name == "StateMachineLayer").then(|| context.latest(ImportStackKey::StateMachine))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "StateMachineLayer" {
        context.make_latest(ImportStackKey::StateMachineLayer);
    }
}

#[derive(Debug)]
struct CppStateMachineLayerResolution {
    object_id: u32,
    owner_artboard_resolve_boundary: usize,
    importer_resolve_boundary: usize,
    state_count: usize,
    has_any_state: bool,
    has_entry_state: bool,
    has_exit_state: bool,
    transitions: Vec<CppStateTransitionResolution>,
}

#[derive(Debug)]
struct CppStateTransitionResolution {
    object_id: u32,
    file_index: usize,
    type_name: &'static str,
    state_to_id: u64,
}

pub(super) fn validate_cpp_state_machine_layers(
    objects: &[Option<RuntimeObject>],
    import_statuses: &[RuntimeImportStatus],
) -> Result<()> {
    let null_object_consumers = runtime_null_object_consumers(objects, import_statuses);
    let artboard_ranges = runtime_artboard_ranges(objects, import_statuses);
    let artboard_indices = runtime_artboard_indices_by_file_index(objects, import_statuses);
    let mut current_artboard_index = None;
    let mut current_state_machine_owner_artboard_boundary = None;
    let mut layers = Vec::<CppStateMachineLayerResolution>::new();
    let mut current_layer: Option<usize> = None;
    // LayerState has its own ImportStack key and therefore survives a later
    // StateMachineLayer, StateMachine, or Artboard until another LayerState
    // replaces it. Transitions attach to that exact retained state owner.
    let mut current_layer_state_owner: Option<usize> = None;

    for file_index in 0..objects.len() {
        let Some(object) = objects[file_index].as_ref() else {
            if null_object_consumers.get(file_index)
                == Some(&Some(NullObjectConsumer::StateMachineLayer))
                && let Some(layer_index) = current_layer
            {
                layers[layer_index].state_count += 1;
            }
            continue;
        };

        if import_statuses.get(file_index) != Some(&RuntimeImportStatus::Imported) {
            continue;
        }

        let Some(definition) = definition_by_type_key(object.type_key) else {
            continue;
        };

        if definition.name == "Artboard" {
            current_artboard_index = artboard_indices[file_index];
            continue;
        }

        if definition.name == "StateMachine" {
            current_state_machine_owner_artboard_boundary = current_artboard_index
                .and_then(|index| artboard_ranges.get(index))
                .map(|(_, end)| *end);
            continue;
        }

        if definition.name == "StateMachineLayer" {
            if let Some(previous_layer_index) = current_layer {
                layers[previous_layer_index].importer_resolve_boundary = file_index;
            }
            layers.push(CppStateMachineLayerResolution {
                object_id: object.id,
                owner_artboard_resolve_boundary: current_state_machine_owner_artboard_boundary
                    .unwrap_or(0),
                importer_resolve_boundary: objects.len(),
                state_count: 0,
                has_any_state: false,
                has_entry_state: false,
                has_exit_state: false,
                transitions: Vec::new(),
            });
            current_layer = Some(layers.len() - 1);
            continue;
        }

        if definition.is_a("LayerState") {
            if let Some(layer_index) = current_layer {
                let layer = &mut layers[layer_index];
                layer.state_count += 1;
                if file_index < layer.owner_artboard_resolve_boundary {
                    match definition.name {
                        "AnyState" => layer.has_any_state = true,
                        "EntryState" => layer.has_entry_state = true,
                        "ExitState" => layer.has_exit_state = true,
                        _ => {}
                    }
                }
                current_layer_state_owner = Some(layer_index);
            }
        }

        if definition.is_a("StateTransition") {
            if let Some(layer_index) = current_layer_state_owner {
                layers[layer_index]
                    .transitions
                    .push(CppStateTransitionResolution {
                        object_id: object.id,
                        file_index,
                        type_name: object.type_name,
                        state_to_id: object.uint_property("stateToId").unwrap_or(u64::MAX),
                    });
            }
        }
    }

    for layer in layers {
        validate_cpp_state_machine_layer_transitions(&layer)?;
    }

    Ok(())
}

fn validate_cpp_state_machine_layer_transitions(
    layer: &CppStateMachineLayerResolution,
) -> Result<()> {
    if (layer.object_id as usize) < layer.owner_artboard_resolve_boundary
        && (!layer.has_any_state || !layer.has_entry_state || !layer.has_exit_state)
    {
        bail!(
            "state machine layer {} is missing required AnyState/EntryState/ExitState objects",
            layer.object_id
        );
    }

    for transition in &layer.transitions {
        if transition.file_index >= layer.importer_resolve_boundary {
            continue;
        }
        let Ok(state_to_id) = usize::try_from(transition.state_to_id) else {
            bail!(
                "state transition object {} ({}) targets state {} outside {} states in state machine layer {}",
                transition.object_id,
                transition.type_name,
                transition.state_to_id,
                layer.state_count,
                layer.object_id
            );
        };

        if state_to_id >= layer.state_count {
            bail!(
                "state transition object {} ({}) targets state {} outside {} states in state machine layer {}",
                transition.object_id,
                transition.type_name,
                transition.state_to_id,
                layer.state_count,
                layer.object_id
            );
        }
    }

    Ok(())
}
impl RuntimeFile {
    pub(crate) fn cpp_artboard_state_machine_graphs(
        &self,
        artboard_index: usize,
    ) -> Vec<RuntimeStateMachine<'_>> {
        if self.artboard(artboard_index).is_none() {
            return Vec::new();
        }
        let data_bind_targets = self.cpp_data_bind_targets();
        let artboard_ranges = runtime_artboard_ranges(&self.objects, &self.import_statuses);
        let artboard_indices = self.cpp_artboard_indices_by_file_index();
        let null_object_consumers =
            runtime_null_object_consumers(&self.objects, &self.import_statuses);
        let artboard_animations_by_index = artboard_ranges
            .iter()
            .enumerate()
            .map(|(index, _)| self.cpp_artboard_objects_named(index, "LinearAnimation"))
            .collect::<Vec<_>>();
        let artboard_local_slots_by_index = artboard_ranges
            .iter()
            .map(|range| {
                let mut slots =
                    runtime_artboard_local_slots(&self.objects, &self.import_statuses, *range);
                validate_cpp_artboard_local_slots(&mut slots, &self.objects);
                slots
            })
            .collect::<Vec<_>>();
        let mut current_artboard_index = None;

        let mut state_machines = Vec::<RuntimeStateMachine<'_>>::new();
        let mut state_machine_artboard_owners = Vec::new();
        let mut layer_importer_resolve_boundaries = Vec::<Vec<usize>>::new();
        let mut current_state_machine: Option<usize> = None;
        // StateMachineLayerImporter captures the latest Artboard at the
        // instant the layer is constructed. That identity survives later
        // Artboard and StateMachine importers (`file.cpp:435-447`;
        // `state_machine_layer_importer.cpp:9-12,18-50`).
        let mut current_layer: Option<(usize, usize, Option<usize>)> = None;
        // These two cursors mirror the distinct ImportStack keys. A transition
        // does not replace the latest LayerState, and a new StateMachine does
        // not replace either key.
        let mut current_layer_state: Option<(usize, usize, usize, Option<usize>)> = None;
        let mut current_transition: Option<(usize, usize, usize, usize)> = None;
        // ImportStack retains the latest importer independently for every
        // type key. Listener and ListenerInputType ownership therefore
        // survives unrelated records, layer/state changes, and even a later
        // StateMachine importer until the same importer key is replaced
        // (`import_stack.hpp:23-68`; `listener_action.cpp:14-45`;
        // `listener_input_type.cpp:10-22`).
        let mut current_listener: Option<RuntimeStateMachineListenerOwner> = None;
        let mut current_keyboard_input_type: Option<RuntimeStateMachineListenerInputTypeOwner> =
            None;
        let mut current_gamepad_input_type: Option<RuntimeStateMachineListenerInputTypeOwner> =
            None;
        let mut current_semantic_input_type: Option<RuntimeStateMachineListenerInputTypeOwner> =
            None;
        let mut current_layer_component: Option<RuntimeStateMachineLayerComponentOwner> = None;
        let mut current_state_machine_scripted_object: Option<
            RuntimeStateMachineScriptedObjectOwner,
        > = None;

        // ImportStack cursor keys are file-global: a new Artboard replaces
        // only the Artboard importer, not StateMachine, layer, listener, or
        // scripted-object importers. Replay the complete file once and select
        // the StateMachines whose own importer attached them to this artboard.
        for (file_index, object) in self.objects.iter().enumerate() {
            let Some(object) = object.as_ref() else {
                if self.import_status(file_index) == Some(RuntimeImportStatus::NullObject) {
                    match null_object_consumers[file_index] {
                        Some(NullObjectConsumer::StateMachineLayer) => {
                            if let Some((owner_state_machine_index, layer_index, _)) = current_layer
                            {
                                state_machines[owner_state_machine_index].layers[layer_index]
                                    .states
                                    .push(RuntimeLayerState {
                                        object: None,
                                        animation: None,
                                        blend_animations: Vec::new(),
                                        fire_actions: Vec::new(),
                                        listener_actions: Vec::new(),
                                        transitions: Vec::new(),
                                    });
                                state_machines[owner_state_machine_index].layers[layer_index]
                                    .state_count += 1;
                            }
                        }
                        Some(NullObjectConsumer::StateMachine) => {
                            if let Some(state_machine_index) = current_state_machine {
                                // `StateMachineImporter::readNullObject` appends
                                // a null input occurrence. Keep the slot so
                                // every later `inputId` retains its C++ index.
                                state_machines[state_machine_index].inputs.push(None);
                            }
                        }
                        _ => {}
                    }
                }
                continue;
            };
            if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if definition.name == "Artboard" {
                current_artboard_index = artboard_indices[file_index];
                continue;
            }

            if definition.name == "StateMachine" {
                state_machines.push(RuntimeStateMachine {
                    object,
                    layers: Vec::new(),
                    inputs: Vec::new(),
                    listeners: Vec::new(),
                    data_binds: Vec::new(),
                    scripted_objects: Vec::new(),
                });
                state_machine_artboard_owners.push(current_artboard_index);
                layer_importer_resolve_boundaries.push(Vec::new());
                current_state_machine = Some(state_machines.len() - 1);
                continue;
            }

            let Some(state_machine_index) = current_state_machine else {
                continue;
            };

            if matches!(
                definition.name,
                "GamepadInput" | "KeyboardInput" | "SemanticInput"
            ) {
                let owner = match definition.name {
                    "KeyboardInput" => current_keyboard_input_type,
                    "GamepadInput" => current_gamepad_input_type,
                    "SemanticInput" => current_semantic_input_type,
                    _ => unreachable!("the enclosing match filters concrete listener inputs"),
                };
                if let Some(owner) = owner
                    && let Some(listener_input_type) = state_machines[owner.state_machine_index]
                        .listeners
                        .get_mut(owner.listener_index)
                    && let Some(inputs) = listener_input_type
                        .listener_input_type_inputs
                        .get_mut(owner.input_type_index)
                {
                    inputs.push(object);
                }
                continue;
            }

            if definition_adds_cpp_state_machine_scripted_object(definition) {
                state_machines[state_machine_index]
                    .scripted_objects
                    .push(RuntimeScriptedObject {
                        object,
                        inputs: Vec::new(),
                    });
                current_state_machine_scripted_object =
                    Some(RuntimeStateMachineScriptedObjectOwner {
                        state_machine_index,
                        scripted_object_index: state_machines[state_machine_index]
                            .scripted_objects
                            .len()
                            - 1,
                    });
            } else if definition_is_cpp_scripted_object(definition) {
                current_state_machine_scripted_object = None;
            }

            if definition.name.starts_with("ScriptInput") {
                if let Some(owner) = current_state_machine_scripted_object {
                    state_machines[owner.state_machine_index].scripted_objects
                        [owner.scripted_object_index]
                        .inputs
                        .push(object);
                }
                continue;
            }

            if definition.name == "StateMachineLayer" {
                if let Some((previous_state_machine_index, previous_layer_index, _)) = current_layer
                {
                    layer_importer_resolve_boundaries[previous_state_machine_index]
                        [previous_layer_index] = file_index;
                }
                state_machines[state_machine_index]
                    .layers
                    .push(RuntimeStateMachineLayer {
                        object,
                        state_count: 0,
                        states: Vec::new(),
                    });
                layer_importer_resolve_boundaries[state_machine_index].push(self.objects.len());
                current_layer = Some((
                    state_machine_index,
                    state_machines[state_machine_index].layers.len() - 1,
                    current_artboard_index,
                ));
                continue;
            }

            if definition.is_a("LayerState") {
                if let Some((owner_state_machine_index, layer_index, layer_artboard_index)) =
                    current_layer
                {
                    let layer_artboard_animations = layer_artboard_index
                        .and_then(|index| artboard_animations_by_index.get(index))
                        .map(Vec::as_slice)
                        .unwrap_or_default();
                    state_machines[owner_state_machine_index].layers[layer_index]
                        .states
                        .push(RuntimeLayerState {
                            object: Some(object),
                            animation: cpp_resolved_animation_state_animation(
                                object,
                                layer_artboard_animations,
                            ),
                            blend_animations: Vec::new(),
                            fire_actions: Vec::new(),
                            listener_actions: Vec::new(),
                            transitions: Vec::new(),
                        });
                    let state_index = state_machines[owner_state_machine_index].layers[layer_index]
                        .states
                        .len()
                        - 1;
                    current_layer_state = Some((
                        owner_state_machine_index,
                        layer_index,
                        state_index,
                        layer_artboard_index,
                    ));
                    current_layer_component = Some(RuntimeStateMachineLayerComponentOwner::State {
                        state_machine_index: owner_state_machine_index,
                        layer_index,
                        state_index,
                    });
                    state_machines[owner_state_machine_index].layers[layer_index].state_count += 1;
                }
                continue;
            }

            if definition.is_a("BlendAnimation") {
                if let Some((owner_state_machine_index, layer_index, state_index, _)) =
                    current_layer_state
                {
                    // BlendAnimation::import resolves against the Artboard
                    // importer that is latest for this record, unlike the
                    // enclosing layer's deferred AnimationState resolution
                    // (`blend_animation.cpp:11-38`).
                    let current_artboard_animations = current_artboard_index
                        .and_then(|index| artboard_animations_by_index.get(index))
                        .map(Vec::as_slice)
                        .unwrap_or_default();
                    let animation_index =
                        usize::try_from(object.uint_property("animationId").unwrap_or(u64::MAX))
                            .ok()
                            .filter(|index| *index < current_artboard_animations.len());
                    let animation = animation_index
                        .and_then(|index| current_artboard_animations.get(index))
                        .copied();
                    state_machines[owner_state_machine_index].layers[layer_index].states
                        [state_index]
                        .blend_animations
                        .push(RuntimeBlendAnimation {
                            object,
                            animation_index,
                            animation,
                        });
                }
                continue;
            }

            if definition.is_a("StateTransition") {
                if let Some((owner_state_machine_index, layer_index, state_index, _)) =
                    current_layer_state
                {
                    let owner_artboard_range = state_machine_artboard_owners
                        .get(owner_state_machine_index)
                        .copied()
                        .flatten()
                        .and_then(|index| artboard_ranges.get(index).copied());
                    // StateTransition resolves its interpolator during the
                    // owning Artboard's initialize/onAddedDirty pass. Replacing
                    // that Artboard importer resolves it before a transition
                    // later attached through a stale LayerState importer can
                    // exist, so that late transition keeps a null interpolator
                    // (`import_stack.hpp:33-68`; `artboard.cpp:264-312`;
                    // `state_transition.cpp:28-40`).
                    let interpolator = owner_artboard_range
                        .filter(|(_, end)| file_index < *end)
                        .and_then(|range| {
                            cpp_resolved_state_transition_interpolator(
                                object,
                                range,
                                &self.objects,
                                &self.import_statuses,
                            )
                        });

                    state_machines[owner_state_machine_index].layers[layer_index].states
                        [state_index]
                        .transitions
                        .push(RuntimeStateTransition {
                            object,
                            state_to_index: None,
                            state_to: None,
                            interpolator,
                            exit_blend_animation_index: None,
                            exit_blend_animation: None,
                            exit_animation_index: None,
                            exit_animation: None,
                            fire_actions: Vec::new(),
                            listener_actions: Vec::new(),
                            conditions: Vec::new(),
                        });
                    let transition_index = state_machines[owner_state_machine_index].layers
                        [layer_index]
                        .states[state_index]
                        .transitions
                        .len()
                        - 1;
                    current_transition = Some((
                        owner_state_machine_index,
                        layer_index,
                        state_index,
                        transition_index,
                    ));
                    current_layer_component =
                        Some(RuntimeStateMachineLayerComponentOwner::Transition {
                            state_machine_index: owner_state_machine_index,
                            layer_index,
                            state_index,
                            transition_index,
                        });
                }
                continue;
            }

            if definition.is_a("StateMachineFireAction") {
                if let Some(owner) = current_layer_component {
                    match owner {
                        RuntimeStateMachineLayerComponentOwner::State {
                            state_machine_index,
                            layer_index,
                            state_index,
                        } => {
                            let owner_artboard_slots = state_machine_artboard_owners
                                .get(state_machine_index)
                                .copied()
                                .flatten()
                                .and_then(|index| artboard_local_slots_by_index.get(index))
                                .map(Vec::as_slice)
                                .unwrap_or_default();
                            state_machines[state_machine_index].layers[layer_index].states
                                [state_index]
                                .fire_actions
                                .push(cpp_runtime_state_machine_fire_action(
                                    object,
                                    owner_artboard_slots,
                                    &self.objects,
                                ));
                        }
                        RuntimeStateMachineLayerComponentOwner::Transition {
                            state_machine_index,
                            layer_index,
                            state_index,
                            transition_index,
                        } => {
                            let owner_artboard_slots = state_machine_artboard_owners
                                .get(state_machine_index)
                                .copied()
                                .flatten()
                                .and_then(|index| artboard_local_slots_by_index.get(index))
                                .map(Vec::as_slice)
                                .unwrap_or_default();
                            state_machines[state_machine_index].layers[layer_index].states
                                [state_index]
                                .transitions[transition_index]
                                .fire_actions
                                .push(cpp_runtime_state_machine_fire_action(
                                    object,
                                    owner_artboard_slots,
                                    &self.objects,
                                ));
                        }
                    }
                }
                continue;
            }

            if definition.is_a("TransitionCondition") {
                if let Some((
                    owner_state_machine_index,
                    layer_index,
                    state_index,
                    transition_index,
                )) = current_transition
                {
                    state_machines[owner_state_machine_index].layers[layer_index].states
                        [state_index]
                        .transitions[transition_index]
                        .conditions
                        .push(object);
                }
                continue;
            }

            if definition.is_a("StateMachineInput") {
                state_machines[state_machine_index]
                    .inputs
                    .push(Some(object));
                continue;
            }

            if definition.is_a("StateMachineListener") {
                state_machines[state_machine_index]
                    .listeners
                    .push(RuntimeStateMachineListener {
                        object,
                        actions: Vec::new(),
                        listener_input_types: Vec::new(),
                        listener_input_type_inputs: Vec::new(),
                    });
                current_listener = Some(RuntimeStateMachineListenerOwner {
                    state_machine_index,
                    listener_index: state_machines[state_machine_index].listeners.len() - 1,
                });
                continue;
            }

            if definition.is_a("ListenerAction") {
                if listener_action_parent_kind_is_listener(object) {
                    if let Some(owner) = current_listener {
                        let owner_artboard_slots = state_machine_artboard_owners
                            .get(owner.state_machine_index)
                            .copied()
                            .flatten()
                            .and_then(|index| artboard_local_slots_by_index.get(index))
                            .map(Vec::as_slice)
                            .unwrap_or_default();
                        state_machines[owner.state_machine_index].listeners[owner.listener_index]
                            .actions
                            .push(cpp_runtime_listener_action(
                                object,
                                owner_artboard_slots,
                                &self.objects,
                                (object.type_name == "ListenerViewModelChange")
                                    .then(|| self.latest_bindable_property_for_object(object))
                                    .flatten(),
                            ));
                    }
                } else if let Some(owner) = current_layer_component {
                    match owner {
                        RuntimeStateMachineLayerComponentOwner::State {
                            state_machine_index,
                            layer_index,
                            state_index,
                        } => {
                            let owner_artboard_slots = state_machine_artboard_owners
                                .get(state_machine_index)
                                .copied()
                                .flatten()
                                .and_then(|index| artboard_local_slots_by_index.get(index))
                                .map(Vec::as_slice)
                                .unwrap_or_default();
                            state_machines[state_machine_index].layers[layer_index].states
                                [state_index]
                                .listener_actions
                                .push(cpp_runtime_listener_action(
                                    object,
                                    owner_artboard_slots,
                                    &self.objects,
                                    (object.type_name == "ListenerViewModelChange")
                                        .then(|| self.latest_bindable_property_for_object(object))
                                        .flatten(),
                                ));
                        }
                        RuntimeStateMachineLayerComponentOwner::Transition {
                            state_machine_index,
                            layer_index,
                            state_index,
                            transition_index,
                        } => {
                            let owner_artboard_slots = state_machine_artboard_owners
                                .get(state_machine_index)
                                .copied()
                                .flatten()
                                .and_then(|index| artboard_local_slots_by_index.get(index))
                                .map(Vec::as_slice)
                                .unwrap_or_default();
                            state_machines[state_machine_index].layers[layer_index].states
                                [state_index]
                                .transitions[transition_index]
                                .listener_actions
                                .push(cpp_runtime_listener_action(
                                    object,
                                    owner_artboard_slots,
                                    &self.objects,
                                    (object.type_name == "ListenerViewModelChange")
                                        .then(|| self.latest_bindable_property_for_object(object))
                                        .flatten(),
                                ));
                        }
                    }
                }
                continue;
            }

            if definition.is_a("ListenerInputType") {
                if let Some(owner) = current_listener {
                    let listener = &mut state_machines[owner.state_machine_index].listeners
                        [owner.listener_index];
                    listener.listener_input_types.push(object);
                    listener.listener_input_type_inputs.push(Vec::new());
                    let input_owner = Some(RuntimeStateMachineListenerInputTypeOwner {
                        state_machine_index: owner.state_machine_index,
                        listener_index: owner.listener_index,
                        input_type_index: listener.listener_input_types.len() - 1,
                    });
                    match definition.name {
                        "ListenerInputTypeKeyboard" => {
                            current_keyboard_input_type = input_owner;
                        }
                        "ListenerInputTypeGamepad" => {
                            current_gamepad_input_type = input_owner;
                        }
                        "ListenerInputTypeSemantic" => {
                            current_semantic_input_type = input_owner;
                        }
                        _ => {}
                    }
                }
                continue;
            }

            if definition.is_a("DataBind")
                && data_bind_target_is_cpp_state_machine_owned(
                    data_bind_targets[file_index].map(|target| target.object),
                )
            {
                state_machines[state_machine_index].data_binds.push(object);
            }
        }

        resolve_runtime_state_machine_transition_targets(
            &mut state_machines,
            &layer_importer_resolve_boundaries,
        );
        state_machines
            .into_iter()
            .zip(state_machine_artboard_owners)
            .filter_map(|(state_machine, owner)| {
                (owner == Some(artboard_index)).then_some(state_machine)
            })
            .collect()
    }
}
