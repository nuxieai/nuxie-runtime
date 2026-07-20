use crate::functions::lua_l_checkvector::lua_l_checkvector;
use crate::macros::lua_l_opt::luaL_opt;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub fn lua_l_optvector(L: *mut lua_State, narg: core::ffi::c_int, def: *const f32) -> *const f32 {
    unsafe { luaL_opt!(L, lua_l_checkvector, narg, def) }
}
