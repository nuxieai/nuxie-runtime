//! One-for-one ports of `tests/unit_tests/runtime/scripting/scripting_renderer_test.cpp`.
#![cfg(all(feature = "luau", feature = "upstream-test-seams"))]

use luaur_rt::{Function, Table};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
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
    let table = draw_table(&vm);
    let mut renderer = factory.borrow().make_renderer();
    let balanced = vm
        .upstream_test_call_draw_with_balance(&table, &mut factory, &mut renderer)
        .unwrap();
    assert!(balanced);

    let afterwards: Function = vm.lua().globals().get("afterwards").unwrap();
    let error = afterwards.call::<()>(()).unwrap_err().to_string();
    assert_eq!(
        error,
        "runtime error: test_source:9: Renderer is no longer valid."
    );
}
