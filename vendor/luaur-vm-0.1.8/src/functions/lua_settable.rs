use core::ffi::c_int;

use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_v_settable::lua_v_settable;
use crate::macros::api_check::api_check;
use crate::macros::api_checknelems::api_checknelems;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::records::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_settable(L: *mut lua_State, idx: c_int) {
    api_checknelems!(L, 2);

    let t: StkId = index2addr(L, idx);
    api_check!(L, t != luaO_nilobject as StkId);
    lua_v_settable(L, t, (*L).top.sub(2), (*L).top.sub(1));
    (*L).top = (*L).top.sub(2);
}
