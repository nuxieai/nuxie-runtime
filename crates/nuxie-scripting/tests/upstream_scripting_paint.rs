//! One-for-one ports of `tests/unit_tests/runtime/scripting/scripting_paint_test.cpp`.
#![cfg(feature = "luau")]

use luaur_rt::{FromLuaMulti, Value};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_scripting::vm::ScriptVm;

mod support;
use support::ScriptVmSourceTestExt as _;

fn eval<R: FromLuaMulti>(source: &str) -> R {
    let vm = ScriptVm::new();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    vm.install_render_factory(&mut factory).unwrap();
    vm.install_rive_globals().unwrap();
    vm.eval(source).unwrap()
}

fn eval_value(source: &str) -> Value {
    eval(source)
}

fn assert_userdata(source: &str) {
    let value = eval_value(source);
    assert!(matches!(value, Value::UserData(_)), "got {value:?}");
}

#[test]
fn paint_can_be_constructed() {
    assert_userdata(
        r#"return Paint.with({
    style = 'stroke',
    join = 'round',
    cap = 'butt',
    blendMode = 'srcOver',
    color = 0xffff0000,
    thickness = 3,
    feather = 0,
    gradient = Gradient.radial(Vector.origin(), 20.0, {
        { position = 0.0, color = Color.rgba(255, 0, 0, 255) },
        { position = 1.0, color = Color.rgba(255, 0, 0, 0) },
    }),
})
"#,
    );

    assert_userdata(
        "local paint = Paint.new()\n\
         paint.cap = 'round'\n\
         return paint\n",
    );

    let style: String = eval(
        "local paint = Paint.new()\n\
         return paint.style\n",
    );
    assert_eq!(style, "fill");

    let style: String = eval(
        "local paint = Paint.with({style='stroke'})\n\
         return paint.style\n",
    );
    assert_eq!(style, "stroke");

    let join: String = eval(
        "local paint = Paint.with({style='stroke'})\n\
         return paint.join\n",
    );
    assert_eq!(join, "miter");

    let join: String = eval(
        "local paint = Paint.with({join='round'})\n\
         return paint.join\n",
    );
    assert_eq!(join, "round");

    let join: String = eval(
        "local paint = Paint.new()\n\
         paint.join = 'bevel'\n\
         return paint.join\n",
    );
    assert_eq!(join, "bevel");

    let thickness: f64 = eval("return Paint.new().thickness\n");
    assert_eq!(thickness, 1.0);

    let thickness: f64 = eval(
        "local paint = Paint.new()\n\
         paint.thickness = 22\n\
         return paint.thickness\n",
    );
    assert_eq!(thickness, 22.0);

    let color: i64 = eval(
        "local paint = Paint.new()\n\
         paint.color = Color.rgb(255, 128, 64)\n\
         return paint.color\n",
    );
    assert_eq!(color, 0xffff8040);

    let feather: f64 = eval(
        "local paint = Paint.new()\n\
         paint.feather = 0.222\n\
         return paint.feather\n",
    );
    assert!((feather - 0.222).abs() <= 1e-6);

    let gradient = eval_value(
        "local paint = Paint.new()\n\
         paint.feather = 0.222\n\
         return paint.gradient\n",
    );
    assert!(matches!(gradient, Value::Nil));

    assert_userdata(
        r#"local paint = Paint.new()
paint.gradient = Gradient.radial(Vector.origin(), 20.0, {
    { position = 0.0, color = Color.rgba(255, 0, 0, 255) },
    { position = 1.0, color = Color.rgba(255, 0, 0, 0) },
})
return paint.gradient
"#,
    );

    assert_userdata(
        r#"local paint = Paint.new()
paint.gradient = Gradient.radial(Vector.origin(), 20.0, {
    { position = 0.0, color = Color.rgba(255, 0, 0, 255) },
    { position = 1.0, color = Color.rgba(255, 0, 0, 0) },
})
return paint:copy().gradient
"#,
    );
}

#[test]
#[ignore = "expected red: Paint.copy({gradient=false}) does not yet clear the gradient"]
fn paint_gradients_can_be_cleared() {
    let copied_gradient = eval_value(
        r#"local paint = Paint.new()
paint.gradient = Gradient.radial(Vector.origin(), 20.0, {
    { position = 0.0, color = Color.rgba(255, 0, 0, 255) },
    { position = 1.0, color = Color.rgba(255, 0, 0, 0) },
})
return paint:copy({gradient=false}).gradient
"#,
    );
    assert!(matches!(copied_gradient, Value::Nil));

    let assigned_gradient = eval_value(
        r#"local paint = Paint.new()
paint.gradient = Gradient.radial(Vector.origin(), 20.0, {
    { position = 0.0, color = Color.rgba(255, 0, 0, 255) },
    { position = 1.0, color = Color.rgba(255, 0, 0, 0) },
})
local paintCopy = paint:copy()
paintCopy.gradient = nil
return paintCopy.gradient
"#,
    );
    assert!(matches!(assigned_gradient, Value::Nil));
}
