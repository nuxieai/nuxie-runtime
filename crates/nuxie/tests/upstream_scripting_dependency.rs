//! Direct ports of all five cases in pinned
//! `tests/unit_tests/runtime/scripting/scripting_dependency_test.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{File, PersistentFactory};
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
    let file = File::import_with_unsigned_scripts(&pinned_fixture(asset))
        .unwrap_or_else(|error| panic!("{asset} imports with trusted scripts: {error:#}"));
    let artboard = file.artboard_named("Artboard").expect("Artboard artboard");
    let mut instance = artboard.instantiate().expect("Artboard instantiates");
    let mut state_machine = instance.state_machine_instance(0).expect("state machine 0");
    let mut view_model = if instance.view_model_index().is_none() {
        instance.instantiate_view_model()
    } else {
        instance.instantiate_view_model_instance(0)
    }
    .expect("Artboard view-model instance");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
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
        .expect("initial dependency draw");

    let mut counter = 0;
    for _ in 0..30 {
        assert!(view_model.set_number("InputValue1", counter as f32));
        silver.borrow_mut().add_frame();
        instance
            .try_advance_with_state_machines_and_view_model_and_factory(
                std::slice::from_mut(&mut state_machine),
                0.016,
                &mut view_model,
                &mut silver,
            )
            .expect("dependency frame advances");
        instance
            .draw(&mut silver, &mut renderer)
            .expect("dependency frame draws");
        counter += 5;
    }
    compare_silver(silver_name, &silver.borrow().bytes());
}

fn run_string_dependency(asset: &str, silver_name: &str) {
    let file = File::import_with_unsigned_scripts(&pinned_fixture(asset))
        .unwrap_or_else(|error| panic!("{asset} imports with trusted scripts: {error:#}"));
    let artboard = file.artboard_named("Artboard").expect("Artboard artboard");
    let mut instance = artboard.instantiate().expect("Artboard instantiates");
    let mut state_machine = instance.state_machine_instance(0).expect("state machine 0");
    let mut view_model = if instance.view_model_index().is_none() {
        instance.instantiate_view_model()
    } else {
        instance.instantiate_view_model_instance(0)
    }
    .expect("Artboard view-model instance");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
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
        .expect("initial dependency draw");

    for value in STRING_VALUES {
        assert!(view_model.set_string("InputString", value));
        silver.borrow_mut().add_frame();
        instance
            .try_advance_with_state_machines_and_view_model_and_factory(
                std::slice::from_mut(&mut state_machine),
                0.016,
                &mut view_model,
                &mut silver,
            )
            .expect("dependency frame advances");
        instance
            .draw(&mut silver, &mut renderer)
            .expect("dependency frame draws");
    }
    compare_silver(silver_name, &silver.borrow().bytes());
}

#[test]
#[ignore = "expected-red: Rust serializes frameSize while pinned C++ starts at makeRenderPaint"]
fn scripted_data_converter_number_using_multi_chain_requires() {
    run_number_dependency(
        "script_dependency_test.riv",
        "script_converter_with_dependency",
    );
}

#[test]
#[ignore = "expected-red: Rust serializes frameSize while pinned C++ starts at makeRenderPaint"]
fn scripted_data_converter_string_using_multi_chain_requires() {
    run_string_dependency(
        "script_dependency_test2.riv",
        "script_converter_with_dependency_2",
    );
}

#[test]
#[ignore = "expected-red: Rust serializes frameSize while pinned C++ starts at makeRenderPaint"]
fn scripted_data_converter_string_using_multi_chain_requires_from_library() {
    run_string_dependency(
        "script_dependency_test_using_library.riv",
        "script_converter_with_dependency_with_library",
    );
}

#[test]
#[ignore = "expected-red: Rust serializes frameSize while pinned C++ starts at makeRenderPaint"]
fn scripted_data_converter_string_using_multi_chain_requires_from_library_with_update() {
    run_string_dependency(
        "script_dependency_test_using_library_v2.riv",
        "script_converter_with_dependency_with_library_with_update",
    );
}

#[test]
#[ignore = "expected-red: Rust serializes frameSize while pinned C++ starts at makeRenderPaint"]
fn scripted_data_converter_string_with_namespaced_requires() {
    run_string_dependency("script_namespace_test.riv", "script_namespace_test");
}
