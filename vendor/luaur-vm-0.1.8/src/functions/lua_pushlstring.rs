use core::ffi::c_char;

use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_s_newlstr::luaS_newlstr;
use crate::macros::api_check::api_check;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::setsvalue::setsvalue;
use crate::records::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_pushlstring(L: *mut lua_State, s: *const c_char, len: usize) {
    api_check!(L, !s.is_null());
    luaC_checkGC!(L);
    lua_c_threadbarrier_lapi(L);
    setsvalue!(L, (*L).top, luaS_newlstr(L, s, len));
    api_incr_top!(L);
}
