use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_l_checkvector::lua_l_checkvector;
use crate::functions::lua_pushvector_lapi::lua_pushvector_lua_state_f32_f32_f32_f32;
use crate::functions::lua_pushvector_lapi_alt_b::lua_pushvector_lua_state_f32_f32_f32;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn vector_min(L: *mut lua_State) -> core::ffi::c_int {
    let n = lua_gettop(L);
    let v = lua_l_checkvector(L, 1);

    let mut result = [0.0f32; 4];
    if LUA_VECTOR_SIZE == 4 {
        result[0] = *v.offset(0);
        result[1] = *v.offset(1);
        result[2] = *v.offset(2);
        result[3] = *v.offset(3);
    } else {
        result[0] = *v.offset(0);
        result[1] = *v.offset(1);
        result[2] = *v.offset(2);
    }

    for i in 2..=n {
        let b = lua_l_checkvector(L, i);

        if *b.offset(0) < result[0] {
            result[0] = *b.offset(0);
        }
        if *b.offset(1) < result[1] {
            result[1] = *b.offset(1);
        }
        if *b.offset(2) < result[2] {
            result[2] = *b.offset(2);
        }
        if LUA_VECTOR_SIZE == 4 {
            if *b.offset(3) < result[3] {
                result[3] = *b.offset(3);
            }
        }
    }

    if LUA_VECTOR_SIZE == 4 {
        lua_pushvector_lua_state_f32_f32_f32_f32(L, result[0], result[1], result[2], result[3]);
    } else {
        lua_pushvector_lua_state_f32_f32_f32(L, result[0], result[1], result[2]);
    }

    1
}
