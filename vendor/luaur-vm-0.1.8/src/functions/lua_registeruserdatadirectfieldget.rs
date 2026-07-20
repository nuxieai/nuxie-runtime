use core::ffi::{c_char, c_int, c_void};

use crate::functions::lua_h_new::lua_h_new;
use crate::functions::lua_h_setstr::lua_h_setstr;
use crate::macros::api_check::api_check;
use crate::macros::fixedbit::FIXEDBIT;
use crate::macros::l_setbit::l_setbit;
use crate::macros::lua_s_new::luaS_new;
use crate::macros::lua_utag_limit::LUA_UTAG_LIMIT;
use crate::macros::setpvalue::setpvalue;
use crate::records::global_state::global_State;
use crate::records::lua_state::lua_State;
use crate::records::t_string::TString;
use crate::type_aliases::lua_userdata_direct_field_get::lua_UserdataDirectFieldGet;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_registeruserdatadirectfieldget(
    L: *mut lua_State,
    tag: c_int,
    field: *const c_char,
    fn_: lua_UserdataDirectFieldGet,
) {
    if !luaur_common::FFlag::LuauDirectFieldGet.get() {
        return;
    }

    api_check!(L, (tag as u32) < LUA_UTAG_LIMIT as u32);
    api_check!(L, !field.is_null());
    api_check!(L, fn_.is_some());

    let g: *mut global_State = (*L).global;

    if (*g).udatadirectfields[tag as usize].is_null() {
        (*g).udatadirectfields[tag as usize] = lua_h_new(L, 0, 1);
    }

    let ts: *mut TString = luaS_new(L, field);
    l_setbit!((*ts).hdr.marked, FIXEDBIT);

    let slot: *mut TValue = lua_h_setstr(L, (*g).udatadirectfields[tag as usize], ts);
    setpvalue!(slot, fn_.unwrap() as *mut c_void, 0);
}
