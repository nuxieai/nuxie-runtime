use crate::functions::lua_newuserdatatagged::lua_newuserdatatagged;

#[inline]
pub fn lua_newuserdata(
    l: *mut crate::records::lua_state::lua_State,
    s: usize,
) -> *mut core::ffi::c_void {
    lua_newuserdatatagged(l, s, 0)
}
