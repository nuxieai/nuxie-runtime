//! Literal state_machine_action_vm_sync_test.cpp cases from upstream 115c4862.
#![cfg(feature = "testing")]
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        animation_state::AnimationState, linear_animation::LinearAnimation,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_trigger::ViewModelInstanceTrigger,
    },
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};

struct RouterScene {
    // Match reverse C++ member teardown: view model, machine, artboard, file.
    view_model: CoreHandle,
    machine: RuntimeStateMachineInstanceHandle,
    _artboard: RuntimeArtboardInstanceHandle,
    _file: RuntimeFileHandle,
}
impl RouterScene {
    fn new(name: &str) -> Self {
        let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream");
        let bytes = std::fs::read(
            std::path::PathBuf::from(root).join("tests/unit_tests/assets/state_action_vm_sync.riv"),
        )
        .unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let source = file.with_file(|f| f.artboard_named_source(name)).unwrap();
        let artboard = Artboard::instance_from_handle(&source).unwrap();
        let machine = artboard.state_machine_at(0).unwrap();
        let model_id = artboard.with_artboard(|a| a.view_model_id());
        let view_model = file
            .with_file(|f| f.create_view_model_instance_at(model_id as usize, 0))
            .unwrap();
        machine.with_instance_mut(|m| m.bind_view_model_instance(view_model.clone()));
        machine.advance_and_apply(0.0);
        Self {
            machine,
            _artboard: artboard,
            _file: file,
            view_model,
        }
    }
    fn state(&self) -> String {
        self.machine
            .with_instance_mut(|m| m.layer_state(0))
            .and_then(|s| {
                s.with_downcast::<AnimationState, _>(AnimationState::animation)
                    .flatten()
            })
            .and_then(|a| a.with_downcast::<LinearAnimation, _>(|a| a.base.name().to_owned()))
            .unwrap_or_default()
    }
    fn property(&self, name: &str) -> CoreHandle {
        self.view_model
            .with_downcast::<ViewModelInstance, _>(|vm| vm.property_value_named(name))
            .flatten()
            .unwrap()
    }
    fn ready(&self) -> bool {
        self.property("ready")
            .with_downcast::<ViewModelInstanceBoolean, _>(|v| v.base.property_value())
            .unwrap()
    }
    fn fire_go(&self) {
        assert!(ViewModelInstanceTrigger::trigger_handle(
            &self.property("go")
        ));
    }
}
#[test]
fn state_enter_action_write_is_visible_in_the_same_advance() {
    let scene = RouterScene::new("StateEnter");
    assert_eq!(scene.state(), "idle");
    assert!(!scene.ready());
    scene.fire_go();
    scene.machine.advance_and_apply(0.016);
    assert!(scene.ready());
    assert_eq!(scene.state(), "done");
}
#[test]
fn timed_transition_end_action_write_is_visible_in_the_same_advance() {
    let scene = RouterScene::new("TransitionEnd");
    assert_eq!(scene.state(), "idle");
    scene.fire_go();
    scene.machine.advance_and_apply(0.016);
    assert_eq!(scene.state(), "mid");
    assert!(!scene.ready());
    for _ in 0..10 {
        scene.machine.advance_and_apply(0.016);
    }
    assert!(scene.ready());
    assert_eq!(scene.state(), "done");
}
