//! Node: `cxx:Function:Luau.VM:VM/src/lapi.cpp:1350:lua_next`
//! Source: `VM/src/lapi.cpp:1350-1363` (hand-ported)

use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_h_next::lua_h_next;
use crate::macros::api_check::api_check;
use crate::macros::api_checknelems::api_checknelems;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::hvalue::hvalue;
use crate::macros::ttistable::ttistable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_next(L: *mut lua_State, idx: c_int) -> c_int {
    api_checknelems!(L, 1);
    lua_c_threadbarrier_lapi(L);
    let t: StkId = index2addr(L, idx);
    api_check!(L, ttistable!(t));

    let more = lua_h_next(L, hvalue!(t), (*L).top.sub(1));
    if more != 0 {
        api_incr_top!(L);
    } else {
        (*L).top = (*L).top.sub(1);
    }
    more
}
