//! Frozen-render ports from pinned
//! `tests/unit_tests/runtime/scripting/scripting_converter_test.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{File, PersistentFactory};
use nuxie_render_api::SerializingFactory;
use silver_corpus::{compare_sriv, parse_sriv};

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

#[test]
#[ignore = "expected-red: Rust serializes frameSize while pinned C++ starts at makeRenderPaint"]
fn scripted_string_converter() {
    let file =
        File::import_with_unsigned_scripts(&pinned_fixture("script_string_converter_test.riv"))
            .expect("script_string_converter_test.riv imports with trusted scripts");
    let artboard = file
        .artboard_named("Converter")
        .expect("Converter artboard");
    let mut artboard = artboard.instantiate().expect("Converter instantiates");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = if artboard.view_model_index().is_none() {
        artboard.instantiate_view_model()
    } else {
        artboard.instantiate_view_model_instance(0)
    }
    .expect("Converter view-model instance");

    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("initial converter advance");
    let mut renderer = silver.borrow().make_renderer();
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("initial converter draw");

    assert!(view_model.set_string("Field1", "H#e%l&l*o"));
    silver.borrow_mut().add_frame();
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
            &mut silver,
        )
        .expect("Field1 frame advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("Field1 frame draws");

    assert!(view_model.set_string("Field2", "____one two three___"));
    silver.borrow_mut().add_frame();
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
            &mut silver,
        )
        .expect("Field2 frame advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("Field2 frame draws");

    assert!(view_model.set_string("Field3", "  **This uses a string converter@@. "));
    silver.borrow_mut().add_frame();
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
            &mut silver,
        )
        .expect("Field3 frame advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("Field3 frame draws");

    assert!(view_model.set_string("Field4", "It strips special characters like *&^%$#@!)()",));
    silver.borrow_mut().add_frame();
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
            &mut silver,
        )
        .expect("Field4 frame advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("Field4 frame draws");

    compare_silver("script_string_converter", &silver.borrow().bytes());
}

#[test]
#[ignore = "expected-red: Rust serializes frameSize while pinned C++ starts at makeRenderPaint"]
fn data_converter_with_bound_inputs_in_artboard_and_state_machine() {
    let file = File::import_with_unsigned_scripts(&pinned_fixture(
        "scripted_data_converter_bound_input.riv",
    ))
    .expect("scripted_data_converter_bound_input.riv imports with trusted scripts");
    let artboard = file.default_artboard().expect("default artboard");
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = artboard
        .instantiate_default_view_model_instance()
        .expect("default view-model instance");

    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("initial bound-converter advance");
    let mut renderer = silver.borrow().make_renderer();
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("initial bound-converter draw");

    silver.borrow_mut().add_frame();
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("second bound-converter advance");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("second bound-converter draw");

    compare_silver(
        "scripted_data_converter_bound_input",
        &silver.borrow().bytes(),
    );
}
