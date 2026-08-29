//! Silver-test ports from pinned
//! `tests/unit_tests/runtime/scripting/scripting_context_test.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{
    FileImportLimits, PersistentFactory, RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle, RuntimeViewModelInstanceHandle, ScriptExecutionLimits,
    ViewModelInstanceRuntime, import_unsigned_scripted,
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

fn authored_or_fresh_view_model(
    file: &nuxie::RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
) -> RuntimeViewModelInstanceHandle {
    let model_id = artboard.with_artboard(|artboard| artboard.view_model_id());
    file.with_file(|file| {
        if model_id == u32::MAX {
            file.create_view_model_instance_for_artboard(artboard.core_handle())
        } else {
            file.create_view_model_instance_at(model_id as usize, 0)
        }
    })
    .map(ViewModelInstanceRuntime::new)
    .map(ViewModelInstanceRuntime::into_handle)
    .expect("artboard view-model instance")
}

fn default_view_model(
    file: &nuxie::RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
) -> RuntimeViewModelInstanceHandle {
    file.with_file(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    })
    .map(ViewModelInstanceRuntime::new)
    .map(ViewModelInstanceRuntime::into_handle)
    .expect("default view-model instance")
}

fn bind_view_model(
    machine: &RuntimeStateMachineInstanceHandle,
    view_model: &RuntimeViewModelInstanceHandle,
) {
    machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(view_model.instance());
    });
}

#[test]
fn script_has_access_to_user_created_view_models_via_data() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let scripted = import_unsigned_scripted(
        &pinned_fixture("script_create_viewmodel_instance.riv"),
        &mut silver,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("script_create_viewmodel_instance.riv imports with trusted scripts");
    let file = scripted.native_file();
    let artboard = file
        .with_file(|file| file.artboard_named("main"))
        .expect("main artboard");
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = authored_or_fresh_view_model(file, &artboard);
    bind_view_model(&machine, &view_model);
    machine.advance_and_apply(0.1);
    let mut renderer = silver.borrow().make_renderer();
    artboard.draw(&mut renderer);

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
        let trigger_property = view_model
            .property_trigger(trigger)
            .unwrap_or_else(|| panic!("trigger {trigger}"));
        for _ in 0..count {
            trigger_property.trigger();
        }
        machine.advance_and_apply(0.1);
        artboard.draw(&mut renderer);
    }

    compare_silver("script_create_viewmodel_instance", &silver.borrow().bytes());
}

#[test]
fn script_has_access_to_the_data_bound_view_model() {
    two_frame_context_silver(
        "viewmodel_from_context.riv",
        "main",
        0.1,
        "viewmodel_from_context",
        false,
    );
}

#[test]
fn script_has_access_to_the_data_root_view_model() {
    two_frame_context_silver(
        "scripting_root_viewmodel.riv",
        "parent",
        0.1,
        "scripting_root_viewmodel",
        true,
    );
}

fn two_frame_context_silver(
    fixture: &str,
    artboard_name: &str,
    dt: f32,
    silver_name: &str,
    uses_default_view_model: bool,
) {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let scripted = import_unsigned_scripted(
        &pinned_fixture(fixture),
        &mut silver,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .unwrap_or_else(|error| panic!("{fixture} imports with trusted scripts: {error:#}"));
    let file = scripted.native_file();
    let artboard = file
        .with_file(|file| file.artboard_named(artboard_name))
        .unwrap_or_else(|| panic!("{artboard_name} artboard"));
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = if uses_default_view_model {
        default_view_model(file, &artboard)
    } else {
        authored_or_fresh_view_model(file, &artboard)
    };
    bind_view_model(&machine, &view_model);

    machine.advance_and_apply(dt);
    let mut renderer = silver.borrow().make_renderer();
    artboard.draw(&mut renderer);
    silver.borrow_mut().add_frame();
    machine.advance_and_apply(dt);
    artboard.draw(&mut renderer);
    compare_silver(silver_name, &silver.borrow().bytes());
}

#[test]
fn expose_data_context_to_scripts_through_context() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let scripted = import_unsigned_scripted(
        &pinned_fixture("scripted_data_context.riv"),
        &mut silver,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("scripted_data_context.riv imports with trusted scripts");
    let file = scripted.native_file();
    let artboard = file
        .with_file(|file| file.artboard_named("Main"))
        .expect("Main artboard");
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = default_view_model(file, &artboard);
    bind_view_model(&machine, &view_model);
    let mut renderer = silver.borrow().make_renderer();
    machine.advance_and_apply(0.016);
    artboard.draw(&mut renderer);
    compare_silver("scripted_data_context", &silver.borrow().bytes());
}

#[test]
fn provide_data_context_and_view_model_instance_to_artboard() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let scripted = import_unsigned_scripted(
        &pinned_fixture("viewmodel_instance_to_artboard.riv"),
        &mut silver,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("viewmodel_instance_to_artboard.riv imports with trusted scripts");
    let file = scripted.native_file();
    let artboard = file
        .with_file(|file| file.artboard_default())
        .expect("default artboard");
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = default_view_model(file, &artboard);
    bind_view_model(&machine, &view_model);
    let mut renderer = silver.borrow().make_renderer();
    machine.advance_and_apply(0.016);
    artboard.draw(&mut renderer);

    let frames = (1.0_f32 / 0.016_f32) as i32;
    for _ in 0..frames {
        silver.borrow_mut().add_frame();
        machine.advance_and_apply(0.016);
        artboard.draw(&mut renderer);
    }
    compare_silver("viewmodel_instance_to_artboard", &silver.borrow().bytes());
}
