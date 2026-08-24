//! Silver-test ports from pinned
//! `tests/unit_tests/runtime/scripting/scripting_context_test.cpp`.
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
#[ignore = "expected-red: exact scripted Data list silver awaits renderer stream parity"]
fn script_has_access_to_user_created_view_models_via_data() {
    let file =
        File::import_with_unsigned_scripts(&pinned_fixture("script_create_viewmodel_instance.riv"))
            .expect("script_create_viewmodel_instance.riv imports with trusted scripts");
    let artboard = file.artboard_named("main").expect("main artboard");
    let mut artboard = artboard.instantiate().expect("main artboard instantiates");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = if artboard.view_model_index().is_none() {
        artboard.instantiate_view_model()
    } else {
        artboard.instantiate_view_model_instance(0)
    }
    .expect("main view-model instance");
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("initial scripted Data advance");
    let mut renderer = silver.borrow().make_renderer();
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("initial scripted Data draw");

    for (trigger, count) in [
        ("newButton/onClick", 1),
        ("newAtButton/onClick", 1),
        ("swapButton/onClick", 1),
        ("shiftButton/onClick", 1),
        ("popButton/onClick", 1),
        ("popButton/onClick", 4),
        ("newButton/onClick", 2),
    ] {
        silver.borrow_mut().add_frame();
        for _ in 0..count {
            assert!(view_model.fire_trigger(trigger), "trigger {trigger}");
        }
        artboard
            .try_advance_with_state_machines_and_view_model_and_factory(
                std::slice::from_mut(&mut state_machine),
                0.1,
                &mut view_model,
                &mut silver,
            )
            .expect("scripted Data frame advances");
        artboard
            .draw(&mut silver, &mut renderer)
            .expect("scripted Data frame draws");
    }

    compare_silver("script_create_viewmodel_instance", &silver.borrow().bytes());
}

#[test]
#[ignore = "expected-red: exact context-bound view-model silver awaits renderer stream parity"]
fn script_has_access_to_the_data_bound_view_model() {
    two_frame_context_silver(
        "viewmodel_from_context.riv",
        "main",
        0.1,
        "viewmodel_from_context",
    );
}

#[test]
#[ignore = "expected-red: exact root-view-model silver awaits renderer stream parity"]
fn script_has_access_to_the_data_root_view_model() {
    two_frame_context_silver(
        "scripting_root_viewmodel.riv",
        "parent",
        0.1,
        "scripting_root_viewmodel",
    );
}

fn two_frame_context_silver(fixture: &str, artboard_name: &str, dt: f32, silver_name: &str) {
    let file = File::import_with_unsigned_scripts(&pinned_fixture(fixture))
        .unwrap_or_else(|error| panic!("{fixture} imports with trusted scripts: {error}"));
    let artboard = file
        .artboard_named(artboard_name)
        .unwrap_or_else(|| panic!("{artboard_name} artboard"));
    let mut artboard = artboard
        .instantiate()
        .unwrap_or_else(|_| panic!("{artboard_name} artboard instantiates"));
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = if artboard.view_model_index().is_none() {
        artboard.instantiate_view_model()
    } else {
        artboard.instantiate_view_model_instance(0)
    }
    .expect("bound view-model instance");
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            dt,
            &mut view_model,
            &mut silver,
        )
        .expect("initial context frame advances");
    let mut renderer = silver.borrow().make_renderer();
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("initial context frame draws");
    silver.borrow_mut().add_frame();
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            dt,
            &mut view_model,
            &mut silver,
        )
        .expect("second context frame advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("second context frame draws");
    compare_silver(silver_name, &silver.borrow().bytes());
}

#[test]
#[ignore = "expected-red: exact scripted data-context silver awaits renderer stream parity"]
fn expose_data_context_to_scripts_through_context() {
    let file = File::import_with_unsigned_scripts(&pinned_fixture("scripted_data_context.riv"))
        .expect("scripted_data_context.riv imports with trusted scripts");
    let artboard = file.artboard_named("Main").expect("Main artboard");
    let mut artboard = artboard.instantiate().expect("Main artboard instantiates");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = artboard
        .instantiate_default_view_model_instance()
        .expect("default view-model instance");
    let mut renderer = silver.borrow().make_renderer();
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
            &mut silver,
        )
        .expect("data-context frame advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("data-context frame draws");
    compare_silver("scripted_data_context", &silver.borrow().bytes());
}

#[test]
#[ignore = "expected-red: exact artboard data-context silver awaits renderer stream parity"]
fn provide_data_context_and_view_model_instance_to_artboard() {
    let file =
        File::import_with_unsigned_scripts(&pinned_fixture("viewmodel_instance_to_artboard.riv"))
            .expect("viewmodel_instance_to_artboard.riv imports with trusted scripts");
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
    let mut renderer = silver.borrow().make_renderer();
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
            &mut silver,
        )
        .expect("initial artboard-context frame advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("initial artboard-context frame draws");

    let frames = (1.0_f32 / 0.016_f32) as i32;
    for _ in 0..frames {
        silver.borrow_mut().add_frame();
        artboard
            .try_advance_with_state_machines_and_view_model_and_factory(
                std::slice::from_mut(&mut state_machine),
                0.016,
                &mut view_model,
                &mut silver,
            )
            .expect("artboard-context frame advances");
        artboard
            .draw(&mut silver, &mut renderer)
            .expect("artboard-context frame draws");
    }
    compare_silver("viewmodel_instance_to_artboard", &silver.borrow().bytes());
}
