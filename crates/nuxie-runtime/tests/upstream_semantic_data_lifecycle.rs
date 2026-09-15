//! Exact executable port of lifecycle case 9 from pinned
//! `semantic_data_lifecycle_test.cpp`.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        semantic_listener_group::SemanticActionType,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    semantic::semantic_state::{SemanticState, has_semantic_state},
    semantic::{semantic_data::SemanticData, semantic_manager::RuntimeSemanticManagerHandle},
};
use nuxie_runtime::{File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle};

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

struct Dropdown {
    _file: RuntimeFileHandle,
    _artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    manager: RuntimeSemanticManagerHandle,
    button_id: u32,
}

fn dropdown() -> Dropdown {
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
    Dropdown {
        _file: file,
        _artboard: artboard,
        machine: state_machine,
        manager,
        button_id,
    }
}

#[test]
fn wave_c15_019_state_machine_property_change_appears_in_updated_semantic() {
    let fixture = dropdown();
    let Dropdown {
        machine: state_machine,
        manager,
        button_id,
        ..
    } = &fixture;
    let button_id = *button_id;

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

#[test]
fn disabled_and_hidden_semantic_nodes_do_not_execute_taps() {
    for (hidden, queued) in [(false, false), (true, false), (false, true), (true, true)] {
        let fixture = dropdown();
        let Dropdown {
            machine,
            manager,
            button_id,
            ..
        } = &fixture;
        let button_id = *button_id;
        let data = manager
            .with_semantic_manager(|manager| manager.node_by_id(button_id))
            .expect("dropdown node")
            .borrow()
            .semantic_data
            .clone()
            .expect("authored semantics");
        if queued {
            machine.fire_semantic_action(button_id, SemanticActionType::Tap as u8);
        }
        data.with_downcast_mut::<SemanticData, _>(|data| {
            if hidden {
                data.set_is_hidden(true);
            } else {
                data.set_is_disabled(true);
            }
        })
        .expect("semantic data");
        if !queued {
            machine.fire_semantic_action(button_id, SemanticActionType::Tap as u8);
        }
        for _ in 0..10 {
            machine.advance_and_apply(0.1);
        }
        assert!(
            data.with_downcast::<SemanticData, _>(|data| data.is_expanded())
                .unwrap(),
            "ineligible semantic tap must not close the dropdown (hidden={hidden}, queued={queued})"
        );
    }
}

#[test]
fn full_snapshot_survives_diff_drain_and_tracks_authored_actions() {
    let fixture = dropdown();
    let snapshot = fixture
        .manager
        .with_semantic_manager_mut(|manager| manager.snapshot().to_vec());
    let button = snapshot
        .iter()
        .find(|node| node.id == fixture.button_id)
        .expect("dropdown in full tree after initial diff drain");
    assert_eq!(button.label, DROPDOWN_LABEL);
    assert!(has_semantic_state(
        button.state_flags,
        SemanticState::EXPANDED
    ));
    fixture
        .machine
        .fire_semantic_action(fixture.button_id, SemanticActionType::Tap as u8);
    for _ in 0..10 {
        fixture.machine.advance_and_apply(0.1);
    }
    let updated = fixture
        .manager
        .with_semantic_manager_mut(|manager| manager.snapshot().to_vec());
    let button = updated
        .iter()
        .find(|node| node.id == fixture.button_id)
        .expect("same dropdown occurrence");
    assert!(!has_semantic_state(
        button.state_flags,
        SemanticState::EXPANDED
    ));
    let diff = fixture
        .manager
        .with_semantic_manager_mut(|manager| manager.drain_diff());
    assert!(
        diff.updated_semantic
            .iter()
            .any(|node| node.id == fixture.button_id)
    );
    assert_eq!(
        fixture
            .manager
            .with_semantic_manager_mut(|manager| manager.snapshot().to_vec()),
        updated
    );
}
