use crate::macros::api_check::api_check;
use crate::macros::getstr::getstr;
use crate::macros::lua_lutag_limit::LUA_LUTAG_LIMIT;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_string::TString;

pub fn lua_getlightuserdataname(L: *mut lua_State, tag: i32) -> *const core::ffi::c_char {
    api_check!(L, (tag as u32) < LUA_LUTAG_LIMIT as u32);

    unsafe {
        let global = (*L).global;
        let name = (*global).lightuserdataname[tag as usize];
        if name.is_null() {
            core::ptr::null()
        } else {
            getstr(name as *const TString)
        }
    }
}
