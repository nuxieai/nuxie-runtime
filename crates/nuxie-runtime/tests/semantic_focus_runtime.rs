//! Direct ports of the four focus-dependent cases in pinned
//! `tests/unit_tests/runtime/semantic_state_machine_test.cpp`.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard::RuntimeArtboardInstanceHandle,
    math::vec2d::Vec2D,
    semantic::{
        semantic_role::SemanticRole,
        semantic_snapshot::{SemanticsDiff, SemanticsDiffNode},
        semantic_state::{SemanticState, has_semantic_state},
        semantic_trait::{SemanticTrait, has_semantic_trait},
    },
};
use nuxie_runtime::{File, RuntimeFactoryHandle, RuntimeFileHandle};

fn native_file(bytes: &[u8]) -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("semantic fixture imports")
}

fn drain(machine: &RuntimeStateMachineInstanceHandle) -> SemanticsDiff {
    machine
        .with_instance(|machine| machine.semantic_manager())
        .unwrap()
        .with_semantic_manager_mut(|manager| manager.drain_diff())
}

fn scene(
    bytes: &[u8],
    require_model: bool,
) -> (
    RuntimeFileHandle,
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
    SemanticsDiff,
) {
    let file = native_file(bytes);
    let artboard = file.with_file(|file| file.artboard_default()).unwrap();
    let machine = artboard.state_machine_instance_handle(0).unwrap();
    machine.with_instance_mut(|machine| machine.enable_semantics());
    let model = file.with_file_mut(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    });
    if require_model {
        assert!(model.is_some());
    }
    if let Some(model) = model {
        artboard.bind_view_model_instance(Some(model.clone()));
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(model));
    }
    settle(&machine);
    let initial = drain(&machine);
    (file, artboard, machine, initial)
}

fn upstream_semantic_fixture(
    asset: &str,
) -> (
    RuntimeFileHandle,
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
    SemanticsDiff,
) {
    let fixture = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets/semantic")
    .join(asset);
    let bytes = std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read {}: {error}", fixture.display()));
    scene(&bytes, false)
}

fn tabs(diff: &SemanticsDiff) -> Vec<&SemanticsDiffNode> {
    diff.added
        .iter()
        .filter(|node| node.role == SemanticRole::Tab as u32)
        .collect()
}

fn selected(node: &SemanticsDiffNode) -> bool {
    has_semantic_state(node.state_flags, SemanticState::SELECTED)
}

fn semantic_snapshot(
    initial: &SemanticsDiff,
) -> std::collections::BTreeMap<u32, SemanticsDiffNode> {
    initial
        .added
        .iter()
        .cloned()
        .map(|node| (node.id, node))
        .collect()
}

fn apply_semantic_diff(
    snapshot: &mut std::collections::BTreeMap<u32, SemanticsDiffNode>,
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

fn settle(machine: &RuntimeStateMachineInstanceHandle) {
    for _ in 0..10 {
        machine.advance_and_apply(0.1);
    }
}

#[test]
fn upstream_simpsons_exposes_one_tab_list_and_labelled_single_selection() {
    let (_file, _artboard, _machine, initial) = upstream_semantic_fixture("simpsons.riv");
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
    let (_file, _artboard, machine, initial) = upstream_semantic_fixture("simpsons.riv");
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
        machine.with_instance_mut(|machine| machine.pointer_down(Vec2D::new(x, y), 0));
        machine.with_instance_mut(|machine| machine.pointer_up(Vec2D::new(x, y), 0));
        settle(&machine);
        let update = drain(&machine);
        apply_semantic_diff(&mut snapshot, &update);
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
    let file = native_file(&std::fs::read(fixture).unwrap());
    let artboard = file.with_file(|file| file.artboard_default()).unwrap();
    let machine = artboard.state_machine_instance_handle(0).unwrap();
    assert!(
        machine
            .with_instance(|machine| machine.semantic_manager())
            .is_none()
    );
    machine.with_instance_mut(|machine| machine.enable_semantics());
    let manager = machine
        .with_instance(|machine| machine.semantic_manager())
        .unwrap();
    machine.with_instance_mut(|machine| machine.enable_semantics());
    let again = machine
        .with_instance(|machine| machine.semantic_manager())
        .unwrap();
    assert!(
        manager.ptr_eq(&again),
        "enableSemantics retains the selected manager"
    );
}

#[test]
fn upstream_first_semantic_drain_delivers_tree_and_second_is_empty() {
    let (_file, _artboard, machine, initial) = upstream_semantic_fixture("simpsons.riv");
    assert!(initial.tree_version > 0);
    assert!(!initial.added.is_empty());
    assert!(
        initial
            .added
            .iter()
            .any(|node| node.role == SemanticRole::TabList as u32)
    );
    assert!(drain(&machine).is_empty());
}

#[test]
fn upstream_data_binding_lists_exposes_dropdown_button() {
    let (_file, _artboard, _machine, initial) = upstream_semantic_fixture("data_binding_lists.riv");
    let button = initial
        .added
        .iter()
        .find(|node| node.label == "Select a fandom")
        .expect("Select a fandom");
    assert_eq!(button.role, SemanticRole::Button as u32);
}

#[test]
fn upstream_list_scroll_focus_exposes_one_list_and_five_labelled_items() {
    let (_file, _artboard, _machine, initial) = list_focus_fixture();
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

fn list_focus_fixture() -> (
    RuntimeFileHandle,
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
    SemanticsDiff,
) {
    scene(
        include_bytes!("../../../fixtures/semantic/semantic_list_scroll_focus_fixed.riv"),
        true,
    )
}

#[test]
fn upstream_list_scroll_focus_items_expose_the_focusable_trait() {
    let (_file, _artboard, _machine, initial) = list_focus_fixture();
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
    let (_file, _artboard, machine, initial) = list_focus_fixture();
    let items = initial
        .added
        .iter()
        .filter(|node| node.role == SemanticRole::ListItem as u32)
        .collect::<Vec<_>>();
    let target = items
        .iter()
        .find(|node| node.label == "Element 3")
        .expect("third semantic list item");

    assert!(
        machine
            .with_instance(|machine| machine.semantic_manager())
            .unwrap()
            .request_focus(target.id)
    );
    for _ in 0..10 {
        machine.advance_and_apply(0.1);
    }
    let follow = drain(&machine);
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
    let (_file, _artboard, machine, initial) = list_focus_fixture();
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

    assert!(
        machine
            .with_instance(|machine| machine.semantic_manager())
            .unwrap()
            .request_focus(first.id)
    );
    machine.advance_and_apply(0.1);
    drain(&machine);

    assert!(
        machine
            .with_instance(|machine| machine.semantic_manager())
            .unwrap()
            .request_focus(third.id)
    );
    machine.advance_and_apply(0.1);
    let handoff = drain(&machine);
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
    let (_file, _artboard, machine, initial) = list_focus_fixture();
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

    assert!(
        machine
            .with_instance(|machine| machine.semantic_manager())
            .unwrap()
            .request_focus(last_id)
    );
    for _ in 0..10 {
        machine.advance_and_apply(0.1);
    }
    let scroll = drain(&machine);
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
