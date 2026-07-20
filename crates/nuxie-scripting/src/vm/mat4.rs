//! Rive's single-precision, column-major `Mat4` Luau userdata.

use core::ffi::{CStr, c_int, c_void};

use luaur_rt::ffi::{lua_CFunction, lua_State};
use luaur_rt::{Lua, Result, Table};
use luaur_vm::enums::lua_type::LUA_T_COUNT;
use luaur_vm::functions::lua_isnumber::lua_isnumber;
use luaur_vm::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use luaur_vm::functions::lua_l_checkinteger::lua_l_checkinteger;
use luaur_vm::functions::lua_l_checklstring::lua_l_checklstring;
use luaur_vm::functions::lua_l_checknumber::lua_l_checknumber;
use luaur_vm::functions::lua_l_checkvector::lua_l_checkvector;
use luaur_vm::functions::lua_l_error_l::lua_l_error_l;
use luaur_vm::functions::lua_l_typeerror_l::lua_l_typeerror_l;
use luaur_vm::functions::lua_namecallatom::lua_namecallatom;
use luaur_vm::functions::lua_newuserdatataggedwithmetatable::lua_newuserdatataggedwithmetatable;
use luaur_vm::functions::lua_pushboolean::lua_pushboolean;
use luaur_vm::functions::lua_pushnil::lua_pushnil;
use luaur_vm::functions::lua_pushnumber::lua_pushnumber;
use luaur_vm::functions::lua_pushvalue::lua_pushvalue;
use luaur_vm::functions::lua_pushvector_lapi_alt_b::lua_pushvector_lua_state_f32_f32_f32;
use luaur_vm::functions::lua_registeruserdatadirectfieldget::lua_registeruserdatadirectfieldget;
use luaur_vm::functions::lua_setuserdatametatable::lua_setuserdatametatable;
use luaur_vm::functions::lua_touserdatatagged::lua_touserdatatagged;
use luaur_vm::functions::lua_userdatadirectfield_setnumber::lua_userdatadirectfield_setnumber;

// Keep this synchronized with ScriptedMat4::luaTag in rive_lua_libs.hpp.
const MAT4_USERDATA_TAG: c_int = LUA_T_COUNT as c_int + 62;

const IDENTITY: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, // column 0
    0.0, 1.0, 0.0, 0.0, // column 1
    0.0, 0.0, 1.0, 0.0, // column 2
    0.0, 0.0, 0.0, 1.0, // column 3
];

#[derive(Clone, Copy)]
#[repr(transparent)]
struct ScriptedMat4 {
    values: [f32; 16],
}

impl ScriptedMat4 {
    fn transform_vec4(&self, x: f32, y: f32, z: f32, w: f32) -> [f32; 4] {
        std::array::from_fn(|row| {
            self.values[row] * x
                + self.values[4 + row] * y
                + self.values[8 + row] * z
                + self.values[12 + row] * w
        })
    }
}

pub(super) fn install_mat4_global(lua: &Lua) -> Result<()> {
    // Upstream registers one metatable per userdata tag. Every result can then
    // allocate only its inline 64-byte payload and attach that shared table.
    let metatable = lua.create_table();
    set_c_function(lua, &metatable, "__index", mat4_index)?;
    set_c_function(lua, &metatable, "__newindex", mat4_newindex)?;
    set_c_function(lua, &metatable, "__mul", mat4_mul)?;
    set_c_function(lua, &metatable, "__eq", mat4_eq)?;
    set_c_function(lua, &metatable, "__namecall", mat4_namecall)?;
    metatable.set_readonly(true);
    let _: () = unsafe {
        lua.exec_raw(metatable, |state| {
            lua_setuserdatametatable(state, MAT4_USERDATA_TAG);
            register_direct_fields(state);
        })?
    };

    let table = lua.create_table();
    for (name, function) in [
        (
            "identity",
            mat4_identity as unsafe fn(*mut lua_State) -> c_int,
        ),
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
        set_c_function(lua, &table, name, function)?;
    }
    table.set_readonly(true);
    lua.globals().set("Mat4", table)?;
    Ok(())
}

fn set_c_function(
    lua: &Lua,
    table: &Table,
    name: &str,
    function: unsafe fn(*mut lua_State) -> c_int,
) -> Result<()> {
    let function: lua_CFunction = Some(function);
    table.set(name, unsafe { lua.create_c_function(function)? })
}

unsafe fn push_mat4(state: *mut lua_State, values: [f32; 16]) -> *mut ScriptedMat4 {
    let storage = unsafe {
        lua_newuserdatataggedwithmetatable(
            state,
            core::mem::size_of::<ScriptedMat4>(),
            MAT4_USERDATA_TAG,
        )
    }
    .cast::<ScriptedMat4>();
    unsafe { storage.write(ScriptedMat4 { values }) };
    storage
}

unsafe fn check_mat4(state: *mut lua_State, index: c_int) -> *mut ScriptedMat4 {
    let matrix = unsafe { lua_touserdatatagged(state, index, MAT4_USERDATA_TAG) };
    if matrix.is_null() {
        unsafe { lua_l_typeerror_l(state, index, "Mat4") };
    }
    matrix.cast::<ScriptedMat4>()
}

unsafe fn mat4_values(state: *mut lua_State) -> c_int {
    let values = std::array::from_fn(|index| lua_l_checknumber(state, index as c_int + 1) as f32);
    unsafe { push_mat4(state, values) };
    1
}

unsafe fn mat4_identity(state: *mut lua_State) -> c_int {
    unsafe { push_mat4(state, IDENTITY) };
    1
}

unsafe fn mat4_from_translation(state: *mut lua_State) -> c_int {
    let mut values = IDENTITY;
    values[12] = lua_l_checknumber(state, 1) as f32;
    values[13] = lua_l_checknumber(state, 2) as f32;
    values[14] = lua_l_checknumber(state, 3) as f32;
    unsafe { push_mat4(state, values) };
    1
}

unsafe fn mat4_from_scale(state: *mut lua_State) -> c_int {
    let scale_x = lua_l_checknumber(state, 1) as f32;
    let scale_y = if unsafe { lua_isnumber(state, 2) } != 0 {
        lua_l_checknumber(state, 2) as f32
    } else {
        scale_x
    };
    let scale_z = if unsafe { lua_isnumber(state, 3) } != 0 {
        lua_l_checknumber(state, 3) as f32
    } else {
        scale_x
    };
    let mut values = IDENTITY;
    values[0] = scale_x;
    values[5] = scale_y;
    values[10] = scale_z;
    unsafe { push_mat4(state, values) };
    1
}

unsafe fn mat4_from_rotation_x(state: *mut lua_State) -> c_int {
    let radians = lua_l_checknumber(state, 1) as f32;
    let cosine = radians.cos();
    let sine = radians.sin();
    let mut values = IDENTITY;
    values[5] = cosine;
    values[6] = sine;
    values[9] = -sine;
    values[10] = cosine;
    unsafe { push_mat4(state, values) };
    1
}

unsafe fn mat4_from_rotation_y(state: *mut lua_State) -> c_int {
    let radians = lua_l_checknumber(state, 1) as f32;
    let cosine = radians.cos();
    let sine = radians.sin();
    let mut values = IDENTITY;
    values[0] = cosine;
    values[2] = -sine;
    values[8] = sine;
    values[10] = cosine;
    unsafe { push_mat4(state, values) };
    1
}

unsafe fn mat4_from_rotation_z(state: *mut lua_State) -> c_int {
    let radians = lua_l_checknumber(state, 1) as f32;
    let cosine = radians.cos();
    let sine = radians.sin();
    let mut values = IDENTITY;
    values[0] = cosine;
    values[1] = sine;
    values[4] = -sine;
    values[5] = cosine;
    unsafe { push_mat4(state, values) };
    1
}

unsafe fn mat4_perspective(state: *mut lua_State) -> c_int {
    let fov_y = lua_l_checknumber(state, 1) as f32;
    let aspect = lua_l_checknumber(state, 2) as f32;
    let near = lua_l_checknumber(state, 3) as f32;
    let far = lua_l_checknumber(state, 4) as f32;
    let focal_length = 1.0 / (fov_y * 0.5).tan();
    let inverse_depth = 1.0 / (near - far);
    let mut values = [0.0; 16];
    values[0] = focal_length / aspect;
    values[5] = focal_length;
    values[10] = far * inverse_depth;
    values[11] = -1.0;
    values[14] = far * near * inverse_depth;
    unsafe { push_mat4(state, values) };
    1
}

unsafe fn mat4_perspective_reverse_z(state: *mut lua_State) -> c_int {
    let fov_y = lua_l_checknumber(state, 1) as f32;
    let aspect = lua_l_checknumber(state, 2) as f32;
    let near = lua_l_checknumber(state, 3) as f32;
    let focal_length = 1.0 / (fov_y * 0.5).tan();
    let mut values = [0.0; 16];
    values[0] = focal_length / aspect;
    values[5] = focal_length;
    values[11] = -1.0;
    values[14] = near;
    unsafe { push_mat4(state, values) };
    1
}

unsafe fn mat4_look_at(state: *mut lua_State) -> c_int {
    let eye_ptr = lua_l_checkvector(state, 1);
    let center_ptr = lua_l_checkvector(state, 2);
    let up_ptr = lua_l_checkvector(state, 3);
    let eye = unsafe { [*eye_ptr, *eye_ptr.add(1), *eye_ptr.add(2)] };
    let center = unsafe { [*center_ptr, *center_ptr.add(1), *center_ptr.add(2)] };
    let up = unsafe { [*up_ptr, *up_ptr.add(1), *up_ptr.add(2)] };

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
    unsafe { push_mat4(state, values) };
    1
}

unsafe fn mat4_ortho(state: *mut lua_State) -> c_int {
    let left = lua_l_checknumber(state, 1) as f32;
    let right = lua_l_checknumber(state, 2) as f32;
    let bottom = lua_l_checknumber(state, 3) as f32;
    let top = lua_l_checknumber(state, 4) as f32;
    let near = lua_l_checknumber(state, 5) as f32;
    let far = lua_l_checknumber(state, 6) as f32;
    let mut values = IDENTITY;
    values[0] = 2.0 / (right - left);
    values[5] = 2.0 / (top - bottom);
    values[10] = -1.0 / (far - near);
    values[12] = -(right + left) / (right - left);
    values[13] = -(top + bottom) / (top - bottom);
    values[14] = -near / (far - near);
    unsafe { push_mat4(state, values) };
    1
}

unsafe fn mat4_static_multiply(state: *mut lua_State) -> c_int {
    let output = unsafe { check_mat4(state, 1) };
    let lhs = unsafe { (*check_mat4(state, 2)).values };
    let rhs = unsafe { (*check_mat4(state, 3)).values };
    unsafe { (*output).values = multiply(lhs, rhs) };
    unsafe { lua_pushvalue(state, 1) };
    1
}

unsafe fn mat4_static_multiply_affine(state: *mut lua_State) -> c_int {
    let output = unsafe { check_mat4(state, 1) };
    let lhs = unsafe { (*check_mat4(state, 2)).values };
    let rhs = unsafe { (*check_mat4(state, 3)).values };
    unsafe { (*output).values = multiply_affine(lhs, rhs) };
    unsafe { lua_pushvalue(state, 1) };
    1
}

unsafe fn mat4_static_invert(state: *mut lua_State) -> c_int {
    let output = unsafe { check_mat4(state, 1) };
    let input = unsafe { (*check_mat4(state, 2)).values };
    if let Some(values) = invert(input) {
        unsafe { (*output).values = values };
        unsafe { lua_pushboolean(state, 1) };
    } else {
        unsafe { lua_pushboolean(state, 0) };
    }
    1
}

unsafe fn mat4_static_invert_affine(state: *mut lua_State) -> c_int {
    let output = unsafe { check_mat4(state, 1) };
    let input = unsafe { (*check_mat4(state, 2)).values };
    if let Some(values) = invert_affine(input) {
        unsafe { (*output).values = values };
        unsafe { lua_pushboolean(state, 1) };
    } else {
        unsafe { lua_pushboolean(state, 0) };
    }
    1
}

fn matrix_index(name: &[u8]) -> Option<usize> {
    if let [b'm', row, column] = name {
        let row = row.wrapping_sub(b'0');
        let column = column.wrapping_sub(b'0');
        if (1..=4).contains(&row) && (1..=4).contains(&column) {
            return Some(usize::from(column - 1) * 4 + usize::from(row - 1));
        }
    }

    // Mirror C strtol's useful subset for the upstream 1-2 byte fallback:
    // leading ASCII whitespace and an optional sign, then decimal digits.
    if !(1..=2).contains(&name.len()) {
        return None;
    }
    let mut cursor = 0;
    while cursor < name.len() && name[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    let negative = if cursor < name.len() && matches!(name[cursor], b'+' | b'-') {
        let negative = name[cursor] == b'-';
        cursor += 1;
        negative
    } else {
        false
    };
    let first_digit = cursor;
    let mut value = 0_usize;
    while cursor < name.len() && name[cursor].is_ascii_digit() {
        value = value * 10 + usize::from(name[cursor] - b'0');
        cursor += 1;
    }
    (cursor > first_digit && cursor == name.len() && !negative && (1..=16).contains(&value))
        .then_some(value - 1)
}

unsafe fn mat4_index(state: *mut lua_State) -> c_int {
    let matrix = unsafe { check_mat4(state, 1) };
    let mut name_len = 0;
    let name = unsafe { lua_l_checklstring(state, 2, &mut name_len) };
    let name_bytes = unsafe { core::slice::from_raw_parts(name.cast::<u8>(), name_len) };
    if let Some(index) = matrix_index(name_bytes) {
        unsafe { lua_pushnumber(state, f64::from((*matrix).values[index])) };
        return 1;
    }
    unsafe { invalid_index(state, name) }
}

unsafe fn mat4_newindex(state: *mut lua_State) -> c_int {
    let matrix = unsafe { check_mat4(state, 1) };
    let mut name_len = 0;
    let name = unsafe { lua_l_checklstring(state, 2, &mut name_len) };
    let value = lua_l_checknumber(state, 3) as f32;
    let name_bytes = unsafe { core::slice::from_raw_parts(name.cast::<u8>(), name_len) };
    if let Some(index) = matrix_index(name_bytes) {
        unsafe { (*matrix).values[index] = value };
        return 0;
    }
    unsafe { invalid_index(state, name) }
}

unsafe fn invalid_index(state: *mut lua_State, name: *const core::ffi::c_char) -> ! {
    let name = unsafe { CStr::from_ptr(name) }.to_string_lossy();
    unsafe {
        lua_l_error_l(
            state,
            c"'%s' is not a valid index of Mat4".as_ptr(),
            format_args!("'{name}' is not a valid index of Mat4"),
        )
    };
    unsafe { core::hint::unreachable_unchecked() }
}

unsafe fn mat4_mul(state: *mut lua_State) -> c_int {
    let lhs = unsafe { (*check_mat4(state, 1)).values };
    let rhs = unsafe { (*check_mat4(state, 2)).values };
    unsafe { push_mat4(state, multiply(lhs, rhs)) };
    1
}

unsafe fn mat4_eq(state: *mut lua_State) -> c_int {
    let lhs = unsafe { (*check_mat4(state, 1)).values };
    let rhs = unsafe { (*check_mat4(state, 2)).values };
    unsafe { lua_pushboolean(state, c_int::from(lhs == rhs)) };
    1
}

unsafe fn mat4_invert(state: *mut lua_State) -> c_int {
    let matrix = unsafe { (*check_mat4(state, 1)).values };
    if let Some(values) = invert(matrix) {
        unsafe { push_mat4(state, values) };
    } else {
        unsafe { lua_pushnil(state) };
    }
    1
}

unsafe fn mat4_invert_affine(state: *mut lua_State) -> c_int {
    let matrix = unsafe { (*check_mat4(state, 1)).values };
    if let Some(values) = invert_affine(matrix) {
        unsafe { push_mat4(state, values) };
    } else {
        unsafe { lua_pushnil(state) };
    }
    1
}

unsafe fn mat4_transpose(state: *mut lua_State) -> c_int {
    let matrix = unsafe { (*check_mat4(state, 1)).values };
    let mut values = [0.0; 16];
    for row in 0..4 {
        for column in 0..4 {
            values[row * 4 + column] = matrix[column * 4 + row];
        }
    }
    unsafe { push_mat4(state, values) };
    1
}

unsafe fn mat4_transform_point(state: *mut lua_State) -> c_int {
    let matrix = unsafe { &*check_mat4(state, 1) };
    let x = lua_l_checknumber(state, 2) as f32;
    let y = lua_l_checknumber(state, 3) as f32;
    let z = lua_l_checknumber(state, 4) as f32;
    let out = matrix.transform_vec4(x, y, z, 1.0);
    if out[3] != 0.0 && out[3] != 1.0 {
        let inverse_w = 1.0 / out[3];
        unsafe {
            lua_pushvector_lua_state_f32_f32_f32(
                state,
                out[0] * inverse_w,
                out[1] * inverse_w,
                out[2] * inverse_w,
            )
        };
    } else {
        unsafe { lua_pushvector_lua_state_f32_f32_f32(state, out[0], out[1], out[2]) };
    }
    1
}

unsafe fn mat4_transform_vec4(state: *mut lua_State) -> c_int {
    let matrix = unsafe { &*check_mat4(state, 1) };
    let x = lua_l_checknumber(state, 2) as f32;
    let y = lua_l_checknumber(state, 3) as f32;
    let z = lua_l_checknumber(state, 4) as f32;
    let w = lua_l_checknumber(state, 5) as f32;
    for value in matrix.transform_vec4(x, y, z, w) {
        unsafe { lua_pushnumber(state, f64::from(value)) };
    }
    4
}

unsafe fn mat4_write_to_buffer(state: *mut lua_State) -> c_int {
    let matrix = unsafe { &*check_mat4(state, 1) };
    let mut buffer_len = 0;
    let buffer = lua_l_checkbuffer(state, 2, &mut buffer_len).cast::<u8>();
    let offset = lua_l_checkinteger(state, 3);
    if offset < 0 || offset as usize + 64 > buffer_len {
        unsafe {
            lua_l_error_l(
                state,
                c"Mat4:writeToBuffer offset out of range".as_ptr(),
                format_args!("Mat4:writeToBuffer offset out of range"),
            )
        };
        unsafe { core::hint::unreachable_unchecked() }
    }
    unsafe {
        core::ptr::copy_nonoverlapping(
            matrix.values.as_ptr().cast::<u8>(),
            buffer.add(offset as usize),
            64,
        )
    };
    0
}

unsafe fn mat4_namecall(state: *mut lua_State) -> c_int {
    let mut atom = 0;
    let name = unsafe { lua_namecallatom(state, &mut atom) };
    if !name.is_null() {
        match unsafe { CStr::from_ptr(name) }.to_bytes() {
            b"invert" => return unsafe { mat4_invert(state) },
            b"invertAffine" => return unsafe { mat4_invert_affine(state) },
            b"transpose" => return unsafe { mat4_transpose(state) },
            b"transformPoint" => return unsafe { mat4_transform_point(state) },
            b"transformVec4" => return unsafe { mat4_transform_vec4(state) },
            b"writeToBuffer" => return unsafe { mat4_write_to_buffer(state) },
            _ => {}
        }
    }
    let name = if name.is_null() {
        "<unknown>".into()
    } else {
        unsafe { CStr::from_ptr(name) }.to_string_lossy()
    };
    unsafe {
        lua_l_error_l(
            state,
            c"%s is not a valid method of Mat4".as_ptr(),
            format_args!("{name} is not a valid method of Mat4"),
        )
    };
    0
}

macro_rules! direct_field_getter {
    ($name:ident, $index:expr) => {
        unsafe extern "C" fn $name(userdata: *mut c_void, result: *mut c_void) {
            let matrix = unsafe { &*userdata.cast::<ScriptedMat4>() };
            unsafe { lua_userdatadirectfield_setnumber(result, f64::from(matrix.values[$index])) };
        }
    };
}

direct_field_getter!(mat4_get_m11, 0);
direct_field_getter!(mat4_get_m21, 1);
direct_field_getter!(mat4_get_m31, 2);
direct_field_getter!(mat4_get_m41, 3);
direct_field_getter!(mat4_get_m12, 4);
direct_field_getter!(mat4_get_m22, 5);
direct_field_getter!(mat4_get_m32, 6);
direct_field_getter!(mat4_get_m42, 7);
direct_field_getter!(mat4_get_m13, 8);
direct_field_getter!(mat4_get_m23, 9);
direct_field_getter!(mat4_get_m33, 10);
direct_field_getter!(mat4_get_m43, 11);
direct_field_getter!(mat4_get_m14, 12);
direct_field_getter!(mat4_get_m24, 13);
direct_field_getter!(mat4_get_m34, 14);
direct_field_getter!(mat4_get_m44, 15);

unsafe fn register_direct_fields(state: *mut lua_State) {
    for (name, getter) in [
        (c"m11", mat4_get_m11 as unsafe extern "C" fn(_, _)),
        (c"m21", mat4_get_m21 as unsafe extern "C" fn(_, _)),
        (c"m31", mat4_get_m31 as unsafe extern "C" fn(_, _)),
        (c"m41", mat4_get_m41 as unsafe extern "C" fn(_, _)),
        (c"m12", mat4_get_m12 as unsafe extern "C" fn(_, _)),
        (c"m22", mat4_get_m22 as unsafe extern "C" fn(_, _)),
        (c"m32", mat4_get_m32 as unsafe extern "C" fn(_, _)),
        (c"m42", mat4_get_m42 as unsafe extern "C" fn(_, _)),
        (c"m13", mat4_get_m13 as unsafe extern "C" fn(_, _)),
        (c"m23", mat4_get_m23 as unsafe extern "C" fn(_, _)),
        (c"m33", mat4_get_m33 as unsafe extern "C" fn(_, _)),
        (c"m43", mat4_get_m43 as unsafe extern "C" fn(_, _)),
        (c"m14", mat4_get_m14 as unsafe extern "C" fn(_, _)),
        (c"m24", mat4_get_m24 as unsafe extern "C" fn(_, _)),
        (c"m34", mat4_get_m34 as unsafe extern "C" fn(_, _)),
        (c"m44", mat4_get_m44 as unsafe extern "C" fn(_, _)),
    ] {
        unsafe {
            lua_registeruserdatadirectfieldget(
                state,
                MAT4_USERDATA_TAG,
                name.as_ptr(),
                Some(getter),
            )
        };
    }
}

fn multiply(lhs: [f32; 16], rhs: [f32; 16]) -> [f32; 16] {
    let mut output = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = lhs[row] * rhs[column * 4]
                + lhs[4 + row] * rhs[column * 4 + 1]
                + lhs[8 + row] * rhs[column * 4 + 2]
                + lhs[12 + row] * rhs[column * 4 + 3];
        }
    }
    output
}

fn multiply_affine(lhs: [f32; 16], rhs: [f32; 16]) -> [f32; 16] {
    let mut output = [0.0; 16];
    for column in 0..3 {
        for row in 0..4 {
            output[column * 4 + row] = lhs[row] * rhs[column * 4]
                + lhs[4 + row] * rhs[column * 4 + 1]
                + lhs[8 + row] * rhs[column * 4 + 2];
        }
    }
    for row in 0..4 {
        output[12 + row] =
            lhs[row] * rhs[12] + lhs[4 + row] * rhs[13] + lhs[8 + row] * rhs[14] + lhs[12 + row];
    }
    output
}

fn invert(matrix: [f32; 16]) -> Option<[f32; 16]> {
    let m = matrix;
    let mut inverse = [0.0; 16];

    inverse[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
        + m[9] * m[7] * m[14]
        + m[13] * m[6] * m[11]
        - m[13] * m[7] * m[10];
    inverse[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
        - m[8] * m[7] * m[14]
        - m[12] * m[6] * m[11]
        + m[12] * m[7] * m[10];
    inverse[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
        + m[8] * m[7] * m[13]
        + m[12] * m[5] * m[11]
        - m[12] * m[7] * m[9];
    inverse[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
        - m[8] * m[6] * m[13]
        - m[12] * m[5] * m[10]
        + m[12] * m[6] * m[9];
    inverse[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
        - m[9] * m[3] * m[14]
        - m[13] * m[2] * m[11]
        + m[13] * m[3] * m[10];
    inverse[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
        + m[8] * m[3] * m[14]
        + m[12] * m[2] * m[11]
        - m[12] * m[3] * m[10];
    inverse[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
        - m[8] * m[3] * m[13]
        - m[12] * m[1] * m[11]
        + m[12] * m[3] * m[9];
    inverse[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
        + m[8] * m[2] * m[13]
        + m[12] * m[1] * m[10]
        - m[12] * m[2] * m[9];
    inverse[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
        + m[5] * m[3] * m[14]
        + m[13] * m[2] * m[7]
        - m[13] * m[3] * m[6];
    inverse[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
        - m[4] * m[3] * m[14]
        - m[12] * m[2] * m[7]
        + m[12] * m[3] * m[6];
    inverse[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
        + m[4] * m[3] * m[13]
        + m[12] * m[1] * m[7]
        - m[12] * m[3] * m[5];
    inverse[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
        - m[4] * m[2] * m[13]
        - m[12] * m[1] * m[6]
        + m[12] * m[2] * m[5];
    inverse[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
        - m[5] * m[3] * m[10]
        - m[9] * m[2] * m[7]
        + m[9] * m[3] * m[6];
    inverse[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
        + m[4] * m[3] * m[10]
        + m[8] * m[2] * m[7]
        - m[8] * m[3] * m[6];
    inverse[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
        - m[4] * m[3] * m[9]
        - m[8] * m[1] * m[7]
        + m[8] * m[3] * m[5];
    inverse[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
        + m[4] * m[2] * m[9]
        + m[8] * m[1] * m[6]
        - m[8] * m[2] * m[5];

    let determinant =
        m[0] * inverse[0] + m[1] * inverse[4] + m[2] * inverse[8] + m[3] * inverse[12];
    if determinant == 0.0 {
        return None;
    }
    let inverse_determinant = 1.0 / determinant;
    for value in &mut inverse {
        *value *= inverse_determinant;
    }
    Some(inverse)
}

fn invert_affine(m: [f32; 16]) -> Option<[f32; 16]> {
    let cofactor_00 = m[5] * m[10] - m[6] * m[9];
    let cofactor_10 = m[6] * m[8] - m[4] * m[10];
    let cofactor_20 = m[4] * m[9] - m[5] * m[8];
    let determinant = m[0] * cofactor_00 + m[1] * cofactor_10 + m[2] * cofactor_20;
    if determinant == 0.0 {
        return None;
    }
    let inverse_determinant = 1.0 / determinant;
    let cofactor_01 = m[2] * m[9] - m[1] * m[10];
    let cofactor_02 = m[1] * m[6] - m[2] * m[5];
    let cofactor_11 = m[0] * m[10] - m[2] * m[8];
    let cofactor_12 = m[2] * m[4] - m[0] * m[6];
    let cofactor_21 = m[1] * m[8] - m[0] * m[9];
    let cofactor_22 = m[0] * m[5] - m[1] * m[4];

    let r0_0 = cofactor_00 * inverse_determinant;
    let r0_1 = cofactor_10 * inverse_determinant;
    let r0_2 = cofactor_20 * inverse_determinant;
    let r1_0 = cofactor_01 * inverse_determinant;
    let r1_1 = cofactor_11 * inverse_determinant;
    let r1_2 = cofactor_21 * inverse_determinant;
    let r2_0 = cofactor_02 * inverse_determinant;
    let r2_1 = cofactor_12 * inverse_determinant;
    let r2_2 = cofactor_22 * inverse_determinant;

    let translation_x = m[12];
    let translation_y = m[13];
    let translation_z = m[14];
    let inverse_x = -(r0_0 * translation_x + r0_1 * translation_y + r0_2 * translation_z);
    let inverse_y = -(r1_0 * translation_x + r1_1 * translation_y + r1_2 * translation_z);
    let inverse_z = -(r2_0 * translation_x + r2_1 * translation_y + r2_2 * translation_z);

    Some([
        r0_0, r1_0, r2_0, 0.0, r0_1, r1_1, r2_1, 0.0, r0_2, r1_2, r2_2, 0.0, inverse_x, inverse_y,
        inverse_z, 1.0,
    ])
}
