use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_g_readonlyerror::lua_g_readonlyerror;
use crate::functions::lua_h_setnum::luaH_setnum;
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
pub unsafe fn lua_rawseti(L: *mut lua_State, idx: core::ffi::c_int, n: core::ffi::c_int) {
    api_checknelems!(L, 1);
    let o: StkId = index2addr(L, idx);
    api_check!(L, ttistable!(o));
    if (*hvalue!(o)).readonly != 0 {
        lua_g_readonlyerror(L);
    }
    setobj2t!(L, luaH_setnum(L, hvalue!(o), n), (*L).top.offset(-1));
    luaC_barriert!(L, hvalue!(o), (*L).top.offset(-1));
    (*L).top = (*L).top.offset(-1);
}
