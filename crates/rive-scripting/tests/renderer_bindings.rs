#![cfg(feature = "luau")]

use rive_render_api::RecordingFactory;
use rive_runtime::{NoopScriptHost, ScriptInstance};
use rive_scripting::vm::ScriptVm;

#[test]
fn scripted_draw_can_emit_renderer_path_calls() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    let chunk = vm
        .load(
            "scripted-draw",
            "return function(_)\n\
                 return {\n\
                   draw = function(self, renderer)\n\
                     renderer:save()\n\
                     renderer:transform(Mat2D.withTranslation(3, 4))\n\
                     local path = Path.new()\n\
                     path:moveTo(Vector(0, 0))\n\
                     path:lineTo(Vector(10, 0))\n\
                     path:lineTo(Vector(10, 20))\n\
                     path:close()\n\
                     local paint = Paint.with({ color = 0xffff0000 })\n\
                     renderer:drawPath(path, paint)\n\
                   end,\n\
                 }\n\
               end",
        )
        .unwrap();
    let generator: luaur_rt::Function = chunk.call(()).unwrap();
    let table: luaur_rt::Table = generator.call(luaur_rt::Value::Nil).unwrap();
    let mut instance = vm.script_instance_from_table(table);
    let mut host = NoopScriptHost;

    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    factory.add_sample(0.0);
    instance
        .call_draw(&mut factory, &mut renderer, &mut host)
        .unwrap();
    factory.add_frame();

    let stream = factory.stream();
    assert!(
        stream.contains("save\ntransform matrix=[1,0,0,1,3,4]\n"),
        "{stream}"
    );
    assert!(
        stream.contains("makeRenderPaint {id=1,style=fill,color=0xff000000"),
        "{stream}"
    );
    assert!(
        stream.contains(
            "drawPath path={id=1,fillRule=2,path={verbs=[move,line,line,close],points=[(0,0),(10,0),(10,20)]}} paint={id=1,style=fill,color=0xffff0000"
        ),
        "{stream}"
    );
    assert!(stream.contains("restore\nframe\n"), "{stream}");
}
