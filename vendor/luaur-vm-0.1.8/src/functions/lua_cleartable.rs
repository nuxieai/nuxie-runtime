use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_g_readonlyerror::lua_g_readonlyerror;
use crate::functions::lua_h_clear::lua_h_clear;
use crate::macros::api_check::api_check;
use crate::macros::hvalue::hvalue;
use crate::macros::ttistable::ttistable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_cleartable(L: *mut lua_State, idx: core::ffi::c_int) {
    let t: StkId = index2addr(L, idx);
    api_check!(L, ttistable!(t));
    let tt = hvalue!(t);
    if (*tt).readonly != 0 {
        lua_g_readonlyerror(L);
    }
    lua_h_clear(tt);
}
