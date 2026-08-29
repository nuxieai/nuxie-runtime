//! Direct ports of all three cases in pinned
//! `tests/unit_tests/runtime/scripting/scripting_listener_action_test.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{
    FileImportLimits, PersistentFactory, RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle, RuntimeViewModelInstanceHandle, ScriptExecutionLimits,
    ScriptedFile, Vec2D, ViewModelInstanceRuntime, import_unsigned_scripted,
    runtime::input::focusable::{Key, KeyModifiers},
};
use nuxie_render_api::{NullFactory, SerializingFactory};
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

fn default_artboard(file: &ScriptedFile) -> RuntimeArtboardInstanceHandle {
    file.native_file()
        .with_file(|file| file.artboard_default())
        .expect("default artboard")
}

fn authored_or_fresh_view_model(
    file: &ScriptedFile,
    artboard: &RuntimeArtboardInstanceHandle,
) -> RuntimeViewModelInstanceHandle {
    let view_model_id = artboard.with_artboard(|artboard| artboard.view_model_id());
    file.native_file()
        .with_file(|file| {
            if view_model_id == u32::MAX {
                file.create_view_model_instance_for_artboard(artboard.core_handle())
            } else {
                file.create_view_model_instance_at(view_model_id as usize, 0)
            }
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("artboard view-model instance")
}

fn fresh_view_model(
    file: &ScriptedFile,
    artboard: &RuntimeArtboardInstanceHandle,
) -> RuntimeViewModelInstanceHandle {
    file.native_file()
        .with_file(|file| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("fresh artboard view-model instance")
}

fn bind_view_model(
    machine: &RuntimeStateMachineInstanceHandle,
    view_model: &RuntimeViewModelInstanceHandle,
) {
    machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(view_model.instance());
    });
}

fn pointer_down(machine: &RuntimeStateMachineInstanceHandle, x: f32, y: f32, pointer_id: i32) {
    machine.with_instance_mut(|machine| machine.pointer_down(Vec2D::new(x, y), pointer_id));
}

fn pointer_up(machine: &RuntimeStateMachineInstanceHandle, x: f32, y: f32, pointer_id: i32) {
    machine.with_instance_mut(|machine| machine.pointer_up(Vec2D::new(x, y), pointer_id));
}

#[test]
fn scripted_listener_action() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let file = import_unsigned_scripted(
        &pinned_fixture("scripted_listener_action.riv"),
        &mut silver,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("scripted_listener_action.riv imports with trusted scripts");
    let artboard = default_artboard(&file);
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = fresh_view_model(&file, &artboard);
    bind_view_model(&machine, &view_model);

    machine.advance_and_apply(0.1);
    let mut renderer = silver.borrow().make_renderer();
    artboard.draw(&mut renderer);
    silver.borrow_mut().add_frame();

    for (x, pointer_id) in [(200.0, 1), (300.0, 2), (400.0, 3)] {
        pointer_down(&machine, x, 20.0, pointer_id);
        pointer_up(&machine, x, 20.0, pointer_id);
        machine.advance_and_apply(0.016);
        artboard.draw(&mut renderer);
    }

    compare_silver("scripted_listener_action", &silver.borrow().bytes());
}

#[test]
fn listener_action_inputs() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let file = import_unsigned_scripted(
        &pinned_fixture("listener_action_inputs.riv"),
        &mut silver,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("listener_action_inputs.riv imports with trusted scripts");
    let artboard = default_artboard(&file);
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = authored_or_fresh_view_model(&file, &artboard);
    bind_view_model(&machine, &view_model);

    let mut renderer = silver.borrow().make_renderer();
    machine.advance_and_apply(0.016);
    artboard.draw(&mut renderer);
    pointer_down(&machine, width / 2.0, height / 2.0, 3);
    pointer_up(&machine, width / 2.0, height / 2.0, 3);
    silver.borrow_mut().add_frame();
    machine.advance_and_apply(0.016);
    artboard.draw(&mut renderer);

    compare_silver("listener_action_inputs", &silver.borrow().bytes());
}

fn assert_context(
    view_model: &RuntimeViewModelInstanceHandle,
    key: &str,
    pointer: &str,
    text: &str,
    x: f32,
    y: f32,
    focus: bool,
) {
    assert_eq!(
        view_model
            .property_string("keyInput")
            .expect("keyInput")
            .value(),
        key
    );
    assert_eq!(
        view_model
            .property_string("pointerType")
            .expect("pointerType")
            .value(),
        pointer
    );
    assert_eq!(
        view_model
            .property_string("stringInput")
            .expect("stringInput")
            .value(),
        text
    );
    assert_eq!(view_model.property_number("posX").expect("posX").value(), x);
    assert_eq!(view_model.property_number("posY").expect("posY").value(), y);
    assert_eq!(
        view_model
            .property_boolean("isFocus")
            .expect("isFocus")
            .value(),
        focus
    );
}

#[test]
fn listener_action_script_receives_pointer_types_and_the_data() {
    let mut factory = PersistentFactory::new(NullFactory);
    let file = import_unsigned_scripted(
        &pinned_fixture("scripted_listener_context.riv"),
        &mut factory,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("scripted_listener_context.riv imports with trusted scripts");
    let artboard = default_artboard(&file);
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = fresh_view_model(&file, &artboard);

    assert_context(&view_model, "", "", "", 0.0, 0.0, false);
    assert!(
        !view_model
            .property_boolean("eventReported")
            .unwrap()
            .value()
    );
    assert!(
        !view_model
            .property_boolean("viewModelChanged")
            .unwrap()
            .value()
    );

    bind_view_model(&machine, &view_model);
    machine.advance_and_apply(0.016);

    machine.with_instance_mut(|machine| machine.pointer_move(Vec2D::new(200.0, 210.0), 0.0, 0));
    machine.advance_and_apply(0.016);
    assert_context(&view_model, "", "pointerEnter", "", 200.0, 210.0, false);

    pointer_down(&machine, 250.0, 251.0, 0);
    pointer_up(&machine, 250.0, 251.0, 0);
    machine.advance_and_apply(0.016);
    assert_context(&view_model, "", "click", "", 250.0, 251.0, false);

    let focus_manager = machine.with_instance(|machine| machine.focus_manager());
    focus_manager.with_focus_manager_mut(|manager| manager.focus_next());
    machine.advance_and_apply(0.016);
    assert_context(&view_model, "", "click", "", 250.0, 251.0, true);

    focus_manager.with_focus_manager_mut(|manager| {
        manager.key_input(Key::A, KeyModifiers::NONE, false, false)
    });
    machine.advance_and_apply(0.016);
    assert_context(
        &view_model,
        "65, no shift, no meta, no control, no alt, phase: up",
        "click",
        "",
        250.0,
        251.0,
        true,
    );

    focus_manager.with_focus_manager_mut(|manager| manager.text_input("With text input"));
    machine.advance_and_apply(0.016);
    assert_context(
        &view_model,
        "65, no shift, no meta, no control, no alt, phase: up",
        "click",
        "With text input",
        250.0,
        251.0,
        true,
    );

    focus_manager.with_focus_manager_mut(|manager| {
        manager.key_input(
            Key::B,
            KeyModifiers::META | KeyModifiers::SHIFT,
            true,
            false,
        )
    });
    machine.advance_and_apply(0.016);
    assert_context(
        &view_model,
        "66, with shift, with meta, no control, no alt, phase: down",
        "click",
        "With text input",
        250.0,
        251.0,
        true,
    );

    focus_manager.with_focus_manager_mut(|manager| manager.focus_next());
    machine.advance_and_apply(0.016);
    assert_context(
        &view_model,
        "66, with shift, with meta, no control, no alt, phase: down",
        "click",
        "With text input",
        250.0,
        251.0,
        false,
    );

    focus_manager.with_focus_manager_mut(|manager| {
        manager.key_input(Key::A, KeyModifiers::NONE, false, false)
    });
    machine.advance_and_apply(0.016);
    assert_context(
        &view_model,
        "66, with shift, with meta, no control, no alt, phase: down",
        "click",
        "With text input",
        250.0,
        251.0,
        false,
    );
    assert!(
        view_model
            .property_boolean("eventReported")
            .unwrap()
            .value()
    );
    assert!(
        view_model
            .property_boolean("viewModelChanged")
            .unwrap()
            .value()
    );
}
