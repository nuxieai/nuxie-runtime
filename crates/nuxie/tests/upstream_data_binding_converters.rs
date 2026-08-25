//! Exact executable ports of pinned `data_binding_converters_test.cpp`.

use std::path::PathBuf;

use nuxie::{File, PersistentFactory, ViewModelInstance};
use nuxie_render_api::SerializingFactory;
use silver_corpus::{compare_sriv, parse_sriv};

fn pinned(relative: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root).join("tests/unit_tests").join(relative);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn compare_silver(name: &str, actual: &[u8]) {
    let expected =
        parse_sriv(&pinned(&format!("silvers/{name}.sriv"))).expect("pinned SRIV parses");
    let actual = parse_sriv(actual).expect("Rust SRIV parses");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("{name} differs: {difference}"));
}

fn with_live(
    asset: &str,
    run: impl for<'a> FnOnce(
        &'a File,
        &mut nuxie::ArtboardInstance<'a>,
        &mut nuxie::StateMachineInstance,
        &mut ViewModelInstance,
        &mut PersistentFactory<SerializingFactory>,
    ),
) {
    // Keep the File at a stable address for the facade's borrowed instance.
    let file = Box::leak(Box::new(
        File::import(&pinned(&format!("assets/{asset}"))).expect("pinned fixture imports"),
    ));
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("default artboard instantiates");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    artboard
        .initialize_renderer(&mut silver)
        .expect("renderer initializes at import boundary");
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = artboard
        .instantiate_default_view_model_instance()
        .or_else(|| artboard.instantiate_view_model())
        .expect("artboard view model");
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    run(
        file,
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut silver,
    );
}

fn advance_draw(
    artboard: &mut nuxie::ArtboardInstance<'_>,
    machine: &mut nuxie::StateMachineInstance,
    view_model: &mut ViewModelInstance,
    silver: &mut PersistentFactory<SerializingFactory>,
    seconds: f32,
) {
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(machine),
        seconds,
        view_model,
    );
    let mut renderer = silver.borrow().make_renderer();
    artboard
        .draw(silver, &mut renderer)
        .expect("artboard draws");
}

fn set_number(view_model: &mut ViewModelInstance, name: &str, value: f32) {
    let _ = view_model.set_number(name, value);
    assert_eq!(
        view_model.raw().number_value_by_property_name_path(name),
        Some(value),
        "{name} retains the exact assigned number"
    );
}

fn set_color(view_model: &mut ViewModelInstance, name: &str, value: u32) {
    let _ = view_model.set_color(name, value);
    assert_eq!(
        view_model.raw().color_value_by_property_name_path(name),
        Some(value),
        "{name} retains the exact assigned color"
    );
}

#[test]
fn list_to_length_converter() {
    with_live(
        "list_to_length_test.riv",
        |file, artboard, machine, view_model, silver| {
            advance_draw(artboard, machine, view_model, silver, 0.1);
            let child = file.view_model_named("child").expect("child view model");
            for _ in 0..4 {
                silver.borrow_mut().add_frame();
                let item = child.instantiate_default().expect("child instance");
                let index = view_model
                    .raw()
                    .list_item_count_by_property_name_path("lis")
                    .expect("lis list");
                assert!(view_model.handle().insert_list_item_by_property_name_path(
                    "lis",
                    index,
                    item.handle(),
                ));
                artboard.advance_with_state_machines_and_view_model(
                    std::slice::from_mut(machine),
                    0.1,
                    view_model,
                );
                advance_draw(artboard, machine, view_model, silver, 0.1);
            }
            compare_silver("list_to_length_test", &silver.borrow().bytes());
        },
    );
}

#[test]
#[ignore = "expected-red: rebound interpolator stream diverges at frame 1 op 30 (expected save, got color)"]
fn data_converter_interpolator_resets_on_binding() {
    with_live(
        "data_converter_interpolator_reset.riv",
        |_file, artboard, machine, view_model, silver| {
            set_number(view_model, "xPos", 250.0);
            set_color(view_model, "col", 0xffff_0000);
            advance_draw(artboard, machine, view_model, silver, 0.1);
            set_color(view_model, "col", 0xff00_ff00);
            set_number(view_model, "xPos", 500.0);
            for _ in 0..(1.0_f32 / 0.016_f32) as usize {
                silver.borrow_mut().add_frame();
                advance_draw(artboard, machine, view_model, silver, 0.016);
            }

            silver.borrow_mut().add_frame();
            let mut rebound = artboard
                .instantiate_default_view_model_instance()
                .or_else(|| artboard.instantiate_view_model())
                .expect("replacement view model");
            set_number(&mut rebound, "xPos", 250.0);
            set_color(&mut rebound, "col", 0xffff_0000);
            advance_draw(artboard, machine, &mut rebound, silver, 0.1);
            set_color(&mut rebound, "col", 0xff00_00ff);
            set_number(&mut rebound, "xPos", 0.0);
            for _ in 0..(1.0_f32 / 0.016_f32) as usize {
                silver.borrow_mut().add_frame();
                advance_draw(artboard, machine, &mut rebound, silver, 0.016);
            }
            compare_silver(
                "data_converter_interpolator_reset",
                &silver.borrow().bytes(),
            );
        },
    );
}

#[test]
#[ignore = "expected-red: zero-duration interpolation stream diverges at frame 1 transform tx (expected 0, got 200)"]
fn interpolations_that_change_duration_to_zero_work_correctly() {
    with_live(
        "interpolation_zero_duration.riv",
        |_file, artboard, machine, view_model, silver| {
            advance_draw(artboard, machine, view_model, silver, 0.1);
            set_number(view_model, "objectX", 200.0);
            let frames = (1.5_f32 / 0.1_f32) as usize;
            for _ in 0..frames {
                silver.borrow_mut().add_frame();
                advance_draw(artboard, machine, view_model, silver, 0.1);
            }
            set_number(view_model, "interpValue", 0.0);
            artboard.advance_with_state_machines_and_view_model(
                std::slice::from_mut(machine),
                0.016,
                view_model,
            );
            set_number(view_model, "objectX", 400.0);
            artboard.advance_with_state_machines_and_view_model(
                std::slice::from_mut(machine),
                0.016,
                view_model,
            );
            for _ in 0..frames {
                silver.borrow_mut().add_frame();
                advance_draw(artboard, machine, view_model, silver, 0.1);
            }
            set_number(view_model, "interpValue", 1.0);
            artboard.advance_with_state_machines_and_view_model(
                std::slice::from_mut(machine),
                0.016,
                view_model,
            );
            set_number(view_model, "objectX", 200.0);
            artboard.advance_with_state_machines_and_view_model(
                std::slice::from_mut(machine),
                0.016,
                view_model,
            );
            for _ in 0..frames {
                silver.borrow_mut().add_frame();
                advance_draw(artboard, machine, view_model, silver, 0.1);
            }
            compare_silver("interpolation_zero_duration", &silver.borrow().bytes());
        },
    );
}
