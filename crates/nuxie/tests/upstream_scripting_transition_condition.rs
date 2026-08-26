//! Direct port of pinned
//! `tests/unit_tests/runtime/scripting/scripting_transition_condition_test.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{File, PersistentFactory};
use nuxie_render_api::SerializingFactory;
use silver_corpus::{compare_sriv, parse_sriv};

fn pinned_bytes(directory: &str, name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests")
        .join(directory)
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read pinned {}: {error}", path.display()))
}

#[test]
#[ignore = "expected-red: frame 1 operation 30 expects color but live scripted transition emits save"]
fn scripted_transition_condition() {
    let file = File::import_with_unsigned_scripts(&pinned_bytes(
        "assets",
        "scripted_transition_condition.riv",
    ))
    .expect("scripted_transition_condition.riv imports with trusted scripts");
    let artboard = file.default_artboard().expect("default artboard");
    let mut instance = artboard
        .instantiate()
        .expect("default artboard instantiates");
    let mut state_machine = instance.state_machine_instance(0).expect("state machine 0");
    let mut view_model = instance
        .instantiate_view_model()
        .expect("artboard ViewModel instance");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    instance
        .initialize_renderer(&mut silver)
        .expect("renderer and File script VM initialize");
    let (width, height) = instance.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);

    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("initial scripted advance");
    let mut renderer = silver.borrow().make_renderer();
    instance
        .draw(&mut silver, &mut renderer)
        .expect("initial draw");

    silver.borrow_mut().add_frame();
    assert!(view_model.set_bool("timelineBool", true));
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
            &mut silver,
        )
        .expect("timeline transition advance");
    instance
        .draw(&mut silver, &mut renderer)
        .expect("timeline transition draw");

    silver.borrow_mut().add_frame();
    assert!(view_model.set_bool("anyStateBool", true));
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
            &mut silver,
        )
        .expect("any-state transition advance");
    instance
        .draw(&mut silver, &mut renderer)
        .expect("any-state transition draw");

    let actual = parse_sriv(&silver.borrow().bytes()).expect("valid Rust SRIV stream");
    let expected = parse_sriv(&pinned_bytes(
        "silvers",
        "scripted_transition_condition.sriv",
    ))
    .expect("valid pinned SRIV stream");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("scripted_transition_condition differs: {difference}"));
}
