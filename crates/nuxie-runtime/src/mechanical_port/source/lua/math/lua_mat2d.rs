#![cfg(feature = "rive_scripting")]
use crate::mechanical_port::source::{
    lua::rive_lua_libs::*,
    math::{mat2d::Mat2D, vec2d::Vec2D},
};
fn push(s: &mut LuaState, m: Mat2D) -> i32 {
    s.new_rive(ScriptedMat2D { value: m });
    1
}
fn values(s: &mut LuaState) -> i32 {
    let mut m = [0.0; 6];
    for (i, v) in m.iter_mut().enumerate() {
        *v = s.check_number(1 + i) as f32;
    }
    push(s, Mat2D::new(m))
}
fn identity(s: &mut LuaState) -> i32 {
    push(s, Mat2D::identity())
}
fn translation(s: &mut LuaState) -> i32 {
    let v = s
        .to_vec2d(1)
        .copied()
        .unwrap_or_else(|| Vec2D::new(s.check_number(1) as f32, s.check_number(2) as f32));
    push(s, Mat2D::from_translation(v))
}
fn rotation(s: &mut LuaState) -> i32 {
    push(s, Mat2D::from_rotation(s.check_number(1) as f32))
}
fn scale(s: &mut LuaState) -> i32 {
    if let Some(v) = s.to_vec2d(1) {
        push(s, Mat2D::from_scale(v.x, v.y))
    } else {
        let x = s.check_number(1) as f32;
        let y = if s.is_number(2) {
            s.check_number(2) as f32
        } else {
            x
        };
        push(s, Mat2D::from_scale(x, y))
    }
}
fn scale_translation(s: &mut LuaState) -> i32 {
    if let Some(v) = s.to_vec2d(1).copied() {
        let t = *s.check_vec2d(2);
        push(s, Mat2D::from_scale_and_translation(v.x, v.y, t.x, t.y))
    } else {
        let (x, y, tx, ty) = (
            s.check_number(1) as f32,
            s.check_number(2) as f32,
            s.check_number(3) as f32,
            s.check_number(4) as f32,
        );
        push(s, Mat2D::from_scale_and_translation(x, y, tx, ty))
    }
}
fn key(name: &str) -> Option<usize> {
    match name {
        "xx" | "1" => Some(0),
        "xy" | "2" => Some(1),
        "yx" | "3" => Some(2),
        "yy" | "4" => Some(3),
        "tx" | "5" => Some(4),
        "ty" | "6" => Some(5),
        _ => None,
    }
}
fn newindex(s: &mut LuaState) -> i32 {
    let name = s.check_string(2);
    let Some(index) = key(&name) else {
        return s.error(format!(
            "'{name}' is not a valid index of {}",
            ScriptedMat2D::LUA_NAME
        ));
    };
    let value = s.check_number(3) as f32;
    s.to_rive_mut::<ScriptedMat2D>(1).value[index] = value;
    0
}
fn index(s: &mut LuaState) -> i32 {
    let name = s.check_string(2);
    let Some(index) = key(&name) else {
        return s.error(format!(
            "'{name}' is not a valid index of {}",
            ScriptedMat2D::LUA_NAME
        ));
    };
    let value = s.to_rive::<ScriptedMat2D>(1).value[index];
    s.push_number(value as f64);
    1
}
fn direct<const I: usize>(v: &ScriptedMat2D, r: &mut DirectFieldResult) {
    r.set_number(v.value[I] as f64)
}
fn mul(s: &mut LuaState) -> i32 {
    let lhs = s.to_rive::<ScriptedMat2D>(1).value;
    if let Some(v) = s.to_vec2d(2) {
        s.push_vec2d(lhs * *v);
        1
    } else {
        let rhs = s.to_rive::<ScriptedMat2D>(2).value;
        push(s, lhs * rhs)
    }
}
fn invert(s: &mut LuaState) -> i32 {
    let m = s.to_rive::<ScriptedMat2D>(1).value;
    if let Some(v) = m.invert() {
        push(s, v)
    } else {
        s.push_nil();
        1
    }
}
fn is_identity(s: &mut LuaState) -> i32 {
    let value = s.to_rive::<ScriptedMat2D>(1).value == Mat2D::identity();
    s.push_boolean(value);
    1
}
fn equal(s: &mut LuaState) -> i32 {
    let value = s.to_rive::<ScriptedMat2D>(1).value == s.to_rive::<ScriptedMat2D>(2).value;
    s.push_boolean(value);
    1
}
fn namecall(s: &mut LuaState) -> i32 {
    let (_, atom) = s.namecall_atom();
    match atom {
        LuaAtoms::Invert => invert(s),
        LuaAtoms::IsIdentity => is_identity(s),
        _ => s.error(format!(
            "{} is not a valid method of {}",
            s.check_string(1),
            ScriptedMat2D::LUA_NAME
        )),
    }
}
fn static_invert(s: &mut LuaState) -> i32 {
    let input = s.to_rive::<ScriptedMat2D>(2).value;
    let result = input.invert();
    if let Some(value) = result {
        s.to_rive_mut::<ScriptedMat2D>(1).value = value;
    }
    s.push_boolean(result.is_some());
    1
}
const METHODS: &[LuaReg] = &[
    LuaReg::new("withTranslation", translation),
    LuaReg::new("withRotation", rotation),
    LuaReg::new("withScale", scale),
    LuaReg::new("withScaleAndTranslation", scale_translation),
    LuaReg::new("identity", identity),
    LuaReg::new("values", values),
    LuaReg::new("invert", static_invert),
    LuaReg::END,
];
pub fn luaopen_rive_mat2d(s: &mut LuaState) -> i32 {
    s.register(ScriptedMat2D::LUA_NAME, METHODS);
    s.register_rive::<ScriptedMat2D>();
    for (name, function) in [
        ("__index", index as LuaFunction),
        ("__newindex", newindex),
        ("__mul", mul),
        ("__eq", equal),
        ("__namecall", namecall),
    ] {
        s.push_function(function);
        s.set_field(-2, name);
    }
    s.set_readonly(-1, true);
    s.pop(1);
    s.register_userdata_direct_field_get::<ScriptedMat2D>("xx", direct::<0>);
    s.register_userdata_direct_field_get::<ScriptedMat2D>("xy", direct::<1>);
    s.register_userdata_direct_field_get::<ScriptedMat2D>("yx", direct::<2>);
    s.register_userdata_direct_field_get::<ScriptedMat2D>("yy", direct::<3>);
    s.register_userdata_direct_field_get::<ScriptedMat2D>("tx", direct::<4>);
    s.register_userdata_direct_field_get::<ScriptedMat2D>("ty", direct::<5>);
    1
}
