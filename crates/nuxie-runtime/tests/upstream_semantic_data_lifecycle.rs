//! Exact executable port of lifecycle case 9 from pinned
//! `semantic_data_lifecycle_test.cpp`.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::semantic_listener_group::SemanticActionType,
    semantic::semantic_state::{SemanticState, has_semantic_state},
};
use nuxie_runtime::{File, RuntimeFactoryHandle};

const DROPDOWN_LABEL: &str = "Select a fandom";

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets/semantic")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

#[test]
fn wave_c15_019_state_machine_property_change_appears_in_updated_semantic() {
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(
        &pinned_fixture("data_binding_lists.riv"),
        factory,
        None,
        None,
        None,
    )
    .expect("data_binding_lists imports");
    let artboard = file
        .with_file(|file| file.artboard_default())
        .expect("default artboard");
    let state_machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine zero");
    state_machine.with_instance_mut(|machine| machine.enable_semantics());
    if let Some(instance) = file.with_file_mut(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    }) {
        artboard.bind_view_model_instance(Some(instance.clone()));
        state_machine.with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    }
    for _ in 0..10 {
        state_machine.advance_and_apply(0.1);
    }

    let manager = state_machine
        .with_instance(|machine| machine.semantic_manager())
        .expect("semantic manager");
    let initial = manager.with_semantic_manager_mut(|manager| manager.drain_diff());
    let initial_button = initial
        .added
        .iter()
        .find(|node| node.label == DROPDOWN_LABEL)
        .expect("initial dropdown button");
    assert!(has_semantic_state(
        initial_button.state_flags,
        SemanticState::EXPANDED
    ));
    let button_id = initial_button.id;

    state_machine.fire_semantic_action(button_id, SemanticActionType::Tap as u8);
    for _ in 0..10 {
        state_machine.advance_and_apply(0.1);
    }
    let follow = manager.with_semantic_manager_mut(|manager| manager.drain_diff());

    let updated = follow
        .updated_semantic
        .iter()
        .find(|node| node.id == button_id)
        .expect("dropdown semantic update");
    assert!(!has_semantic_state(
        updated.state_flags,
        SemanticState::EXPANDED
    ));
}
