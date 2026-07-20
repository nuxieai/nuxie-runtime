use crate::functions::lua_l_checknumber::lua_l_checknumber;
use crate::functions::lua_l_checkvector::lua_l_checkvector;
use crate::functions::lua_pushvector_lapi::lua_pushvector_lua_state_f32_f32_f32_f32;
use crate::functions::lua_pushvector_lapi_alt_b::lua_pushvector_lua_state_f32_f32_f32;
use crate::functions::luai_lerpf::luai_lerpf;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn vector_lerp(L: *mut lua_State) -> core::ffi::c_int {
    let a = lua_l_checkvector(L, 1);
    let b = lua_l_checkvector(L, 2);
    let t = lua_l_checknumber(L, 3) as f32;

    if LUA_VECTOR_SIZE == 4 {
        lua_pushvector_lua_state_f32_f32_f32_f32(
            L,
            luai_lerpf(*a.offset(0), *b.offset(0), t),
            luai_lerpf(*a.offset(1), *b.offset(1), t),
            luai_lerpf(*a.offset(2), *b.offset(2), t),
            luai_lerpf(*a.offset(3), *b.offset(3), t),
        );
    } else {
        lua_pushvector_lua_state_f32_f32_f32(
            L,
            luai_lerpf(*a.offset(0), *b.offset(0), t),
            luai_lerpf(*a.offset(1), *b.offset(1), t),
            luai_lerpf(*a.offset(2), *b.offset(2), t),
        );
    }

    1
}
