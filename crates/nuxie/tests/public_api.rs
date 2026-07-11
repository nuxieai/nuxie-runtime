// Test code is deliberately outside the panic-freedom lint gate (the crate
// lints table denies these for src/; unwrap/indexing are fine in tests).
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

use nuxie::{File, RecordingFactory, StateMachineInputKind};
use std::path::PathBuf;

fn repo_fixture(relative: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    std::fs::read(&path).expect("read repo fixture")
}

fn external_fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets")
    .join(name);
    std::fs::read(&path).expect("read external fixture")
}

/// Instantiate artboard 0, optionally set a view-model property before binding,
/// then advance and draw, returning the recorded stream.
fn render_with_view_model(
    bytes: &[u8],
    set: impl FnOnce(&mut nuxie::ViewModelInstance) -> bool,
) -> String {
    let file = File::import(bytes).expect("import file");
    let mut instance = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("instantiate artboard");
    let mut view_model = instance
        .instantiate_view_model()
        .expect("artboard has a view model");
    let _ = set(&mut view_model);
    instance.bind_view_model(&view_model);
    instance.advance(0.0);

    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    instance
        .draw(&mut factory, &mut renderer)
        .expect("draw artboard");
    factory.stream()
}

#[test]
fn public_api_imports_lists_instantiates_and_draws() {
    let fixture = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets/shapetest.riv");
    let bytes = std::fs::read(&fixture).expect("read fixture");
    let file = File::import(&bytes).expect("import file");

    assert!(file.artboard_count() >= 1);
    let names = file
        .artboards()
        .map(|artboard| artboard.name().unwrap_or("<unnamed>").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(names.len(), file.artboard_count());

    let artboard = file.default_artboard().expect("default artboard");
    let mut instance = artboard.instantiate().expect("instantiate artboard");
    instance.advance(0.0);

    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    instance
        .draw(&mut factory, &mut renderer)
        .expect("draw artboard");

    let stream = factory.stream();
    assert!(stream.contains("rive-golden-stream-v1"));
    assert!(stream.contains("drawPath"));
}

#[test]
fn public_api_drives_default_state_machine_and_inputs() {
    let bytes = repo_fixture("fixtures/animation/smi_test.riv");
    let file = File::import(&bytes).expect("import file");

    let artboard = file
        .artboard_named("artboard to nest")
        .expect("artboard to nest");
    assert!(artboard.state_machine_count() >= 1);
    assert_eq!(artboard.state_machine_name(0), Some("State Machine 1"));
    assert_eq!(artboard.state_machine_name(99), None);
    assert_eq!(artboard.default_state_machine_index(), Some(0));

    let mut instance = artboard.instantiate().expect("instantiate artboard");
    let mut state_machine = instance
        .default_state_machine_instance()
        .expect("default state machine instance");

    let bool_index = state_machine.input_index_named("bool").expect("bool input");
    assert_eq!(
        state_machine.input(bool_index).map(|input| input.kind()),
        Some(StateMachineInputKind::Bool)
    );
    assert!(state_machine.set_bool(bool_index, true));

    let number_index = state_machine.input_index_named("num").expect("num input");
    assert!(state_machine.set_number(number_index, 42.0));

    let trigger_index = state_machine.input_index_named("trig").expect("trig input");
    assert!(state_machine.fire_trigger(trigger_index));

    // Wrong-kind writes are rejected.
    assert!(!state_machine.set_number(bool_index, 1.0));

    instance.advance_with_state_machine(&mut state_machine, 0.016);
    instance.advance_with_state_machine(&mut state_machine, 0.016);
}

#[test]
fn public_api_view_model_number_set_changes_stream() {
    // `data_binding_test_2.riv` artboard 0 binds a shape to the view model's
    // `num` number property, so setting it visibly changes the draw stream.
    let bytes = external_fixture("data_binding_test_2.riv");

    let baseline = render_with_view_model(&bytes, |_| false);
    let mutated = render_with_view_model(&bytes, |view_model| {
        assert!(
            view_model.set_number("num", 137.0),
            "num is a settable number property"
        );
        true
    });

    assert!(baseline.contains("rive-golden-stream-v1"));
    assert_ne!(
        baseline, mutated,
        "setting a bound number property must change the draw stream"
    );
    // Setting a property that does not exist reports no change.
    let mut probe = File::import(&bytes)
        .unwrap()
        .default_artboard()
        .unwrap()
        .instantiate()
        .unwrap()
        .instantiate_view_model()
        .unwrap();
    assert!(!probe.set_number("does-not-exist", 1.0));
    assert!(!probe.set_bool("num", true), "wrong-kind write is rejected");
}

#[test]
fn public_api_view_model_string_set_changes_stream() {
    // `relative_data_binding.riv` artboard 0 binds text to the view model's
    // `str` string property.
    let bytes = external_fixture("relative_data_binding.riv");

    let baseline = render_with_view_model(&bytes, |_| false);
    let mutated = render_with_view_model(&bytes, |view_model| {
        assert!(
            view_model.set_string("str", "nuxie view model string"),
            "str is a settable string property"
        );
        true
    });

    assert_ne!(
        baseline, mutated,
        "setting a bound string property must change the draw stream"
    );
}

#[test]
fn public_api_view_model_instance_selection_and_missing() {
    // Artboards without a view model yield no context.
    let bytes = repo_fixture("fixtures/animation/smi_test.riv");
    let file = File::import(&bytes).expect("import file");
    let instance = file
        .default_artboard()
        .unwrap()
        .instantiate()
        .expect("instantiate artboard");
    assert!(instance.view_model_index().is_none());
    assert!(instance.instantiate_view_model().is_none());
    assert!(instance.instantiate_view_model_instance(0).is_none());

    // A databind fixture exposes a view model and a source instance at index 0;
    // an out-of-range instance index yields nothing.
    let bytes = external_fixture("data_binding_test_2.riv");
    let file = File::import(&bytes).expect("import file");
    let instance = file
        .default_artboard()
        .unwrap()
        .instantiate()
        .expect("instantiate artboard");
    assert!(instance.view_model_index().is_some());
    assert!(instance.instantiate_view_model().is_some());
    assert!(instance.instantiate_view_model_instance(0).is_some());
    assert!(instance.instantiate_view_model_instance(9_999).is_none());
}
