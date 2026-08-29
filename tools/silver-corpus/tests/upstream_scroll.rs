//! Exact direct Silver ports of all four pinned `scroll_test.cpp` cases.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, SerializingFactory, SerializingRenderer};
use nuxie_runtime::source::math::vec2d::Vec2D;
use nuxie_runtime::{
    File, ImportResult, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    RuntimeStateMachineInstanceHandle,
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
    state_machine: RuntimeStateMachineInstanceHandle,
    artboard: RuntimeArtboardInstanceHandle,
    // Retain the defining File alongside its live native instance.
    _file: RuntimeFileHandle,
}

impl Fixture {
    fn load(
        asset: &str,
        artboard_name: Option<&str>,
        bind_view_model_instance_zero: bool,
        silver: &mut PersistentFactory<SerializingFactory>,
    ) -> Self {
        let retained = RuntimeFactoryHandle::from_factory(silver)
            .expect("explicit retained serializing factory");
        let mut result = ImportResult::Malformed;
        let file = File::import(
            &pinned_fixture(asset),
            retained,
            Some(&mut result),
            None,
            None,
        )
        .unwrap_or_else(|| panic!("{asset} imports: {result:?}"));
        assert_eq!(result, ImportResult::Success);
        let artboard = file
            .with_file(|file| match artboard_name {
                Some(name) => file.artboard_named(name),
                None => file.artboard_default(),
            })
            .expect("pinned artboard instance");
        // Match the pinned setup order: frameSize precedes state-machine creation.
        let (width, height) =
            artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
        silver.borrow_mut().frame_size(width as u32, height as u32);
        let state_machine = artboard
            .state_machine_instance_handle(0)
            .expect("state machine 0");
        if bind_view_model_instance_zero {
            let model_id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
            let view_model = file
                .with_file(|file| file.create_view_model_instance_at(model_id as usize, 0))
                .expect("authored view-model instance zero");
            state_machine
                .with_instance_mut(|machine| machine.bind_view_model_instance_handle(view_model));
        }
        Self {
            _file: file,
            artboard,
            state_machine,
        }
    }

    fn dimensions(&self) -> (f32, f32) {
        self.artboard
            .with_artboard(|artboard| (artboard.width(), artboard.height()))
    }

    fn draw(&self, renderer: &mut SerializingRenderer) {
        self.artboard.draw(renderer);
    }

    fn advance(&self, seconds: f32) {
        self.state_machine.advance_and_apply(seconds);
    }

    fn advance_and_draw(&self, seconds: f32, renderer: &mut SerializingRenderer) {
        self.advance(seconds);
        self.draw(renderer);
    }

    fn pointer_down(&self, x: f32, y: f32) {
        self.state_machine.with_instance_mut(|machine| {
            machine.pointer_down(Vec2D::new(x, y), 0);
        });
    }

    fn pointer_move(&self, x: f32, y: f32) {
        self.state_machine.with_instance_mut(|machine| {
            machine.pointer_move(Vec2D::new(x, y), 0.0, 0);
        });
    }

    fn pointer_up(&self, x: f32, y: f32) {
        self.state_machine.with_instance_mut(|machine| {
            machine.pointer_up(Vec2D::new(x, y), 0);
        });
    }
}

fn assert_matches(silver_name: &str, actual: &PersistentFactory<SerializingFactory>) {
    let expected_path = runtime_root()
        .join("tests/unit_tests/silvers")
        .join(format!("{silver_name}.sriv"));
    let expected_bytes = std::fs::read(&expected_path)
        .unwrap_or_else(|error| panic!("read pinned SRIV {}: {error}", expected_path.display()));
    let expected = parse_sriv(&expected_bytes).expect("parse pinned SRIV");
    let actual = parse_sriv(&actual.borrow().bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("{silver_name}: {difference}"));
}

#[test]
fn wave_c14_scroll_001_multiple_scrolling_artboards() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let fixture = Fixture::load("scroll_test.riv", None, false, &mut silver);

    fixture.advance(0.1);
    let mut renderer = silver.borrow().make_renderer();
    fixture.draw(&mut renderer);
    silver.borrow_mut().add_frame();

    let (width, height) = fixture.dimensions();
    fixture.pointer_down(width / 2.0, height / 2.0);
    fixture.advance(0.1);
    fixture.advance(1.0);
    fixture.draw(&mut renderer);
    silver.borrow_mut().add_frame();

    fixture.pointer_down(260.0, 500.0);
    fixture.advance(0.1);
    fixture.advance(1.0);
    fixture.draw(&mut renderer);

    let y_movement = 400.0_f32;
    let x_movement = 100.0_f32;
    let frames = (1.0_f32 / 0.016_f32) as usize;
    for i in 0..frames {
        silver.borrow_mut().add_frame();
        fixture.pointer_move(
            260.0 - i as f32 * x_movement / frames as f32,
            500.0 - i as f32 * y_movement / frames as f32,
        );
        fixture.advance(0.1);
        fixture.advance(0.016);
        fixture.draw(&mut renderer);
    }
    silver.borrow_mut().add_frame();
    fixture.pointer_up(260.0 - x_movement, 500.0 - y_movement);
    fixture.advance(0.1);
    fixture.advance(0.016);
    fixture.draw(&mut renderer);

    silver.borrow_mut().add_frame();
    fixture.pointer_down(50.0, 500.0);
    fixture.advance(0.1);
    fixture.advance(1.0);
    fixture.draw(&mut renderer);

    for i in 0..frames {
        silver.borrow_mut().add_frame();
        fixture.pointer_move(
            50.0 + i as f32 * x_movement / frames as f32,
            500.0 - i as f32 * y_movement / frames as f32,
        );
        fixture.advance(0.1);
        fixture.advance(0.016);
        fixture.draw(&mut renderer);
    }
    silver.borrow_mut().add_frame();
    fixture.pointer_up(50.0 + x_movement, 500.0 - y_movement);
    fixture.advance(0.1);
    fixture.advance(0.016);
    fixture.draw(&mut renderer);

    assert_matches("scroll_test", &silver);
}

#[test]
fn wave_c14_scroll_002_vertical_scroll_with_threshold() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let fixture = Fixture::load(
        "scroll_threshold.riv",
        Some("vertical-scroll"),
        true,
        &mut silver,
    );
    fixture.advance(0.1);
    let mut renderer = silver.borrow().make_renderer();
    fixture.draw(&mut renderer);
    silver.borrow_mut().add_frame();

    let mut pos = 70.0_f32;
    fixture.pointer_down(fixture.dimensions().0 / 2.0, pos);
    fixture.advance_and_draw(0.1, &mut renderer);
    while pos > 40.0 {
        silver.borrow_mut().add_frame();
        fixture.pointer_move(fixture.dimensions().0 / 2.0, pos);
        fixture.advance_and_draw(0.1, &mut renderer);
        pos -= 8.0;
    }
    silver.borrow_mut().add_frame();
    fixture.pointer_up(fixture.dimensions().0 / 2.0, pos);
    fixture.advance_and_draw(0.1, &mut renderer);

    pos = 70.0;
    fixture.pointer_down(fixture.dimensions().0 / 2.0, pos);
    fixture.advance_and_draw(0.1, &mut renderer);
    while pos > 10.0 {
        silver.borrow_mut().add_frame();
        fixture.pointer_move(fixture.dimensions().0 / 2.0, pos);
        fixture.advance_and_draw(0.1, &mut renderer);
        pos -= 8.0;
    }
    silver.borrow_mut().add_frame();
    fixture.pointer_up(fixture.dimensions().0 / 2.0, pos);
    fixture.advance_and_draw(0.1, &mut renderer);

    assert_matches("scroll_threshold-vertical-scroll", &silver);
}

#[test]
fn wave_c14_scroll_003_horizontal_scroll_with_threshold() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let fixture = Fixture::load(
        "scroll_threshold.riv",
        Some("horizontal-scroll"),
        true,
        &mut silver,
    );
    fixture.advance(0.1);
    let mut renderer = silver.borrow().make_renderer();
    fixture.draw(&mut renderer);
    silver.borrow_mut().add_frame();

    let mut pos = 70.0_f32;
    fixture.pointer_down(pos, fixture.dimensions().1 / 2.0);
    fixture.advance_and_draw(0.1, &mut renderer);
    while pos > 40.0 {
        silver.borrow_mut().add_frame();
        fixture.pointer_move(pos, fixture.dimensions().1 / 2.0);
        fixture.advance_and_draw(0.1, &mut renderer);
        pos -= 8.0;
    }
    silver.borrow_mut().add_frame();
    fixture.pointer_up(pos, fixture.dimensions().1 / 2.0);
    fixture.advance_and_draw(0.1, &mut renderer);

    pos = 70.0;
    fixture.pointer_down(pos, fixture.dimensions().1 / 2.0);
    fixture.advance_and_draw(0.1, &mut renderer);
    while pos > 10.0 {
        silver.borrow_mut().add_frame();
        fixture.pointer_move(pos, fixture.dimensions().1 / 2.0);
        fixture.advance_and_draw(0.1, &mut renderer);
        pos -= 8.0;
    }
    silver.borrow_mut().add_frame();
    fixture.pointer_up(pos, fixture.dimensions().1 / 2.0);
    fixture.advance_and_draw(0.1, &mut renderer);

    assert_matches("scroll_threshold-horizontal-scroll", &silver);
}

#[test]
fn wave_c14_scroll_004_multidirectional_scroll_with_threshold() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let fixture = Fixture::load(
        "scroll_threshold.riv",
        Some("all-scroll"),
        true,
        &mut silver,
    );

    fixture.advance(0.1);
    let mut renderer = silver.borrow().make_renderer();
    fixture.draw(&mut renderer);
    silver.borrow_mut().add_frame();

    let mut pos = 70.0_f32;
    fixture.pointer_down(pos, pos);
    fixture.advance_and_draw(0.1, &mut renderer);
    while pos > 50.0 {
        silver.borrow_mut().add_frame();
        fixture.pointer_move(pos, pos);
        fixture.advance_and_draw(0.1, &mut renderer);
        pos -= 8.0;
    }
    silver.borrow_mut().add_frame();
    fixture.pointer_up(pos, pos);
    fixture.advance_and_draw(0.1, &mut renderer);

    pos = 70.0;
    fixture.pointer_down(pos, pos);
    fixture.advance_and_draw(0.1, &mut renderer);
    while pos > 32.0 {
        silver.borrow_mut().add_frame();
        fixture.pointer_move(pos, pos);
        fixture.advance_and_draw(0.1, &mut renderer);
        pos -= 8.0;
    }
    silver.borrow_mut().add_frame();
    fixture.pointer_up(pos, pos);
    fixture.advance_and_draw(0.1, &mut renderer);

    assert_matches("scroll_threshold-all-scroll", &silver);
}
