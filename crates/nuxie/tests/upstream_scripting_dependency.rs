//! Direct ports of all five cases in pinned
//! `tests/unit_tests/runtime/scripting/scripting_dependency_test.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{
    FileImportLimits, PersistentFactory, ScriptExecutionLimits, ViewModelInstanceRuntime,
    import_unsigned_scripted,
};
use nuxie_render_api::SerializingFactory;
use silver_corpus::{compare_sriv, parse_sriv};

const STRING_VALUES: [&str; 5] = [
    "Hello world!",
    "1,2,3",
    "rive scripting",
    "testing testing testing",
    "Script Data Converter",
];

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

fn pinned_silver(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let silver = PathBuf::from(root)
        .join("tests/unit_tests/silvers")
        .join(format!("{name}.sriv"));
    std::fs::read(&silver)
        .unwrap_or_else(|error| panic!("read pinned silver {}: {error}", silver.display()))
}

fn compare_silver(name: &str, actual: &[u8]) {
    let actual = parse_sriv(actual).expect("valid Rust SRIV stream");
    let expected = parse_sriv(&pinned_silver(name)).expect("valid pinned SRIV stream");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("{name} differs: {difference}"));
}

fn run_number_dependency(asset: &str, silver_name: &str) {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let scripted = import_unsigned_scripted(
        &pinned_fixture(asset),
        &mut silver,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .unwrap_or_else(|error| panic!("{asset} imports with trusted scripts: {error:#}"));
    let file = scripted.native_file();
    let artboard = file
        .with_file(|file| file.artboard_named("Artboard"))
        .expect("Artboard artboard");
    let state_machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                .or_else(|| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("Artboard view-model instance");
    state_machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(view_model.instance()));
    artboard.bind_view_model_instance(Some(view_model.instance()));
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);

    state_machine.advance_and_apply(0.1);
    let mut renderer = silver.borrow().make_renderer();
    artboard.draw(&mut renderer);

    let mut counter = 0;
    for _ in 0..30 {
        let input = view_model
            .property_number("InputValue1")
            .expect("InputValue1 number");
        input.set_value(counter as f32);
        assert_eq!(input.value(), counter as f32);
        silver.borrow_mut().add_frame();
        state_machine.advance_and_apply(0.016);
        artboard.draw(&mut renderer);
        counter += 5;
    }
    compare_silver(silver_name, &silver.borrow().bytes());
}

fn run_string_dependency(asset: &str, silver_name: &str) {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let scripted = import_unsigned_scripted(
        &pinned_fixture(asset),
        &mut silver,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .unwrap_or_else(|error| panic!("{asset} imports with trusted scripts: {error:#}"));
    let file = scripted.native_file();
    let artboard = file
        .with_file(|file| file.artboard_named("Artboard"))
        .expect("Artboard artboard");
    let state_machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                .or_else(|| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("Artboard view-model instance");
    state_machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(view_model.instance()));
    artboard.bind_view_model_instance(Some(view_model.instance()));
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);

    state_machine.advance_and_apply(0.1);
    let mut renderer = silver.borrow().make_renderer();
    artboard.draw(&mut renderer);

    for value in STRING_VALUES {
        let input = view_model
            .property_string("InputString")
            .expect("InputString string");
        input.set_value(value);
        assert_eq!(input.value(), value);
        silver.borrow_mut().add_frame();
        state_machine.advance_and_apply(0.016);
        artboard.draw(&mut renderer);
    }
    compare_silver(silver_name, &silver.borrow().bytes());
}

#[test]
fn scripted_data_converter_number_using_multi_chain_requires() {
    run_number_dependency(
        "script_dependency_test.riv",
        "script_converter_with_dependency",
    );
}

#[test]
fn scripted_data_converter_string_using_multi_chain_requires() {
    run_string_dependency(
        "script_dependency_test2.riv",
        "script_converter_with_dependency_2",
    );
}

#[test]
fn scripted_data_converter_string_using_multi_chain_requires_from_library() {
    run_string_dependency(
        "script_dependency_test_using_library.riv",
        "script_converter_with_dependency_with_library",
    );
}

#[test]
fn scripted_data_converter_string_using_multi_chain_requires_from_library_with_update() {
    run_string_dependency(
        "script_dependency_test_using_library_v2.riv",
        "script_converter_with_dependency_with_library_with_update",
    );
}

#[test]
fn scripted_data_converter_string_with_namespaced_requires() {
    run_string_dependency("script_namespace_test.riv", "script_namespace_test");
}
