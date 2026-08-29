//! Direct port of pinned
//! `tests/unit_tests/runtime/scripting/scripting_transition_condition_test.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{
    File, FileImportLimits, PersistentFactory, ScriptExecutionLimits, ViewModelInstanceRuntime,
    import_unsigned_scripted,
};
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
fn scripted_transition_condition() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let scripted = import_unsigned_scripted(
        &pinned_bytes("assets", "scripted_transition_condition.riv"),
        &mut silver,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("scripted_transition_condition.riv imports with trusted scripts");
    let file = scripted.native_file();
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let state_machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                .or_else(|| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("artboard ViewModel instance");
    state_machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(view_model.instance()));
    artboard.bind_view_model_instance(Some(view_model.instance()));
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);

    state_machine.advance_and_apply(0.1);
    let mut renderer = silver.borrow().make_renderer();
    artboard.draw(&mut renderer);

    silver.borrow_mut().add_frame();
    let timeline = view_model
        .property_boolean("timelineBool")
        .expect("timelineBool property");
    timeline.set_value(true);
    assert!(timeline.value());
    state_machine.advance_and_apply(0.016);
    artboard.draw(&mut renderer);

    silver.borrow_mut().add_frame();
    let any_state = view_model
        .property_boolean("anyStateBool")
        .expect("anyStateBool property");
    any_state.set_value(true);
    assert!(any_state.value());
    state_machine.advance_and_apply(0.016);
    artboard.draw(&mut renderer);

    let actual = parse_sriv(&silver.borrow().bytes()).expect("valid Rust SRIV stream");
    let expected = parse_sriv(&pinned_bytes(
        "silvers",
        "scripted_transition_condition.sriv",
    ))
    .expect("valid pinned SRIV stream");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("scripted_transition_condition differs: {difference}"));
}
