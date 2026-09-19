use crate::functions::lua_getfield::lua_getfield;
use crate::macros::lua_registryindex::LUA_REGISTRYINDEX;

#[allow(non_snake_case)]
#[inline(always)]
pub fn luaL_getmetatable(
    l: *mut crate::records::lua_state::lua_State,
    n: *const core::ffi::c_char,
) -> crate::records::lua_exception::LuaResult<core::ffi::c_int> {
    unsafe { lua_getfield(l, LUA_REGISTRYINDEX, n) }
}
