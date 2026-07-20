use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_insert::lua_insert;
use crate::functions::lua_pushboolean::lua_pushboolean;
use crate::functions::lua_rawcheckstack::lua_rawcheckstack;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_b_pcallcont(L: *mut lua_State, status: core::ffi::c_int) -> core::ffi::c_int {
    if status == 0 {
        lua_rawcheckstack(L, 1);
        lua_pushboolean(L, 1);
        lua_insert(L, 1);
        lua_gettop(L)
    } else {
        lua_rawcheckstack(L, 1);
        lua_pushboolean(L, 0);
        lua_insert(L, -2);
        2
    }
}
