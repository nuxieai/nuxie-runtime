//! One-for-one ports of all four cases in pinned
//! `tests/unit_tests/runtime/scroll_test.cpp`.
//!
//! The complete frame, pointer, advance, and draw sequences are retained. The
//! tests are honest expected-red ports because this crate does not yet expose
//! the pinned C++ silver matcher used by the four final assertions.

use std::collections::BTreeMap;
use std::path::PathBuf;

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::GraphFile;
use nuxie_render_api::SerializingFactory;
use nuxie_runtime::{
    ArtboardInstance, RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance,
    StateMachineInstance,
};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

struct Fixture {
    file: RuntimeFile,
    graphs: GraphFile,
    graph_index: usize,
    artboard: ArtboardInstance,
    state_machine: StateMachineInstance,
}

fn fixture(asset: &str, artboard_name: Option<&str>, bind_default_view_model: bool) -> Fixture {
    let file = read_runtime_file(&pinned_fixture(asset))
        .unwrap_or_else(|error| panic!("{asset} imports: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&file)
        .unwrap_or_else(|error| panic!("{asset} graph builds: {error:#}"));
    let graph_index = artboard_name
        .map(|name| {
            graphs
                .artboards
                .iter()
                .position(|graph| graph.name.as_deref() == Some(name))
                .unwrap_or_else(|| panic!("{asset} has artboard {name}"))
        })
        .unwrap_or(0);
    let graph = &graphs.artboards[graph_index];
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .unwrap_or_else(|error| panic!("{asset} artboard instantiates: {error:#}"));
    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");

    if bind_default_view_model {
        let view_model_index = usize::try_from(
            file.artboard(graph_index)
                .and_then(|artboard| artboard.uint_property("viewModelId"))
                .expect("artboard viewModelId"),
        )
        .expect("viewModelId fits usize");
        let instance_index = file
            .view_model_default_instance(view_model_index)
            .expect("authored default view-model instance")
            .instance_index;
        let view_model = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::from_instance(&file, view_model_index, instance_index)
                .expect("default view-model instance builds"),
        );
        state_machine.bind_owned_view_model_handle(&view_model);
    }

    Fixture {
        file,
        graphs,
        graph_index,
        artboard,
        state_machine,
    }
}

fn draw(fixture: &mut Fixture, silver: &mut SerializingFactory) {
    let mut renderer = silver.make_renderer();
    fixture
        .artboard
        .draw_artboard(
            &fixture.file,
            &fixture.graphs.artboards[fixture.graph_index],
            &fixture.graphs.artboards,
            silver,
            &mut renderer,
            &BTreeMap::new(),
            None,
            true,
        )
        .expect("scroll fixture draws");
}

fn advance_and_draw(fixture: &mut Fixture, silver: &mut SerializingFactory, seconds: f32) {
    fixture
        .state_machine
        .advance_and_apply(&mut fixture.artboard, seconds)
        .expect("state machine advances");
    draw(fixture, silver);
}

fn missing_silver_match(_: &str, _: &[u8]) -> bool {
    panic!("nuxie-runtime tests do not own the pinned SRIV comparator")
}

#[test]
#[ignore = "expected-red: pinned scroll_test silver comparator is not wired here"]
fn multiple_scrolling_artboards() {
    let mut fixture = fixture("scroll_test.riv", None, false);
    let mut silver = SerializingFactory::new();
    let (width, height) = fixture.artboard.artboard_dimensions();
    silver.frame_size(width as u32, height as u32);

    advance_and_draw(&mut fixture, &mut silver, 0.1);
    silver.add_frame();

    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, width / 2.0, height / 2.0, 0);
    fixture
        .state_machine
        .advance_and_apply(&mut fixture.artboard, 0.1)
        .expect("transition advance");
    advance_and_draw(&mut fixture, &mut silver, 1.0);
    silver.add_frame();

    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, 260.0, 500.0, 0);
    fixture
        .state_machine
        .advance_and_apply(&mut fixture.artboard, 0.1)
        .expect("right scroll transition advance");
    advance_and_draw(&mut fixture, &mut silver, 1.0);

    let y_movement = 400.0_f32;
    let x_movement = 100.0_f32;
    let frames = (1.0_f32 / 0.016_f32) as usize;
    for i in 0..frames {
        silver.add_frame();
        fixture.state_machine.pointer_move(
            &mut fixture.artboard,
            260.0 - i as f32 * x_movement / frames as f32,
            500.0 - i as f32 * y_movement / frames as f32,
            0.0,
            0,
        );
        fixture
            .state_machine
            .advance_and_apply(&mut fixture.artboard, 0.1)
            .expect("right scroll transition advance");
        advance_and_draw(&mut fixture, &mut silver, 0.016);
    }
    silver.add_frame();
    fixture.state_machine.pointer_up(
        &mut fixture.artboard,
        260.0 - x_movement,
        500.0 - y_movement,
        0,
    );
    fixture
        .state_machine
        .advance_and_apply(&mut fixture.artboard, 0.1)
        .expect("right release transition advance");
    advance_and_draw(&mut fixture, &mut silver, 0.016);

    silver.add_frame();
    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, 50.0, 500.0, 0);
    fixture
        .state_machine
        .advance_and_apply(&mut fixture.artboard, 0.1)
        .expect("left scroll transition advance");
    advance_and_draw(&mut fixture, &mut silver, 1.0);

    for i in 0..frames {
        silver.add_frame();
        fixture.state_machine.pointer_move(
            &mut fixture.artboard,
            50.0 + i as f32 * x_movement / frames as f32,
            500.0 - i as f32 * y_movement / frames as f32,
            0.0,
            0,
        );
        fixture
            .state_machine
            .advance_and_apply(&mut fixture.artboard, 0.1)
            .expect("left scroll transition advance");
        advance_and_draw(&mut fixture, &mut silver, 0.016);
    }
    silver.add_frame();
    fixture.state_machine.pointer_up(
        &mut fixture.artboard,
        50.0 + x_movement,
        500.0 - y_movement,
        0,
    );
    fixture
        .state_machine
        .advance_and_apply(&mut fixture.artboard, 0.1)
        .expect("left release transition advance");
    advance_and_draw(&mut fixture, &mut silver, 0.016);

    assert!(missing_silver_match("scroll_test", &silver.bytes()));
}

fn threshold_case(artboard_name: &str, silver_name: &str) {
    let mut fixture = fixture("scroll_threshold.riv", Some(artboard_name), true);
    let mut silver = SerializingFactory::new();
    let (width, height) = fixture.artboard.artboard_dimensions();
    silver.frame_size(width as u32, height as u32);

    advance_and_draw(&mut fixture, &mut silver, 0.1);
    silver.add_frame();

    let (first_limit, second_limit) = match artboard_name {
        "vertical-scroll" | "horizontal-scroll" => (40.0, 10.0),
        "all-scroll" => (50.0, 32.0),
        _ => unreachable!(),
    };

    let mut pos = 70.0_f32;
    let (x, y) = threshold_point(artboard_name, width, height, pos);
    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, x, y, 0);
    advance_and_draw(&mut fixture, &mut silver, 0.1);
    while pos > first_limit {
        silver.add_frame();
        let (x, y) = threshold_point(artboard_name, width, height, pos);
        fixture
            .state_machine
            .pointer_move(&mut fixture.artboard, x, y, 0.0, 0);
        advance_and_draw(&mut fixture, &mut silver, 0.1);
        pos -= 8.0;
    }
    silver.add_frame();
    let (x, y) = threshold_point(artboard_name, width, height, pos);
    fixture
        .state_machine
        .pointer_up(&mut fixture.artboard, x, y, 0);
    advance_and_draw(&mut fixture, &mut silver, 0.1);

    pos = 70.0;
    let (x, y) = threshold_point(artboard_name, width, height, pos);
    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, x, y, 0);
    advance_and_draw(&mut fixture, &mut silver, 0.1);
    while pos > second_limit {
        silver.add_frame();
        let (x, y) = threshold_point(artboard_name, width, height, pos);
        fixture
            .state_machine
            .pointer_move(&mut fixture.artboard, x, y, 0.0, 0);
        advance_and_draw(&mut fixture, &mut silver, 0.1);
        pos -= 8.0;
    }
    silver.add_frame();
    let (x, y) = threshold_point(artboard_name, width, height, pos);
    fixture
        .state_machine
        .pointer_up(&mut fixture.artboard, x, y, 0);
    advance_and_draw(&mut fixture, &mut silver, 0.1);

    assert!(missing_silver_match(silver_name, &silver.bytes()));
}

fn threshold_point(artboard_name: &str, width: f32, height: f32, pos: f32) -> (f32, f32) {
    match artboard_name {
        "vertical-scroll" => (width / 2.0, pos),
        "horizontal-scroll" => (pos, height / 2.0),
        "all-scroll" => (pos, pos),
        _ => unreachable!(),
    }
}

#[test]
#[ignore = "expected-red: pinned scroll_threshold silver comparator is not wired here"]
fn vertical_scroll_with_threshold() {
    threshold_case("vertical-scroll", "scroll_threshold-vertical-scroll");
}

#[test]
#[ignore = "expected-red: pinned scroll_threshold silver comparator is not wired here"]
fn horizontal_scroll_with_threshold() {
    threshold_case("horizontal-scroll", "scroll_threshold-horizontal-scroll");
}

#[test]
#[ignore = "expected-red: pinned scroll_threshold silver comparator is not wired here"]
fn multidirectional_scroll_with_threshold() {
    threshold_case("all-scroll", "scroll_threshold-all-scroll");
}
