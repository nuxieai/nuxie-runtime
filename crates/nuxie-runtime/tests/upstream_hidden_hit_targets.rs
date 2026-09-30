//! Complete added hittest_test.cpp cases at upstream 8f3690df.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        nested_state_machine::NestedStateMachine,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    artboard::RuntimeArtboardInstanceHandle,
    artboard_component_list::ArtboardComponentList,
    math::vec2d::Vec2D,
    nested_artboard::NestedArtboard,
};
use nuxie_runtime::{File, RuntimeFactoryHandle, RuntimeFileHandle};
use std::path::PathBuf;

fn load() -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes =
        std::fs::read(PathBuf::from(root).join("tests/unit_tests/assets/hidden_hit_targets.riv"))
            .expect("pinned hidden_hit_targets.riv");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("hidden_hit_targets imports")
}

// Retain the actual machine that owns the input, releasing its borrow before
// pointer dispatch. The definition and input table do not change in these cases.
struct ClickedInput(RuntimeStateMachineInstanceHandle);
impl ClickedInput {
    fn new(machine: RuntimeStateMachineInstanceHandle) -> Self {
        assert!(machine.with_instance(|machine| machine.get_bool("Clicked").is_some()));
        Self(machine)
    }
    fn value(&self) -> bool {
        self.0
            .with_instance(|machine| machine.get_bool("Clicked").unwrap().value())
    }
}

fn nested_clicked(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> ClickedInput {
    let nested = artboard
        .with_artboard(|board| board.find_handle::<NestedArtboard>(name))
        .expect("named nested artboard");
    let animation = nested
        .with_downcast::<NestedArtboard, _>(|nested| nested.nested_animations()[0].clone())
        .unwrap();
    let machine = animation
        .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
        .expect("nested animation is a NestedStateMachine")
        .expect("nested machine instance");
    ClickedInput::new(machine)
}

fn list_clicked(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> ClickedInput {
    let list = artboard
        .with_artboard(|board| board.find_handle::<ArtboardComponentList>(name))
        .expect("named artboard component list");
    let machine = list
        .with_downcast::<ArtboardComponentList, _>(|list| {
            assert_eq!(list.artboard_count(), 1);
            list.state_machine_instance(0)
                .expect("row machine instance")
        })
        .unwrap();
    ClickedInput::new(machine)
}

fn click(machine: &RuntimeStateMachineInstanceHandle) {
    machine.with_instance_mut(|machine| {
        machine.pointer_down(Vec2D::new(100.0, 100.0), 0);
    });
    machine.with_instance_mut(|machine| {
        machine.pointer_up(Vec2D::new(100.0, 100.0), 0);
    });
    machine.advance_and_apply(0.0);
}

#[test]
fn hidden_nested_artboard_does_not_take_hits() {
    let file = load();
    let artboard = file
        .with_file(|file| file.artboard_named("NestedHost"))
        .expect("NestedHost");
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine0");
    let hidden_clicked = nested_clicked(&artboard, "HiddenOver");
    let visible_clicked = nested_clicked(&artboard, "Visible");
    machine.advance_and_apply(0.0);
    assert!(!hidden_clicked.value());
    assert!(!visible_clicked.value());
    click(&machine);
    assert!(visible_clicked.value());
    assert!(!hidden_clicked.value());
}

#[test]
fn hidden_artboard_component_list_does_not_take_hits() {
    let file = load();
    let artboard = file
        .with_file(|file| file.artboard_named("ListHost"))
        .expect("ListHost");
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine0");
    let model = file.with_file(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    });
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(model));
    machine.advance_and_apply(0.0);
    let hidden_clicked = list_clicked(&artboard, "HiddenList");
    let visible_clicked = list_clicked(&artboard, "VisibleList");
    assert!(!hidden_clicked.value());
    assert!(!visible_clicked.value());
    click(&machine);
    assert!(visible_clicked.value());
    assert!(!hidden_clicked.value());
}
