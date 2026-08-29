use crate::mechanical_port::source::lua::rive_lua_libs::{LuaFunction, LuaState};

use super::{
    lua_color::luaopen_rive_color, lua_mat2d::luaopen_rive_mat2d, lua_mat4::luaopen_rive_mat4,
    lua_vec2d::luaopen_rive_vector,
};

pub fn luaopen_rive_math(state: &mut LuaState) -> i32 {
    let math_types: [LuaFunction; 4] = [
        luaopen_rive_vector,
        luaopen_rive_mat2d,
        luaopen_rive_mat4,
        luaopen_rive_color,
    ];
    math_types.into_iter().map(|open| open(state)).sum()
}
