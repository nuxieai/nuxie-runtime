//! One-for-one expected-red ports of pinned
//! `tests/unit_tests/runtime/scroll_velocity_test.cpp`.
//!
//! Scroll physics and pointer routing are present, but Rust's public retained
//! owner does not expose the exact C++ `velocityX`, `velocityY`, or
//! `scrollActive` observables. The complete four bodies remain explicit.

use std::path::PathBuf;

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::GraphFile;
use nuxie_runtime::{ArtboardInstance, StateMachineInstance};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name).expect("schema definition");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            nuxie_schema::definition_by_name(ancestor)
                .expect("ancestor definition")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .unwrap_or_else(|| panic!("property {type_name}.{property_name}"))
        .key
        .int
}

struct Fixture {
    _file: RuntimeFile,
    _graphs: GraphFile,
    artboard: ArtboardInstance,
    state_machine: Option<StateMachineInstance>,
    scroll_local: usize,
}

fn fixture(name: &str, with_state_machine: bool) -> Fixture {
    let file = read_runtime_file(&pinned_fixture(name))
        .unwrap_or_else(|error| panic!("{name} imports: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&file)
        .unwrap_or_else(|error| panic!("{name} graph builds: {error:#}"));
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let state_machine = with_state_machine.then(|| {
        let index = graph
            .state_machines
            .iter()
            .position(|machine| machine.name.as_deref() == Some("State Machine 1"))
            .expect("State Machine 1");
        artboard
            .state_machine_instance(index)
            .expect("State Machine 1 instantiates")
    });
    let scroll_local = artboard
        .scroll_constraint_occurrences()
        .first()
        .expect("fixture has a ScrollConstraint")
        .constraint_local_id;
    artboard.advance(0.0).expect("initial artboard advance");
    Fixture {
        _file: file,
        _graphs: graphs,
        artboard,
        state_machine,
        scroll_local,
    }
}

fn velocity_x(_: &ArtboardInstance, _: usize) -> f32 {
    missing_scroll_velocity_owner()
}

fn velocity_y(_: &ArtboardInstance, _: usize) -> f32 {
    missing_scroll_velocity_owner()
}

fn scroll_active(_: &ArtboardInstance, _: usize) -> bool {
    missing_scroll_velocity_owner()
}

fn missing_scroll_velocity_owner() -> ! {
    panic!("Rust does not expose the retained ScrollConstraint velocity/scrollActive owner")
}

#[test]
#[ignore = "expected-red: Rust does not expose ScrollConstraint velocity/scrollActive"]
fn scroll_constraint_velocity_and_scroll_active_during_drag() {
    let mut fixture = fixture("layout/layout_scroll_vertical.riv", true);
    let state_machine = fixture.state_machine.as_mut().expect("state machine");

    assert_eq!(velocity_x(&fixture.artboard, fixture.scroll_local), 0.0);
    assert_eq!(velocity_y(&fixture.artboard, fixture.scroll_local), 0.0);
    assert!(!scroll_active(&fixture.artboard, fixture.scroll_local));

    state_machine.pointer_move(&mut fixture.artboard, 50.0, 250.0, 0.0, 0);
    state_machine.pointer_down(&mut fixture.artboard, 50.0, 250.0, 0);
    state_machine
        .advance_and_apply(&mut fixture.artboard, 0.1)
        .expect("drag-start advance");

    assert!(scroll_active(&fixture.artboard, fixture.scroll_local));
    assert_eq!(velocity_y(&fixture.artboard, fixture.scroll_local), 0.0);

    state_machine.pointer_move(&mut fixture.artboard, 50.0, 50.0, 0.0, 0);
    state_machine
        .advance_and_apply(&mut fixture.artboard, 0.0)
        .expect("drag-move advance");
    assert_ne!(velocity_y(&fixture.artboard, fixture.scroll_local), 0.0);
    assert!(scroll_active(&fixture.artboard, fixture.scroll_local));

    state_machine
        .advance_and_apply(&mut fixture.artboard, 0.1)
        .expect("drag-pause advance");
    assert!(scroll_active(&fixture.artboard, fixture.scroll_local));

    state_machine.pointer_up(&mut fixture.artboard, 50.0, 50.0, 0);
    let scroll = fixture.artboard.scroll_constraint_occurrences()[0];
    assert!(scroll.physics_running);
    assert!(scroll_active(&fixture.artboard, fixture.scroll_local));
}

#[test]
#[ignore = "expected-red: Rust does not expose ScrollConstraint velocity/scrollActive"]
fn scroll_constraint_velocity_resets_after_physics_settles() {
    let mut fixture = fixture("layout/layout_scroll_vertical.riv", true);
    let state_machine = fixture.state_machine.as_mut().expect("state machine");

    state_machine.pointer_move(&mut fixture.artboard, 50.0, 250.0, 0.0, 0);
    state_machine.pointer_down(&mut fixture.artboard, 50.0, 250.0, 0);
    state_machine
        .advance_and_apply(&mut fixture.artboard, 0.1)
        .expect("drag-start advance");
    state_machine.pointer_move(&mut fixture.artboard, 50.0, 50.0, 0.0, 0);
    state_machine
        .advance_and_apply(&mut fixture.artboard, 0.0)
        .expect("drag-move advance");
    state_machine.pointer_up(&mut fixture.artboard, 50.0, 50.0, 0);

    assert!(fixture.artboard.scroll_constraint_occurrences()[0].physics_running);
    assert!(scroll_active(&fixture.artboard, fixture.scroll_local));

    for _ in 0..600 {
        state_machine
            .advance_and_apply(&mut fixture.artboard, 0.016)
            .expect("physics settle advance");
        if !fixture.artboard.scroll_constraint_occurrences()[0].physics_running {
            break;
        }
    }

    assert!(!fixture.artboard.scroll_constraint_occurrences()[0].physics_running);
    assert!(!scroll_active(&fixture.artboard, fixture.scroll_local));
    assert_eq!(velocity_x(&fixture.artboard, fixture.scroll_local), 0.0);
    assert_eq!(velocity_y(&fixture.artboard, fixture.scroll_local), 0.0);
}

#[test]
#[ignore = "expected-red: Rust does not expose ScrollConstraint velocity/scrollActive"]
fn scroll_constraint_horizontal_velocity() {
    let mut fixture = fixture("layout/layout_scroll_horizontal.riv", true);
    let state_machine = fixture.state_machine.as_mut().expect("state machine");

    state_machine.pointer_move(&mut fixture.artboard, 250.0, 50.0, 0.0, 0);
    state_machine.pointer_down(&mut fixture.artboard, 250.0, 50.0, 0);
    state_machine
        .advance_and_apply(&mut fixture.artboard, 0.1)
        .expect("drag-start advance");
    state_machine.pointer_move(&mut fixture.artboard, 50.0, 50.0, 0.0, 0);
    state_machine
        .advance_and_apply(&mut fixture.artboard, 0.0)
        .expect("drag-move advance");

    assert_ne!(velocity_x(&fixture.artboard, fixture.scroll_local), 0.0);
    assert_eq!(velocity_y(&fixture.artboard, fixture.scroll_local), 0.0);
    assert!(scroll_active(&fixture.artboard, fixture.scroll_local));
    state_machine.pointer_up(&mut fixture.artboard, 50.0, 50.0, 0);
}

#[test]
#[ignore = "expected-red: Rust does not expose ScrollConstraint velocity/scrollActive"]
fn scroll_constraint_scroll_active_false_when_idle() {
    let mut fixture = fixture("layout/layout_scroll_vertical.riv", false);
    let percent_y = property_key("ScrollConstraint", "percentY");
    assert!(
        fixture
            .artboard
            .set_double_property(fixture.scroll_local, percent_y, 0.5)
    );

    assert!(!scroll_active(&fixture.artboard, fixture.scroll_local));
    assert_eq!(velocity_x(&fixture.artboard, fixture.scroll_local), 0.0);
    assert_eq!(velocity_y(&fixture.artboard, fixture.scroll_local), 0.0);
}
