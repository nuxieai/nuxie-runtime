use crate::functions::lua_l_checknumber::lua_l_checknumber;
use crate::macros::lua_l_opt::luaL_opt;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_l_optnumber(L: *mut lua_State, narg: core::ffi::c_int, def: f64) -> f64 {
    unsafe { luaL_opt!(L, lua_l_checknumber, narg, def) }
}
