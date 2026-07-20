use crate::functions::lua_concat::lua_concat;
use crate::functions::lua_error::lua_error;
use crate::functions::lua_l_where::lua_l_where;
use crate::functions::lua_pushvfstring::lua_pushvfstring;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

#[allow(non_snake_case)]
pub unsafe fn lua_l_error_l(L: *mut lua_State, fmt: *const c_char, args: core::fmt::Arguments<'_>) {
    lua_l_where(L, 1);
    lua_pushvfstring(L, fmt, args);
    lua_concat(L, 2);
    lua_error(L);
}
