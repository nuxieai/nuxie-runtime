//! Direct ports of the four focus-dependent cases in pinned
//! `tests/unit_tests/runtime/semantic_state_machine_test.cpp`.

use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_runtime::{
    ArtboardInstance, SemanticRole, SemanticState, SemanticTrait, SemanticsDiff,
    StateMachineInstance, has_semantic_state, has_semantic_trait,
};

fn upstream_semantic_fixture(
    asset: &str,
) -> (ArtboardInstance, StateMachineInstance, SemanticsDiff) {
    let fixture = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets/semantic")
    .join(asset);
    let bytes = std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read {}: {error}", fixture.display()));
    let file = read_runtime_file(&bytes).expect("semantic fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("semantic fixture graph builds");
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let _ = machine.enable_semantics();
    let _ = machine.bind_default_view_model_context_on_artboard(&mut artboard);
    for _ in 0..10 {
        machine
            .advance_and_apply(&mut artboard, 0.1)
            .expect("semantic fixture settles");
    }
    let initial = machine
        .drain_semantics_diff(&mut artboard)
        .expect("initial semantic tree drains");
    (artboard, machine, initial)
}

fn tabs(diff: &SemanticsDiff) -> Vec<&nuxie_runtime::SemanticsDiffNode> {
    diff.added
        .iter()
        .filter(|node| node.role == SemanticRole::Tab as u32)
        .collect()
}

fn selected(node: &nuxie_runtime::SemanticsDiffNode) -> bool {
    has_semantic_state(node.state_flags, SemanticState::SELECTED)
}

fn semantic_snapshot(
    initial: &SemanticsDiff,
) -> std::collections::BTreeMap<u32, nuxie_runtime::SemanticsDiffNode> {
    initial
        .added
        .iter()
        .cloned()
        .map(|node| (node.id, node))
        .collect()
}

fn apply_semantic_diff(
    snapshot: &mut std::collections::BTreeMap<u32, nuxie_runtime::SemanticsDiffNode>,
    diff: &SemanticsDiff,
) {
    for id in &diff.removed {
        snapshot.remove(id);
    }
    for node in diff
        .added
        .iter()
        .chain(&diff.moved)
        .chain(&diff.updated_semantic)
    {
        snapshot.insert(node.id, node.clone());
    }
    for geometry in &diff.updated_geometry {
        if let Some(node) = snapshot.get_mut(&geometry.id) {
            node.min_x = geometry.min_x;
            node.min_y = geometry.min_y;
            node.max_x = geometry.max_x;
            node.max_y = geometry.max_y;
        }
    }
}

fn settle(machine: &mut StateMachineInstance, artboard: &mut ArtboardInstance) {
    for _ in 0..10 {
        machine
            .advance_and_apply(artboard, 0.1)
            .expect("semantic action settles");
    }
}

#[test]
fn upstream_simpsons_exposes_one_tab_list_and_labelled_single_selection() {
    let (_, _, initial) = upstream_semantic_fixture("simpsons.riv");
    assert_eq!(
        initial
            .added
            .iter()
            .filter(|node| node.role == SemanticRole::TabList as u32)
            .count(),
        1
    );
    let tabs = tabs(&initial);
    assert!(tabs.len() >= 2);
    assert!(tabs.iter().all(|tab| !tab.label.is_empty()));
    assert_eq!(tabs.iter().filter(|tab| selected(tab)).count(), 1);
}

#[test]
fn upstream_simpsons_tabs_produce_two_three_and_five_list_items() {
    let (mut artboard, mut machine, initial) = upstream_semantic_fixture("simpsons.riv");
    let tab_nodes = tabs(&initial).into_iter().cloned().collect::<Vec<_>>();
    assert_eq!(tab_nodes.len(), 3);
    let mut snapshot = semantic_snapshot(&initial);
    assert_eq!(
        snapshot
            .values()
            .filter(|node| node.role == SemanticRole::List as u32)
            .count(),
        1
    );
    let mut counts = Vec::new();
    for tab in tab_nodes {
        let x = (tab.min_x + tab.max_x) * 0.5;
        let y = (tab.min_y + tab.max_y) * 0.5;
        machine.pointer_down(&mut artboard, x, y, 0);
        machine.pointer_up(&mut artboard, x, y, 0);
        settle(&mut machine, &mut artboard);
        apply_semantic_diff(
            &mut snapshot,
            &machine.drain_semantics_diff(&mut artboard).unwrap(),
        );
        counts.push(
            snapshot
                .values()
                .filter(|node| node.role == SemanticRole::ListItem as u32)
                .count(),
        );
    }
    counts.sort_unstable();
    assert_eq!(counts, [2, 3, 5]);
}

#[test]
fn upstream_enable_semantics_is_idempotent_and_manager_stays_selected() {
    let fixture = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets/semantic/simpsons.riv");
    let file = read_runtime_file(&std::fs::read(fixture).unwrap()).unwrap();
    let graphs = GraphFile::from_runtime_file(&file).unwrap();
    let mut artboard =
        ArtboardInstance::from_graph_with_artboards(&file, &graphs.artboards[0], &graphs.artboards)
            .unwrap();
    let mut machine = artboard.state_machine_instance(0).unwrap();
    assert!(!machine.semantic_manager());
    assert!(machine.enable_semantics());
    assert!(machine.semantic_manager());
    assert!(!machine.enable_semantics());
    assert!(machine.semantic_manager());
}

#[test]
fn upstream_first_semantic_drain_delivers_tree_and_second_is_empty() {
    let (mut artboard, mut machine, initial) = upstream_semantic_fixture("simpsons.riv");
    assert!(initial.tree_version > 0);
    assert!(!initial.added.is_empty());
    assert!(
        initial
            .added
            .iter()
            .any(|node| node.role == SemanticRole::TabList as u32)
    );
    assert!(
        machine
            .drain_semantics_diff(&mut artboard)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn upstream_data_binding_lists_exposes_dropdown_button() {
    let (_, _, initial) = upstream_semantic_fixture("data_binding_lists.riv");
    let button = initial
        .added
        .iter()
        .find(|node| node.label == "Select a fandom")
        .expect("Select a fandom");
    assert_eq!(button.role, SemanticRole::Button as u32);
}

#[test]
fn upstream_list_scroll_focus_exposes_one_list_and_five_labelled_items() {
    let (_, _, initial) = list_focus_fixture();
    assert_eq!(
        initial
            .added
            .iter()
            .filter(|node| node.role == SemanticRole::List as u32)
            .count(),
        1
    );
    let items = initial
        .added
        .iter()
        .filter(|node| node.role == SemanticRole::ListItem as u32)
        .collect::<Vec<_>>();
    for index in 1..=5 {
        let label = format!("Element {index}");
        let item = items
            .iter()
            .find(|item| item.label == label)
            .expect("authored visible list item");
        assert_eq!(item.role, SemanticRole::ListItem as u32);
    }
    assert_eq!(items.len(), 5);
}

fn list_focus_fixture() -> (ArtboardInstance, StateMachineInstance, SemanticsDiff) {
    let file = read_runtime_file(include_bytes!(
        "../../../fixtures/semantic/semantic_list_scroll_focus_fixed.riv"
    ))
    .expect("semantic list-focus fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("semantic list-focus graph builds");
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("default state machine instantiates");
    assert!(machine.enable_semantics());
    assert!(machine.bind_default_view_model_context_on_artboard(&mut artboard));
    for _ in 0..10 {
        machine
            .advance_and_apply(&mut artboard, 0.1)
            .expect("semantic list-focus fixture settles");
    }
    let initial = machine
        .drain_semantics_diff(&mut artboard)
        .expect("retained semantic tree drains after enable");
    (artboard, machine, initial)
}

#[test]
fn upstream_list_scroll_focus_items_expose_the_focusable_trait() {
    let (_, _, initial) = list_focus_fixture();
    let items = initial
        .added
        .iter()
        .filter(|node| node.role == SemanticRole::ListItem as u32)
        .collect::<Vec<_>>();

    for index in 1..=5 {
        let label = format!("Element {index}");
        let item = items
            .iter()
            .find(|item| item.label == label)
            .expect("authored visible list item");
        assert!(has_semantic_trait(
            item.trait_flags,
            SemanticTrait::FOCUSABLE
        ));
    }
}

#[test]
fn upstream_request_focus_sets_focused_only_on_the_target_item() {
    let (mut artboard, mut machine, initial) = list_focus_fixture();
    let items = initial
        .added
        .iter()
        .filter(|node| node.role == SemanticRole::ListItem as u32)
        .collect::<Vec<_>>();
    let target = items
        .iter()
        .find(|node| node.label == "Element 3")
        .expect("third semantic list item");

    assert!(machine.request_semantic_focus(target.id));
    for _ in 0..10 {
        machine
            .advance_and_apply(&mut artboard, 0.1)
            .expect("focus request settles");
    }
    let follow = machine
        .drain_semantics_diff(&mut artboard)
        .expect("focused semantic diff drains");
    let focused = follow
        .updated_semantic
        .iter()
        .filter(|node| has_semantic_state(node.state_flags, SemanticState::FOCUSED))
        .collect::<Vec<_>>();

    assert_eq!(focused.len(), 1);
    assert_eq!(focused[0].id, target.id);
}

#[test]
fn upstream_moving_focus_hands_the_focused_bit_between_items() {
    let (mut artboard, mut machine, initial) = list_focus_fixture();
    let first = initial
        .added
        .iter()
        .find(|node| node.label == "Element 1")
        .expect("first semantic list item");
    let third = initial
        .added
        .iter()
        .find(|node| node.label == "Element 3")
        .expect("third semantic list item");

    assert!(machine.request_semantic_focus(first.id));
    machine
        .advance_and_apply(&mut artboard, 0.1)
        .expect("first focus request settles");
    machine
        .drain_semantics_diff(&mut artboard)
        .expect("first focus diff drains");

    assert!(machine.request_semantic_focus(third.id));
    machine
        .advance_and_apply(&mut artboard, 0.1)
        .expect("third focus request settles");
    let handoff = machine
        .drain_semantics_diff(&mut artboard)
        .expect("focus handoff diff drains");
    let first_update = handoff
        .updated_semantic
        .iter()
        .find(|node| node.id == first.id)
        .expect("first item emits its blur state");
    let third_update = handoff
        .updated_semantic
        .iter()
        .find(|node| node.id == third.id)
        .expect("third item emits its focused state");

    assert!(!has_semantic_state(
        first_update.state_flags,
        SemanticState::FOCUSED
    ));
    assert!(has_semantic_state(
        third_update.state_flags,
        SemanticState::FOCUSED
    ));
}

#[test]
fn upstream_focusing_the_bottom_slot_scrolls_every_item_upward() {
    let (mut artboard, mut machine, initial) = list_focus_fixture();
    let items = initial
        .added
        .iter()
        .filter(|node| node.role == SemanticRole::ListItem as u32)
        .map(|node| (node.id, (node.label.clone(), node.min_y)))
        .collect::<std::collections::BTreeMap<_, _>>();
    let last_id = items
        .iter()
        .find_map(|(id, (label, _))| (label == "Element 5").then_some(*id))
        .expect("bottom semantic list item");

    assert!(machine.request_semantic_focus(last_id));
    for _ in 0..10 {
        machine
            .advance_and_apply(&mut artboard, 0.1)
            .expect("focus-driven scroll settles");
    }
    let scroll = machine
        .drain_semantics_diff(&mut artboard)
        .expect("focus-driven geometry diff drains");
    let shifted = scroll
        .updated_geometry
        .iter()
        .filter(|update| items.contains_key(&update.id))
        .collect::<Vec<_>>();

    assert_eq!(
        shifted.len(),
        items.len(),
        "initial items: {items:#?}\nscroll diff: {scroll:#?}"
    );
    for update in shifted {
        let (label, start_y) = &items[&update.id];
        assert!(
            update.min_y < *start_y,
            "{label} must move upward: {} !< {start_y}",
            update.min_y
        );
    }
}
