//! One-for-one ports of pinned
//! `tests/unit_tests/runtime/scripting/scripting_canvas_drawing_phase_test.cpp`.

use std::path::PathBuf;

use luaur_rt::Table;
use nuxie_render_api::{Factory, PersistentFactory, RecordingFactory, Renderer};
use nuxie_runtime::{
    NoopScriptHost, ScriptArtboard, ScriptError, ScriptInstance, ScriptMethod, ScriptValue,
    ScriptViewModel,
};
use nuxie_scripting::vm::{CanvasDrawingPhase, ScopedCanvasDrawingPhase, ScriptVm};

mod support;
use support::compile_source;

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

#[derive(Debug, Default)]
struct EmptyScriptArtboard;

impl ScriptArtboard for EmptyScriptArtboard {
    fn width(&self) -> f32 {
        0.0
    }

    fn height(&self) -> f32 {
        0.0
    }

    fn frame_origin(&self) -> bool {
        false
    }

    fn set_width(&mut self, _width: f32) {}

    fn set_height(&mut self, _height: f32) {}

    fn set_frame_origin(&mut self, _frame_origin: bool) {}

    fn instance(
        &self,
        _view_model: Option<ScriptViewModel>,
    ) -> Result<Box<dyn ScriptArtboard>, ScriptError> {
        Ok(Box::new(Self))
    }

    fn draw(
        &mut self,
        _factory: &mut dyn Factory,
        _renderer: &mut dyn Renderer,
    ) -> Result<(), ScriptError> {
        Ok(())
    }
}

#[test]
fn scoped_canvas_drawing_phase_toggles_the_flag_and_restores_it() {
    let context = CanvasDrawingPhase::default();
    assert!(!context.is_active());
    {
        let _phase = ScopedCanvasDrawingPhase::new(Some(&context));
        assert!(context.is_active());
        {
            let _nested = ScopedCanvasDrawingPhase::new(Some(&context));
            assert!(context.is_active());
        }
        assert!(context.is_active());
    }
    assert!(!context.is_active());
}

#[test]
fn scoped_canvas_drawing_phase_tolerates_a_null_context() {
    let _phase = ScopedCanvasDrawingPhase::new(None);
}

#[test]
fn artboard_draw_canvas_is_callable_regardless_of_drawing_phase() {
    let script = "function callDrawCanvas(artboard:Artboard):()\n  artboard:drawCanvas()\nend\n";
    let coin = pinned_fixture("coin.riv");
    let file = nuxie_binary::read_runtime_file(&coin).expect("coin.riv parses");
    assert!(file.artboard(0).is_some());

    let vm = ScriptVm::new();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    vm.install_render_factory(&mut factory).unwrap();
    vm.install_rive_globals().unwrap();
    vm.eval_bytecode::<()>("test_source", &compile_source(script).unwrap())
        .unwrap();
    let table: Table = vm
        .eval_bytecode(
            "test_instance",
            &compile_source("return { update = function(self) callDrawCanvas(self.artboard) end }")
                .unwrap(),
        )
        .unwrap();
    let mut instance = vm.script_instance_from_table(table);
    instance
        .set_artboard_input("artboard", Box::new(EmptyScriptArtboard))
        .unwrap();

    assert!(!vm.canvas_drawing_phase().is_active());
    assert_eq!(
        instance
            .call_method(ScriptMethod::Update, &[], &mut NoopScriptHost)
            .unwrap(),
        ScriptValue::Nil
    );
    {
        let _phase = vm.canvas_drawing_phase().scoped();
        assert!(vm.canvas_drawing_phase().is_active());
        assert_eq!(
            instance
                .call_method(ScriptMethod::Update, &[], &mut NoopScriptHost)
                .unwrap(),
            ScriptValue::Nil
        );
    }
    assert!(!vm.canvas_drawing_phase().is_active());
}
