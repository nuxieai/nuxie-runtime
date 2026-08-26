//! One case per pinned `listener_align_target_test.cpp` owner flow.

use std::path::PathBuf;

use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_runtime::ArtboardInstance;

fn run_case(artboard_name: &str, expected_y: f32) {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let path = root.join("tests/unit_tests/assets/align_target.riv");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let runtime = read_runtime_file(&bytes).expect("import align_target.riv");
    let graphs = GraphFile::from_runtime_file(&runtime).expect("build align_target graph");
    let graph = graphs
        .artboards
        .iter()
        .find(|graph| graph.name.as_deref() == Some(artboard_name))
        .unwrap_or_else(|| panic!("missing artboard {artboard_name}"));
    let circle = graph
        .local_objects
        .iter()
        .find(|object| {
            object.type_name == Some("Shape") && object.name.as_deref() == Some("circle")
        })
        .expect("circle Shape")
        .local_id;
    assert_eq!(graph.state_machines.len(), 1);
    assert_eq!(
        graph.state_machines[0].name.as_deref(),
        Some("align-state-machine")
    );

    let mut artboard =
        ArtboardInstance::from_graph_with_artboards(&runtime, graph, &graphs.artboards)
            .unwrap_or_else(|error| panic!("instantiate {artboard_name}: {error:#}"));
    let mut state_machine = artboard
        .state_machine_instance(0)
        .expect("align-state-machine instance");

    artboard.advance(0.0).expect("Artboard::advance(0)");
    let _ = artboard.advance_state_machine_instance(&mut state_machine, 0.0);
    let _ = artboard.advance_state_machine_instance(&mut state_machine, 0.0);
    let _ = state_machine.pointer_move(&mut artboard, 100.0, 50.0, 0.0, 0);
    let _ = state_machine.pointer_move(&mut artboard, 100.0, 51.0, 0.0, 0);
    let _ = artboard.advance_state_machine_instance(&mut state_machine, 1.0);
    let _ = artboard.advance_state_machine_instance(&mut state_machine, 0.0);

    let transform = artboard
        .component_world_transform_with_scroll(circle)
        .expect("circle world transform");
    assert_eq!(transform.tx(), 100.0);
    assert_eq!(transform.ty(), expected_y);
}

#[test]
fn wave_c2_listener_align_001_preserve_offset_off() {
    run_case("preserve-inactive", 51.0);
}

#[test]
fn wave_c2_listener_align_002_preserve_offset_on() {
    run_case("preserve-active", 101.0);
}
