use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::macros::lua_l_opt::luaL_opt;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_l_optinteger(
    L: *mut lua_State,
    narg: core::ffi::c_int,
    def: core::ffi::c_int,
) -> core::ffi::c_int {
    unsafe { luaL_opt!(L, lua_l_checkinteger, narg, def) }
}
