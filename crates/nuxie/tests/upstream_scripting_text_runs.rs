//! Direct port of pinned
//! `tests/unit_tests/runtime/scripting/scripting_text_runs.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{File, PersistentFactory, ViewModelInstance};
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
        .join(name);
    std::fs::read(&silver)
        .unwrap_or_else(|error| panic!("read pinned silver {}: {error}", silver.display()))
}

fn fire_button(view_model: &mut ViewModelInstance, button: &str, count: usize) {
    let trigger = format!("{button}/onClick");
    for _ in 0..count {
        assert!(view_model.fire_trigger(&trigger), "trigger {trigger}");
    }
}

#[test]
#[ignore = "expected-red: Node Script 1 init receives nil for its authored lis input"]
fn script_creates_view_models_that_map_to_text_runs() {
    let file = File::import_with_unsigned_scripts(&pinned_fixture("script_create_text_runs.riv"))
        .expect("script_create_text_runs.riv imports with trusted scripts");
    let artboard = file.artboard_named("main").expect("main artboard");
    let mut instance = artboard.instantiate().expect("main artboard instantiates");
    let mut state_machine = instance.state_machine_instance(0).expect("state machine 0");
    let mut view_model = if instance.view_model_index().is_none() {
        instance.instantiate_view_model()
    } else {
        instance.instantiate_view_model_instance(0)
    }
    .expect("main view-model instance");
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
        .expect("initial text-run draw");

    silver.borrow_mut().add_frame();
    fire_button(&mut view_model, "newButton", 1);
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("push scripted advance");
    instance
        .draw(&mut silver, &mut renderer)
        .expect("push text-run draw");

    silver.borrow_mut().add_frame();
    fire_button(&mut view_model, "newAtButton", 1);
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("indexed-push scripted advance");
    instance
        .draw(&mut silver, &mut renderer)
        .expect("indexed-push text-run draw");

    silver.borrow_mut().add_frame();
    fire_button(&mut view_model, "swapButton", 1);
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("swap scripted advance");
    instance
        .draw(&mut silver, &mut renderer)
        .expect("swap text-run draw");

    silver.borrow_mut().add_frame();
    fire_button(&mut view_model, "shiftButton", 1);
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("shift scripted advance");
    instance
        .draw(&mut silver, &mut renderer)
        .expect("shift text-run draw");

    silver.borrow_mut().add_frame();
    fire_button(&mut view_model, "popButton", 1);
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("pop scripted advance");
    instance
        .draw(&mut silver, &mut renderer)
        .expect("pop text-run draw");

    silver.borrow_mut().add_frame();
    fire_button(&mut view_model, "popButton", 4);
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("pop-through-empty scripted advance");
    instance
        .draw(&mut silver, &mut renderer)
        .expect("pop-through-empty text-run draw");

    silver.borrow_mut().add_frame();
    fire_button(&mut view_model, "newButton", 2);
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut state_machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("double-push scripted advance");
    instance
        .draw(&mut silver, &mut renderer)
        .expect("double-push text-run draw");

    let actual = parse_sriv(&silver.borrow().bytes()).expect("valid Rust SRIV stream");
    let expected = parse_sriv(&pinned_silver("script_create_text_runs.sriv"))
        .expect("valid pinned SRIV stream");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("script_create_text_runs differs: {difference}"));
}
