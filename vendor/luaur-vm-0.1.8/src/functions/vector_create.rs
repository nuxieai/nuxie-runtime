use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_l_checknumber::lua_l_checknumber;
use crate::functions::lua_pushvector_lapi::lua_pushvector_lua_state_f32_f32_f32_f32;
use crate::functions::lua_pushvector_lapi_alt_b::lua_pushvector_lua_state_f32_f32_f32;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn vector_create(L: *mut lua_State) -> core::ffi::c_int {
    let count = lua_gettop(L);

    let x = lua_l_checknumber(L, 1);
    let y = lua_l_checknumber(L, 2);
    let z = if count >= 3 {
        lua_l_checknumber(L, 3)
    } else {
        0.0
    };

    if LUA_VECTOR_SIZE == 4 {
        let w = if count >= 4 {
            lua_l_checknumber(L, 4)
        } else {
            0.0
        };
        lua_pushvector_lua_state_f32_f32_f32_f32(L, x as crate::type_aliases::lua_vector_type::LuaVectorType, y as crate::type_aliases::lua_vector_type::LuaVectorType, z as crate::type_aliases::lua_vector_type::LuaVectorType, w as crate::type_aliases::lua_vector_type::LuaVectorType);
    } else {
        lua_pushvector_lua_state_f32_f32_f32(L, x as crate::type_aliases::lua_vector_type::LuaVectorType, y as crate::type_aliases::lua_vector_type::LuaVectorType, z as crate::type_aliases::lua_vector_type::LuaVectorType);
    }

    1
}
