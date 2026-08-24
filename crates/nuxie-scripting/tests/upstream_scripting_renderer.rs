//! One-for-one ports of `tests/unit_tests/runtime/scripting/scripting_renderer_test.cpp`.
#![cfg(feature = "luau")]

use luaur_rt::{Function, Table};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{NoopScriptHost, ScriptInstance};
use nuxie_scripting::vm::ScriptVm;

mod support;
use support::ScriptVmSourceTestExt as _;
use support::compile_source;

fn configured_vm() -> (ScriptVm, PersistentFactory<RecordingFactory>) {
    let vm = ScriptVm::new();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    vm.install_render_factory(&mut factory).unwrap();
    vm.install_rive_globals().unwrap();
    (vm, factory)
}

fn run_named(vm: &ScriptVm, name: &str, source: &str) {
    vm.eval_bytecode::<()>(name, &compile_source(source).unwrap())
        .unwrap();
}

fn draw_table(vm: &ScriptVm) -> Table {
    vm.eval("return { draw = function(self, renderer) return render(renderer) end }")
        .unwrap()
}

#[test]
#[ignore = "expected red: retained-renderer error loses the pinned source line and punctuation"]
fn can_call_renderer() {
    const SOURCE: &str = "local storedRenderer:Renderer\n\
function render(renderer:Renderer):()\n\
  storedRenderer = renderer\n\
  local path:Path = Path.new()\n\
  local paint:Paint = Paint.new()\n\
  renderer:drawPath(path, paint)\n\
end\n\
function afterwards(): ()\n\
  storedRenderer:save()\n\
end";

    let (vm, mut factory) = configured_vm();
    run_named(&vm, "test_source", SOURCE);
    let mut instance = vm.script_instance_from_table(draw_table(&vm));
    let mut renderer = factory.borrow().make_renderer();
    instance
        .call_draw(&mut factory, &mut renderer, &mut NoopScriptHost)
        .unwrap();

    let afterwards: Function = vm.lua().globals().get("afterwards").unwrap();
    let error = afterwards.call::<()>(()).unwrap_err().to_string();
    assert_eq!(
        error,
        "runtime error: test_source:9: Renderer is no longer valid."
    );
}

fn missing_renderer_end_balance_result() -> bool {
    panic!("Rust ScriptedRenderer cleanup does not expose upstream end()'s balance result")
}

#[test]
#[ignore = "expected red: ScriptedRenderer cleanup does not expose end() balance status"]
fn renderer_checks_its_balanced() {
    const SOURCE: &str = "function render(renderer:Renderer):()\n\
  local path:Path = Path.new()\n\
  local paint:Paint = Paint.new()\n\
  renderer:save()\n\
  renderer:drawPath(path, paint)\n\
  renderer:save()\n\
end\n";

    let (vm, mut factory) = configured_vm();
    run_named(&vm, "test_source", SOURCE);
    let mut instance = vm.script_instance_from_table(draw_table(&vm));
    let mut renderer = factory.borrow().make_renderer();
    instance
        .call_draw(&mut factory, &mut renderer, &mut NoopScriptHost)
        .unwrap();
    assert!(!missing_renderer_end_balance_result());
}

const ADD_OVAL_SOURCE: &str = r#"function addOval(path: Path, x: number, y: number, width: number, height: number)
	local c: number = 0.5519150244935105707435627
	local unit: { Vector } = {
		Vector.xy(1, 0),
		Vector.xy(1, c),
		Vector.xy(c, 1), -- quadrant 1 ( 4:30)
		Vector.xy(0, 1),
		Vector.xy(-c, 1),
		Vector.xy(-1, c), -- quadrant 2 ( 7:30)
		Vector.xy(-1, 0),
		Vector.xy(-1, -c),
		Vector.xy(-c, -1), -- quadrant 3 (10:30)
		Vector.xy(0, -1),
		Vector.xy(c, -1),
		Vector.xy(1, -c), -- quadrant 4 ( 1:30)
		Vector.xy(1, 0),
	}

	local dx: number = x - width / 2
	local dy: number = y - height / 2
	local sx: number = width * 0.5
	local sy: number = height * 0.5

	local map = function(p: Vector): Vector
		return Vector.xy(p.x * sx + dx, p.y * sy + dy)
	end
	path:moveTo(map(unit[1]))
	for i = 1, 12, 3 do
		path:cubicTo(map(unit[i + 1]), map(unit[i + 2]), map(unit[i + 3]))
	end
	path:close()
end
"#;

fn missing_silver_match(_: &str, _: &str) -> bool {
    panic!("recording renderer has no pinned C++ silver matcher for this case")
}

#[test]
#[ignore = "expected red: pinned scripted_oval silver matcher is not wired"]
fn renderer_can_draw_an_oval() {
    let source = format!(
        "{ADD_OVAL_SOURCE}\n\
         function render(renderer: Renderer): ()\n\
         \tlocal path: Path = Path.new()\n\
         \tlocal paint: Paint = Paint.with({{color=0xFFFF0000, feather=20}})\n\
         \taddOval(path, 600, 500, 100, 180)\n\
         \trenderer:drawPath(path, paint)\n\
         end\n"
    );
    let (vm, mut factory) = configured_vm();
    run_named(&vm, "test_source", &source);
    let mut instance = vm.script_instance_from_table(draw_table(&vm));
    let mut renderer = factory.borrow().make_renderer();
    instance
        .call_draw(&mut factory, &mut renderer, &mut NoopScriptHost)
        .unwrap();
    assert!(missing_silver_match(
        "scripted_oval",
        &factory.borrow().stream()
    ));
}

fn missing_stack_top_check() {
    panic!("safe luaur wrapper does not expose the pinned raw lua stack-top check")
}

#[test]
#[ignore = "expected red: animated silver and raw stack-top seams are not wired"]
fn renderer_can_draw_and_animate_oval() {
    let source = format!(
        "{ADD_OVAL_SOURCE}\n\
         local path: Path = Path.new()\n\
         local rotation:number = 0\n\
         local paint: Paint = Paint.with({{color=0xFFFF0000, feather=20}})\n\
         function advance(seconds:number): boolean\n\
             path:reset()\n\
             addOval(path, 600, 500, 100, 180)\n\
             rotation += seconds\n\
             return true\n\
         end\n\
         function render(renderer: Renderer): ()\n\
         \trenderer:save()\n\
             renderer:transform(Mat2D.withRotation(rotation))\n\
         \trenderer:drawPath(path, paint)\n\
              renderer:restore()\n\
         end\n"
    );
    let (vm, mut factory) = configured_vm();
    run_named(&vm, "test_source", &source);
    let advance: Function = vm.lua().globals().get("advance").unwrap();
    let mut instance = vm.script_instance_from_table(draw_table(&vm));
    let elapsed_seconds = 1.0_f32 / 60.0;

    for frame in 0..1000 {
        if frame != 0 {
            factory.borrow_mut().add_frame();
        }
        assert!(advance.call::<bool>(elapsed_seconds).unwrap());
        let mut renderer = factory.borrow().make_renderer();
        instance
            .call_draw(&mut factory, &mut renderer, &mut NoopScriptHost)
            .unwrap();
        missing_stack_top_check();
        vm.eval::<()>("collectgarbage('collect')").unwrap();
    }

    assert!(missing_silver_match(
        "scripted_animated_oval",
        &factory.borrow().stream()
    ));
}
