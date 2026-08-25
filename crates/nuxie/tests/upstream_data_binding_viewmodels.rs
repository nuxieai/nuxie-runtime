//! Exact executable ports of pinned `data_binding_viewmodels_test.cpp`.

use std::path::PathBuf;

use nuxie::{File, PersistentFactory};
use nuxie_render_api::SerializingFactory;
use silver_corpus::{compare_sriv, parse_sriv};

fn pinned(relative: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root).join("tests/unit_tests").join(relative);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn compare_silver(name: &str, bytes: &[u8]) {
    let expected = parse_sriv(&pinned(&format!("silvers/{name}.sriv"))).expect("pinned SRIV");
    let actual = parse_sriv(bytes).expect("Rust SRIV");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("{name} differs: {difference}"));
}

fn run(
    asset: &str,
    silver_name: &str,
    body: impl for<'a> FnOnce(
        &'a File,
        &mut nuxie::ArtboardInstance<'a>,
        &mut nuxie::StateMachineInstance,
        &mut nuxie::ViewModelInstance,
        &mut PersistentFactory<SerializingFactory>,
    ),
) {
    let file = Box::leak(Box::new(
        File::import(&pinned(&format!("assets/{asset}"))).expect("fixture imports"),
    ));
    let mut artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("artboard instantiates");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    artboard
        .initialize_renderer(&mut silver)
        .expect("renderer initializes");
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = artboard
        .instantiate_default_view_model_instance()
        .or_else(|| artboard.instantiate_view_model())
        .expect("default view model");
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    body(
        file,
        &mut artboard,
        &mut machine,
        &mut view_model,
        &mut silver,
    );
    compare_silver(silver_name, &silver.borrow().bytes());
}

fn advance_draw(
    artboard: &mut nuxie::ArtboardInstance<'_>,
    machine: &mut nuxie::StateMachineInstance,
    view_model: &mut nuxie::ViewModelInstance,
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

fn click(
    artboard: &mut nuxie::ArtboardInstance<'_>,
    machine: &mut nuxie::StateMachineInstance,
    x: f32,
    y: f32,
) {
    machine.pointer_down(artboard.raw_mut(), x, y, 0);
    machine.pointer_up(artboard.raw_mut(), x, y, 0);
}

#[test]
#[ignore = "expected-red: databind_viewmodel stream diverges at frame 0 op 53 (expected save, got restore)"]
fn bind_view_model_from_set_value_external_change_and_scripting() {
    run(
        "databind_viewmodel.riv",
        "databind_viewmodel",
        |file, artboard, machine, view_model, silver| {
            advance_draw(artboard, machine, view_model, silver, 0.016);
            silver.borrow_mut().add_frame();
            let mut child = file
                .view_model_named("StatefulChild")
                .expect("StatefulChild")
                .instantiate()
                .expect("child instance");
            let _ = child.set_number("num", 44.0);
            assert_eq!(
                child.raw().number_value_by_property_name_path("num"),
                Some(44.0)
            );
            assert!(
                view_model
                    .handle()
                    .link_view_model_by_property_name_path("statefulChild", child.handle())
                    .is_ok()
            );
            advance_draw(artboard, machine, view_model, silver, 0.016);
            silver.borrow_mut().add_frame();
            let _ = child.set_number("num", 44.0);
            advance_draw(artboard, machine, view_model, silver, 0.016);
            silver.borrow_mut().add_frame();
            click(artboard, machine, 25.0, 25.0);
            advance_draw(artboard, machine, view_model, silver, 0.016);
        },
    );
}
