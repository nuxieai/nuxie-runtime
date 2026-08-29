//! Frozen-render ports from pinned
//! `tests/unit_tests/runtime/scripting/scripted_path_test.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{
    File, FileImportLimits, PersistentFactory, ScriptExecutionLimits, ViewModelInstanceRuntime,
    import_unsigned_scripted,
};
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
        .with_file(|file| file.artboard_named(artboard_name))
        .unwrap_or_else(|| panic!("{artboard_name} artboard"));
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let state_machine = artboard.state_machine_at(0).expect("state machine 0");

    state_machine.advance_and_apply(0.1);
    let mut renderer = silver.borrow().make_renderer();
    artboard.draw(&mut renderer);

    for _ in 0..60 {
        for _ in 0..frames_per_iteration {
            silver.borrow_mut().add_frame();
        }
        state_machine.advance_and_apply(0.016);
        artboard.draw(&mut renderer);
    }

    compare_silver(silver_name, &silver.borrow().bytes());
}

#[test]
fn path_drawing_examples() {
    run_silver("script_paths_test.riv", "PathsScript", "script_paths", 1);
}

#[test]
#[ignore = "expected-red: script_path_effects differs at frame 1 op 86 (expected save, got frame)"]
fn path_effects_examples() {
    run_silver(
        "script_path_effects_test.riv",
        "PathEffects",
        "script_path_effects",
        2,
    );
}

#[test]
fn paths_with_opacity_applied() {
    run_silver(
        "script_paths_opacity_test.riv",
        "Artboard",
        "script_path_opacity",
        1,
    );
}

#[test]
#[ignore = "expected-red: scripted_as_path differs at frame 0 op 31 (expected save, got makeRenderPaint)"]
fn access_paint_and_path_data() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let scripted = import_unsigned_scripted(
        &pinned_fixture("scripted_as_path.riv"),
        &mut silver,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("scripted_as_path.riv imports with trusted scripts");
    let file = scripted.native_file();
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let state_machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                .or_else(|| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("default view-model instance");
    state_machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(view_model.instance()));
    artboard.bind_view_model_instance(Some(view_model.instance()));
    let mut renderer = silver.borrow().make_renderer();

    state_machine.advance_and_apply(0.016);
    artboard.draw(&mut renderer);

    compare_silver("scripted_as_path", &silver.borrow().bytes());
}
