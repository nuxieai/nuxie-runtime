use super::*;
use crate::math::hit_test::{HitTestArea, HitTester};
use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::GraphFile;
use nuxie_render_api::FillRule;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets")
    .join(name)
}

fn fixture(
    fixture_name: &str,
    artboard_name: &str,
    machine_name: &str,
) -> (
    RuntimeFile,
    GraphFile,
    usize,
    ArtboardInstance,
    StateMachineInstance,
) {
    let path = fixture_path(fixture_name);
    let runtime = read_runtime_file(
        &std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("import {fixture_name}: {error}"));
    let graph = GraphFile::from_runtime_file(&runtime)
        .unwrap_or_else(|error| panic!("graph {fixture_name}: {error}"));
    let artboard_index = graph
        .artboards
        .iter()
        .position(|artboard| artboard.name.as_deref() == Some(artboard_name))
        .unwrap_or_else(|| panic!("artboard {artboard_name}"));
    let artboard_graph = &graph.artboards[artboard_index];
    assert_eq!(artboard_graph.state_machines.len(), 1);
    let machine_index = artboard_graph
        .state_machines
        .iter()
        .position(|machine| machine.name.as_deref() == Some(machine_name))
        .unwrap_or_else(|| panic!("state machine {machine_name}"));
    let mut artboard =
        ArtboardInstance::from_graph_with_artboards(&runtime, artboard_graph, &graph.artboards)
            .unwrap_or_else(|error| panic!("instantiate {fixture_name}: {error}"));
    let machine = artboard
        .state_machine_instance(machine_index)
        .unwrap_or_else(|| panic!("instantiate {machine_name}"));
    (runtime, graph, artboard_index, artboard, machine)
}

fn initialize(artboard: &mut ArtboardInstance, machine: &mut StateMachineInstance) {
    artboard.advance_state_machine_instance(machine, 0.0);
    artboard.update_components();
    assert!(machine.needs_advance());
    artboard.advance_state_machine_instance(machine, 0.0);
}

fn bool_input(machine: &StateMachineInstance, name: &str) -> bool {
    machine
        .get_bool(name)
        .and_then(|input| input.bool_value())
        .unwrap_or_else(|| panic!("bool input {name}"))
}

fn current_animation_name<'a>(
    graph: &'a GraphFile,
    artboard_index: usize,
    machine: &StateMachineInstance,
) -> Option<&'a str> {
    let animation_index = machine.current_animation(0)?.animation_index();
    graph.artboards[artboard_index]
        .animations
        .get(animation_index)?
        .name
        .as_deref()
}

fn nested_bool_input(
    artboard: &ArtboardInstance,
    host_name: &str,
    input_name: &str,
) -> Option<bool> {
    let host_local = artboard
        .slots()
        .iter()
        .find(|slot| slot.name.as_deref() == Some(host_name))?
        .local_id;
    artboard
        .nested_artboards
        .get(&host_local)?
        .animations
        .iter()
        .find_map(|animation| match animation {
            crate::artboard::RuntimeNestedAnimationInstance::StateMachine(occurrence) => occurrence
                .state_machine()
                .and_then(|machine| machine.get_bool(input_name))
                .and_then(|input| input.bool_value()),
            _ => None,
        })
}

fn retained_early_out_count(
    machine: &StateMachineInstance,
    hit_component_index: usize,
) -> Option<usize> {
    machine.hit_component(hit_component_index)?;
    // The concrete HitComponent occurrence exists, but Rust retains only the
    // event-routing early-out policy. Pinned TESTING increments a per-owner
    // counter at every skipped event; there is no retained Rust value to read.
    None
}

#[test]
fn wave_b5_hittest_basics() {
    let mut tester = HitTester::new(HitTestArea::new(10, 10, 12, 12));
    tester.move_to((0.0, 0.0));
    tester.line_to((20.0, 0.0));
    tester.line_to((20.0, 20.0));
    tester.line_to((0.0, 20.0));
    tester.close();
    assert!(tester.test(FillRule::NonZero));

    tester.reset(HitTestArea::new(81, 156, 84, 159));
    let points = [
        (29.9785, 32.5261),
        (231.102, 32.5261),
        (231.102, 269.898),
        (29.9785, 269.898),
    ];
    tester.move_to(points[0]);
    for point in points.into_iter().skip(1) {
        tester.line_to(point);
    }
    tester.close();
    assert!(tester.test(FillRule::NonZero));
}

#[test]
fn wave_b5_hittest_mesh() {
    assert!(HitTester::test_mesh_area(
        HitTestArea::new(10, 10, 12, 12),
        &[(0.0, 0.0), (20.0, 10.0), (0.0, 20.0)],
        &[0, 1, 2],
    ));
}

#[test]
fn wave_b5_hit_test_on_opaque_target() {
    let (_, _, _, mut artboard, mut machine) =
        fixture("opaque_hit_test.riv", "main", "main-state-machine");
    initialize(&mut artboard, &mut machine);
    assert_eq!(bool_input(&machine, "toGreen"), false);
    assert_eq!(bool_input(&machine, "grayToggle"), false);

    machine.pointer_down(&mut artboard, 100.0, 50.0, 0);
    assert_eq!(bool_input(&machine, "grayToggle"), true);
    assert_eq!(bool_input(&machine, "toGreen"), false);

    machine.pointer_down(&mut artboard, 100.0, 250.0, 0);
    assert_eq!(bool_input(&machine, "grayToggle"), false);
    assert_eq!(bool_input(&machine, "toGreen"), true);

    machine.pointer_down(&mut artboard, 100.0, 110.0, 0);
    assert_eq!(bool_input(&machine, "grayToggle"), true);
    assert_eq!(bool_input(&machine, "toGreen"), false);
}

#[test]
#[ignore = "expected-red: pointer down at x=301 toggles second-gray-toggle outside the 300px opaque nested-artboard bounds"]
fn wave_b5_hit_test_on_opaque_nested_artboard() {
    let (_, _, _, mut artboard, mut machine) =
        fixture("opaque_hit_test.riv", "second", "second-state-machine");
    assert_eq!(
        nested_bool_input(&artboard, "second-nested", "bool-target"),
        Some(false)
    );
    artboard.update_components();
    artboard.advance_state_machine_instance(&mut machine, 0.0);
    assert_eq!(bool_input(&machine, "second-gray-toggle"), false);

    machine.pointer_down(&mut artboard, 100.0, 250.0, 0);
    assert_eq!(bool_input(&machine, "second-gray-toggle"), true);
    machine.pointer_down(&mut artboard, 301.0, 50.0, 0);
    assert_eq!(bool_input(&machine, "second-gray-toggle"), true);
    machine.pointer_down(&mut artboard, 100.0, 50.0, 0);
    assert_eq!(bool_input(&machine, "second-gray-toggle"), true);
    assert_eq!(
        nested_bool_input(&artboard, "second-nested", "bool-target"),
        Some(true)
    );

    artboard.advance_state_machine_instance(&mut machine, 1.0);
    artboard.advance_state_machine_instance(&mut machine, 0.0);
    machine.pointer_down(&mut artboard, 100.0, 50.0, 0);
    assert_eq!(bool_input(&machine, "second-gray-toggle"), false);
    assert_eq!(
        nested_bool_input(&artboard, "second-nested", "bool-target"),
        Some(true)
    );
}

#[test]
#[ignore = "expected-red: exact four HitComponent owners exist, but Rust does not retain the TESTING earlyOutCount observable"]
fn wave_b5_early_out_on_listeners() {
    let (_, _, _, mut artboard, mut machine) = fixture("pointer_events.riv", "art-1", "sm-1");
    initialize(&mut artboard, &mut machine);
    assert_eq!(machine.hit_components_count(), 4);
    assert!(machine.hit_component(0).is_some());
    assert_eq!(retained_early_out_count(&machine, 0), Some(0));
}

#[test]
fn wave_b5_click_event() {
    let (_, graph, artboard_index, mut artboard, mut machine) =
        fixture("click_event.riv", "art-1", "sm-1");
    initialize(&mut artboard, &mut machine);
    assert_eq!(machine.hit_components_count(), 2);
    assert_eq!(
        graph.artboards[artboard_index].state_machines[0]
            .layers
            .len(),
        1
    );
    assert_eq!(machine.reported_event_count(), 0);

    let mut counts = Vec::new();
    for (down, up) in [
        ((75.0, 75.0), (75.0, 75.0)),
        ((75.0, 75.0), (300.0, 75.0)),
        ((300.0, 75.0), (75.0, 75.0)),
        ((75.0, 75.0), (225.0, 225.0)),
        ((150.0, 150.0), (150.0, 150.0)),
    ] {
        machine.pointer_down(&mut artboard, down.0, down.1, 0);
        machine.pointer_up(&mut artboard, up.0, up.1, 0);
        counts.push(machine.reported_event_count());
    }
    assert_eq!(counts, [1, 1, 1, 2, 3]);
}

#[test]
fn wave_b5_multiple_shapes_with_mouse_movement_behavior() {
    let (_, graph, artboard_index, mut artboard, mut machine) =
        fixture("click_event.riv", "art-2", "sm-1");
    initialize(&mut artboard, &mut machine);
    assert_eq!(machine.hit_components_count(), 2);
    assert_eq!(
        graph.artboards[artboard_index].state_machines[0]
            .layers
            .len(),
        1
    );

    for (x, expected) in [
        (75.0, "green"),
        (200.0, "green"),
        (400.0, "red"),
        (200.0, "green"),
    ] {
        machine.pointer_move(&mut artboard, x, 75.0, 0.0, 0);
        artboard.update_components();
        artboard.advance_state_machine_instance(&mut machine, 0.0);
        assert_eq!(
            current_animation_name(&graph, artboard_index, &machine),
            Some(expected)
        );
    }
}
