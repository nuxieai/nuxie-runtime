//! One-for-one silver ports of
//! `tests/unit_tests/runtime/scripting/scripting_renderer_test.cpp`.

use std::path::{Path, PathBuf};

use luaur_rt::{Function, Table};
use luaur_vm::enums::lua_gc_op::lua_GCOp;
use luaur_vm::functions::lua_gc::lua_gc;
use luaur_vm::functions::lua_gettop::lua_gettop;
use nuxie_render_api::{PersistentFactory, SerializingFactory};
use nuxie_scripting::vm::ScriptVm;
use silver_corpus::{compare_sriv, parse_sriv};

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

fn runtime_root() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

fn configured_vm() -> (ScriptVm, PersistentFactory<SerializingFactory>) {
    let vm = ScriptVm::new();
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    vm.install_render_factory(&mut factory).unwrap();
    vm.install_rive_globals().unwrap();
    (vm, factory)
}

fn run_named(vm: &ScriptVm, name: &str, source: &str) {
    vm.lua().load(source).set_name(name).exec().unwrap();
}

fn draw_table(vm: &ScriptVm) -> Table {
    vm.lua()
        .load("return { draw = function(self, renderer) return render(renderer) end }")
        .eval()
        .unwrap()
}

fn compare_pinned_silver(
    id: &str,
    runtime: &Path,
    factory: &PersistentFactory<SerializingFactory>,
) -> anyhow::Result<()> {
    let expected_path = runtime
        .join("tests/unit_tests/silvers")
        .join(format!("{id}.sriv"));
    let expected_bytes = std::fs::read(&expected_path)?;
    let actual_bytes = {
        let factory = factory.borrow();
        factory.bytes().to_vec()
    };
    let expected = parse_sriv(&expected_bytes)?;
    let actual = parse_sriv(&actual_bytes)?;
    compare_sriv(&expected, &actual).map_err(|difference| anyhow::anyhow!("{id}: {difference}"))
}

#[test]
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
    let runtime = runtime_root();
    let (vm, mut factory) = configured_vm();
    run_named(&vm, "test_source", &source);
    let table = draw_table(&vm);
    let mut renderer = factory.borrow().make_renderer();

    let balanced = vm
        .upstream_test_call_draw_with_balance(&table, &mut factory, &mut renderer)
        .unwrap();

    assert!(balanced);
    compare_pinned_silver("scripted_oval", &runtime, &factory)
        .unwrap_or_else(|error| panic!("{error:#}"));
}

#[test]
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
    let runtime = runtime_root();
    let (vm, mut factory) = configured_vm();
    run_named(&vm, "test_source", &source);
    let advance: Function = vm.lua().globals().get("advance").unwrap();
    let table = draw_table(&vm);
    let elapsed_seconds = 1.0_f32 / 60.0;
    let state = vm.lua().current_thread().state();

    for frame in 0..1000 {
        let top = unsafe { lua_gettop(state) };
        if frame != 0 {
            factory.borrow_mut().add_frame();
        }
        assert!(advance.call::<bool>(elapsed_seconds).unwrap());
        let mut renderer = factory.borrow().make_renderer();
        let balanced = vm
            .upstream_test_call_draw_with_balance(&table, &mut factory, &mut renderer)
            .unwrap();
        assert!(balanced);
        assert_eq!(top, unsafe { lua_gettop(state) });
        lua_gc(state, lua_GCOp::LUA_GCCOLLECT as i32, 0);
    }

    compare_pinned_silver("scripted_animated_oval", &runtime, &factory)
        .unwrap_or_else(|error| panic!("{error:#}"));
}
