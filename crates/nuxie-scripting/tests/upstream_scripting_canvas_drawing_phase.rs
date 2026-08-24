//! One-for-one expected-red ports of pinned
//! `tests/unit_tests/runtime/scripting/scripting_canvas_drawing_phase_test.cpp`.

use std::path::PathBuf;

#[derive(Debug, Default)]
struct ScriptingContext;

fn canvas_drawing_phase(_: &ScriptingContext) -> bool {
    missing_canvas_drawing_phase_owner()
}

fn with_scoped_canvas_drawing_phase<F>(_: Option<&mut ScriptingContext>, _: F)
where
    F: FnOnce(&mut ScriptingContext),
{
    missing_canvas_drawing_phase_owner()
}

fn call_draw_canvas(_: &str, _: &[u8], _: bool) -> bool {
    missing_canvas_drawing_phase_owner()
}

fn missing_canvas_drawing_phase_owner() -> ! {
    panic!("Rust scripting has no canvasDrawingPhase state/scoped-guard owner")
}

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

#[test]
#[ignore = "expected-red: Rust scripting has no canvasDrawingPhase/scoped-guard owner"]
fn scoped_canvas_drawing_phase_toggles_the_flag_and_restores_it() {
    let mut context = ScriptingContext;
    assert!(!canvas_drawing_phase(&context));
    with_scoped_canvas_drawing_phase(Some(&mut context), |context| {
        assert!(canvas_drawing_phase(context));
        with_scoped_canvas_drawing_phase(Some(context), |context| {
            assert!(canvas_drawing_phase(context));
        });
        assert!(canvas_drawing_phase(context));
    });
    assert!(!canvas_drawing_phase(&context));
}

#[test]
#[ignore = "expected-red: Rust scripting has no nullable scoped canvas-drawing guard owner"]
fn scoped_canvas_drawing_phase_tolerates_a_null_context() {
    with_scoped_canvas_drawing_phase(None, |_| {});
}

#[test]
#[ignore = "expected-red: Rust scripting has no canvasDrawingPhase/drawCanvas binding owner"]
fn artboard_draw_canvas_is_callable_regardless_of_drawing_phase() {
    let script = "function callDrawCanvas(artboard:Artboard):()\n  artboard:drawCanvas()\nend\n";
    let coin = pinned_fixture("coin.riv");
    let mut context = ScriptingContext;
    assert!(!canvas_drawing_phase(&context));
    assert!(call_draw_canvas(script, &coin, false));
    with_scoped_canvas_drawing_phase(Some(&mut context), |context| {
        assert!(canvas_drawing_phase(context));
        assert!(call_draw_canvas(script, &coin, true));
    });
    assert!(!canvas_drawing_phase(&context));
}
