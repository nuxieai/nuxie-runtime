//! Direct ports of pinned `runtime/scripting/scripting_artboard_test.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::runtime::math::vec2d::Vec2D;
use nuxie::{
    FileImportLimits, PersistentFactory, RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle, RuntimeViewModelInstanceHandle, ScriptExecutionLimits,
    ScriptedFile, ViewModelInstanceRuntime, import_unsigned_scripted,
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
    let actual = parse_sriv(actual).expect("valid Rust SRIV");
    let expected = parse_sriv(&pinned(&format!("silvers/{name}.sriv"))).expect("pinned SRIV");
    compare_sriv(&expected, &actual).unwrap_or_else(|difference| panic!("{name}: {difference}"));
}

struct Fixture {
    _file: ScriptedFile,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    view_model: Option<RuntimeViewModelInstanceHandle>,
    silver: PersistentFactory<SerializingFactory>,
}

#[derive(Clone, Copy)]
enum ViewModelKind {
    None,
    Fresh,
    Authored,
    Default,
}

impl Fixture {
    fn new(asset: &str, artboard_name: Option<&str>, kind: ViewModelKind) -> Self {
        let mut silver = PersistentFactory::new(SerializingFactory::new());
        let file = import_unsigned_scripted(
            &pinned(&format!("assets/{asset}")),
            &mut silver,
            None,
            FileImportLimits::new(),
            ScriptExecutionLimits::new(),
        )
        .unwrap_or_else(|error| panic!("{asset} imports: {error:#}"));
        let artboard = file
            .native_file()
            .with_file(|file| match artboard_name {
                Some(name) => file.artboard_named(name),
                None => file.artboard_default(),
            })
            .expect("artboard instance");
        let (width, height) =
            artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
        silver.borrow_mut().frame_size(width as u32, height as u32);
        let machine = artboard.state_machine_at(0).expect("state machine 0");
        let view_model = match kind {
            ViewModelKind::None => None,
            kind => file
                .native_file()
                .with_file_mut(|native| match kind {
                    ViewModelKind::Fresh => {
                        native.create_view_model_instance_for_artboard(artboard.core_handle())
                    }
                    ViewModelKind::Authored => {
                        let id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
                        if id == u32::MAX {
                            native.create_view_model_instance_for_artboard(artboard.core_handle())
                        } else {
                            native.create_view_model_instance_at(id as usize, 0)
                        }
                    }
                    ViewModelKind::Default => native
                        .create_default_view_model_instance_for_artboard(artboard.core_handle()),
                    ViewModelKind::None => unreachable!(),
                })
                .map(ViewModelInstanceRuntime::new)
                .map(ViewModelInstanceRuntime::into_handle),
        };
        if let Some(view_model) = &view_model {
            machine.with_instance_mut(|machine| {
                machine.bind_view_model_instance(view_model.instance())
            });
        }
        Self {
            _file: file,
            artboard,
            machine,
            view_model,
            silver,
        }
    }

    fn advance(&self, seconds: f32) {
        self.machine.advance_and_apply(seconds);
    }

    fn draw(&mut self) {
        let mut renderer = self.silver.borrow().make_renderer();
        self.artboard.draw(&mut renderer);
    }

    fn click(&self, x: f32, y: f32) {
        self.machine
            .with_instance_mut(|machine| machine.pointer_down(Vec2D::new(x, y), 0));
        self.machine
            .with_instance_mut(|machine| machine.pointer_up(Vec2D::new(x, y), 0));
    }

    fn view_model(&self) -> &RuntimeViewModelInstanceHandle {
        self.view_model.as_ref().expect("fixture view model")
    }
}

fn sixty_frame_artboard_silver(asset: &str, silver: &str) {
    let mut fixture = Fixture::new(asset, Some("Artboard"), ViewModelKind::None);
    fixture.advance(0.1);
    fixture.draw();
    for _ in 0..60 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.016);
        fixture.draw();
    }
    compare_silver(silver, &fixture.silver.borrow().bytes());
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
fn script_node_advance_affects_did_change_via_dirt() {
    let mut fixture = Fixture::new(
        "script_affects_has_changed.riv",
        Some("Main"),
        ViewModelKind::Fresh,
    );
    fixture.advance(0.1);
    assert!(
        fixture
            .artboard
            .with_artboard(|artboard| artboard.did_change())
    );
    fixture.draw();
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(1.0);
    assert!(
        !fixture
            .artboard
            .with_artboard(|artboard| artboard.did_change())
    );
    fixture.draw();
    fixture
        .view_model()
        .property_boolean("toLeft")
        .expect("toLeft")
        .set_value(true);
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    assert!(
        fixture
            .artboard
            .with_artboard(|artboard| artboard.did_change())
    );
    fixture.draw();
    fixture.silver.borrow_mut().add_frame();
    fixture.advance(0.1);
    assert!(
        !fixture
            .artboard
            .with_artboard(|artboard| artboard.did_change())
    );
    fixture.draw();
    compare_silver(
        "script_affects_has_changed",
        &fixture.silver.borrow().bytes(),
    );
}

fn double_advance(fixture: &Fixture) {
    fixture.advance(0.016);
    fixture.advance(0.016);
}

#[test]
fn script_instance_linear_animations() {
    let mut fixture = Fixture::new(
        "scripting_linear_animation.riv",
        Some("Main"),
        ViewModelKind::Authored,
    );
    fixture.advance(0.1);
    fixture.draw();
    for _ in 0..60 {
        fixture.silver.borrow_mut().add_frame();
        fixture.advance(0.064);
        fixture.draw();
    }
    let time_property = fixture.view_model().property_number("time").expect("time");
    let mode_property = fixture.view_model().property_string("mode").expect("mode");
    for time in [0.55, -1.0, 3.8, 40.0] {
        time_property.set_value(time);
        double_advance(&fixture);
        fixture.silver.borrow_mut().add_frame();
        fixture.draw();
        mode_property.set_value("frames");
        double_advance(&fixture);
        fixture.silver.borrow_mut().add_frame();
        fixture.draw();
        mode_property.set_value("percentage");
        double_advance(&fixture);
        fixture.silver.borrow_mut().add_frame();
        fixture.draw();
    }
    compare_silver(
        "scripting_linear_animation",
        &fixture.silver.borrow().bytes(),
    );
}

#[test]
fn script_instances_artboard_with_opacity_applied() {
    sixty_frame_artboard_silver(
        "script_artboard_opacity_test.riv",
        "script_artboards_opacity",
    );
}

#[test]
fn view_model_source_cache_is_cleared_when_instance_changes() {
    let mut fixture = Fixture::new("scripted_viewmodel_cache.riv", None, ViewModelKind::Default);
    fixture.advance(0.016);
    fixture.draw();
    fixture.silver.borrow_mut().add_frame();
    fixture.click(450.0, 50.0);
    fixture.advance(0.016);
    fixture
        .view_model()
        .property_trigger("createInstance")
        .expect("createInstance")
        .trigger();
    fixture.advance(0.016);
    fixture.draw();
    fixture.silver.borrow_mut().add_frame();
    fixture.click(450.0, 150.0);
    fixture.advance(0.016);
    fixture.draw();
    fixture.silver.borrow_mut().add_frame();
    fixture
        .view_model()
        .property_trigger("createInstance")
        .expect("createInstance")
        .trigger();
    fixture.advance(0.016);
    fixture.draw();
    compare_silver("scripted_viewmodel_cache", &fixture.silver.borrow().bytes());
}
