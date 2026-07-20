use crate::functions::lua_getfield::lua_getfield;
use crate::macros::lua_globalsindex::LUA_GLOBALSINDEX;
use core::ffi::{c_char, c_int};

#[inline(always)]
pub unsafe fn lua_getglobal(
    L: *mut crate::records::lua_state::lua_State,
    s: *const c_char,
) -> c_int {
    lua_getfield(L, LUA_GLOBALSINDEX, s)
}
