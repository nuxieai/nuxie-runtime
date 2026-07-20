use core::ffi::{c_char, c_int};

use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_v_settable::lua_v_settable;
use crate::macros::api_check::api_check;
use crate::macros::api_checknelems::api_checknelems;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::lua_s_new::luaS_new;
use crate::macros::setsvalue::setsvalue;
use crate::records::lua_state::lua_State;
use crate::records::lua_t_value::TValue;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_setfield(L: *mut lua_State, idx: c_int, k: *const c_char) {
    api_checknelems!(L, 1);

    let t: StkId = index2addr(L, idx);
    api_check!(L, t != luaO_nilobject as StkId);

    let mut key = TValue::default();
    setsvalue!(L, &mut key, luaS_new(L, k));
    lua_v_settable(L, t, &mut key, (*L).top.sub(1));
    (*L).top = (*L).top.sub(1);
}
