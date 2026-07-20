use core::ffi::{c_char, c_int};

use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_v_gettable::lua_v_gettable;
use crate::macros::api_check::api_check;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::lua_s_new::luaS_new;
use crate::macros::setsvalue::setsvalue;
use crate::macros::ttype::ttype;
use crate::records::lua_state::lua_State;
use crate::records::lua_t_value::TValue;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_getfield(L: *mut lua_State, idx: c_int, k: *const c_char) -> c_int {
    lua_c_threadbarrier_lapi(L);

    let t: StkId = index2addr(L, idx);
    api_check!(L, t != luaO_nilobject as StkId);

    let mut key = TValue::default();
    setsvalue!(L, &mut key, luaS_new(L, k));
    lua_v_gettable(L, t, &mut key, (*L).top);
    api_incr_top!(L);

    ttype!((*L).top.sub(1))
}
