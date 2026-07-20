use crate::functions::lua_setfield::lua_setfield;
use crate::macros::lua_pushcclosure::LUA_PUSHCCLOSURE;
use crate::macros::lua_pushcfunction::LUA_PUSHCFUNCTION;
use crate::type_aliases::lua_c_function::lua_CFunction;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

#[allow(non_snake_case)]
pub(crate) unsafe fn auxopen(
    L: *mut lua_State,
    name: *const c_char,
    f: lua_CFunction,
    u: lua_CFunction,
) {
    LUA_PUSHCFUNCTION(L, u, core::ptr::null());
    LUA_PUSHCCLOSURE(L, f, name, 1);
    lua_setfield(L, -2, name);
}
