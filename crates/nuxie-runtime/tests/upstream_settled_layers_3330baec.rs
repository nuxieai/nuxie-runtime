//! Direct translations of all five cases in upstream 3330baec
//! tests/unit_tests/runtime/settled_layers_test.cpp.
#![cfg(feature = "testing")]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        state_machine::StateMachine,
        state_machine_instance::{RuntimeStateMachineInstanceHandle, StateMachineInstance},
        state_machine_layer::StateMachineLayer,
    },
    shapes::{
        paint::{fill::Fill, solid_color::SolidColor},
        shape::Shape,
    },
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_enum::ViewModelInstanceEnum,
        viewmodel_instance_trigger::ViewModelInstanceTrigger,
    },
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};

fn read_file(asset: &str) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR");
    let path = std::path::PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(asset);
    let bytes = std::fs::read(path).expect("upstream asset");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("upstream asset imports")
}

fn instance(file: &RuntimeFileHandle, name: &str) -> RuntimeArtboardInstanceHandle {
    let source = file
        .with_file(|file| file.artboard_named_source(name))
        .expect("named artboard");
    Artboard::instance_from_handle(&source).expect("artboard instance")
}

fn default_vm(file: &RuntimeFileHandle, artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    file.with_file(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    })
    .expect("default view model instance")
}

fn shape(artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    artboard
        .with_artboard(|artboard| artboard.find_handle::<Shape>("color_rectangle"))
        .expect("color_rectangle")
}

fn fill_color(artboard: &RuntimeArtboardInstanceHandle) -> u32 {
    let fill = shape(artboard)
        .with_downcast::<Shape, _>(|shape| shape.base.children()[1].clone())
        .unwrap();
    let paint = fill
        .with_downcast::<Fill, _>(|fill| fill.base.paint())
        .expect("second child is Fill")
        .unwrap();
    paint
        .with_downcast::<SolidColor, _>(|paint| paint.base.color_value() as u32)
        .expect("SolidColor")
}

fn position(shape: &CoreHandle) -> (f32, f32) {
    shape
        .with_downcast::<Shape, _>(|shape| (shape.base.x(), shape.base.y()))
        .unwrap()
}

fn idle(machine: &RuntimeStateMachineInstanceHandle) {
    for _ in 0..10 {
        machine.advance_and_apply(1.0 / 60.0);
    }
}

#[test]
fn view_model_conditions_settle_until_a_bound_value_changes() {
    let file = read_file("data_binding_test.riv");
    let artboard = instance(&file, "artboard-2");
    let vm = default_vm(&file, &artboard);
    let machine = artboard.default_state_machine_handle().unwrap();
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(vm.clone()));
    let shape = shape(&artboard);
    let state = vm
        .with_downcast::<ViewModelInstance, _>(|vm| vm.property_value_named("state"))
        .flatten()
        .unwrap();
    let trigger = vm
        .with_downcast::<ViewModelInstance, _>(|vm| vm.property_value_named("trigger-prop"))
        .flatten()
        .unwrap();
    machine.advance_and_apply(0.0);
    assert_eq!(fill_color(&artboard), 0xffff0000);
    let skips = StateMachineInstance::settled_layer_skips();
    idle(&machine);
    assert!(StateMachineInstance::settled_layer_skips() > skips);
    assert_eq!(fill_color(&artboard), 0xffff0000);
    state
        .with_downcast_mut::<ViewModelInstanceEnum, _>(|state| state.set_value_at(1))
        .unwrap();
    machine.advance_and_apply(0.0);
    assert_eq!(fill_color(&artboard), 0xff00ff00);
    assert_eq!(position(&shape), (150.0, 250.0));
    idle(&machine);
    state
        .with_downcast_mut::<ViewModelInstanceEnum, _>(|state| state.set_value_named("state-blue"))
        .unwrap();
    assert!(ViewModelInstanceTrigger::trigger_handle(&trigger));
    machine.advance_and_apply(0.0);
    assert_eq!(fill_color(&artboard), 0xff0000ff);
    assert_eq!(position(&shape), (350.0, 250.0));
    idle(&machine);
    assert!(ViewModelInstanceTrigger::trigger_handle(&trigger));
    machine.advance_and_apply(0.0);
    assert_eq!(position(&shape), (350.0, 350.0));
}

#[test]
fn states_reading_state_machine_inputs_are_always_searched() {
    let file = read_file("state_machine_triggers.riv");
    let artboard = instance(&file, "main");
    let machine = artboard.state_machine_named("State Machine 1").unwrap();
    machine.advance_and_apply(0.1);
    let idle_state = machine
        .with_instance_mut(|machine| machine.layer_state(0))
        .expect("idle state");
    let skips = StateMachineInstance::settled_layer_skips();
    idle(&machine);
    assert_eq!(StateMachineInstance::settled_layer_skips(), skips);
    assert_eq!(
        machine.with_instance_mut(|machine| machine.layer_state(0)),
        Some(idle_state.clone())
    );
    machine.with_instance_mut(|machine| machine.get_trigger_mut("Trigger 1").unwrap().fire());
    machine.advance_and_apply(0.1);
    assert_ne!(
        machine.with_instance_mut(|machine| machine.layer_state(0)),
        Some(idle_state)
    );
}

#[test]
fn binding_a_data_context_wakes_settled_layers() {
    let file = read_file("data_binding_test.riv");
    let artboard = instance(&file, "artboard-2");
    let machine = artboard.default_state_machine_handle().unwrap();
    let skips = StateMachineInstance::settled_layer_skips();
    let frozen = StateMachineInstance::frozen_layer_advances();
    idle(&machine);
    assert!(StateMachineInstance::settled_layer_skips() > skips);
    assert!(StateMachineInstance::frozen_layer_advances() > frozen);
    machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(default_vm(&file, &artboard))
    });
    machine.advance_and_apply(0.0);
    assert_eq!(fill_color(&artboard), 0xffff0000);
}

#[test]
fn setting_a_view_model_instance_without_binding_wakes_settled_layers() {
    let file = read_file("data_binding_test.riv");
    let settled_artboard = instance(&file, "artboard-2");
    let settled = settled_artboard.default_state_machine_handle().unwrap();
    idle(&settled);
    settled.with_instance_mut(|machine| {
        machine.set_view_model_instance(default_vm(&file, &settled_artboard))
    });
    settled.advance_and_apply(0.0);
    let fresh_artboard = instance(&file, "artboard-2");
    let fresh = fresh_artboard.default_state_machine_handle().unwrap();
    fresh.with_instance_mut(|machine| {
        machine.set_view_model_instance(default_vm(&file, &fresh_artboard))
    });
    fresh.advance_and_apply(0.0);
    let count = settled
        .with_instance(|machine| machine.state_machine())
        .with_downcast::<StateMachine, _>(|machine| machine.layer_count())
        .unwrap();
    for i in 0..count {
        assert_eq!(
            settled.with_instance_mut(|machine| machine.layer_state(i)),
            fresh.with_instance_mut(|machine| machine.layer_state(i))
        );
    }
    assert_eq!(fill_color(&settled_artboard), fill_color(&fresh_artboard));
}

#[test]
fn states_whose_transitions_wait_on_an_exit_time_never_freeze() {
    let file = read_file("bidirectional_binding_source.riv");
    let artboard = file
        .with_file(|file| file.artboard_named_source("avatar_child_artboard"))
        .expect("avatar_child_artboard");
    let machines = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.state_machine_handles().to_vec())
        .unwrap();
    let (mut timed, mut untimed) = (0, 0);
    for machine in machines {
        let layers = machine
            .with_downcast::<StateMachine, _>(|machine| {
                (0..machine.layer_count())
                    .map(|i| machine.layer(i).unwrap())
                    .collect::<Vec<_>>()
            })
            .unwrap();
        for layer in layers {
            let states = layer
                .with_downcast::<StateMachineLayer, _>(|layer| {
                    (0..layer.state_count())
                        .map(|i| layer.state(i).unwrap())
                        .collect::<Vec<_>>()
                })
                .unwrap();
            for state in states {
                let waits_on_exit_time = state
                    .with(|state| {
                        (0..state.layer_state_transition_count().unwrap()).any(|i| {
                            state
                                .layer_state_transition(i)
                                .unwrap()
                                .with(|transition| {
                                    let transition = transition.as_state_transition().unwrap();
                                    !transition.is_disabled() && transition.enable_exit_time()
                                })
                                .unwrap()
                        })
                    })
                    .unwrap();
                assert_eq!(
                    state
                        .with(|state| state.layer_state_settle_flags().unwrap().1)
                        .unwrap(),
                    !waits_on_exit_time
                );
                if waits_on_exit_time {
                    timed += 1;
                } else {
                    untimed += 1;
                }
            }
        }
    }
    assert!(timed > 0);
    assert!(untimed > 0);
}
