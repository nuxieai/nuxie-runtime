use crate::functions::lua_o_pushvfstring::luaO_pushvfstring;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

pub fn luaO_pushfstring(
    L: *mut lua_State,
    fmt: *const c_char,
    args: core::fmt::Arguments<'_>,
) -> *const c_char {
    // In the Luau Rust port, printf-style varargs are handled by passing core::fmt::Arguments.
    unsafe { luaO_pushvfstring(L, fmt, args) }
}
