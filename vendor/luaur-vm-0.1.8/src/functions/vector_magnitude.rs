use crate::functions::lua_l_checkvector::lua_l_checkvector;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn vector_magnitude(L: *mut lua_State) -> core::ffi::c_int {
    let v = lua_l_checkvector(L, 1);

    if LUA_VECTOR_SIZE == 4 {
        lua_pushnumber(
            L,
            ((*v.offset(0)) * (*v.offset(0))
                + (*v.offset(1)) * (*v.offset(1))
                + (*v.offset(2)) * (*v.offset(2))
                + (*v.offset(3)) * (*v.offset(3)))
            .sqrt() as f64,
        );
    } else {
        lua_pushnumber(
            L,
            ((*v.offset(0)) * (*v.offset(0))
                + (*v.offset(1)) * (*v.offset(1))
                + (*v.offset(2)) * (*v.offset(2)))
            .sqrt() as f64,
        );
    }

    1
}
