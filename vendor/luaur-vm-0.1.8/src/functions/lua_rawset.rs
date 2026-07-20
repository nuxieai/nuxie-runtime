use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_g_readonlyerror::lua_g_readonlyerror;
use crate::functions::lua_h_set::luaH_set;
use crate::macros::api_check::api_check;
use crate::macros::api_checknelems::api_checknelems;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_c_barriert::luaC_barriert;
use crate::macros::setobj_2_t::setobj2t;
use crate::macros::ttistable::ttistable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_rawset(L: *mut lua_State, idx: core::ffi::c_int) {
    api_checknelems!(L, 2);
    let t: StkId = index2addr(L, idx);
    api_check!(L, ttistable!(t));
    if (*hvalue!(t)).readonly != 0 {
        lua_g_readonlyerror(L);
    }
    let key = (*L).top.offset(-2);
    let value = (*L).top.offset(-1);
    let slot = luaH_set(L, hvalue!(t), key);
    setobj2t!(L, slot, value);
    luaC_barriert!(L, hvalue!(t), value);
    (*L).top = (*L).top.offset(-2);
}
