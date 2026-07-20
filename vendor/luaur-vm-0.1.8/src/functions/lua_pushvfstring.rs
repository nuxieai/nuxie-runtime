use core::ffi::c_char;

use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_o_pushvfstring::luaO_pushvfstring;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::records::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_pushvfstring(
    L: *mut lua_State,
    fmt: *const c_char,
    argp: core::fmt::Arguments<'_>,
) -> *const c_char {
    luaC_checkGC!(L);
    lua_c_threadbarrier_lapi(L);
    luaO_pushvfstring(L, fmt, argp)
}
