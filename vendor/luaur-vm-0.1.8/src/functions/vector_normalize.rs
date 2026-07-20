use crate::functions::lua_l_checkvector::lua_l_checkvector;
use crate::functions::lua_pushvector_lapi::lua_pushvector_lua_state_f32_f32_f32_f32;
use crate::functions::lua_pushvector_lapi_alt_b::lua_pushvector_lua_state_f32_f32_f32;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn vector_normalize(L: *mut lua_State) -> core::ffi::c_int {
    let v = lua_l_checkvector(L, 1);

    if LUA_VECTOR_SIZE == 4 {
        let v0 = *v.offset(0);
        let v1 = *v.offset(1);
        let v2 = *v.offset(2);
        let v3 = *v.offset(3);

        let inv_sqrt = 1.0f32 / (v0 * v0 + v1 * v1 + v2 * v2 + v3 * v3).sqrt();
        lua_pushvector_lua_state_f32_f32_f32_f32(
            L,
            v0 * inv_sqrt,
            v1 * inv_sqrt,
            v2 * inv_sqrt,
            v3 * inv_sqrt,
        );
    } else {
        let v0 = *v.offset(0);
        let v1 = *v.offset(1);
        let v2 = *v.offset(2);

        let inv_sqrt = 1.0f32 / (v0 * v0 + v1 * v1 + v2 * v2).sqrt();
        lua_pushvector_lua_state_f32_f32_f32(L, v0 * inv_sqrt, v1 * inv_sqrt, v2 * inv_sqrt);
    }

    1
}
