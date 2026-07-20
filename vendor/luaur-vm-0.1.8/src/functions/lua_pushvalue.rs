//! Node: `cxx:Function:Luau.VM:VM/src/lapi.cpp:310:lua_pushvalue`
//! Source: `VM/src/lapi.cpp:310-316` (hand-ported)

use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::setobj_2_s::setobj2s;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_pushvalue(L: *mut lua_State, idx: c_int) {
    lua_c_threadbarrier_lapi(L);
    let o: StkId = index2addr(L, idx);
    setobj2s!(L, (*L).top, o);
    api_incr_top!(L);
}
