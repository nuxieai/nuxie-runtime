use crate::functions::lua_l_checkvector::lua_l_checkvector;
use crate::functions::lua_pushvector_lapi::lua_pushvector_lua_state_f32_f32_f32_f32;
use crate::functions::lua_pushvector_lapi_alt_b::lua_pushvector_lua_state_f32_f32_f32;
use crate::functions::luaui_signf::luaui_signf;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn vector_sign(L: *mut lua_State) -> core::ffi::c_int {
    let v = lua_l_checkvector(L, 1);

    if LUA_VECTOR_SIZE == 4 {
        lua_pushvector_lua_state_f32_f32_f32_f32(
            L,
            luaui_signf(*v.offset(0)),
            luaui_signf(*v.offset(1)),
            luaui_signf(*v.offset(2)),
            luaui_signf(*v.offset(3)),
        );
    } else {
        lua_pushvector_lua_state_f32_f32_f32(
            L,
            luaui_signf(*v.offset(0)),
            luaui_signf(*v.offset(1)),
            luaui_signf(*v.offset(2)),
        );
    }

    1
}
