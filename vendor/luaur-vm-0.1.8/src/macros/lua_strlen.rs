use crate::functions::lua_objlen::lua_objlen;

#[allow(non_snake_case)]
pub unsafe extern "C" fn lua_strlen(
    L: *mut crate::records::lua_state::lua_State,
    idx: core::ffi::c_int,
) -> usize {
    lua_objlen(L, idx) as usize
}
