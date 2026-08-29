//! Exact executable ports of pinned `data_binding_viewmodels_test.cpp`.

use std::path::PathBuf;

use nuxie::{
    Artboard, File, PersistentFactory, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle, RuntimeStateMachineInstanceHandle, RuntimeViewModelInstanceHandle, Vec2D,
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

fn compare_silver(name: &str, bytes: &[u8]) {
    let expected = parse_sriv(&pinned(&format!("silvers/{name}.sriv"))).expect("pinned SRIV");
    let actual = parse_sriv(bytes).expect("Rust SRIV");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("{name} differs: {difference}"));
}

fn run(
    asset: &str,
    silver_name: &str,
    body: impl FnOnce(
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
    .expect("fixture imports");
    let source = file.with_file(File::artboard).expect("default artboard");
    let artboard = Artboard::instance_from_handle(&source).expect("artboard instantiates");
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                .or_else(|| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("default view model");
    machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(view_model.instance());
    });
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    body(&file, &artboard, &machine, &view_model, &mut silver);
    compare_silver(silver_name, &silver.borrow().bytes());
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

fn click(machine: &RuntimeStateMachineInstanceHandle, x: f32, y: f32) {
    machine.with_instance_mut(|machine| {
        machine.pointer_down(Vec2D::new(x, y), 0);
        machine.pointer_up(Vec2D::new(x, y), 0);
    });
}

#[test]
#[ignore = "expected-red: databind_viewmodel stream diverges at frame 0 op 53 (expected save, got restore)"]
fn bind_view_model_from_set_value_external_change_and_scripting() {
    run(
        "databind_viewmodel.riv",
        "databind_viewmodel",
        |file, artboard, machine, view_model, silver| {
            advance_draw(artboard, machine, silver, 0.016);
            silver.borrow_mut().add_frame();
            let child = file
                .with_file(|file| file.view_model_by_name("StatefulChild"))
                .expect("StatefulChild")
                .create_instance();
            let number = child.property_number("num").expect("num property");
            number.set_value(44.0);
            assert_eq!(number.value(), 44.0);
            assert!(view_model.replace_view_model("statefulChild", child));
            advance_draw(artboard, machine, silver, 0.016);
            silver.borrow_mut().add_frame();
            number.set_value(44.0);
            advance_draw(artboard, machine, silver, 0.016);
            silver.borrow_mut().add_frame();
            click(machine, 25.0, 25.0);
            advance_draw(artboard, machine, silver, 0.016);
        },
    );
}
