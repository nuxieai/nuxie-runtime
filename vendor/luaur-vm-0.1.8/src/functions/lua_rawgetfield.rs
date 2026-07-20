use core::ffi::{c_char, c_int};

use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_h_getstr::lua_h_getstr;
use crate::macros::api_check::api_check;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_s_new::luaS_new;
use crate::macros::setobj_2_s::setobj2s;
use crate::macros::setsvalue::setsvalue;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttistable::ttistable;
use crate::macros::ttype::ttype;
use crate::records::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_rawgetfield(L: *mut lua_State, idx: c_int, k: *const c_char) -> c_int {
    lua_c_threadbarrier_lapi(L);

    let t: StkId = index2addr(L, idx);
    api_check!(L, ttistable!(t));

    let mut key = TValue::default();
    setsvalue!(L, &mut key, luaS_new(L, k));
    setobj2s!(
        L,
        (*L).top,
        lua_h_getstr(hvalue!(t), tsvalue!(&key) as *mut _)
    );
    api_incr_top!(L);

    ttype!((*L).top.sub(1))
}
