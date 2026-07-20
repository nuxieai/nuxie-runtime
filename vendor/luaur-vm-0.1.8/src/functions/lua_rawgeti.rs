use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_h_getnum::lua_h_getnum;
use crate::macros::api_check::api_check;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::hvalue::hvalue;
use crate::macros::setobj_2_s::setobj2s;
use crate::macros::ttistable::ttistable;
use crate::macros::ttype::ttype;
use crate::records::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_rawgeti(L: *mut lua_State, idx: c_int, n: c_int) -> c_int {
    lua_c_threadbarrier_lapi(L);

    let t: StkId = index2addr(L, idx);
    api_check!(L, ttistable!(t));

    setobj2s!(L, (*L).top, lua_h_getnum(hvalue!(t), n));
    api_incr_top!(L);

    ttype!((*L).top.sub(1))
}
