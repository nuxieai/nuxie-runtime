use rive::{File, RecordingFactory, StateMachineInputKind};
use std::path::PathBuf;

fn repo_fixture(relative: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    std::fs::read(&path).expect("read repo fixture")
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
