//! Direct ports of both cases in pinned
//! `tests/unit_tests/runtime/data_binding_fonts_test.cpp`.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_render_api::RecordingFactory;
use nuxie_runtime::{ArtboardInstance, RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

fn fixture() -> (RuntimeFile, GraphFile) {
    let file = read_runtime_file(&pinned_fixture("data_bind_font_test.riv"))
        .expect("data_bind_font_test.riv imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("data_bind_font_test.riv graph builds");
    (file, graphs)
}

fn default_context(file: &RuntimeFile) -> RuntimeOwnedViewModelHandle {
    RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file, 0, 0)
            .expect("default view-model instance builds"),
    )
}

fn draw(
    artboard: &mut ArtboardInstance,
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    graphs: &GraphFile,
    factory: &mut RecordingFactory,
) {
    let mut renderer = factory.make_renderer();
    artboard
        .draw_artboard(
            file,
            graph,
            &graphs.artboards,
            factory,
            &mut renderer,
            &BTreeMap::new(),
            None,
            true,
        )
        .expect("font-bound artboard draws");
}

fn missing_silver_match(_: &str, _: &str) -> bool {
    panic!("recording renderer has no pinned C++ silver matcher for this case")
}

#[test]
#[ignore = "expected-red: pinned data_bind_font_test silver matcher is not wired"]
fn data_bind_font() {
    let (file, graphs) = fixture();
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let context = default_context(&file);
    assert!(artboard.bind_owned_view_model_artboard_handle(&file, &context));
    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");
    state_machine.bind_owned_view_model_handle(&context);

    let mut factory = RecordingFactory::new();
    let (width, height) = artboard.artboard_dimensions();
    factory.frame_size(width as u32, height as u32);

    state_machine
        .advance_and_apply(&mut artboard, 0.0)
        .expect("initial state-machine advance");
    draw(&mut artboard, &file, graph, &graphs, &mut factory);
    factory.add_frame();
    state_machine
        .advance_and_apply(&mut artboard, 0.016)
        .expect("second state-machine advance");
    draw(&mut artboard, &file, graph, &graphs, &mut factory);

    factory.add_frame();

    let kablammo: Arc<[u8]> = pinned_fixture("kablammo.ttf").into();
    assert!(
        context
            .borrow_mut()
            .set_live_font_bytes_by_property_name("fontProperty", Some(kablammo))
    );
    state_machine
        .advance_and_apply(&mut artboard, 0.016)
        .expect("live-font state-machine advance");
    draw(&mut artboard, &file, graph, &graphs, &mut factory);
    factory.add_frame();

    state_machine.pointer_down(&mut artboard, 490.0, 490.0, 0);
    state_machine.pointer_up(&mut artboard, 490.0, 490.0, 0);
    state_machine
        .advance_and_apply(&mut artboard, 0.016)
        .expect("first listener state-machine advance");
    draw(&mut artboard, &file, graph, &graphs, &mut factory);

    factory.add_frame();
    state_machine.pointer_down(&mut artboard, 490.0, 20.0, 0);
    state_machine.pointer_up(&mut artboard, 490.0, 20.0, 0);
    state_machine
        .advance_and_apply(&mut artboard, 0.016)
        .expect("second listener state-machine advance");
    draw(&mut artboard, &file, graph, &graphs, &mut factory);
    assert!(missing_silver_match(
        "data_bind_font_test",
        &factory.stream()
    ));
}

#[test]
fn font_data_bind_stores_and_clears_the_font_on_the_property() {
    let (file, graphs) = fixture();
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let context = default_context(&file);
    assert!(artboard.bind_owned_view_model_artboard_handle(&file, &context));
    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");
    state_machine.bind_owned_view_model_handle(&context);
    state_machine
        .advance_and_apply(&mut artboard, 0.0)
        .expect("initial state-machine advance");

    assert!(
        context
            .borrow()
            .font_asset_value_by_property_name("fontProperty")
            .is_some()
    );

    let kablammo: Arc<[u8]> = pinned_fixture("kablammo.ttf").into();
    assert!(
        context
            .borrow_mut()
            .set_live_font_bytes_by_property_name("fontProperty", Some(Arc::clone(&kablammo)))
    );
    state_machine
        .advance_and_apply(&mut artboard, 0.0)
        .expect("kablammo state-machine advance");
    assert_eq!(
        context
            .borrow()
            .font_asset_value_by_property_name("fontProperty")
            .expect("fontProperty")
            .live_font_bytes()
            .map(<[u8]>::as_ptr),
        Some(kablammo.as_ptr())
    );

    let nabla: Arc<[u8]> = pinned_fixture("nabla.ttf").into();
    assert!(
        context
            .borrow_mut()
            .set_live_font_bytes_by_property_name("fontProperty", Some(Arc::clone(&nabla)))
    );
    state_machine
        .advance_and_apply(&mut artboard, 0.0)
        .expect("nabla state-machine advance");
    assert_eq!(
        context
            .borrow()
            .font_asset_value_by_property_name("fontProperty")
            .expect("fontProperty")
            .live_font_bytes()
            .map(<[u8]>::as_ptr),
        Some(nabla.as_ptr())
    );

    assert!(
        context
            .borrow_mut()
            .set_live_font_bytes_by_property_name("fontProperty", None)
    );
    state_machine
        .advance_and_apply(&mut artboard, 0.0)
        .expect("clear-font state-machine advance");
    assert!(
        context
            .borrow()
            .font_asset_value_by_property_name("fontProperty")
            .expect("fontProperty")
            .live_font_bytes()
            .is_none()
    );
}
