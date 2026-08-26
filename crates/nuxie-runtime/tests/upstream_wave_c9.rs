//! Distinct strict ports for the Wave C9 state-machine rows with retained
//! Rust owner authority. Unsupported rows remain ledger-only pending.

use std::path::PathBuf;

use nuxie_binary::{read_runtime_file, RuntimeFile};
use nuxie_graph::GraphFile;
use nuxie_runtime::{ArtboardInstance, StateMachineInstance};
use nuxie_schema::definition_by_name;

fn pinned_fixture(name: &str) -> PathBuf {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name)
}

fn import_fixture(name: &str) -> (RuntimeFile, GraphFile) {
    let path = pinned_fixture(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let file = read_runtime_file(&bytes)
        .unwrap_or_else(|error| panic!("import pinned fixture {}: {error}", path.display()));
    let graphs = GraphFile::from_runtime_file(&file)
        .unwrap_or_else(|error| panic!("graph pinned fixture {}: {error}", path.display()));
    (file, graphs)
}

fn artboard_index(graphs: &GraphFile, name: Option<&str>) -> usize {
    name.map_or(0, |name| {
        graphs
            .artboards
            .iter()
            .position(|artboard| artboard.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("missing artboard {name}"))
    })
}

fn live_machine(
    file: &RuntimeFile,
    graphs: &GraphFile,
    artboard_index: usize,
    machine_index: usize,
) -> (ArtboardInstance, StateMachineInstance) {
    let graph = &graphs.artboards[artboard_index];
    let mut artboard = ArtboardInstance::from_graph_with_artboards(file, graph, &graphs.artboards)
        .expect("pinned artboard instantiates");
    let machine = artboard
        .state_machine_instance(machine_index)
        .expect("pinned state machine instantiates");
    (artboard, machine)
}

#[test]
fn wave_c9_state_machine_001_file_with_state_machine_can_be_read() {
    let (file, graphs) = import_fixture("rocket.riv");
    let artboard = &graphs.artboards[0];
    assert_eq!(artboard.animations.len(), 3);
    assert_eq!(artboard.state_machines.len(), 1);

    let machine_index = artboard
        .state_machines
        .iter()
        .position(|machine| machine.name.as_deref() == Some("Button"))
        .expect("Button state machine");
    let machine = &file.artboard_state_machine_graphs(0)[machine_index];
    assert_eq!(machine.layers.len(), 1);
    assert_eq!(machine.inputs.len(), 2);

    let hover = machine
        .inputs
        .iter()
        .flatten()
        .find(|input| input.string_property("name") == Some("Hover"))
        .expect("Hover input");
    assert!(definition_by_name(hover.type_name).is_some_and(|kind| kind.is_a("StateMachineBool")));
    let press = machine
        .inputs
        .iter()
        .flatten()
        .find(|input| input.string_property("name") == Some("Press"))
        .expect("Press input");
    assert!(definition_by_name(press.type_name).is_some_and(|kind| kind.is_a("StateMachineBool")));

    let layer = &machine.layers[0];
    assert_eq!(layer.state_count, 6);
    assert!(layer.states.iter().any(|state| {
        state
            .object
            .is_some_and(|state| state.type_name == "AnyState")
    }));
    let entry = layer
        .states
        .iter()
        .find(|state| {
            state
                .object
                .is_some_and(|state| state.type_name == "EntryState")
        })
        .expect("entry state");
    assert!(layer.states.iter().any(|state| {
        state
            .object
            .is_some_and(|state| state.type_name == "ExitState")
    }));

    let mut found_animation_states = 0;
    for state in &layer.states {
        if state
            .object
            .is_some_and(|state| state.type_name == "AnimationState")
        {
            found_animation_states += 1;
            assert!(state.animation.is_some());
        }
    }
    assert_eq!(found_animation_states, 3);

    assert_eq!(entry.transitions.len(), 1);
    let state_to = entry.transitions[0]
        .state_to
        .expect("entry transition target");
    assert!(definition_by_name(state_to.type_name).is_some_and(|kind| kind.is_a("AnimationState")));
    let idle_state = layer
        .states
        .iter()
        .find(|state| state.object.is_some_and(|state| state.id == state_to.id))
        .expect("idle animation state");
    let idle_animation = idle_state.animation.expect("idle animation");
    assert_eq!(idle_animation.string_property("name"), Some("idle"));
    assert_eq!(idle_state.transitions.len(), 2);
    for transition in &idle_state.transitions {
        let target = transition.state_to.expect("idle transition target");
        let target_animation = layer
            .states
            .iter()
            .find(|state| state.object.is_some_and(|state| state.id == target.id))
            .and_then(|state| state.animation)
            .expect("idle target animation");
        if target_animation.string_property("name") == Some("Roll_over") {
            assert_eq!(transition.conditions.len(), 1);
        }
    }

    let (mut instance, state_machine) = live_machine(&file, &graphs, 0, machine_index);
    assert_eq!(
        state_machine
            .get_bool("Hover")
            .and_then(|input| input.name()),
        Some("Hover")
    );
    assert_eq!(
        state_machine
            .get_bool("Press")
            .and_then(|input| input.name()),
        Some("Press")
    );
    assert!(state_machine.get_bool("Hover").is_some());
    assert!(state_machine.get_bool("Press").is_some());
    assert_eq!(state_machine.changed_state_count(), 0);
    assert_eq!(state_machine.current_animation_count(), 0);
    let _ = &mut instance;
}

#[test]
fn wave_c9_state_machine_002_file_with_blend_states_loads_correctly() {
    let (file, graphs) = import_fixture("blend_test.riv");
    let artboard = &graphs.artboards[0];
    assert_eq!(artboard.animations.len(), 4);
    assert_eq!(artboard.state_machines.len(), 2);
    let machine_index = artboard
        .state_machines
        .iter()
        .position(|machine| machine.name.as_deref() == Some("blend"))
        .expect("blend state machine");
    let machine = &file.artboard_state_machine_graphs(0)[machine_index];
    assert_eq!(machine.layers.len(), 1);
    let layer = &machine.layers[0];
    assert_eq!(layer.state_count, 5);
    assert!(layer.states.iter().any(|state| {
        state
            .object
            .is_some_and(|state| state.type_name == "AnyState")
    }));
    assert!(layer.states.iter().any(|state| {
        state
            .object
            .is_some_and(|state| state.type_name == "EntryState")
    }));
    assert!(layer.states.iter().any(|state| {
        state
            .object
            .is_some_and(|state| state.type_name == "ExitState")
    }));
    assert!(layer.states[1].object.is_some_and(|state| {
        definition_by_name(state.type_name).is_some_and(|kind| kind.is_a("BlendState1D"))
    }));
    assert!(layer.states[2].object.is_some_and(|state| {
        definition_by_name(state.type_name).is_some_and(|kind| kind.is_a("BlendState1D"))
    }));
    let blend_a = &layer.states[1];
    let blend_b = &layer.states[2];
    assert_eq!(blend_a.blend_animations.len(), 3);
    assert_eq!(blend_b.blend_animations.len(), 3);

    let animation = &blend_a.blend_animations[0];
    assert!(definition_by_name(animation.object.type_name)
        .is_some_and(|kind| kind.is_a("BlendAnimation1D")));
    let animation_target = animation.animation.expect("horizontal animation");
    assert_eq!(animation_target.string_property("name"), Some("horizontal"));
    assert_eq!(animation.object.double_property("value"), Some(0.0));

    let animation = &blend_a.blend_animations[1];
    assert!(definition_by_name(animation.object.type_name)
        .is_some_and(|kind| kind.is_a("BlendAnimation1D")));
    let animation_target = animation.animation.expect("vertical animation");
    assert_eq!(animation_target.string_property("name"), Some("vertical"));
    assert_eq!(animation.object.double_property("value"), Some(100.0));

    let animation = &blend_a.blend_animations[2];
    assert!(definition_by_name(animation.object.type_name)
        .is_some_and(|kind| kind.is_a("BlendAnimation1D")));
    let animation_target = animation.animation.expect("rotate animation");
    assert_eq!(animation_target.string_property("name"), Some("rotate"));
    assert_eq!(animation.object.double_property("value"), Some(0.0));

    assert_eq!(blend_a.transitions.len(), 1);
    assert!(definition_by_name(blend_a.transitions[0].object.type_name)
        .is_some_and(|kind| kind.is_a("BlendStateTransition")));
    assert!(blend_a.transitions[0].exit_blend_animation.is_some());
}

#[test]
fn wave_c9_state_machine_003_animation_state_without_animation_does_not_crash() {
    let (file, graphs) = import_fixture("multiple_state_machines.riv");
    let artboard = &graphs.artboards[0];
    assert_eq!(artboard.animations.len(), 1);
    assert_eq!(artboard.state_machines.len(), 4);
    let machine_index = artboard
        .state_machines
        .iter()
        .position(|machine| machine.name.as_deref() == Some("two"))
        .expect("two state machine");
    let machine = &file.artboard_state_machine_graphs(0)[machine_index];
    assert_eq!(machine.layers.len(), 1);
    let layer = &machine.layers[0];
    assert_eq!(layer.state_count, 4);
    assert!(layer.states.iter().any(|state| {
        state
            .object
            .is_some_and(|state| state.type_name == "AnyState")
    }));
    assert!(layer.states.iter().any(|state| {
        state
            .object
            .is_some_and(|state| state.type_name == "EntryState")
    }));
    assert!(layer.states.iter().any(|state| {
        state
            .object
            .is_some_and(|state| state.type_name == "ExitState")
    }));
    assert!(layer.states[3].object.is_some_and(|state| {
        definition_by_name(state.type_name).is_some_and(|kind| kind.is_a("AnimationState"))
    }));
    assert!(layer.states[3].animation.is_none());
    let (mut artboard, mut state_machine) = live_machine(&file, &graphs, 0, machine_index);
    artboard.advance_state_machine_instance(&mut state_machine, 0.0);
}

#[test]
fn wave_c9_state_machine_004_oneshot_blend_keeps_going_after_animations_stop() {
    let (file, graphs) = import_fixture("oneshotblend.riv");
    let machine_index = graphs.artboards[0]
        .state_machines
        .iter()
        .position(|machine| machine.name.as_deref() == Some("State Machine 1"))
        .expect("State Machine 1");
    let (mut artboard, mut state_machine) = live_machine(&file, &graphs, 0, machine_index);
    artboard.advance_state_machine_instance(&mut state_machine, 0.0);
    assert!(state_machine.needs_advance());
    artboard.advance_state_machine_instance(&mut state_machine, 0.5);
    assert!(state_machine.needs_advance());
    artboard.advance_state_machine_instance(&mut state_machine, 1.0);
    assert!(state_machine.needs_advance());
}

fn event_fixture(
    name: &str,
    artboard_name: Option<&str>,
) -> (
    RuntimeFile,
    GraphFile,
    usize,
    ArtboardInstance,
    StateMachineInstance,
) {
    let (file, graphs) = import_fixture(name);
    let index = artboard_index(&graphs, artboard_name);
    let (artboard, machine) = live_machine(&file, &graphs, index, 0);
    (file, graphs, index, artboard, machine)
}

#[test]
fn wave_c9_event_001_file_with_state_machine_listeners_can_be_read() {
    let (file, graphs, index, _, _) = event_fixture("bullet_man.riv", Some("Bullet Man"));
    let artboard = &graphs.artboards[index];
    assert_eq!(artboard.state_machines.len(), 1);
    let machine = &file.artboard_state_machine_graphs(index)[0];
    assert_eq!(machine.listeners.len(), 3);
    assert_eq!(machine.inputs.len(), 4);

    let listener = &machine.listeners[0];
    let target = &artboard.components[listener.object.uint_property("targetId").unwrap() as usize];
    assert!(definition_by_name(target.type_name).is_some_and(|kind| kind.is_a("Node")));
    assert_eq!(target.name.as_deref(), Some("HandWickHit"));
    assert_eq!(listener.actions.len(), 1);
    let action = listener.actions[0].object;
    assert!(
        definition_by_name(action.type_name).is_some_and(|kind| kind.is_a("ListenerInputChange"))
    );
    assert_eq!(action.uint_property("inputId"), Some(0));

    let listener = &machine.listeners[1];
    let target = &artboard.components[listener.object.uint_property("targetId").unwrap() as usize];
    assert!(definition_by_name(target.type_name).is_some_and(|kind| kind.is_a("Node")));
    assert_eq!(target.name.as_deref(), Some("HandCannonHit"));
    assert_eq!(listener.actions.len(), 1);
    let action = listener.actions[0].object;
    assert!(
        definition_by_name(action.type_name).is_some_and(|kind| kind.is_a("ListenerInputChange"))
    );
    assert_eq!(action.uint_property("inputId"), Some(1));

    let listener = &machine.listeners[2];
    let target = &artboard.components[listener.object.uint_property("targetId").unwrap() as usize];
    assert!(definition_by_name(target.type_name).is_some_and(|kind| kind.is_a("Node")));
    assert_eq!(target.name.as_deref(), Some("HandHelmetHit"));
    assert_eq!(listener.actions.len(), 1);
    let action = listener.actions[0].object;
    assert!(
        definition_by_name(action.type_name).is_some_and(|kind| kind.is_a("ListenerInputChange"))
    );
    assert_eq!(action.uint_property("inputId"), Some(2));
}

#[test]
fn wave_c9_event_002_hit_testing_via_state_machine_works() {
    let (_, graphs, index, mut artboard, mut machine) =
        event_fixture("bullet_man.riv", Some("Bullet Man"));
    assert_eq!(graphs.artboards[index].state_machines.len(), 1);
    artboard.advance(0.0).expect("initial artboard advance");
    artboard.advance_state_machine_instance(&mut machine, 0.0);
    let light = machine.input_index_named("Light").expect("Light trigger");
    machine.pointer_down(&mut artboard, 71.0, 263.0, 0);
    assert_eq!(
        machine.input(light).and_then(|input| input.trigger_fired()),
        Some(true)
    );
}

#[test]
fn wave_c9_event_003_hit_toggle_boolean_listener() {
    let (_, graphs, index, mut artboard, mut machine) = event_fixture("light_switch.riv", None);
    assert_eq!(graphs.artboards[index].state_machines.len(), 1);
    artboard.advance(0.0).expect("initial artboard advance");
    artboard.advance_state_machine_instance(&mut machine, 0.0);
    let on = machine.input_index_named("On").expect("On boolean");
    assert_eq!(
        machine.input(on).and_then(|input| input.bool_value()),
        Some(true)
    );
    machine.pointer_down(&mut artboard, 150.0, 258.0, 0);
    machine.pointer_up(&mut artboard, 150.0, 258.0, 0);
    assert_eq!(
        machine.input(on).and_then(|input| input.bool_value()),
        Some(false)
    );
    machine.pointer_down(&mut artboard, 150.0, 258.0, 0);
    machine.pointer_up(&mut artboard, 150.0, 258.0, 0);
    assert_eq!(
        machine.input(on).and_then(|input| input.bool_value()),
        Some(true)
    );
}

fn event_components<'a>(
    graphs: &'a GraphFile,
    index: usize,
) -> Vec<&'a nuxie_graph::ComponentNode> {
    graphs.artboards[index]
        .components
        .iter()
        .filter(|component| {
            definition_by_name(component.type_name).is_some_and(|kind| kind.is_a("Event"))
        })
        .collect()
}

#[test]
fn wave_c9_event_004_can_query_all_rive_events() {
    let (_, graphs) = import_fixture("event_on_listener.riv");
    assert_eq!(event_components(&graphs, 0).len(), 4);
}

#[test]
fn wave_c9_event_005_can_query_rive_event_at_index() {
    let (_, graphs) = import_fixture("event_on_listener.riv");
    let events = event_components(&graphs, 0);
    assert_eq!(events[0].name.as_deref(), Some("Somewhere.com"));
}

#[test]
fn wave_c9_event_006_events_load_on_listener() {
    let (file, graphs, index, mut artboard, mut machine) =
        event_fixture("event_on_listener.riv", None);
    assert_eq!(graphs.artboards[index].state_machines.len(), 1);
    artboard.advance(0.0).expect("initial artboard advance");
    artboard.advance_state_machine_instance(&mut machine, 0.0);
    assert_eq!(event_components(&graphs, index).len(), 4);
    let definition = &file.artboard_state_machine_graphs(index)[0];
    assert_eq!(definition.listeners.len(), 1);
    let listener = &definition.listeners[0];
    let target = &graphs.artboards[index].components
        [listener.object.uint_property("targetId").unwrap() as usize];
    assert!(definition_by_name(target.type_name).is_some_and(|kind| kind.is_a("Shape")));
    assert_eq!(listener.actions.len(), 2);
    let fire = &listener.actions[0];
    assert!(definition_by_name(fire.object.type_name)
        .is_some_and(|kind| kind.is_a("ListenerFireEvent")));
    assert_ne!(fire.object.uint_property("eventId"), Some(0));
    let event = fire.event.expect("listener event");
    assert!(definition_by_name(event.type_name).is_some_and(|kind| kind.is_a("Event")));
    assert_eq!(event.string_property("name"), Some("Footstep"));
    assert_eq!(machine.reported_event_count(), 0);
    machine.pointer_down(&mut artboard, 343.0, 116.0, 0);
    machine.pointer_up(&mut artboard, 343.0, 116.0, 0);
    assert_eq!(machine.reported_event_count(), 2);
    assert_eq!(
        machine
            .reported_event(&artboard, 0)
            .and_then(|event| event.name()),
        Some("Footstep")
    );
    assert_eq!(
        machine
            .reported_event(&artboard, 1)
            .and_then(|event| event.name()),
        Some("Event 3")
    );
    artboard.advance_state_machine_instance(&mut machine, 0.0);
    assert_eq!(machine.reported_event_count(), 0);
}

#[test]
fn wave_c9_event_007_events_load_on_state_and_transition() {
    let (file, graphs, index, mut artboard, mut machine) =
        event_fixture("events_on_states.riv", None);
    assert_eq!(graphs.artboards[index].state_machines.len(), 1);
    artboard.advance(0.0).expect("initial artboard advance");
    artboard.advance_state_machine_instance(&mut machine, 0.0);
    let definition = &file.artboard_state_machine_graphs(index)[0];
    assert_eq!(definition.layers.len(), 1);
    let layer = &definition.layers[0];
    assert_eq!(layer.state_count, 5);
    let entry = layer
        .states
        .iter()
        .find(|state| {
            state
                .object
                .is_some_and(|state| state.type_name == "EntryState")
        })
        .expect("entry state");
    assert_eq!(entry.transitions.len(), 1);
    let mut transition = &entry.transitions[0];
    assert_eq!(transition.fire_actions.len(), 0);
    let state_to = transition.state_to.expect("first animation state");
    assert!(definition_by_name(state_to.type_name).is_some_and(|kind| kind.is_a("AnimationState")));
    let first = layer
        .states
        .iter()
        .find(|state| state.object.is_some_and(|state| state.id == state_to.id))
        .expect("first animation state");
    assert_eq!(first.fire_actions.len(), 2);
    assert_eq!(first.transitions.len(), 1);
    transition = &first.transitions[0];
    assert_eq!(transition.fire_actions.len(), 2);
    assert_eq!(machine.reported_event_count(), 1);
    assert_eq!(
        machine
            .reported_event(&artboard, 0)
            .and_then(|event| event.name()),
        Some("First")
    );
    artboard.advance_state_machine_instance(&mut machine, 1.0);
    assert_eq!(machine.reported_event_count(), 0);
    artboard.advance_state_machine_instance(&mut machine, 1.0);
    assert_eq!(machine.reported_event_count(), 2);
    assert_eq!(
        machine
            .reported_event(&artboard, 0)
            .and_then(|event| event.name()),
        Some("Second")
    );
    assert_eq!(
        machine
            .reported_event(&artboard, 1)
            .and_then(|event| event.name()),
        Some("Third")
    );
    artboard.advance_state_machine_instance(&mut machine, 1.0);
    assert_eq!(machine.reported_event_count(), 1);
    assert_eq!(
        machine
            .reported_event(&artboard, 0)
            .and_then(|event| event.name()),
        Some("Fourth")
    );
}

#[test]
fn wave_c9_event_008_timeline_events_load_and_report() {
    let (_, graphs, index, mut artboard, mut machine) =
        event_fixture("timeline_event_test.riv", None);
    assert_eq!(graphs.artboards[index].state_machines.len(), 1);
    artboard.advance(0.0).expect("initial artboard advance");
    artboard.advance_state_machine_instance(&mut machine, 0.0);
    assert_eq!(machine.reported_event_count(), 0);
    artboard.advance_state_machine_instance(&mut machine, 0.4);
    assert_eq!(machine.reported_event_count(), 0);
    artboard.advance_state_machine_instance(&mut machine, 0.2);
    assert_eq!(machine.reported_event_count(), 1);
    let event = machine.reported_event(&artboard, 0).expect("Half event");
    assert_eq!(event.name(), Some("Half"));
    assert!((event.seconds_delay() - 0.1).abs() <= 0.00001);
}

#[test]
fn wave_c9_event_011_view_model_listener_event_is_host_visible() {
    let (file, graphs) = import_fixture("vm_listener_fire_event.riv");
    let definition = &file.artboard_state_machine_graphs(0)[0];
    let listener = &definition.listeners[0];
    assert_eq!(listener.listener_input_types.len(), 1);
    assert_eq!(
        listener.listener_input_types[0].uint_property("listenerTypeValue"),
        Some(11)
    );
    let (mut artboard, mut machine) = live_machine(&file, &graphs, 0, 0);
    let mut vmi = artboard
        .imported_view_model_instance_context(0, 0)
        .expect("authored VMI");
    assert!(machine.bind_imported_view_model_context(&file, &vmi));
    machine
        .advance_and_apply(&mut artboard, 0.0)
        .expect("initial advance");
    assert_eq!(machine.reported_event_count(), 0);
    assert!(vmi.set_trigger_by_property_name(&file, "go", 1));
    machine
        .advance_and_apply(&mut artboard, 0.016)
        .expect("listener advance");
    assert_eq!(machine.reported_event_count(), 1);
    assert_eq!(
        machine
            .reported_event(&artboard, 0)
            .and_then(|event| event.name()),
        Some("ding")
    );
    machine
        .advance_and_apply(&mut artboard, 0.016)
        .expect("event reset advance");
    assert_eq!(machine.reported_event_count(), 0);
}

#[test]
fn wave_c9_input_001_file_with_state_machine_inputs_loads() {
    let (file, graphs) = import_fixture("smi_test.riv");
    let artboard = &graphs.artboards[0];
    let nested_artboard = artboard
        .local_objects
        .iter()
        .find(|object| {
            object.type_name == Some("NestedArtboard")
                && object.name.as_deref() == Some("artboard to nest component")
        })
        .expect("named nested artboard");
    let nested_artboard = file
        .object(nested_artboard.global_id as usize)
        .expect("nested artboard object");
    assert_eq!(nested_artboard.double_property("x"), Some(100.0));
    assert_eq!(nested_artboard.double_property("y"), Some(100.0));
    assert_eq!(
        nested_artboard.string_property("name"),
        Some("artboard to nest component")
    );
    assert_eq!(nested_artboard.uint_property("artboardId"), Some(1));

    let nested_machine = artboard
        .local_objects
        .iter()
        .find(|object| {
            object.type_name == Some("NestedStateMachine")
                && object.name.as_deref().unwrap_or("").is_empty()
        })
        .expect("unnamed nested state machine");
    let nested_machine = file
        .object(nested_machine.global_id as usize)
        .expect("nested state machine object");
    assert_eq!(nested_machine.string_property("name").unwrap_or(""), "");
    assert_eq!(nested_machine.uint_property("animationId"), Some(0));

    let nested_trigger = artboard
        .local_objects
        .iter()
        .find(|object| {
            object.type_name == Some("NestedTrigger")
                && object.name.as_deref().unwrap_or("").is_empty()
        })
        .expect("unnamed nested trigger");
    let nested_trigger = file
        .object(nested_trigger.global_id as usize)
        .expect("nested trigger object");
    assert_eq!(nested_trigger.string_property("name").unwrap_or(""), "");
    assert_eq!(nested_trigger.uint_property("inputId"), Some(0));

    let nested_bool = artboard
        .local_objects
        .iter()
        .find(|object| {
            object.type_name == Some("NestedBool")
                && object.name.as_deref().unwrap_or("").is_empty()
        })
        .expect("unnamed nested bool");
    let nested_bool = file
        .object(nested_bool.global_id as usize)
        .expect("nested bool object");
    assert_eq!(nested_bool.string_property("name").unwrap_or(""), "");
    assert_eq!(nested_bool.uint_property("inputId"), Some(1));

    let nested_number = artboard
        .local_objects
        .iter()
        .find(|object| {
            object.type_name == Some("NestedNumber")
                && object.name.as_deref().unwrap_or("").is_empty()
        })
        .expect("unnamed nested number");
    let nested_number = file
        .object(nested_number.global_id as usize)
        .expect("nested number object");
    assert_eq!(nested_number.string_property("name").unwrap_or(""), "");
    assert_eq!(nested_number.uint_property("inputId"), Some(2));
}
