//! Direct ports of all three cases in pinned
//! `tests/unit_tests/runtime/scripting/scripting_listener_action_test.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{File, PersistentFactory, StateMachineInstance, ViewModelInstance};
use nuxie_render_api::SerializingFactory;
use silver_corpus::{compare_sriv, parse_sriv};

trait UpstreamAdvance {
    fn try_advance_with_state_machines_and_view_model(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        view_model: &mut ViewModelInstance,
    ) -> Result<bool, &'static str>;
}

impl UpstreamAdvance for nuxie::ArtboardInstance<'_> {
    fn try_advance_with_state_machines_and_view_model(
        &mut self,
        state_machines: &mut [StateMachineInstance],
        elapsed_seconds: f32,
        view_model: &mut ViewModelInstance,
    ) -> Result<bool, &'static str> {
        Ok(self.advance_with_state_machines_and_view_model(
            state_machines,
            elapsed_seconds,
            view_model,
        ))
    }
}

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

fn pointer_down(
    machine: &mut StateMachineInstance,
    artboard: &mut nuxie::ArtboardInstance<'_>,
    view_model: &ViewModelInstance,
    x: f32,
    y: f32,
    pointer_id: i32,
) {
    let mut view_model = view_model.raw_mut();
    machine.pointer_down_with_owned_view_model_context(
        artboard.raw_mut(),
        x,
        y,
        pointer_id,
        &mut view_model,
    );
}

fn pointer_up(
    machine: &mut StateMachineInstance,
    artboard: &mut nuxie::ArtboardInstance<'_>,
    view_model: &ViewModelInstance,
    x: f32,
    y: f32,
    pointer_id: i32,
) {
    let mut view_model = view_model.raw_mut();
    machine.pointer_up_with_owned_view_model_context(
        artboard.raw_mut(),
        x,
        y,
        pointer_id,
        &mut view_model,
    );
}

#[test]
#[ignore = "expected-red: Rust serializes frameSize while pinned C++ starts at makeRenderPaint"]
fn scripted_listener_action() {
    let file = File::import_with_unsigned_scripts(&pinned_fixture("scripted_listener_action.riv"))
        .expect("scripted_listener_action.riv imports with trusted scripts");
    let artboard = file.default_artboard().expect("default artboard");
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");

    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");

    let mut view_model = artboard
        .instantiate_view_model()
        .expect("default artboard view-model instance");
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.1,
            &mut view_model,
        )
        .expect("initial scripted advance");

    let mut renderer = silver.borrow().make_renderer();
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("initial scripted-listener draw");

    silver.borrow_mut().add_frame();

    pointer_down(
        &mut state_machine,
        &mut artboard,
        &view_model,
        200.0,
        20.0,
        1,
    );
    pointer_up(
        &mut state_machine,
        &mut artboard,
        &view_model,
        200.0,
        20.0,
        1,
    );
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("first listener frame advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("first listener frame draws");

    pointer_down(
        &mut state_machine,
        &mut artboard,
        &view_model,
        300.0,
        20.0,
        2,
    );
    pointer_up(
        &mut state_machine,
        &mut artboard,
        &view_model,
        300.0,
        20.0,
        2,
    );
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("second listener frame advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("second listener frame draws");

    pointer_down(
        &mut state_machine,
        &mut artboard,
        &view_model,
        400.0,
        20.0,
        3,
    );
    pointer_up(
        &mut state_machine,
        &mut artboard,
        &view_model,
        400.0,
        20.0,
        3,
    );
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("third listener frame advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("third listener frame draws");

    compare_silver("scripted_listener_action", &silver.borrow().bytes());
}

#[test]
#[ignore = "expected-red: Rust serializes frameSize while pinned C++ starts at makeRenderPaint"]
fn listener_action_inputs() {
    let file = File::import_with_unsigned_scripts(&pinned_fixture("listener_action_inputs.riv"))
        .expect("listener_action_inputs.riv imports with trusted scripts");
    let artboard = file.default_artboard().expect("default artboard");
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);

    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = if artboard.view_model_index().is_none() {
        artboard.instantiate_view_model()
    } else {
        artboard.instantiate_view_model_instance(0)
    }
    .expect("default artboard view-model instance");

    let mut renderer = silver.borrow().make_renderer();
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("initial listener-input advance");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("initial listener-input draw");

    pointer_down(
        &mut state_machine,
        &mut artboard,
        &view_model,
        width / 2.0,
        height / 2.0,
        3,
    );
    pointer_up(
        &mut state_machine,
        &mut artboard,
        &view_model,
        width / 2.0,
        height / 2.0,
        3,
    );
    silver.borrow_mut().add_frame();
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("listener-input click frame advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("listener-input click frame draws");

    compare_silver("listener_action_inputs", &silver.borrow().bytes());
}

fn string_value(view_model: &ViewModelInstance, name: &str) -> String {
    let value = view_model
        .raw()
        .string_value_by_property_name_path(name)
        .unwrap_or_else(|| panic!("string view-model property {name}"));
    String::from_utf8(value.to_vec()).expect("view-model strings are UTF-8")
}

fn number_value(view_model: &ViewModelInstance, name: &str) -> f32 {
    view_model
        .raw()
        .number_value_by_property_name_path(name)
        .unwrap_or_else(|| panic!("number view-model property {name}"))
}

fn boolean_value(view_model: &ViewModelInstance, name: &str) -> bool {
    view_model
        .raw()
        .boolean_value_by_property_name_path(name)
        .unwrap_or_else(|| panic!("boolean view-model property {name}"))
}

fn assert_context(
    view_model: &ViewModelInstance,
    key: &str,
    pointer: &str,
    text: &str,
    x: f32,
    y: f32,
    focus: bool,
) {
    assert_eq!(string_value(view_model, "keyInput"), key);
    assert_eq!(string_value(view_model, "pointerType"), pointer);
    assert_eq!(string_value(view_model, "stringInput"), text);
    assert_eq!(number_value(view_model, "posX"), x);
    assert_eq!(number_value(view_model, "posY"), y);
    assert_eq!(boolean_value(view_model, "isFocus"), focus);
}

#[test]
#[ignore = "expected-red: pointer move does not update pointerType to pointerEnter"]
fn listener_action_script_receives_pointer_types_and_the_data() {
    let file = File::import_with_unsigned_scripts(&pinned_fixture("scripted_listener_context.riv"))
        .expect("scripted_listener_context.riv imports with trusted scripts");
    let artboard = file.default_artboard().expect("default artboard");
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");
    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = artboard
        .instantiate_view_model()
        .expect("default artboard view-model instance");

    assert_context(&view_model, "", "", "", 0.0, 0.0, false);
    assert!(!boolean_value(&view_model, "eventReported"));
    assert!(!boolean_value(&view_model, "viewModelChanged"));

    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("initial listener-context advance");

    {
        let mut raw_view_model = view_model.raw_mut();
        state_machine.pointer_move_with_owned_view_model_context(
            artboard.raw_mut(),
            200.0,
            210.0,
            0.0,
            0,
            &mut raw_view_model,
        );
    }
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("pointer-enter frame advances");
    assert_context(&view_model, "", "pointerEnter", "", 200.0, 210.0, false);

    pointer_down(
        &mut state_machine,
        &mut artboard,
        &view_model,
        250.0,
        251.0,
        0,
    );
    pointer_up(
        &mut state_machine,
        &mut artboard,
        &view_model,
        250.0,
        251.0,
        0,
    );
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("click frame advances");
    assert_context(&view_model, "", "click", "", 250.0, 251.0, false);

    state_machine.focus_next(artboard.raw());
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("focus frame advances");
    assert_context(&view_model, "", "click", "", 250.0, 251.0, true);

    state_machine.key_input(artboard.raw_mut(), 65, 0, false, false);
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("key-up frame advances");
    assert_context(
        &view_model,
        "65, no shift, no meta, no control, no alt, phase: up",
        "click",
        "",
        250.0,
        251.0,
        true,
    );

    state_machine.text_input(artboard.raw_mut(), "With text input");
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("text-input frame advances");
    assert_context(
        &view_model,
        "65, no shift, no meta, no control, no alt, phase: up",
        "click",
        "With text input",
        250.0,
        251.0,
        true,
    );

    state_machine.key_input(artboard.raw_mut(), 66, (1 << 3) | 1, true, false);
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("modified key-down frame advances");
    assert_context(
        &view_model,
        "66, with shift, with meta, no control, no alt, phase: down",
        "click",
        "With text input",
        250.0,
        251.0,
        true,
    );

    state_machine.focus_next(artboard.raw());
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("focus-clear frame advances");
    assert_context(
        &view_model,
        "66, with shift, with meta, no control, no alt, phase: down",
        "click",
        "With text input",
        250.0,
        251.0,
        false,
    );

    state_machine.key_input(artboard.raw_mut(), 65, 0, false, false);
    artboard
        .try_advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut state_machine),
            0.016,
            &mut view_model,
        )
        .expect("unfocused key frame advances");
    assert_context(
        &view_model,
        "66, with shift, with meta, no control, no alt, phase: down",
        "click",
        "With text input",
        250.0,
        251.0,
        false,
    );
    assert!(boolean_value(&view_model, "eventReported"));
    assert!(boolean_value(&view_model, "viewModelChanged"));
}
