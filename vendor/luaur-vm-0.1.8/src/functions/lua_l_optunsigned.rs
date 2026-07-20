use crate::functions::lua_l_checkunsigned::lua_l_checkunsigned;
use crate::macros::lua_l_opt::luaL_opt;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_l_optunsigned(
    L: *mut lua_State,
    narg: core::ffi::c_int,
    def: core::ffi::c_uint,
) -> core::ffi::c_uint {
    unsafe { luaL_opt!(L, lua_l_checkunsigned, narg, def) }
}
