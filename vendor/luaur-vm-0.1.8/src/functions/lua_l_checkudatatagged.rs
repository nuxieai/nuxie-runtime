use crate::functions::lua_getuserdataname::lua_getuserdataname;
use crate::functions::lua_l_typeerror_l::lua_l_typeerror_l;
use crate::functions::lua_touserdatatagged::lua_touserdatatagged;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_int, c_void, CStr};

#[allow(non_snake_case)]
pub unsafe fn lua_l_checkudatatagged(L: *mut lua_State, ud: c_int, tag: c_int) -> *mut c_void {
    let p = lua_touserdatatagged(L, ud, tag);
    if !p.is_null() {
        return p;
    }

    let tname = CStr::from_ptr(lua_getuserdataname(L, tag)).to_string_lossy();
    lua_l_typeerror_l(L, ud, &tname)
}
