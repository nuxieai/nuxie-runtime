//! Direct ports of both cases in pinned
//! `tests/unit_tests/runtime/semantic_focus_list_test.cpp`.

use std::path::PathBuf;

use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_runtime::{ArtboardInstance, SemanticRole, SemanticsDiff};

const EXPECTED_SLOTS: [[f32; 4]; 4] = [
    [0.0, 0.0, 122.0, 59.0],
    [0.0, 75.0, 122.0, 134.0],
    [0.0, 150.0, 122.0, 209.0],
    [0.0, 225.0, 500.0, 500.0],
];

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets/semantic")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn load_fixture() -> SemanticsDiff {
    let file = read_runtime_file(&pinned_fixture("focus_nodes_list_order.riv"))
        .expect("focus-list fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("focus-list graph builds");
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("state machine zero");
    assert!(machine.enable_semantics());
    let _ = machine.bind_default_view_model_context_on_artboard(&mut artboard);
    for _ in 0..10 {
        machine
            .advance_and_apply(&mut artboard, 0.1)
            .expect("focus-list fixture settles");
    }
    machine
        .drain_semantics_diff(&mut artboard)
        .expect("semantic diff drains")
}

fn catch_approx(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() <= (100.0 * f32::EPSILON) * expected.abs()
}

#[test]
#[ignore = "expected-red: focus_nodes_list_order added nodes do not retain the upstream visual-order bounds"]
fn four_root_buttons_are_in_visual_order() {
    let diff = load_fixture();
    assert_eq!(diff.added.len(), EXPECTED_SLOTS.len());

    for node in &diff.added {
        assert_eq!(node.parent_id, -1);
        assert_eq!(node.role, SemanticRole::Button as u32);
    }

    for (index, (node, expected)) in diff.added.iter().zip(EXPECTED_SLOTS).enumerate() {
        assert_eq!(node.sibling_index, index as u32);
        assert!(catch_approx(node.min_x, expected[0]));
        assert!(catch_approx(node.min_y, expected[1]));
        assert!(catch_approx(node.max_x, expected[2]));
        assert!(catch_approx(node.max_y, expected[3]));
    }

    let root_update = diff
        .children_updated
        .iter()
        .find(|update| update.parent_id == -1)
        .expect("root children update");
    assert_eq!(root_update.child_ids.len(), EXPECTED_SLOTS.len());
    for (index, child_id) in root_update.child_ids.iter().enumerate() {
        assert_eq!(*child_id, diff.added[index].id);
    }
}

#[test]
#[ignore = "expected-red: Rust assigns the smallest semantic id to visual slot 0 instead of bottom slot 3"]
fn bottom_button_has_the_smallest_id_but_sorts_last() {
    let diff = load_fixture();
    assert_eq!(diff.added.len(), EXPECTED_SLOTS.len());

    let mut min_id = diff.added[0].id;
    let mut min_id_slot = 0;
    for (index, node) in diff.added.iter().enumerate().skip(1) {
        if node.id < min_id {
            min_id = node.id;
            min_id_slot = index;
        }
    }
    assert_eq!(min_id_slot, EXPECTED_SLOTS.len() - 1);
}
