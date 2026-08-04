use crate::functions::lua_h_getstr::luaH_getstr;
use crate::macros::api_check::api_check;
use crate::macros::getstr::getstr;
use crate::macros::lua_utag_limit::LUA_UTAG_LIMIT;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttisstring::ttisstring;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::tms::TMS;
use core::ffi::{c_char, c_int};

#[allow(non_snake_case)]
pub unsafe fn lua_getuserdataname(L: *mut lua_State, tag: c_int) -> *const c_char {
    api_check!(L, (tag as u32) < LUA_UTAG_LIMIT as u32);

    let mut tname = c"userdata".as_ptr();
    let mt = (*(*L).global).udatamt[tag as usize];

    if !mt.is_null() {
        let type_ = luaH_getstr(mt, (*(*L).global).tmname[TMS::TM_TYPE as usize]);
        if ttisstring!(type_) {
            tname = getstr(tsvalue!(type_));
        }
    }

    tname
}
