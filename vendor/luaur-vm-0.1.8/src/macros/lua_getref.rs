use crate::functions::lua_rawgeti::lua_rawgeti;
use crate::macros::lua_registryindex::LUA_REGISTRYINDEX;

#[inline(always)]
pub fn lua_getref(l: *mut crate::records::lua_state::lua_State, ref_: core::ffi::c_int) {
    unsafe {
        lua_rawgeti(l, LUA_REGISTRYINDEX, ref_);
    }
}
