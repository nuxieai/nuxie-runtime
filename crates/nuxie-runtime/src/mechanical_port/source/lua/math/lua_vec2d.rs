#![cfg(feature = "rive_scripting")]
use crate::mechanical_port::source::lua::rive_lua_libs::*;
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn index(s: &mut LuaState) -> i32 {
    let v = s.check_vector(1);
    let name = s.check_string(2);
    if name.len() == 1 {
        match name.as_bytes()[0] {
            b'x' | b'1' => s.push_number(v[0] as f64),
            b'y' | b'2' => s.push_number(v[1] as f64),
            b'z' | b'3' => s.push_number(v[2] as f64),
            _ => return s.error(format!("'{name}' is not a valid index of Vector")),
        }
        return 1;
    }
    s.error(format!("'{name}' is not a valid index of Vector"))
}
fn length(s: &mut LuaState) -> i32 {
    let v = s.check_vector(1);
    s.push_number(dot(v, v).sqrt() as f64);
    1
}
fn length_squared(s: &mut LuaState) -> i32 {
    let v = s.check_vector(1);
    s.push_number(dot(v, v) as f64);
    1
}
fn normalized(s: &mut LuaState) -> i32 {
    let v = s.check_vector(1);
    let d = dot(v, v);
    let k = if d > 0.0 { 1.0 / d.sqrt() } else { 1.0 };
    s.push_vector(v[0] * k, v[1] * k, v[2] * k);
    1
}
fn distance_squared(s: &mut LuaState) -> i32 {
    let (a, b) = (s.check_vector(1), s.check_vector(2));
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    s.push_number(dot(d, d) as f64);
    1
}
fn distance(s: &mut LuaState) -> i32 {
    distance_squared(s);
    let d = s.to_number(-1).unwrap();
    s.pop(1);
    s.push_number(d.sqrt());
    1
}
fn vector_dot(s: &mut LuaState) -> i32 {
    let (a, b) = (s.check_vector(1), s.check_vector(2));
    s.push_number(dot(a, b) as f64);
    1
}
fn cross(s: &mut LuaState) -> i32 {
    let (a, b) = (s.check_vector(1), s.check_vector(2));
    s.push_number((a[0] * b[1] - a[1] * b[0]) as f64);
    1
}
fn cross3(s: &mut LuaState) -> i32 {
    let (a, b) = (s.check_vector(1), s.check_vector(2));
    s.push_vector(
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    );
    1
}
fn scale_add(s: &mut LuaState) -> i32 {
    let (a, b, k) = (
        s.check_vector(1),
        s.check_vector(2),
        s.check_number(3) as f32,
    );
    s.push_vector(a[0] + b[0] * k, a[1] + b[1] * k, a[2] + b[2] * k);
    1
}
fn scale_sub(s: &mut LuaState) -> i32 {
    let (a, b, k) = (
        s.check_vector(1),
        s.check_vector(2),
        s.check_number(3) as f32,
    );
    s.push_vector(a[0] - b[0] * k, a[1] - b[1] * k, a[2] - b[2] * k);
    1
}
fn lerpf(a: f32, b: f32, t: f32) -> f32 {
    if t == 1.0 { b } else { a + (b - a) * t }
}
fn lerp(s: &mut LuaState) -> i32 {
    let (a, b, t) = (
        s.check_vector(1),
        s.check_vector(2),
        s.check_number(3) as f32,
    );
    s.push_vector(
        lerpf(a[0], b[0], t),
        lerpf(a[1], b[1], t),
        lerpf(a[2], b[2], t),
    );
    1
}
fn write(s: &mut LuaState) -> i32 {
    let v = s.check_vector(1);
    let offset = s.check_integer(3);
    let buffer = s.check_buffer_mut(2);
    if offset < 0 || offset as usize + 12 > buffer.len() {
        return s.error("Vector:writeToBuffer offset out of range");
    }
    buffer[offset as usize..offset as usize + 12].copy_from_slice(bytemuck::bytes_of(&v));
    0
}
fn write4(s: &mut LuaState) -> i32 {
    let v = s.check_vector(1);
    let offset = s.check_integer(3);
    let w = s.check_number(4) as f32;
    let buffer = s.check_buffer_mut(2);
    if offset < 0 || offset as usize + 16 > buffer.len() {
        return s.error("Vector:writeVec4 offset out of range");
    }
    buffer[offset as usize..offset as usize + 12].copy_from_slice(bytemuck::bytes_of(&v));
    buffer[offset as usize + 12..offset as usize + 16].copy_from_slice(&w.to_ne_bytes());
    0
}
fn xy(s: &mut LuaState) -> i32 {
    s.push_vector2(
        s.to_number(1).unwrap_or(0.0) as f32,
        s.to_number(2).unwrap_or(0.0) as f32,
    );
    1
}
fn xyz(s: &mut LuaState) -> i32 {
    s.push_vector(
        s.to_number(1).unwrap_or(0.0) as f32,
        s.to_number(2).unwrap_or(0.0) as f32,
        s.to_number(3).unwrap_or(0.0) as f32,
    );
    1
}
fn origin(s: &mut LuaState) -> i32 {
    s.push_vector2(0.0, 0.0);
    1
}
const METHODS: &[LuaReg] = &[
    LuaReg::new("distance", distance),
    LuaReg::new("distanceSquared", distance_squared),
    LuaReg::new("dot", vector_dot),
    LuaReg::new("cross", cross),
    LuaReg::new("cross3", cross3),
    LuaReg::new("scaleAndAdd", scale_add),
    LuaReg::new("scaleAndSub", scale_sub),
    LuaReg::new("lerp", lerp),
    LuaReg::new("xy", xy),
    LuaReg::new("xyz", xyz),
    LuaReg::new("origin", origin),
    LuaReg::new("length", length),
    LuaReg::new("lengthSquared", length_squared),
    LuaReg::new("normalized", normalized),
    LuaReg::END,
];
fn namecall(s: &mut LuaState) -> i32 {
    let (_, atom) = s.namecall_atom();
    match atom {
        LuaAtoms::Length => length(s),
        LuaAtoms::LengthSquared => length_squared(s),
        LuaAtoms::Normalized => normalized(s),
        LuaAtoms::Distance => distance(s),
        LuaAtoms::DistanceSquared => distance_squared(s),
        LuaAtoms::Dot => vector_dot(s),
        LuaAtoms::Lerp => lerp(s),
        LuaAtoms::WriteToBuffer => write(s),
        LuaAtoms::WriteVec4 => write4(s),
        _ => s.error(format!(
            "{} is not a valid method of Vector",
            s.check_string(1)
        )),
    }
}
pub fn luaopen_rive_vector(s: &mut LuaState) -> i32 {
    s.register("Vector", METHODS);
    s.create_table(0, 1);
    s.push_value(-1);
    s.push_vector(0.0, 0.0, 0.0);
    s.insert(-2);
    s.set_metatable(-2);
    s.pop(1);
    s.push_function(index);
    s.set_field(-2, "__index");
    s.push_function(namecall);
    s.set_field(-2, "__namecall");
    s.set_readonly(-1, true);
    s.pop(1);
    1
}
