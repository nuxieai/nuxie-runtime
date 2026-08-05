#![cfg(feature = "luau")]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{NoopScriptHost, ScriptInstance};
use nuxie_scripting::vm::ScriptVm;

#[test]
fn dirty_shared_path_rebuild_across_callbacks_in_one_frame_uses_a_fresh_render_path() {
    let vm = ScriptVm::new();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    vm.install_render_factory(&mut factory).unwrap();
    vm.install_rive_globals().unwrap();
    let chunk = vm
        .load(
            "same-frame-path-rebuild",
            "local path = Path.new()\n\
               path:moveTo(Vector(0, 0))\n\
               path:lineTo(Vector(10, 0))\n\
               local paint = Paint.new()\n\
               return function(_)\n\
                 return {\n\
                   draw = function(self, renderer)\n\
                     renderer:drawPath(path, paint)\n\
                     path:lineTo(Vector(10, 20))\n\
                     renderer:drawPath(path, paint)\n\
                   end,\n\
                 }\n\
               end",
        )
        .unwrap();
    let generator: luaur_rt::Function = chunk.call(()).unwrap();
    let first_table: luaur_rt::Table = generator.call(luaur_rt::Value::Nil).unwrap();
    let second_table: luaur_rt::Table = generator.call(luaur_rt::Value::Nil).unwrap();
    let mut first_instance = vm.script_instance_from_table(first_table);
    let mut second_instance = vm.script_instance_from_table(second_table);
    let mut host = NoopScriptHost;
    let mut renderer = factory.borrow().make_renderer();

    first_instance
        .call_draw(&mut factory, &mut renderer, &mut host)
        .unwrap();
    second_instance
        .call_draw(&mut factory, &mut renderer, &mut host)
        .unwrap();

    let stream = factory.borrow().stream();
    assert!(stream.contains("drawPath path={id=1,"), "{stream}");
    assert!(stream.contains("drawPath path={id=2,"), "{stream}");
    assert!(stream.contains("drawPath path={id=3,"), "{stream}");
}
mod support;
use support::ScriptVmSourceTestExt as _;
