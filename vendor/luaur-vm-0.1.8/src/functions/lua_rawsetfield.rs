use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_g_readonlyerror::lua_g_readonlyerror;
use crate::functions::lua_h_setstr::lua_h_setstr;
use crate::macros::api_check::api_check;
use crate::macros::api_checknelems::api_checknelems;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_c_barriert::luaC_barriert;
use crate::macros::lua_s_new::luaS_new;
use crate::macros::setobj_2_t::setobj2t;
use crate::macros::ttistable::ttistable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use core::ffi::c_char;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn lua_rawsetfield(L: *mut lua_State, idx: c_int, k: *const c_char) {
    api_checknelems!(L, 1);
    let t: StkId = index2addr(L, idx);
    api_check!(L, ttistable!(t));
    if (*hvalue!(t)).readonly != 0 {
        lua_g_readonlyerror(L);
    }
    setobj2t!(
        L,
        lua_h_setstr(L, hvalue!(t), luaS_new(L, k)),
        (*L).top.offset(-1)
    );
    luaC_barriert!(L, hvalue!(t), (*L).top.offset(-1));
    (*L).top = (*L).top.offset(-1);
}
