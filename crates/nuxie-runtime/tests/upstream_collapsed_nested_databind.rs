//! Port of collapsed_nested_databind_test.cpp at upstream 074bfb139e53.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::viewmodel::{
    viewmodel_instance::ViewModelInstance, viewmodel_instance_number::ViewModelInstanceNumber,
};
use nuxie_runtime::{CoreHandle, File, RuntimeFactoryHandle};

fn property(instance: &CoreHandle, name: &str) -> CoreHandle {
    instance
        .with_downcast::<ViewModelInstance, _>(|instance| instance.property_value_named(name))
        .flatten()
        .unwrap_or_else(|| panic!("view-model property {name}"))
}

fn set_number(instance: &CoreHandle, name: &str, value: f32) {
    property(instance, name)
        .with_downcast_mut::<ViewModelInstanceNumber, _>(|number| number.set_value(value))
        .expect("number property");
}

fn check_number(instance: &CoreHandle, name: &str, expected: f32) {
    let actual = property(instance, name)
        .with_downcast::<ViewModelInstanceNumber, _>(ViewModelInstanceNumber::value)
        .expect("number property");
    // Catch Approx's default epsilon, with float operands widened to double.
    let difference = (f64::from(actual) - f64::from(expected)).abs();
    let tolerance = f64::from(f32::EPSILON) * 100.0 * f64::from(expected).abs();
    assert!(
        difference <= tolerance,
        "{name}: expected {expected}, got {actual}"
    );
}

#[test]
fn a_collapsed_nested_artboard_does_not_write_to_its_source() {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root).join("tests/unit_tests/assets/collapsed_nested_databind.riv");
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, retained, None, None, None).expect("fixture imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let state_machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine zero");
    let instance = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .expect("default view-model instance");
    state_machine.with_instance_mut(|machine| machine.bind_view_model_instance(instance.clone()));

    set_number(&instance, "feed", 55.0);

    // The inactive Solo branch mounts bodyB, which itself mounts deep. Neither
    // level may publish its still-uncomputed world transforms while collapsed.
    for _ in 0..10 {
        state_machine.advance_and_apply(0.016);
    }
    check_number(&instance, "neckY", 1234.0);
    check_number(&instance, "deepY", 4321.0);
    check_number(&instance, "echo", 777.0);

    // Uncollapse: queued dirt must catch up without any explicit replay.
    set_number(&instance, "soloIndex", 1.0);
    for _ in 0..10 {
        state_machine.advance_and_apply(0.016);
    }
    check_number(&instance, "neckY", -70.0);
    check_number(&instance, "deepY", -42.0);
    check_number(&instance, "echo", 55.0);

    // Collapse again: the last published values must stay put.
    set_number(&instance, "soloIndex", 0.0);
    for _ in 0..10 {
        state_machine.advance_and_apply(0.016);
    }
    check_number(&instance, "neckY", -70.0);
    check_number(&instance, "deepY", -42.0);
    check_number(&instance, "echo", 55.0);
}
