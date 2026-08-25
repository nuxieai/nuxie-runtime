//! Silver-test ports from pinned
//! `tests/unit_tests/runtime/scripting/scripting_artboard_test.cpp`.
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

fn sixty_frame_artboard_silver(fixture: &str, silver_name: &str) {
    let file = File::import_with_unsigned_scripts(&pinned_fixture(fixture))
        .unwrap_or_else(|error| panic!("{fixture} imports with trusted scripts: {error}"));
    let artboard = file.artboard_named("Artboard").expect("Artboard artboard");
    let mut artboard = artboard.instantiate().expect("Artboard instantiates");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    artboard
        .initialize_renderer(&mut silver)
        .expect("Artboard renderer initializes at the import boundary");
    artboard
        .mount_scripted_drawables(&mut silver)
        .expect("Artboard scripts mount at the import boundary");
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    artboard
        .try_advance_with_state_machines_and_factory(
            std::slice::from_mut(&mut machine),
            0.1,
            &mut silver,
        )
        .expect("initial scripted Artboard frame advances");
    let mut renderer = silver.borrow().make_renderer();
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("initial draw");
    for _ in 0..60 {
        silver.borrow_mut().add_frame();
        artboard
            .try_advance_with_state_machines_and_factory(
                std::slice::from_mut(&mut machine),
                0.016,
                &mut silver,
            )
            .expect("scripted Artboard frame advances");
        artboard
            .draw(&mut silver, &mut renderer)
            .expect("frame draw");
    }
    compare_silver(silver_name, &silver.borrow().bytes());
}

#[test]
fn script_instances_artboard_input() {
    sixty_frame_artboard_silver("script_artboard_test.riv", "script_artboards");
}

#[test]
fn script_instances_artboard_input_with_proper_origin() {
    sixty_frame_artboard_silver("script_artboard_origin_test.riv", "script_artboards_origin");
}

#[test]
#[ignore = "expected-red: didChange lifecycle passes; frame 0 op 11 omits a retained makeRenderPaint"]
fn script_node_advance_affects_did_change_via_dirt() {
    let file =
        File::import_with_unsigned_scripts(&pinned_fixture("script_affects_has_changed.riv"))
            .expect("script_affects_has_changed imports with trusted scripts");
    let artboard = file.artboard_named("Main").expect("Main artboard");
    let mut artboard = artboard.instantiate().expect("Main instantiates");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    artboard
        .initialize_renderer(&mut silver)
        .expect("Main renderer initializes at the import boundary");
    artboard
        .mount_scripted_drawables(&mut silver)
        .expect("Main scripts mount at the import boundary");
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = artboard
        .instantiate_view_model()
        .expect("view-model instance");
    artboard.bind_view_model(&view_model);
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("initial didChange frame advances");
    let mut renderer = silver.borrow().make_renderer();
    assert!(artboard.raw().did_change());
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("first draw");
    silver.borrow_mut().add_frame();
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            1.0,
            &mut view_model,
            &mut silver,
        )
        .expect("stationary didChange frame advances");
    assert!(!artboard.raw().did_change());
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("second draw");
    assert!(view_model.set_bool("toLeft", true));
    silver.borrow_mut().add_frame();
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("dirty didChange frame advances");
    assert!(artboard.raw().did_change());
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("third draw");
    silver.borrow_mut().add_frame();
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("settled didChange frame advances");
    assert!(!artboard.raw().did_change());
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("fourth draw");
    compare_silver("script_affects_has_changed", &silver.borrow().bytes());
}

#[test]
#[ignore = "expected-red: frame 0 op 25 expects another retained paint but Rust updates color"]
fn script_instance_linear_animations() {
    let file =
        File::import_with_unsigned_scripts(&pinned_fixture("scripting_linear_animation.riv"))
            .expect("scripting_linear_animation imports with trusted scripts");
    let artboard = file.artboard_named("Main").expect("Main artboard");
    let mut artboard = artboard.instantiate().expect("Main instantiates");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    artboard
        .initialize_renderer(&mut silver)
        .expect("Main renderer initializes at the import boundary");
    artboard
        .mount_scripted_drawables(&mut silver)
        .expect("Main scripts mount at the import boundary");
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = if artboard.view_model_index().is_none() {
        artboard.instantiate_view_model()
    } else {
        artboard.instantiate_view_model_instance(0)
    }
    .expect("view-model instance");
    artboard.bind_view_model(&view_model);
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.1,
            &mut view_model,
            &mut silver,
        )
        .expect("initial scripted animation frame advances");
    let mut renderer = silver.borrow().make_renderer();
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("initial draw");
    for _ in 0..60 {
        silver.borrow_mut().add_frame();
        artboard
            .try_advance_with_state_machines_and_view_model_and_factory(
                std::slice::from_mut(&mut machine),
                0.064,
                &mut view_model,
                &mut silver,
            )
            .expect("scripted animation frame advances");
        artboard
            .draw(&mut silver, &mut renderer)
            .expect("animation draw");
    }
    for time in [0.55, -1.0, 3.8, 40.0] {
        assert!(view_model.set_number("time", time));
        double_advance(&mut artboard, &mut machine, &mut view_model, &mut silver);
        silver.borrow_mut().add_frame();
        artboard
            .draw(&mut silver, &mut renderer)
            .expect("seconds draw");
        assert!(view_model.set_string("mode", "frames"));
        double_advance(&mut artboard, &mut machine, &mut view_model, &mut silver);
        silver.borrow_mut().add_frame();
        artboard
            .draw(&mut silver, &mut renderer)
            .expect("frames draw");
        assert!(view_model.set_string("mode", "percentage"));
        double_advance(&mut artboard, &mut machine, &mut view_model, &mut silver);
        silver.borrow_mut().add_frame();
        artboard
            .draw(&mut silver, &mut renderer)
            .expect("percentage draw");
    }
    compare_silver("scripting_linear_animation", &silver.borrow().bytes());
}

fn double_advance(
    artboard: &mut nuxie::ArtboardInstance<'_>,
    machine: &mut nuxie::StateMachineInstance,
    view_model: &mut nuxie::ViewModelInstance,
    factory: &mut dyn nuxie_render_api::Factory,
) {
    for _ in 0..2 {
        artboard
            .try_advance_with_state_machines_and_view_model_and_factory(
                std::slice::from_mut(machine),
                0.016,
                view_model,
                factory,
            )
            .expect("scripted animation mutation frame advances");
    }
}

#[test]
fn script_instances_artboard_with_opacity_applied() {
    sixty_frame_artboard_silver(
        "script_artboard_opacity_test.riv",
        "script_artboards_opacity",
    );
}

#[test]
#[ignore = "expected-red: frame 0 op 22 expects another retained paint before frameSize"]
fn view_model_source_cache_is_cleared_when_instance_changes() {
    let file = File::import_with_unsigned_scripts(&pinned_fixture("scripted_viewmodel_cache.riv"))
        .expect("scripted_viewmodel_cache imports with trusted scripts");
    let artboard = file.default_artboard().expect("default artboard");
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    artboard
        .initialize_renderer(&mut silver)
        .expect("default renderer initializes at the import boundary");
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = artboard
        .instantiate_default_view_model_instance()
        .expect("default view-model instance");
    let mut renderer = silver.borrow().make_renderer();
    artboard.bind_view_model(&view_model);
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.016,
            &mut view_model,
            &mut silver,
        )
        .expect("initial source-cache frame advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("initial draw");
    silver.borrow_mut().add_frame();

    machine.pointer_down(artboard.raw_mut(), 450.0, 50.0, 0);
    machine.pointer_up(artboard.raw_mut(), 450.0, 50.0, 0);
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.016,
            &mut view_model,
            &mut silver,
        )
        .expect("first source selection advances");
    assert!(view_model.fire_trigger("createInstance"));
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.016,
            &mut view_model,
            &mut silver,
        )
        .expect("first instance creation advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("source-1 draw");
    silver.borrow_mut().add_frame();

    machine.pointer_down(artboard.raw_mut(), 450.0, 150.0, 0);
    machine.pointer_up(artboard.raw_mut(), 450.0, 150.0, 0);
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.016,
            &mut view_model,
            &mut silver,
        )
        .expect("second source selection advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("source-2 draw");
    silver.borrow_mut().add_frame();
    assert!(view_model.fire_trigger("createInstance"));
    artboard
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.016,
            &mut view_model,
            &mut silver,
        )
        .expect("second instance creation advances");
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("final draw");
    compare_silver("scripted_viewmodel_cache", &silver.borrow().bytes());
}
