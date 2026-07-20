//! Node: `cxx:Function:Luau.VM:VM/src/lapi.cpp:206:lua_xpush`
//! Source: `VM/src/lapi.cpp:206-212` (hand-ported)

use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::macros::api_check::api_check;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::setobj_2_s::setobj2s;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_xpush(from: *mut lua_State, to: *mut lua_State, idx: core::ffi::c_int) {
    api_check!(from, (*from).global == (*to).global);
    lua_c_threadbarrier_lapi(to);
    let o = index2addr(from, idx);
    setobj2s!(to, (*to).top, o);
    api_incr_top!(to);
}
