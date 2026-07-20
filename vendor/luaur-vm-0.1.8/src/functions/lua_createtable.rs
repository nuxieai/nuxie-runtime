use core::ffi::c_int;

use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_h_new::lua_h_new;
use crate::macros::api_check::api_check;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::sethvalue::sethvalue;
use crate::records::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_createtable(L: *mut lua_State, narray: c_int, nrec: c_int) {
    api_check!(L, narray >= 0 && nrec >= 0);
    luaC_checkGC!(L);
    lua_c_threadbarrier_lapi(L);
    sethvalue!(L, (*L).top, lua_h_new(L, narray, nrec));
    api_incr_top!(L);
}
