//! Direct ports of the four focus-dependent cases in pinned
//! `tests/unit_tests/runtime/semantic_state_machine_test.cpp`.

use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_runtime::{
    ArtboardInstance, SemanticRole, SemanticState, SemanticTrait, SemanticsDiff,
    StateMachineInstance, has_semantic_state, has_semantic_trait,
};

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

    assert_eq!(items.len(), 5);
    for (index, item) in items.iter().enumerate() {
        assert_eq!(item.label, format!("Element {}", index + 1));
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
