use crate::functions::lua_l_checkvector::lua_l_checkvector;
use crate::functions::lua_pushvector_lapi::lua_pushvector_lua_state_f32_f32_f32_f32;
use crate::functions::lua_pushvector_lapi_alt_b::lua_pushvector_lua_state_f32_f32_f32;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn vector_cross(L: *mut lua_State) -> core::ffi::c_int {
    let a = lua_l_checkvector(L, 1);
    let b = lua_l_checkvector(L, 2);

    if LUA_VECTOR_SIZE == 4 {
        lua_pushvector_lua_state_f32_f32_f32_f32(
            L,
            (*a.offset(1)) * (*b.offset(2)) - (*a.offset(2)) * (*b.offset(1)),
            (*a.offset(2)) * (*b.offset(0)) - (*a.offset(0)) * (*b.offset(2)),
            (*a.offset(0)) * (*b.offset(1)) - (*a.offset(1)) * (*b.offset(0)),
            0.0f32,
        );
    } else {
        lua_pushvector_lua_state_f32_f32_f32(
            L,
            (*a.offset(1)) * (*b.offset(2)) - (*a.offset(2)) * (*b.offset(1)),
            (*a.offset(2)) * (*b.offset(0)) - (*a.offset(0)) * (*b.offset(2)),
            (*a.offset(0)) * (*b.offset(1)) - (*a.offset(1)) * (*b.offset(0)),
        );
    }

    1
}
