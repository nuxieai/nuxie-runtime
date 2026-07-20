use crate::functions::lua_l_checkboolean::lua_l_checkboolean;
use crate::macros::lua_l_opt::luaL_opt;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub fn lua_l_optboolean(L: *mut lua_State, narg: core::ffi::c_int, def: bool) -> bool {
    unsafe {
        let def_cint = if def { 1 } else { 0 };
        luaL_opt!(L, lua_l_checkboolean, narg, def_cint) != 0
    }
}
