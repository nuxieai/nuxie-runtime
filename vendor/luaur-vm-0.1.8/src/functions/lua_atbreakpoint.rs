use crate::records::lua_state::lua_State;

pub unsafe fn lua_atbreakpoint(l: *mut lua_State) -> core::ffi::c_int {
    crate::functions::lua_g_onbreak::luaG_onbreak(l) as core::ffi::c_int
}
