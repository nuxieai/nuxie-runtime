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
                .fire_semantic_action(fixture.button_id, SemanticActionType::Tap as u8),
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
            assert!(machine.fire_semantic_action(button_id, SemanticActionType::Tap as u8));
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
            assert!(!machine.fire_semantic_action(button_id, SemanticActionType::Tap as u8));
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
fn semantic_actions_refuse_target_parent_and_mounted_host_then_resume() {
    for (hidden, location) in [
        (false, 0),
        (true, 0),
        (false, 1),
        (true, 1),
        (false, 2),
        (true, 2),
    ] {
        let fixture = dropdown();
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
        let state_owner = if location == 2 {
            // Attach the imported occurrence to an actual nested-artboard host.
            // Host semantics deliberately have no manager: action admission must
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
        } else if location == 1 {
            let target = data
                .with(|owner| owner.component_parent_handle())
                .flatten()
                .unwrap();
            let parent = target
                .with(|owner| owner.component_parent_handle())
                .flatten()
                .unwrap();
            assert!(parent.with(|owner| owner.as_artboard().is_none()).unwrap());
            let parent_data = parent.insert_sibling(SemanticData::default()).unwrap();
            parent
                .with_mut(|owner| {
                    owner
                        .as_container_component_mut()
                        .unwrap()
                        .add_child(parent_data.clone());
                })
                .unwrap();
            parent_data
        } else {
            data.clone()
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
        assert!(
            !fixture
                .machine
                .fire_semantic_action(fixture.button_id, SemanticActionType::Tap as u8,)
        );
        for _ in 0..10 {
            fixture.machine.advance_and_apply(0.1);
        }
        assert!(
            data.with_downcast::<SemanticData, _>(|data| data.is_expanded())
                .unwrap(),
            "ineligible action must not close the dropdown"
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
        assert!(
            fixture
                .machine
                .fire_semantic_action(fixture.button_id, SemanticActionType::Tap as u8,)
        );
        for _ in 0..10 {
            fixture.machine.advance_and_apply(0.1);
        }
        assert!(
            !data
                .with_downcast::<SemanticData, _>(|data| data.is_expanded())
                .unwrap(),
            "reenabled action must close the dropdown (hidden={hidden}, location={location})"
        );
    }
}
