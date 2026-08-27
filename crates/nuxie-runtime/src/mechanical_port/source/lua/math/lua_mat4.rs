#![cfg(feature = "rive_scripting")]
use crate::mechanical_port::source::{lua::rive_lua_libs::*, math::mat4::Mat4};
fn push(s: &mut LuaState, m: Mat4) -> i32 {
    s.new_rive(ScriptedMat4 { value: m });
    1
}
fn values(s: &mut LuaState) -> i32 {
    let mut m = Mat4::default();
    for i in 0..16 {
        m[i] = s.check_number(1 + i) as f32;
    }
    push(s, m)
}
fn identity(s: &mut LuaState) -> i32 {
    push(s, Mat4::identity())
}
fn translation(s: &mut LuaState) -> i32 {
    push(
        s,
        Mat4::from_translation(
            s.check_number(1) as f32,
            s.check_number(2) as f32,
            s.check_number(3) as f32,
        ),
    )
}
fn scale(s: &mut LuaState) -> i32 {
    let x = s.check_number(1) as f32;
    let y = if s.is_number(2) {
        s.check_number(2) as f32
    } else {
        x
    };
    let z = if s.is_number(3) {
        s.check_number(3) as f32
    } else {
        x
    };
    push(s, Mat4::from_scale(x, y, z))
}
fn rx(s: &mut LuaState) -> i32 {
    push(s, Mat4::from_rotation_x(s.check_number(1) as f32))
}
fn ry(s: &mut LuaState) -> i32 {
    push(s, Mat4::from_rotation_y(s.check_number(1) as f32))
}
fn rz(s: &mut LuaState) -> i32 {
    push(s, Mat4::from_rotation_z(s.check_number(1) as f32))
}
fn perspective(s: &mut LuaState) -> i32 {
    push(
        s,
        Mat4::perspective(
            s.check_number(1) as f32,
            s.check_number(2) as f32,
            s.check_number(3) as f32,
            s.check_number(4) as f32,
            true,
        ),
    )
}
fn reverse_z(s: &mut LuaState) -> i32 {
    push(
        s,
        Mat4::perspective_reverse_z(
            s.check_number(1) as f32,
            s.check_number(2) as f32,
            s.check_number(3) as f32,
        ),
    )
}
fn look_at(s: &mut LuaState) -> i32 {
    push(
        s,
        Mat4::look_at(s.check_vector(1), s.check_vector(2), s.check_vector(3)),
    )
}
fn ortho(s: &mut LuaState) -> i32 {
    push(
        s,
        Mat4::ortho(
            s.check_number(1) as f32,
            s.check_number(2) as f32,
            s.check_number(3) as f32,
            s.check_number(4) as f32,
            s.check_number(5) as f32,
            s.check_number(6) as f32,
            true,
        ),
    )
}
fn static_multiply(s: &mut LuaState) -> i32 {
    let value = Mat4::multiply(
        s.to_rive::<ScriptedMat4>(2).value,
        s.to_rive::<ScriptedMat4>(3).value,
    );
    s.to_rive_mut::<ScriptedMat4>(1).value = value;
    s.push_value(1);
    1
}
fn static_affine(s: &mut LuaState) -> i32 {
    let value = Mat4::multiply_affine(
        s.to_rive::<ScriptedMat4>(2).value,
        s.to_rive::<ScriptedMat4>(3).value,
    );
    s.to_rive_mut::<ScriptedMat4>(1).value = value;
    s.push_value(1);
    1
}
fn static_invert(s: &mut LuaState) -> i32 {
    let value = s.to_rive::<ScriptedMat4>(2).value.invert();
    if let Some(v) = value {
        s.to_rive_mut::<ScriptedMat4>(1).value = v;
    }
    s.push_boolean(value.is_some());
    1
}
fn static_invert_affine(s: &mut LuaState) -> i32 {
    let value = s.to_rive::<ScriptedMat4>(2).value.invert_affine();
    if let Some(v) = value {
        s.to_rive_mut::<ScriptedMat4>(1).value = v;
    }
    s.push_boolean(value.is_some());
    1
}
fn field(name: &str) -> Option<usize> {
    if name.len() == 3 && name.starts_with('m') {
        let b = name.as_bytes();
        let (row, col) = ((b[1] - b'0') as usize, (b[2] - b'0') as usize);
        if (1..=4).contains(&row) && (1..=4).contains(&col) {
            return Some((col - 1) * 4 + row - 1);
        }
    }
    name.parse::<usize>()
        .ok()
        .filter(|v| (1..=16).contains(v))
        .map(|v| v - 1)
}
fn index(s: &mut LuaState) -> i32 {
    let name = s.check_string(2);
    if let Some(i) = field(&name) {
        s.push_number(s.to_rive::<ScriptedMat4>(1).value[i] as f64);
        1
    } else {
        s.error(format!(
            "'{name}' is not a valid index of {}",
            ScriptedMat4::LUA_NAME
        ))
    }
}
fn newindex(s: &mut LuaState) -> i32 {
    let name = s.check_string(2);
    if let Some(i) = field(&name) {
        let v = s.check_number(3) as f32;
        s.to_rive_mut::<ScriptedMat4>(1).value[i] = v;
        0
    } else {
        s.error(format!(
            "'{name}' is not a valid index of {}",
            ScriptedMat4::LUA_NAME
        ))
    }
}
fn direct<const I: usize>(v: &ScriptedMat4, r: &mut DirectFieldResult) {
    r.set_number(v.value[I] as f64)
}
fn mul(s: &mut LuaState) -> i32 {
    push(
        s,
        Mat4::multiply(
            s.to_rive::<ScriptedMat4>(1).value,
            s.to_rive::<ScriptedMat4>(2).value,
        ),
    )
}
fn equal(s: &mut LuaState) -> i32 {
    let value = s.to_rive::<ScriptedMat4>(1).value == s.to_rive::<ScriptedMat4>(2).value;
    s.push_boolean(value);
    1
}
fn invert(s: &mut LuaState) -> i32 {
    if let Some(v) = s.to_rive::<ScriptedMat4>(1).value.invert() {
        push(s, v)
    } else {
        s.push_nil();
        1
    }
}
fn invert_affine(s: &mut LuaState) -> i32 {
    if let Some(v) = s.to_rive::<ScriptedMat4>(1).value.invert_affine() {
        push(s, v)
    } else {
        s.push_nil();
        1
    }
}
fn transpose(s: &mut LuaState) -> i32 {
    push(s, s.to_rive::<ScriptedMat4>(1).value.transposed())
}
fn transform_point(s: &mut LuaState) -> i32 {
    let out = s.to_rive::<ScriptedMat4>(1).value.transform_vec4(
        s.check_number(2) as f32,
        s.check_number(3) as f32,
        s.check_number(4) as f32,
        1.0,
    );
    if out[3] != 0.0 && out[3] != 1.0 {
        s.push_vector(out[0] / out[3], out[1] / out[3], out[2] / out[3]);
    } else {
        s.push_vector(out[0], out[1], out[2]);
    }
    1
}
fn transform_vec4(s: &mut LuaState) -> i32 {
    let out = s.to_rive::<ScriptedMat4>(1).value.transform_vec4(
        s.check_number(2) as f32,
        s.check_number(3) as f32,
        s.check_number(4) as f32,
        s.check_number(5) as f32,
    );
    for v in out {
        s.push_number(v as f64);
    }
    4
}
fn write(s: &mut LuaState) -> i32 {
    let bytes = bytemuck::bytes_of(s.to_rive::<ScriptedMat4>(1).value.values());
    let offset = s.check_integer(3);
    let buffer = s.check_buffer_mut(2);
    if offset < 0 || offset as usize + 64 > buffer.len() {
        return s.error("Mat4:writeToBuffer offset out of range");
    }
    buffer[offset as usize..offset as usize + 64].copy_from_slice(bytes);
    0
}
fn namecall(s: &mut LuaState) -> i32 {
    let (_, atom) = s.namecall_atom();
    match atom {
        LuaAtoms::Invert => invert(s),
        LuaAtoms::InvertAffine => invert_affine(s),
        LuaAtoms::Transpose => transpose(s),
        LuaAtoms::TransformPoint => transform_point(s),
        LuaAtoms::TransformVec4 => transform_vec4(s),
        LuaAtoms::WriteToBuffer => write(s),
        _ => s.error(format!(
            "{} is not a valid method of {}",
            s.check_string(1),
            ScriptedMat4::LUA_NAME
        )),
    }
}
const METHODS: &[LuaReg] = &[
    LuaReg::new("identity", identity),
    LuaReg::new("values", values),
    LuaReg::new("fromTranslation", translation),
    LuaReg::new("fromScale", scale),
    LuaReg::new("fromRotationX", rx),
    LuaReg::new("fromRotationY", ry),
    LuaReg::new("fromRotationZ", rz),
    LuaReg::new("perspective", perspective),
    LuaReg::new("perspectiveReverseZ", reverse_z),
    LuaReg::new("lookAt", look_at),
    LuaReg::new("ortho", ortho),
    LuaReg::new("multiply", static_multiply),
    LuaReg::new("multiplyAffine", static_affine),
    LuaReg::new("invert", static_invert),
    LuaReg::new("invertAffine", static_invert_affine),
    LuaReg::END,
];
pub fn luaopen_rive_mat4(s: &mut LuaState) -> i32 {
    s.register(ScriptedMat4::LUA_NAME, METHODS);
    s.register_rive::<ScriptedMat4>();
    for (name, f) in [
        ("__index", index as LuaFunction),
        ("__newindex", newindex),
        ("__mul", mul),
        ("__eq", equal),
        ("__namecall", namecall),
    ] {
        s.push_function(f);
        s.set_field(-2, name);
    }
    s.set_readonly(-1, true);
    s.pop(1);
    let names = [
        "m11", "m21", "m31", "m41", "m12", "m22", "m32", "m42", "m13", "m23", "m33", "m43", "m14",
        "m24", "m34", "m44",
    ];
    for (i, name) in names.into_iter().enumerate() {
        s.register_userdata_direct_field_get_index::<ScriptedMat4>(name, i);
    }
    1
}
