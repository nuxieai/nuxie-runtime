use crate::functions::lua_l_checkvector::lua_l_checkvector;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn vector_dot(L: *mut lua_State) -> core::ffi::c_int {
    let a = lua_l_checkvector(L, 1);
    let b = lua_l_checkvector(L, 2);

    if LUA_VECTOR_SIZE == 4 {
        lua_pushnumber(
            L,
            ((*a.offset(0)) * (*b.offset(0))
                + (*a.offset(1)) * (*b.offset(1))
                + (*a.offset(2)) * (*b.offset(2))
                + (*a.offset(3)) * (*b.offset(3))) as f64,
        );
    } else {
        lua_pushnumber(
            L,
            ((*a.offset(0)) * (*b.offset(0))
                + (*a.offset(1)) * (*b.offset(1))
                + (*a.offset(2)) * (*b.offset(2))) as f64,
        );
    }

    1
}
