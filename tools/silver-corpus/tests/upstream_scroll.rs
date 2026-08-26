//! Exact direct Silver ports of all four pinned `scroll_test.cpp` cases.

use std::collections::BTreeMap;
use std::path::PathBuf;

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::GraphFile;
use nuxie_render_api::{SerializingFactory, SerializingRenderer};
use nuxie_runtime::{
    ArtboardInstance, RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance,
    StateMachineInstance,
};
use silver_corpus::{compare_sriv, parse_sriv};

fn runtime_root() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

fn pinned_fixture(name: &str) -> Vec<u8> {
    let path = runtime_root().join("tests/unit_tests/assets").join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

struct Fixture {
    file: RuntimeFile,
    graphs: GraphFile,
    graph_index: usize,
    artboard: ArtboardInstance,
    state_machine: StateMachineInstance,
}

impl Fixture {
    fn load(
        asset: &str,
        artboard_name: Option<&str>,
        bind_view_model_instance_zero: bool,
        silver: &mut SerializingFactory,
    ) -> Self {
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
        let mut artboard =
            ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
                .unwrap_or_else(|error| panic!("{asset} artboard instantiates: {error:#}"));
        artboard
            .initialize_artboard_renderer(
                &file,
                graph,
                &graphs.artboards,
                &BTreeMap::new(),
                silver,
                None,
            )
            .unwrap_or_else(|error| panic!("{asset} renderer initializes: {error:#}"));
        let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");

        if bind_view_model_instance_zero {
            let view_model_index = usize::try_from(
                file.artboard(graph_index)
                    .and_then(|artboard| artboard.uint_property("viewModelId"))
                    .expect("artboard viewModelId"),
            )
            .expect("viewModelId fits usize");
            let view_model = RuntimeOwnedViewModelHandle::new(
                RuntimeOwnedViewModelInstance::from_instance(&file, view_model_index, 0)
                    .expect("view-model instance zero builds"),
            );
            state_machine.bind_owned_view_model_handle(&view_model);
        }

        Self {
            file,
            graphs,
            graph_index,
            artboard,
            state_machine,
        }
    }

    fn draw(&mut self, silver: &mut SerializingFactory, renderer: &mut SerializingRenderer) {
        self.artboard
            .draw_artboard(
                &self.file,
                &self.graphs.artboards[self.graph_index],
                &self.graphs.artboards,
                silver,
                renderer,
                &BTreeMap::new(),
                None,
                true,
            )
            .expect("scroll fixture draws");
    }

    fn advance(&mut self, seconds: f32) {
        self.state_machine
            .advance_and_apply(&mut self.artboard, seconds)
            .expect("state machine advances");
    }

    fn advance_and_draw(
        &mut self,
        seconds: f32,
        silver: &mut SerializingFactory,
        renderer: &mut SerializingRenderer,
    ) {
        self.advance(seconds);
        self.draw(silver, renderer);
    }
}

fn set_frame_size(fixture: &Fixture, silver: &mut SerializingFactory) {
    let (width, height) = fixture.artboard.artboard_dimensions();
    silver.frame_size(width as u32, height as u32);
}

fn assert_matches(silver_name: &str, actual: &SerializingFactory) {
    let expected_path = runtime_root()
        .join("tests/unit_tests/silvers")
        .join(format!("{silver_name}.sriv"));
    let expected_bytes = std::fs::read(&expected_path)
        .unwrap_or_else(|error| panic!("read pinned SRIV {}: {error}", expected_path.display()));
    let expected = parse_sriv(&expected_bytes).expect("parse pinned SRIV");
    let actual = parse_sriv(&actual.bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("{silver_name}: {difference}"));
}

#[test]
#[ignore = "expected-red: exact scroll_test SRIV diverges at frame 0/op53 transform.xy (-0.0 vs 0)"]
fn wave_c14_scroll_001_multiple_scrolling_artboards() {
    let mut silver = SerializingFactory::new();
    let mut fixture = Fixture::load("scroll_test.riv", None, false, &mut silver);
    set_frame_size(&fixture, &mut silver);
    let mut renderer = silver.make_renderer();

    fixture.advance(0.1);
    fixture.draw(&mut silver, &mut renderer);
    silver.add_frame();

    let (width, height) = fixture.artboard.artboard_dimensions();
    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, width / 2.0, height / 2.0, 0);
    fixture.advance(0.1);
    fixture.advance(1.0);
    fixture.draw(&mut silver, &mut renderer);
    silver.add_frame();

    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, 260.0, 500.0, 0);
    fixture.advance(0.1);
    fixture.advance(1.0);
    fixture.draw(&mut silver, &mut renderer);

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
        fixture.advance(0.1);
        fixture.advance(0.016);
        fixture.draw(&mut silver, &mut renderer);
    }
    silver.add_frame();
    fixture.state_machine.pointer_up(
        &mut fixture.artboard,
        260.0 - x_movement,
        500.0 - y_movement,
        0,
    );
    fixture.advance(0.1);
    fixture.advance(0.016);
    fixture.draw(&mut silver, &mut renderer);

    silver.add_frame();
    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, 50.0, 500.0, 0);
    fixture.advance(0.1);
    fixture.advance(1.0);
    fixture.draw(&mut silver, &mut renderer);

    for i in 0..frames {
        silver.add_frame();
        fixture.state_machine.pointer_move(
            &mut fixture.artboard,
            50.0 + i as f32 * x_movement / frames as f32,
            500.0 - i as f32 * y_movement / frames as f32,
            0.0,
            0,
        );
        fixture.advance(0.1);
        fixture.advance(0.016);
        fixture.draw(&mut silver, &mut renderer);
    }
    silver.add_frame();
    fixture.state_machine.pointer_up(
        &mut fixture.artboard,
        50.0 + x_movement,
        500.0 - y_movement,
        0,
    );
    fixture.advance(0.1);
    fixture.advance(0.016);
    fixture.draw(&mut silver, &mut renderer);

    assert_matches("scroll_test", &silver);
}

#[test]
#[ignore = "expected-red: exact vertical-threshold SRIV diverges at frame 0/op69 transform.xy (-0.0 vs 0)"]
fn wave_c14_scroll_002_vertical_scroll_with_threshold() {
    let mut silver = SerializingFactory::new();
    let mut fixture = Fixture::load(
        "scroll_threshold.riv",
        Some("vertical-scroll"),
        true,
        &mut silver,
    );
    set_frame_size(&fixture, &mut silver);
    let mut renderer = silver.make_renderer();
    let (width, _) = fixture.artboard.artboard_dimensions();

    fixture.advance(0.1);
    fixture.draw(&mut silver, &mut renderer);
    silver.add_frame();

    let mut pos = 70.0_f32;
    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, width / 2.0, pos, 0);
    fixture.advance_and_draw(0.1, &mut silver, &mut renderer);
    while pos > 40.0 {
        silver.add_frame();
        fixture
            .state_machine
            .pointer_move(&mut fixture.artboard, width / 2.0, pos, 0.0, 0);
        fixture.advance_and_draw(0.1, &mut silver, &mut renderer);
        pos -= 8.0;
    }
    silver.add_frame();
    fixture
        .state_machine
        .pointer_up(&mut fixture.artboard, width / 2.0, pos, 0);
    fixture.advance_and_draw(0.1, &mut silver, &mut renderer);

    pos = 70.0;
    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, width / 2.0, pos, 0);
    fixture.advance_and_draw(0.1, &mut silver, &mut renderer);
    while pos > 10.0 {
        silver.add_frame();
        fixture
            .state_machine
            .pointer_move(&mut fixture.artboard, width / 2.0, pos, 0.0, 0);
        fixture.advance_and_draw(0.1, &mut silver, &mut renderer);
        pos -= 8.0;
    }
    silver.add_frame();
    fixture
        .state_machine
        .pointer_up(&mut fixture.artboard, width / 2.0, pos, 0);
    fixture.advance_and_draw(0.1, &mut silver, &mut renderer);

    assert_matches("scroll_threshold-vertical-scroll", &silver);
}

#[test]
#[ignore = "expected-red: exact horizontal-threshold SRIV diverges at frame 0/op79 transform.xy (-0.0 vs 0)"]
fn wave_c14_scroll_003_horizontal_scroll_with_threshold() {
    let mut silver = SerializingFactory::new();
    let mut fixture = Fixture::load(
        "scroll_threshold.riv",
        Some("horizontal-scroll"),
        true,
        &mut silver,
    );
    set_frame_size(&fixture, &mut silver);
    let mut renderer = silver.make_renderer();
    let (_, height) = fixture.artboard.artboard_dimensions();

    fixture.advance(0.1);
    fixture.draw(&mut silver, &mut renderer);
    silver.add_frame();

    let mut pos = 70.0_f32;
    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, pos, height / 2.0, 0);
    fixture.advance_and_draw(0.1, &mut silver, &mut renderer);
    while pos > 40.0 {
        silver.add_frame();
        fixture
            .state_machine
            .pointer_move(&mut fixture.artboard, pos, height / 2.0, 0.0, 0);
        fixture.advance_and_draw(0.1, &mut silver, &mut renderer);
        pos -= 8.0;
    }
    silver.add_frame();
    fixture
        .state_machine
        .pointer_up(&mut fixture.artboard, pos, height / 2.0, 0);
    fixture.advance_and_draw(0.1, &mut silver, &mut renderer);

    pos = 70.0;
    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, pos, height / 2.0, 0);
    fixture.advance_and_draw(0.1, &mut silver, &mut renderer);
    while pos > 10.0 {
        silver.add_frame();
        fixture
            .state_machine
            .pointer_move(&mut fixture.artboard, pos, height / 2.0, 0.0, 0);
        fixture.advance_and_draw(0.1, &mut silver, &mut renderer);
        pos -= 8.0;
    }
    silver.add_frame();
    fixture
        .state_machine
        .pointer_up(&mut fixture.artboard, pos, height / 2.0, 0);
    fixture.advance_and_draw(0.1, &mut silver, &mut renderer);

    assert_matches("scroll_threshold-horizontal-scroll", &silver);
}

#[test]
#[ignore = "expected-red: exact multidirectional-threshold SRIV diverges at frame 0/op82 transform.xy (-0.0 vs 0)"]
fn wave_c14_scroll_004_multidirectional_scroll_with_threshold() {
    let mut silver = SerializingFactory::new();
    let mut fixture = Fixture::load(
        "scroll_threshold.riv",
        Some("all-scroll"),
        true,
        &mut silver,
    );
    set_frame_size(&fixture, &mut silver);
    let mut renderer = silver.make_renderer();

    fixture.advance(0.1);
    fixture.draw(&mut silver, &mut renderer);
    silver.add_frame();

    let mut pos = 70.0_f32;
    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, pos, pos, 0);
    fixture.advance_and_draw(0.1, &mut silver, &mut renderer);
    while pos > 50.0 {
        silver.add_frame();
        fixture
            .state_machine
            .pointer_move(&mut fixture.artboard, pos, pos, 0.0, 0);
        fixture.advance_and_draw(0.1, &mut silver, &mut renderer);
        pos -= 8.0;
    }
    silver.add_frame();
    fixture
        .state_machine
        .pointer_up(&mut fixture.artboard, pos, pos, 0);
    fixture.advance_and_draw(0.1, &mut silver, &mut renderer);

    pos = 70.0;
    fixture
        .state_machine
        .pointer_down(&mut fixture.artboard, pos, pos, 0);
    fixture.advance_and_draw(0.1, &mut silver, &mut renderer);
    while pos > 32.0 {
        silver.add_frame();
        fixture
            .state_machine
            .pointer_move(&mut fixture.artboard, pos, pos, 0.0, 0);
        fixture.advance_and_draw(0.1, &mut silver, &mut renderer);
        pos -= 8.0;
    }
    silver.add_frame();
    fixture
        .state_machine
        .pointer_up(&mut fixture.artboard, pos, pos, 0);
    fixture.advance_and_draw(0.1, &mut silver, &mut renderer);

    assert_matches("scroll_threshold-all-scroll", &silver);
}
