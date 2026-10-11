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
fn disabled_and_hidden_semantic_actions_do_not_activate() {
    for (disabled, hidden) in [(true, false), (false, true), (false, false)] {
        let fixture = dropdown();
        let data = fixture
            .manager
            .with_semantic_manager(|manager| manager.node_by_id(fixture.button_id))
            .unwrap()
            .borrow()
            .semantic_data
            .clone()
            .unwrap();
        data.with_downcast_mut::<SemanticData, _>(|data| {
            data.set_is_disabled(disabled);
            data.set_is_hidden(hidden);
        })
        .unwrap();
        assert_eq!(
            fixture
                .machine
                .fire_semantic_action_checked(fixture.button_id, SemanticActionType::Tap as u8),
            !disabled && !hidden
        );
        for _ in 0..10 {
            fixture.machine.advance_and_apply(0.1);
        }
        assert_eq!(
            data.with_downcast::<SemanticData, _>(|data| data.is_expanded())
                .unwrap(),
            disabled || hidden,
            "disabled={disabled}, hidden={hidden}"
        );
    }
}

#[test]
fn host_semantic_actions_refuse_disabled_and_hidden_nodes_before_enqueue() {
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
            assert!(machine.fire_semantic_action_checked(button_id, SemanticActionType::Tap as u8));
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
            assert!(
                !machine.fire_semantic_action_checked(button_id, SemanticActionType::Tap as u8)
            );
        }
        for _ in 0..10 {
            machine.advance_and_apply(0.1);
        }
        assert_eq!(
            data.with_downcast::<SemanticData, _>(|data| data.is_expanded())
                .unwrap(),
            !queued,
            "Disabled and hidden nodes refuse new actions; accepted actions remain queued (hidden={hidden}, queued={queued})"
        );
    }
}

#[test]
fn semantic_actions_refuse_semantic_ancestors_then_resume() {
    for state in [SemanticState::DISABLED, SemanticState::HIDDEN] {
        let fixture = dropdown();
        let node = fixture
            .manager
            .with_semantic_manager(|manager| manager.node_by_id(fixture.button_id))
            .unwrap();
        let data = node.borrow().semantic_data.clone().unwrap();
        // The fixture's dropdown is a semantic root. Give it a registered
        // semantic ancestor through the same public manager API as a host.
        let ancestor = nuxie_runtime::source::semantic::semantic_node::SemanticNode::new(0);
        fixture.manager.add_child(None, ancestor.clone());
        fixture.manager.remove_child(&node);
        fixture.manager.add_child(Some(ancestor.clone()), node);
        let previous = ancestor.borrow().state_flags;
        ancestor.borrow_mut().state_flags |= state.0;
        assert!(
            !fixture
                .machine
                .fire_semantic_action_checked(fixture.button_id, 0)
        );
        for _ in 0..10 {
            fixture.machine.advance_and_apply(0.1);
        }
        assert!(
            data.with_downcast::<SemanticData, _>(|data| data.is_expanded())
                .unwrap()
        );
        ancestor.borrow_mut().state_flags = previous;
        assert!(
            fixture
                .machine
                .fire_semantic_action_checked(fixture.button_id, 0)
        );
        for _ in 0..10 {
            fixture.machine.advance_and_apply(0.1);
        }
        assert!(
            !data
                .with_downcast::<SemanticData, _>(|data| data.is_expanded())
                .unwrap()
        );
    }
}

#[test]
fn raw_upstream_dispatch_stays_available_when_host_dispatch_refuses() {
    let fixture = dropdown();
    let data = fixture
        .manager
        .with_semantic_manager(|manager| manager.node_by_id(fixture.button_id))
        .unwrap()
        .borrow()
        .semantic_data
        .clone()
        .unwrap();
    data.with_downcast_mut::<SemanticData, _>(|data| data.set_is_disabled(true))
        .unwrap();
    assert!(
        !fixture
            .machine
            .fire_semantic_action_checked(fixture.button_id, 0)
    );
    fixture.machine.fire_semantic_action(fixture.button_id, 0);
    for _ in 0..10 {
        fixture.machine.advance_and_apply(0.1);
    }
    assert!(
        !data
            .with_downcast::<SemanticData, _>(|data| data.is_expanded())
            .unwrap(),
        "the unmodified upstream API still delivers an authored action on the same disabled file node"
    );
}

#[test]
fn unavailable_actions_and_zero_opacity_sources_are_refused() {
    use nuxie_runtime::source::generated::{
        core_registry::CoreRegistry, world_transform_component_base::WorldTransformComponentBase,
    };
    let fixture = dropdown();
    assert!(!fixture.machine.fire_semantic_action_checked(u32::MAX, 0));
    assert!(
        !fixture
            .machine
            .fire_semantic_action_checked(fixture.button_id, u8::MAX)
    );
    let data = fixture
        .manager
        .with_semantic_manager(|manager| manager.node_by_id(fixture.button_id))
        .unwrap()
        .borrow()
        .semantic_data
        .clone()
        .unwrap();
    let parent = data
        .with(|data| data.component_parent_handle())
        .flatten()
        .unwrap();
    assert!(CoreRegistry::set_double_handle(
        &parent,
        i32::from(WorldTransformComponentBase::OPACITY_PROPERTY_KEY),
        0.0
    ));
    assert!(
        !fixture
            .machine
            .fire_semantic_action_checked(fixture.button_id, 0)
    );
    for _ in 0..10 {
        fixture.machine.advance_and_apply(0.1);
    }
    assert!(
        data.with_downcast::<SemanticData, _>(|data| data.is_expanded())
            .unwrap()
    );
}

#[test]
fn presented_capture_refuses_a_different_occurrence() {
    let first = dropdown();
    let other = dropdown();
    assert_eq!(
        first.button_id, other.button_id,
        "local IDs are reused across occurrences"
    );
    let capture = first.machine.capture_semantic_actions().unwrap();
    assert!(
        !other
            .machine
            .fire_semantic_action_from_capture(&capture, other.button_id, 0)
    );
    assert!(
        first
            .machine
            .fire_semantic_action_from_capture(&capture, first.button_id, 0)
    );
    for _ in 0..10 {
        first.machine.advance_and_apply(0.1);
    }
    let data = first
        .manager
        .with_semantic_manager(|manager| manager.node_by_id(first.button_id))
        .unwrap()
        .borrow()
        .semantic_data
        .clone()
        .unwrap();
    assert!(
        !data
            .with_downcast::<SemanticData, _>(|data| data.is_expanded())
            .unwrap()
    );
}

#[test]
fn presented_capture_refuses_a_stale_tree_then_accepts_a_new_presentation() {
    use nuxie_runtime::source::generated::{
        core_registry::CoreRegistry, semantic::semantic_data_base::SemanticDataBase,
    };
    let fixture = dropdown();
    let capture = fixture.machine.capture_semantic_actions().unwrap();
    let data = fixture
        .manager
        .with_semantic_manager(|manager| manager.node_by_id(fixture.button_id))
        .unwrap()
        .borrow()
        .semantic_data
        .clone()
        .unwrap();
    assert!(CoreRegistry::set_string_handle(
        &data,
        i32::from(SemanticDataBase::LABEL_PROPERTY_KEY),
        "Changed label".into()
    ));
    assert!(
        !fixture
            .machine
            .fire_semantic_action_from_capture(&capture, fixture.button_id, 0)
    );
    assert!(
        data.with_downcast::<SemanticData, _>(|data| data.is_expanded())
            .unwrap()
    );
    let current = fixture.machine.capture_semantic_actions().unwrap();
    assert!(current.tree_version() > capture.tree_version());
    assert_eq!(
        current
            .nodes()
            .iter()
            .find(|node| node.id == fixture.button_id)
            .unwrap()
            .label,
        "Changed label"
    );
    assert!(
        fixture
            .machine
            .fire_semantic_action_from_capture(&current, fixture.button_id, 0)
    );
    fixture.machine.advance_and_apply(0.1);
    assert!(
        !data
            .with_downcast::<SemanticData, _>(|data| data.is_expanded())
            .unwrap()
    );
}

#[path = "support/semantic_actions.rs"]
mod semantic_actions;

fn authored_tree(absorbed: bool) -> Dropdown {
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let file = File::import(
        &semantic_actions::authored_semantic_tree(absorbed),
        factory,
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(|file| file.artboard_default()).unwrap();
    let machine = artboard.state_machine_instance_handle(0).unwrap();
    machine.with_instance_mut(|machine| machine.enable_semantics());
    machine.advance_and_apply(0.0);
    let manager = machine
        .with_instance(|machine| machine.semantic_manager())
        .unwrap();
    let button_id = artboard
        .with_artboard(|board| board.base.resolve_handle(6))
        .unwrap()
        .with_downcast::<SemanticData, _>(|data| data.semantic_id())
        .unwrap();
    Dropdown {
        _file: file,
        _artboard: artboard,
        machine,
        manager,
        button_id,
    }
}

#[test]
fn a_file_authored_disabled_semantic_parent_refuses_then_readmits_its_child() {
    let fixture = authored_tree(false);
    let capture = fixture.machine.capture_semantic_actions().unwrap();
    assert_eq!(capture.nodes().len(), 2);
    let child = capture
        .nodes()
        .iter()
        .find(|node| node.label == "Action")
        .unwrap();
    let parent = capture
        .nodes()
        .iter()
        .find(|node| node.label == "Group")
        .unwrap();
    assert_eq!(
        child.parent_id, parent.id as i32,
        "the file supplies the semantic ancestor"
    );
    assert!(has_semantic_state(
        parent.state_flags,
        SemanticState::DISABLED
    ));
    assert!(
        !fixture
            .machine
            .fire_semantic_action_from_capture(&capture, child.id, 0)
    );
    assert!(!fixture.machine.fire_semantic_action_checked(child.id, 0));
    let data = fixture
        .manager
        .with_semantic_manager(|manager| manager.node_by_id(parent.id))
        .unwrap()
        .borrow()
        .semantic_data
        .clone()
        .unwrap();
    data.with_downcast_mut::<SemanticData, _>(|data| data.set_is_disabled(false))
        .unwrap();
    let enabled = fixture.machine.capture_semantic_actions().unwrap();
    assert!(
        fixture
            .machine
            .fire_semantic_action_from_capture(&enabled, child.id, 0)
    );
    assert!(fixture.machine.fire_semantic_action_checked(child.id, 0));
}

#[test]
fn a_presented_capture_refuses_an_absorbed_authored_child() {
    let fixture = authored_tree(true);
    let capture = fixture.machine.capture_semantic_actions().unwrap();
    assert_eq!(capture.nodes().len(), 1);
    assert_eq!(capture.nodes()[0].label, "Action");
    assert_eq!(
        capture.nodes()[0].role,
        1,
        "parent button presents the derived child label"
    );
    assert_ne!(capture.nodes()[0].id, fixture.button_id);
    assert!(
        !fixture
            .machine
            .fire_semantic_action_from_capture(&capture, fixture.button_id, 0)
    );
    assert!(
        fixture
            .machine
            .fire_semantic_action_checked(fixture.button_id, 0),
        "the existing checked call remains available for an unpresented but supported child"
    );
}

#[test]
fn a_capture_does_not_retain_a_disposed_occurrence() {
    let (capture, weak, node_id) = {
        let fixture = dropdown();
        (
            fixture.machine.capture_semantic_actions().unwrap(),
            fixture.manager.downgrade(),
            fixture.button_id,
        )
    };
    assert!(
        weak.upgrade().is_none(),
        "presentation token must not retain the native manager"
    );
    let other = dropdown();
    assert!(
        !other
            .machine
            .fire_semantic_action_from_capture(&capture, node_id, 0)
    );
}
