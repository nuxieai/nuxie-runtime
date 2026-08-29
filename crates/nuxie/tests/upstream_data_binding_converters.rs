//! Exact executable ports of pinned `data_binding_converters_test.cpp`.

use std::path::PathBuf;

use nuxie::{
    Artboard, File, PersistentFactory, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle, RuntimeStateMachineInstanceHandle, RuntimeViewModelInstanceHandle,
    ViewModelInstanceRuntime,
};
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
    run: impl FnOnce(
        &RuntimeFileHandle,
        &RuntimeArtboardInstanceHandle,
        &RuntimeStateMachineInstanceHandle,
        &RuntimeViewModelInstanceHandle,
        &mut PersistentFactory<SerializingFactory>,
    ),
) {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut silver).expect("retained factory");
    let file = File::import(
        &pinned(&format!("assets/{asset}")),
        factory,
        None,
        None,
        None,
    )
    .expect("pinned fixture imports");
    let source = file.with_file(File::artboard).expect("default artboard");
    let artboard = Artboard::instance_from_handle(&source).expect("default artboard instantiates");
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = fresh_default_view_model(&file, &artboard);
    bind_view_model(&machine, &view_model);
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    run(&file, &artboard, &machine, &view_model, &mut silver);
}

fn fresh_default_view_model(
    file: &RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
) -> RuntimeViewModelInstanceHandle {
    let instance = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                .or_else(|| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        })
        .expect("artboard view model");
    ViewModelInstanceRuntime::new(instance).into_handle()
}

fn bind_view_model(
    machine: &RuntimeStateMachineInstanceHandle,
    view_model: &RuntimeViewModelInstanceHandle,
) {
    machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(view_model.instance());
    });
}

fn advance_draw(
    artboard: &RuntimeArtboardInstanceHandle,
    machine: &RuntimeStateMachineInstanceHandle,
    silver: &mut PersistentFactory<SerializingFactory>,
    seconds: f32,
) {
    machine.advance_and_apply(seconds);
    let mut renderer = silver.borrow().make_renderer();
    artboard.draw(&mut renderer);
}

fn set_number(view_model: &RuntimeViewModelInstanceHandle, name: &str, value: f32) {
    let property = view_model.property_number(name).expect("number property");
    property.set_value(value);
    assert_eq!(
        property.value(),
        value,
        "{name} retains the exact assigned number"
    );
}

fn set_color(view_model: &RuntimeViewModelInstanceHandle, name: &str, value: u32) {
    let property = view_model.property_color(name).expect("color property");
    property.set_value(value as i32);
    assert_eq!(
        property.value() as u32,
        value,
        "{name} retains the exact assigned color"
    );
}

#[test]
fn list_to_length_converter() {
    with_live(
        "list_to_length_test.riv",
        |file, artboard, machine, view_model, silver| {
            advance_draw(artboard, machine, silver, 0.1);
            let child = file
                .with_file(|file| file.view_model_by_name("child"))
                .expect("child view model");
            let list = view_model.property_list("lis").expect("lis list");
            for _ in 0..4 {
                silver.borrow_mut().add_frame();
                let item = child.create_default_instance();
                assert!(list.add_instance_at(item, list.size() as i32));
                machine.advance_and_apply(0.1);
                advance_draw(artboard, machine, silver, 0.1);
            }
            compare_silver("list_to_length_test", &silver.borrow().bytes());
        },
    );
}

#[test]
fn data_converter_interpolator_resets_on_binding() {
    with_live(
        "data_converter_interpolator_reset.riv",
        |file, artboard, machine, view_model, silver| {
            set_number(view_model, "xPos", 250.0);
            set_color(view_model, "col", 0xffff_0000);
            advance_draw(artboard, machine, silver, 0.1);
            set_color(view_model, "col", 0xff00_ff00);
            set_number(view_model, "xPos", 500.0);
            for _ in 0..(1.0_f32 / 0.016_f32) as usize {
                silver.borrow_mut().add_frame();
                advance_draw(artboard, machine, silver, 0.016);
            }

            silver.borrow_mut().add_frame();
            let rebound = fresh_default_view_model(file, artboard);
            bind_view_model(machine, &rebound);
            set_number(&rebound, "xPos", 250.0);
            set_color(&rebound, "col", 0xffff_0000);
            advance_draw(artboard, machine, silver, 0.1);
            set_color(&rebound, "col", 0xff00_00ff);
            set_number(&rebound, "xPos", 0.0);
            for _ in 0..(1.0_f32 / 0.016_f32) as usize {
                silver.borrow_mut().add_frame();
                advance_draw(artboard, machine, silver, 0.016);
            }
            compare_silver(
                "data_converter_interpolator_reset",
                &silver.borrow().bytes(),
            );
        },
    );
}

#[test]
fn interpolations_that_change_duration_to_zero_work_correctly() {
    with_live(
        "interpolation_zero_duration.riv",
        |_file, artboard, machine, view_model, silver| {
            advance_draw(artboard, machine, silver, 0.1);
            set_number(view_model, "objectX", 200.0);
            let frames = (1.5_f32 / 0.1_f32) as usize;
            for _ in 0..frames {
                silver.borrow_mut().add_frame();
                advance_draw(artboard, machine, silver, 0.1);
            }
            set_number(view_model, "interpValue", 0.0);
            machine.advance_and_apply(0.016);
            set_number(view_model, "objectX", 400.0);
            machine.advance_and_apply(0.016);
            for _ in 0..frames {
                silver.borrow_mut().add_frame();
                advance_draw(artboard, machine, silver, 0.1);
            }
            set_number(view_model, "interpValue", 1.0);
            machine.advance_and_apply(0.016);
            set_number(view_model, "objectX", 200.0);
            machine.advance_and_apply(0.016);
            for _ in 0..frames {
                silver.borrow_mut().add_frame();
                advance_draw(artboard, machine, silver, 0.1);
            }
            compare_silver("interpolation_zero_duration", &silver.borrow().bytes());
        },
    );
}
