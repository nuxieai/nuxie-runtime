use crate::functions::lua_setfield::lua_setfield;
use crate::macros::lua_globalsindex::LUA_GLOBALSINDEX;
use core::ffi::{c_char, c_int};

#[inline(always)]
pub unsafe fn lua_setglobal(l: *mut crate::records::lua_state::lua_State, s: *const c_char) {
    lua_setfield(l, LUA_GLOBALSINDEX, s);
}

#[inline(always)]
pub unsafe fn lua_getglobal(l: *mut crate::records::lua_state::lua_State, s: *const c_char) {
    crate::functions::lua_getfield::lua_getfield(l, LUA_GLOBALSINDEX, s);
}
