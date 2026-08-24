//! Frozen-render ports from pinned
//! `tests/unit_tests/runtime/scripting/scripted_path_test.cpp`.
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

fn run_silver(asset: &str, artboard_name: &str, silver_name: &str, frames_per_iteration: usize) {
    let file = File::import_with_unsigned_scripts(&pinned_fixture(asset))
        .unwrap_or_else(|error| panic!("{asset} imports with trusted scripts: {error:#}"));
    let artboard = file
        .artboard_named(artboard_name)
        .unwrap_or_else(|| panic!("{artboard_name} artboard"));
    let mut artboard = artboard
        .instantiate()
        .unwrap_or_else(|error| panic!("{artboard_name} instantiates: {error:#}"));
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");

    artboard
        .try_advance_with_state_machine_and_factory(&mut state_machine, 0.1, &mut silver)
        .expect("initial scripted path advance");
    let mut renderer = silver.borrow().make_renderer();
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("initial scripted path draw");

    for _ in 0..60 {
        for _ in 0..frames_per_iteration {
            silver.borrow_mut().add_frame();
        }
        artboard
            .try_advance_with_state_machine_and_factory(&mut state_machine, 0.016, &mut silver)
            .expect("scripted path frame advances");
        artboard
            .draw(&mut silver, &mut renderer)
            .expect("scripted path frame draws");
    }

    compare_silver(silver_name, &silver.borrow().bytes());
}

#[test]
#[ignore = "expected-red: Rust serializes frameSize while pinned C++ starts at makeRenderPaint"]
fn path_drawing_examples() {
    run_silver("script_paths_test.riv", "PathsScript", "script_paths", 1);
}

#[test]
#[ignore = "expected-red: path updates fail and Rust serializes frameSize before makeRenderPaint"]
fn path_effects_examples() {
    run_silver(
        "script_path_effects_test.riv",
        "PathEffects",
        "script_path_effects",
        2,
    );
}

#[test]
#[ignore = "expected-red: Rust serializes frameSize while pinned C++ starts at makeRenderPaint"]
fn paths_with_opacity_applied() {
    run_silver(
        "script_paths_opacity_test.riv",
        "Artboard",
        "script_path_opacity",
        1,
    );
}

#[test]
#[ignore = "expected-red: Node Script 1 init indexes nil instance"]
fn access_paint_and_path_data() {
    let file = File::import_with_unsigned_scripts(&pinned_fixture("scripted_as_path.riv"))
        .expect("scripted_as_path.riv imports with trusted scripts");
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
        .expect("scripted-as-path frame advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("scripted-as-path frame draws");

    compare_silver("scripted_as_path", &silver.borrow().bytes());
}
