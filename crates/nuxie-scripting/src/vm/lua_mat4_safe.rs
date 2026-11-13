//! `Mat4` on the safe VM (seam rows R24 to R29): the same tagged plain-data
//! userdata, shared tag metatable, direct `mNN` field reads and functions
//! as the vendored VM's raw C binding had (deleted in Phase 3), written
//! against the safe VM's host-function API. The matrix math is the parent
//! module's.

use std::cell::RefCell;
use std::rc::Rc;

use super::{IDENTITY, ScriptedMat4, invert, invert_affine, matrix_index, multiply, multiply_affine};
use crate::luau::vm::{self, HostCtx, HostError};
use crate::luau::{Lua, Result};

/// `LUA_T_COUNT + 62`, as upstream computes it (`LUA_T_COUNT`
/// is `LUA_TDEADKEY`, 14). Keep in step with `ScriptedMat4::luaTag` in
/// `rive_lua_libs.hpp`.
const MAT4_USERDATA_TAG: u8 = 14 + 62;

/// The debug name the vendored binding's functions carried (its
/// `create_c_function`), which argument errors print; kept so they print
/// the same.
const FUNCTION_NAME: &str = "luaur-rt-c-function";

type HostFn = fn(&mut HostCtx<'_>) -> std::result::Result<usize, HostError>;

pub(in crate::vm) fn install_mat4_global(lua: &Lua) -> Result<()> {
    lua.with_vm(|vm| install(vm).map_err(crate::luau::Error::from))
}

fn install(vm: &mut vm::Vm) -> std::result::Result<(), vm::Error> {
    // One metatable for the tag; every result carries only its 64 bytes.
    let metatable = vm.create_table()?;
    for (name, function) in [
        ("__index", mat4_index as HostFn),
        ("__newindex", mat4_newindex),
        ("__mul", mat4_mul),
        ("__eq", mat4_eq),
        ("__namecall", mat4_namecall),
    ] {
        let f = vm.create_host_function(FUNCTION_NAME, function)?;
        metatable.set(vm, name, f)?;
    }
    metatable.set_readonly(vm, true)?;
    vm.set_tag_metatable(MAT4_USERDATA_TAG, Some(&metatable))?;
    register_direct_fields(vm)?;

    let table = vm.create_table()?;
    for (name, function) in [
        ("identity", mat4_identity as HostFn),
        ("values", mat4_values),
        ("fromTranslation", mat4_from_translation),
        ("fromScale", mat4_from_scale),
        ("fromRotationX", mat4_from_rotation_x),
        ("fromRotationY", mat4_from_rotation_y),
        ("fromRotationZ", mat4_from_rotation_z),
        ("perspective", mat4_perspective),
        ("perspectiveReverseZ", mat4_perspective_reverse_z),
        ("lookAt", mat4_look_at),
        ("ortho", mat4_ortho),
        ("multiply", mat4_static_multiply),
        ("multiplyAffine", mat4_static_multiply_affine),
        ("invert", mat4_static_invert),
        ("invertAffine", mat4_static_invert_affine),
    ] {
        let f = vm.create_host_function(FUNCTION_NAME, function)?;
        table.set(vm, name, f)?;
    }
    table.set_readonly(vm, true)?;
    let globals = vm.globals()?;
    globals.set(vm, "Mat4", table)
}

/// `GETTABLEKS` reads of `m11` .. `m44` skip `__index` (C++
/// `lua_registeruserdatadirectfieldget`).
fn register_direct_fields(vm: &mut vm::Vm) -> std::result::Result<(), vm::Error> {
    vm.register_direct_fields::<ScriptedMat4>(
        MAT4_USERDATA_TAG,
        &[
            ("m11", |m| f64::from(m.values[0])),
            ("m21", |m| f64::from(m.values[1])),
            ("m31", |m| f64::from(m.values[2])),
            ("m41", |m| f64::from(m.values[3])),
            ("m12", |m| f64::from(m.values[4])),
            ("m22", |m| f64::from(m.values[5])),
            ("m32", |m| f64::from(m.values[6])),
            ("m42", |m| f64::from(m.values[7])),
            ("m13", |m| f64::from(m.values[8])),
            ("m23", |m| f64::from(m.values[9])),
            ("m33", |m| f64::from(m.values[10])),
            ("m43", |m| f64::from(m.values[11])),
            ("m14", |m| f64::from(m.values[12])),
            ("m24", |m| f64::from(m.values[13])),
            ("m34", |m| f64::from(m.values[14])),
            ("m44", |m| f64::from(m.values[15])),
        ],
    )
}

fn push_mat4(ctx: &mut HostCtx<'_>, values: [f32; 16]) -> std::result::Result<(), HostError> {
    let ud = ctx.create_userdata_tagged(ScriptedMat4 { values }, MAT4_USERDATA_TAG)?;
    ctx.push(ud)
}

/// Argument `n` as a `Mat4` (C++ `lua_touserdatatagged`, else
/// `luaL_typeerror(n, "Mat4")`).
fn check_mat4(
    ctx: &mut HostCtx<'_>,
    n: usize,
) -> std::result::Result<Rc<RefCell<ScriptedMat4>>, HostError> {
    match ctx.to_userdata_tagged::<ScriptedMat4>(n, MAT4_USERDATA_TAG) {
        Some(cell) => Ok(cell),
        None => Err(ctx.type_error(n, "Mat4")),
    }
}

/// Argument `n`'s matrix values.
fn mat4_at(ctx: &mut HostCtx<'_>, n: usize) -> std::result::Result<[f32; 16], HostError> {
    let cell = check_mat4(ctx, n)?;
    let values = cell
        .try_borrow()
        .map_err(|_| HostError::new("Mat4 is in use"))?
        .values;
    Ok(values)
}

fn write_mat4(
    cell: &Rc<RefCell<ScriptedMat4>>,
    values: [f32; 16],
) -> std::result::Result<(), HostError> {
    cell.try_borrow_mut()
        .map_err(|_| HostError::new("Mat4 is in use"))?
        .values = values;
    Ok(())
}

fn number(ctx: &mut HostCtx<'_>, n: usize) -> std::result::Result<f32, HostError> {
    Ok(ctx.check_number(n)? as f32)
}

/// C++ `lua_isnumber`: a number, or a string that converts to one.
fn is_number(ctx: &mut HostCtx<'_>, n: usize) -> std::result::Result<bool, HostError> {
    let value = ctx.value(isize::try_from(n).unwrap_or(isize::MAX))?;
    Ok(ctx.coerce_number(&value)?.is_some())
}

fn mat4_values(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let mut values = [0.0; 16];
    for (index, value) in values.iter_mut().enumerate() {
        *value = number(ctx, index + 1)?;
    }
    push_mat4(ctx, values)?;
    Ok(1)
}

fn mat4_identity(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    push_mat4(ctx, IDENTITY)?;
    Ok(1)
}

fn mat4_from_translation(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let mut values = IDENTITY;
    values[12] = number(ctx, 1)?;
    values[13] = number(ctx, 2)?;
    values[14] = number(ctx, 3)?;
    push_mat4(ctx, values)?;
    Ok(1)
}

fn mat4_from_scale(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let scale_x = number(ctx, 1)?;
    let scale_y = if is_number(ctx, 2)? {
        number(ctx, 2)?
    } else {
        scale_x
    };
    let scale_z = if is_number(ctx, 3)? {
        number(ctx, 3)?
    } else {
        scale_x
    };
    let mut values = IDENTITY;
    values[0] = scale_x;
    values[5] = scale_y;
    values[10] = scale_z;
    push_mat4(ctx, values)?;
    Ok(1)
}

fn mat4_from_rotation_x(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let radians = number(ctx, 1)?;
    let cosine = radians.cos();
    let sine = radians.sin();
    let mut values = IDENTITY;
    values[5] = cosine;
    values[6] = sine;
    values[9] = -sine;
    values[10] = cosine;
    push_mat4(ctx, values)?;
    Ok(1)
}

fn mat4_from_rotation_y(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let radians = number(ctx, 1)?;
    let cosine = radians.cos();
    let sine = radians.sin();
    let mut values = IDENTITY;
    values[0] = cosine;
    values[2] = -sine;
    values[8] = sine;
    values[10] = cosine;
    push_mat4(ctx, values)?;
    Ok(1)
}

fn mat4_from_rotation_z(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let radians = number(ctx, 1)?;
    let cosine = radians.cos();
    let sine = radians.sin();
    let mut values = IDENTITY;
    values[0] = cosine;
    values[1] = sine;
    values[4] = -sine;
    values[5] = cosine;
    push_mat4(ctx, values)?;
    Ok(1)
}

fn mat4_perspective(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let fov_y = number(ctx, 1)?;
    let aspect = number(ctx, 2)?;
    let near = number(ctx, 3)?;
    let far = number(ctx, 4)?;
    let focal_length = 1.0 / (fov_y * 0.5).tan();
    let inverse_depth = 1.0 / (near - far);
    let mut values = [0.0; 16];
    values[0] = focal_length / aspect;
    values[5] = focal_length;
    values[10] = far * inverse_depth;
    values[11] = -1.0;
    values[14] = far * near * inverse_depth;
    push_mat4(ctx, values)?;
    Ok(1)
}

fn mat4_perspective_reverse_z(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let fov_y = number(ctx, 1)?;
    let aspect = number(ctx, 2)?;
    let near = number(ctx, 3)?;
    let focal_length = 1.0 / (fov_y * 0.5).tan();
    let mut values = [0.0; 16];
    values[0] = focal_length / aspect;
    values[5] = focal_length;
    values[11] = -1.0;
    values[14] = near;
    push_mat4(ctx, values)?;
    Ok(1)
}

fn mat4_look_at(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let eye = ctx.check_vector(1)?;
    let center = ctx.check_vector(2)?;
    let up = ctx.check_vector(3)?;

    let mut forward = [center[0] - eye[0], center[1] - eye[1], center[2] - eye[2]];
    let inverse_forward_length =
        1.0 / (forward[0] * forward[0] + forward[1] * forward[1] + forward[2] * forward[2]).sqrt();
    forward[0] *= inverse_forward_length;
    forward[1] *= inverse_forward_length;
    forward[2] *= inverse_forward_length;

    let mut side = [
        forward[1] * up[2] - forward[2] * up[1],
        forward[2] * up[0] - forward[0] * up[2],
        forward[0] * up[1] - forward[1] * up[0],
    ];
    let inverse_side_length =
        1.0 / (side[0] * side[0] + side[1] * side[1] + side[2] * side[2]).sqrt();
    side[0] *= inverse_side_length;
    side[1] *= inverse_side_length;
    side[2] *= inverse_side_length;

    let corrected_up = [
        side[1] * forward[2] - side[2] * forward[1],
        side[2] * forward[0] - side[0] * forward[2],
        side[0] * forward[1] - side[1] * forward[0],
    ];
    let mut values = IDENTITY;
    values[0] = side[0];
    values[1] = corrected_up[0];
    values[2] = -forward[0];
    values[4] = side[1];
    values[5] = corrected_up[1];
    values[6] = -forward[1];
    values[8] = side[2];
    values[9] = corrected_up[2];
    values[10] = -forward[2];
    values[12] = -(side[0] * eye[0] + side[1] * eye[1] + side[2] * eye[2]);
    values[13] = -(corrected_up[0] * eye[0] + corrected_up[1] * eye[1] + corrected_up[2] * eye[2]);
    values[14] = forward[0] * eye[0] + forward[1] * eye[1] + forward[2] * eye[2];
    push_mat4(ctx, values)?;
    Ok(1)
}

fn mat4_ortho(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let left = number(ctx, 1)?;
    let right = number(ctx, 2)?;
    let bottom = number(ctx, 3)?;
    let top = number(ctx, 4)?;
    let near = number(ctx, 5)?;
    let far = number(ctx, 6)?;
    let mut values = IDENTITY;
    values[0] = 2.0 / (right - left);
    values[5] = 2.0 / (top - bottom);
    values[10] = -1.0 / (far - near);
    values[12] = -(right + left) / (right - left);
    values[13] = -(top + bottom) / (top - bottom);
    values[14] = -near / (far - near);
    push_mat4(ctx, values)?;
    Ok(1)
}

/// `Mat4.multiply(out, a, b)` and `multiplyAffine`: writes `out`, returns
/// it. `out` may be `a` or `b`; both are read before the write.
fn static_product(
    ctx: &mut HostCtx<'_>,
    product: fn([f32; 16], [f32; 16]) -> [f32; 16],
) -> std::result::Result<usize, HostError> {
    let output = check_mat4(ctx, 1)?;
    let lhs = mat4_at(ctx, 2)?;
    let rhs = mat4_at(ctx, 3)?;
    write_mat4(&output, product(lhs, rhs))?;
    ctx.push_arg(1)?;
    Ok(1)
}

fn mat4_static_multiply(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    static_product(ctx, multiply)
}

fn mat4_static_multiply_affine(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    static_product(ctx, multiply_affine)
}

/// `Mat4.invert(out, m)` and `invertAffine`: writes `out` and returns
/// `true`, or returns `false` for a singular matrix.
fn static_inverse(
    ctx: &mut HostCtx<'_>,
    inverse: fn([f32; 16]) -> Option<[f32; 16]>,
) -> std::result::Result<usize, HostError> {
    let output = check_mat4(ctx, 1)?;
    let input = mat4_at(ctx, 2)?;
    if let Some(values) = inverse(input) {
        write_mat4(&output, values)?;
        ctx.push(true)?;
    } else {
        ctx.push(false)?;
    }
    Ok(1)
}

fn mat4_static_invert(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    static_inverse(ctx, invert)
}

fn mat4_static_invert_affine(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    static_inverse(ctx, invert_affine)
}

/// The key up to its first NUL, as upstream reads it through
/// a C string.
fn c_name(bytes: &[u8]) -> String {
    let name = bytes.split(|&b| b == 0).next().unwrap_or_default();
    String::from_utf8_lossy(name).into_owned()
}

fn mat4_index(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let values = mat4_at(ctx, 1)?;
    let name = ctx.check_string(2)?;
    if let Some(value) = matrix_index(&name).and_then(|index| values.get(index)) {
        ctx.push(f64::from(*value))?;
        return Ok(1);
    }
    Err(ctx.error(format!("'{}' is not a valid index of Mat4", c_name(&name))))
}

fn mat4_newindex(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let matrix = check_mat4(ctx, 1)?;
    let name = ctx.check_string(2)?;
    let value = number(ctx, 3)?;
    if let Some(index) = matrix_index(&name) {
        let mut matrix = matrix
            .try_borrow_mut()
            .map_err(|_| HostError::new("Mat4 is in use"))?;
        if let Some(slot) = matrix.values.get_mut(index) {
            *slot = value;
        }
        return Ok(0);
    }
    Err(ctx.error(format!("'{}' is not a valid index of Mat4", c_name(&name))))
}

fn mat4_mul(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let lhs = mat4_at(ctx, 1)?;
    let rhs = mat4_at(ctx, 2)?;
    push_mat4(ctx, multiply(lhs, rhs))?;
    Ok(1)
}

fn mat4_eq(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let lhs = mat4_at(ctx, 1)?;
    let rhs = mat4_at(ctx, 2)?;
    ctx.push(lhs == rhs)?;
    Ok(1)
}

fn mat4_invert(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let matrix = mat4_at(ctx, 1)?;
    match invert(matrix) {
        Some(values) => push_mat4(ctx, values)?,
        None => ctx.push(vm::Value::Nil)?,
    }
    Ok(1)
}

fn mat4_invert_affine(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let matrix = mat4_at(ctx, 1)?;
    match invert_affine(matrix) {
        Some(values) => push_mat4(ctx, values)?,
        None => ctx.push(vm::Value::Nil)?,
    }
    Ok(1)
}

fn mat4_transpose(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let matrix = mat4_at(ctx, 1)?;
    let mut values = [0.0; 16];
    for row in 0..4 {
        for column in 0..4 {
            values[row * 4 + column] = matrix[column * 4 + row];
        }
    }
    push_mat4(ctx, values)?;
    Ok(1)
}

fn mat4_transform_point(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let matrix = ScriptedMat4 {
        values: mat4_at(ctx, 1)?,
    };
    let x = number(ctx, 2)?;
    let y = number(ctx, 3)?;
    let z = number(ctx, 4)?;
    let out = matrix.transform_vec4(x, y, z, 1.0);
    let vector = if out[3] != 0.0 && out[3] != 1.0 {
        let inverse_w = 1.0 / out[3];
        [out[0] * inverse_w, out[1] * inverse_w, out[2] * inverse_w]
    } else {
        [out[0], out[1], out[2]]
    };
    ctx.push(vm::Value::Vector(vector))?;
    Ok(1)
}

fn mat4_transform_vec4(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let matrix = ScriptedMat4 {
        values: mat4_at(ctx, 1)?,
    };
    let x = number(ctx, 2)?;
    let y = number(ctx, 3)?;
    let z = number(ctx, 4)?;
    let w = number(ctx, 5)?;
    for value in matrix.transform_vec4(x, y, z, w) {
        ctx.push(f64::from(value))?;
    }
    Ok(4)
}

fn mat4_write_to_buffer(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let matrix = mat4_at(ctx, 1)?;
    let buffer = ctx.check_buffer(2)?;
    let offset = ctx.check_integer(3)?;
    let len = buffer.len(ctx)?;
    let start = usize::try_from(offset)
        .ok()
        .filter(|start| start.checked_add(64).is_some_and(|end| end <= len));
    let Some(start) = start else {
        return Err(ctx.error("Mat4:writeToBuffer offset out of range"));
    };
    // The bytes as upstream copies them: native order.
    let bytes: Vec<u8> = matrix.iter().flat_map(|v| v.to_ne_bytes()).collect();
    buffer.write_bytes(ctx, start, &bytes)?;
    Ok(0)
}

fn mat4_namecall(ctx: &mut HostCtx<'_>) -> std::result::Result<usize, HostError> {
    let name = ctx.namecall_method();
    let method: Option<HostFn> = match name.as_deref() {
        Some(b"invert") => Some(mat4_invert),
        Some(b"invertAffine") => Some(mat4_invert_affine),
        Some(b"transpose") => Some(mat4_transpose),
        Some(b"transformPoint") => Some(mat4_transform_point),
        Some(b"transformVec4") => Some(mat4_transform_vec4),
        Some(b"writeToBuffer") => Some(mat4_write_to_buffer),
        _ => None,
    };
    if let Some(method) = method {
        return method(ctx);
    }
    let name = name.map_or_else(|| "<unknown>".to_owned(), |name| c_name(&name));
    Err(ctx.error(format!("{name} is not a valid method of Mat4")))
}
