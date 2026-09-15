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

#[test]
fn queued_semantic_action_rechecks_ancestor_eligibility() {
    use nuxie_runtime::source::semantic::semantic_node::SemanticNode;
    for state in [0, 1, 2, 3, 4] {
        let fixture = dropdown();
        let node = fixture
            .manager
            .with_semantic_manager(|manager| manager.node_by_id(fixture.button_id).unwrap());
        let data = node.borrow().semantic_data.clone().unwrap();
        let parent = SemanticNode::new(0);
        fixture.manager.remove_child(&node);
        fixture.manager.add_child(None, parent.clone());
        fixture
            .manager
            .add_child(Some(parent.clone()), node.clone());
        fixture
            .machine
            .fire_semantic_action(fixture.button_id, SemanticActionType::Tap as u8);
        match state {
            1 => parent.borrow_mut().state_flags |= SemanticState::DISABLED.0,
            2 => parent.borrow_mut().state_flags |= SemanticState::HIDDEN.0,
            3 | 4 => fixture.manager.remove_child(&parent),
            _ => {}
        }
        if state == 4 {
            drop(parent);
        }
        for _ in 0..10 {
            fixture.machine.advance_and_apply(0.1);
        }
        assert_eq!(
            data.with_downcast::<SemanticData, _>(|data| data.is_expanded())
                .unwrap(),
            state != 0,
            "only an attached eligible ancestor permits queued activation (state={state})"
        );
    }
}

#[test]
fn disabled_and_hidden_controls_reject_touch_then_resume_when_enabled() {
    use nuxie_runtime::source::math::vec2d::Vec2D;
    for (hidden, on_host) in [(false, false), (true, false), (false, true), (true, true)] {
        let fixture = dropdown();
        let snapshot = fixture
            .manager
            .with_semantic_manager_mut(|manager| manager.snapshot().to_vec());
        let button = snapshot
            .iter()
            .find(|node| node.id == fixture.button_id)
            .unwrap();
        let point = Vec2D::new(
            (button.min_x + button.max_x) * 0.5,
            (button.min_y + button.max_y) * 0.5,
        );
        let node = fixture
            .manager
            .with_semantic_manager(|manager| manager.node_by_id(fixture.button_id))
            .unwrap();
        let data = node.borrow().semantic_data.clone().unwrap();
        let outer = fixture
            ._file
            .with_file(|file| file.artboard_default())
            .unwrap();
        struct HostContext {
            arena: nuxie_runtime::source::core::CoreArena,
            root: nuxie_runtime::CoreHandle,
        }
        impl nuxie_runtime::source::core_context::CoreContext for HostContext {
            fn core_arena(&self) -> &nuxie_runtime::source::core::CoreArena {
                &self.arena
            }
            fn resolve_handle(&self, id: u32) -> Option<nuxie_runtime::CoreHandle> {
                (id == 0).then(|| self.root.clone())
            }
        }
        let state_owner = if on_host {
            // Attach the imported occurrence to an actual nested-artboard host.
            // Host semantics deliberately have no manager: touch admission must
            // use authored state independently of accessibility registration.
            let host = outer.with_artboard(|artboard| {
                artboard
                    .core_arena()
                    .insert(nuxie_runtime::source::nested_artboard::NestedArtboard::new())
            });
            let host_data = fixture
                ._file
                .with_file(|file| file.core_arena().insert(SemanticData::default()));
            host.with_mut(|host| {
                host.as_container_component_mut()
                    .unwrap()
                    .add_child(host_data.clone())
            });
            let mut context = HostContext {
                arena: outer.with_artboard(|artboard| artboard.core_arena().clone()),
                root: outer.core_handle(),
            };
            host.with_mut(|host| {
                assert_eq!(
                    host.as_container_component_mut()
                        .unwrap()
                        .base
                        .base
                        .on_added_dirty(&mut context),
                    nuxie_runtime::source::status_code::StatusCode::Ok
                );
            });
            fixture
                ._artboard
                .with_artboard_mut(|artboard| artboard.set_host_handle(Some(host)));
            host_data
        } else {
            data.clone()
        };
        let touch = || {
            fixture.machine.with_instance_mut(|machine| {
                machine.pointer_down(point, 51);
                machine.pointer_up(point, 51);
            });
            for _ in 0..10 {
                fixture.machine.advance_and_apply(0.1);
            }
        };
        state_owner
            .with_downcast_mut::<SemanticData, _>(|data| {
                if hidden {
                    data.set_is_hidden(true);
                } else {
                    data.set_is_disabled(true);
                }
            })
            .unwrap();
        touch();
        assert!(
            data.with_downcast::<SemanticData, _>(|data| data.is_expanded())
                .unwrap(),
            "ineligible touch must not close the dropdown"
        );
        state_owner
            .with_downcast_mut::<SemanticData, _>(|data| {
                if hidden {
                    data.set_is_hidden(false);
                } else {
                    data.set_is_disabled(false);
                }
            })
            .unwrap();
        touch();
        assert!(
            !data
                .with_downcast::<SemanticData, _>(|data| data.is_expanded())
                .unwrap(),
            "reenabled touch must close the dropdown (hidden={hidden}, on_host={on_host})"
        );
    }
}

#[test]
fn invisible_controls_leave_semantics_and_return_when_visible() {
    use nuxie_runtime::source::generated::{
        core_registry::CoreRegistry, world_transform_component_base::WorldTransformComponentBase,
    };
    for (ancestor, queued) in [(false, false), (true, false), (false, true), (true, true)] {
        let fixture = dropdown();
        let data = fixture
            .manager
            .with_semantic_manager(|manager| manager.node_by_id(fixture.button_id))
            .unwrap()
            .borrow()
            .semantic_data
            .clone()
            .unwrap();
        let parent = data
            .with(|data| data.as_component().unwrap().parent_handle())
            .flatten()
            .unwrap();
        let parent = if ancestor {
            parent
                .with(|parent| parent.component_parent_handle())
                .flatten()
                .unwrap()
        } else {
            parent
        };
        if queued {
            fixture
                .machine
                .fire_semantic_action(fixture.button_id, SemanticActionType::Tap as u8);
        }
        let opacity_key = i32::from(WorldTransformComponentBase::OPACITY_PROPERTY_KEY);
        let original_opacity = CoreRegistry::get_double_handle(&parent, opacity_key).unwrap();
        assert!(original_opacity > 0.0);
        assert!(CoreRegistry::set_double_handle(&parent, opacity_key, 0.0));
        fixture.machine.advance_and_apply(0.0);
        let hidden = fixture
            .manager
            .with_semantic_manager_mut(|manager| manager.snapshot().to_vec());
        assert!(
            !hidden.iter().any(|node| node.id == fixture.button_id),
            "zero-opacity control remains accessible"
        );
        fixture
            .machine
            .fire_semantic_action(fixture.button_id, SemanticActionType::Tap as u8);
        fixture.machine.advance_and_apply(0.0);
        assert!(
            data.with_downcast::<SemanticData, _>(|data| data.is_expanded())
                .unwrap()
        );
        assert!(CoreRegistry::set_double_handle(
            &parent,
            opacity_key,
            original_opacity
        ));
        fixture.machine.advance_and_apply(0.0);
        let visible = fixture
            .manager
            .with_semantic_manager_mut(|manager| manager.snapshot().to_vec());
        assert!(visible.iter().any(|node| node.id == fixture.button_id));
        fixture
            .machine
            .fire_semantic_action(fixture.button_id, SemanticActionType::Tap as u8);
        fixture.machine.advance_and_apply(0.0);
        assert!(
            !data
                .with_downcast::<SemanticData, _>(|data| data.is_expanded())
                .unwrap()
        );
    }
}

#[test]
fn fresh_semantic_registration_uses_authored_opacity_before_the_first_advance() {
    use nuxie_runtime::source::generated::{
        core_registry::CoreRegistry, world_transform_component_base::WorldTransformComponentBase,
    };
    for opacity in [1.0, 0.0] {
        let mut factory = PersistentFactory::new(RecordingFactory::default());
        let file = File::import(
            &pinned_fixture("simpsons.riv"),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let artboard = file.with_file(|file| file.artboard_default()).unwrap();
        let data = artboard.with_artboard(|artboard| {
            artboard
                .objects_typed::<SemanticData>()
                .iter()
                .next()
                .unwrap()
        });
        data.with_downcast_mut::<SemanticData, _>(|data| {
            data.set_label("Fresh semantic control".to_owned());
            data.set_role(1);
            data.set_is_hidden(false);
        })
        .unwrap();
        let owner = data
            .with(|data| data.component_parent_handle())
            .flatten()
            .unwrap();
        assert!(CoreRegistry::set_double_handle(
            &owner,
            i32::from(WorldTransformComponentBase::OPACITY_PROPERTY_KEY),
            opacity,
        ));
        let manager = RuntimeSemanticManagerHandle::new(
            nuxie_runtime::source::semantic::semantic_manager::SemanticManager::new(),
        );
        artboard.build_semantic_tree(Some(manager.clone()), None);
        let nodes = manager.with_semantic_manager_mut(|manager| manager.snapshot().to_vec());
        assert_eq!(
            nodes
                .iter()
                .any(|node| node.label == "Fresh semantic control"),
            opacity > 0.0
        );
    }
}
