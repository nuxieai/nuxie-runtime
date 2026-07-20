use crate::functions::lua_settop::lua_settop;
use crate::records::lua_state::lua_State;
use core::ffi::c_int;

#[inline]
pub unsafe fn lua_pop(l: *mut lua_State, n: c_int) {
    lua_settop(l, -n - 1);
}
