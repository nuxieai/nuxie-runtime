use crate::records::lua_state::lua_State;

pub unsafe fn lua_incustomexecution(
    l: *mut lua_State,
    level: core::ffi::c_int,
) -> core::ffi::c_int {
    crate::functions::lua_g_isnative::luaG_isnative(l, level)
}
