//! Complete listener_viewmodel_assign_test.cpp at upstream c7204ede.
//! Assigning another property's instance must not make Main its own parent;
//! the nested My ViewModel action instead resolves its child data context.
#[path = "support/text_style_global.rs"]
mod support;

use nuxie_runtime::source::viewmodel::{
    viewmodel_instance_trigger::ViewModelInstanceTrigger,
    viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
};
use support::*;

struct AssignScene {
    _file: RuntimeFileHandle,
    _artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    view_model: CoreHandle,
}

impl AssignScene {
    fn new(artboard_name: &str) -> Self {
        let file = load("listener_viewmodel_assign.riv");
        let artboard = file
            .with_file(|file| file.artboard_named(artboard_name))
            .expect("named artboard");
        let machine = artboard.state_machine_at(0).expect("state machine 0");
        let model_id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
        let view_model = file
            .with_file(|file| file.create_view_model_instance_at(model_id as usize, 0))
            .expect("view model instance 0");
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model.clone()));
        machine.advance_and_apply(0.0);
        Self {
            _file: file,
            _artboard: artboard,
            machine,
            view_model,
        }
    }

    fn property(&self, name: &str) -> CoreHandle {
        self.view_model
            .with_downcast::<ViewModelInstance, _>(|model| model.property_value_named(name))
            .flatten()
            .expect("view model property")
    }

    fn reference(&self, name: &str) -> Option<CoreHandle> {
        self.property(name)
            .with_downcast::<ViewModelInstanceViewModel, _>(
                ViewModelInstanceViewModel::reference_view_model_instance,
            )
            .expect("view model reference property")
    }

    fn fire_go(&self) {
        assert!(ViewModelInstanceTrigger::trigger_handle(
            &self.property("go")
        ));
    }
}

#[test]
fn listener_action_sets_a_view_model_property_from_another_one() {
    let scene = AssignScene::new("FromProperty");
    let source = scene.reference("source").expect("source instance");
    assert_ne!(scene.reference("current"), Some(source.clone()));

    scene.fire_go();
    scene.machine.advance_and_apply(0.016);
    assert_eq!(scene.reference("current"), Some(source));
}

#[test]
fn nested_listener_action_sets_a_view_model_property_to_my_view_model() {
    // Child is bound to source, so its own instance is source's.
    let scene = AssignScene::new("MyViewModel");
    let source = scene.reference("source").expect("source instance");
    assert_ne!(scene.reference("current"), Some(source.clone()));

    scene.fire_go();
    scene.machine.advance_and_apply(0.016);
    assert_eq!(scene.reference("current"), Some(source));
}
