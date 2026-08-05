use crate::records::lua_state::lua_State;

pub unsafe fn lua_hascustomexecution(
    l: *mut lua_State,
    level: core::ffi::c_int,
) -> core::ffi::c_int {
    crate::functions::lua_g_hasnative::lua_g_hasnative(l, level)
}
