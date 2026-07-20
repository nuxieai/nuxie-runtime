use core::ffi::c_int;

use crate::enums::lua_type::lua_Type;
use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_g_readonlyerror::lua_g_readonlyerror;
use crate::macros::api_check::api_check;
use crate::macros::api_checknelems::api_checknelems;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_c_objbarrier::luaC_objbarrier;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::ttisnil::ttisnil;
use crate::macros::ttistable::ttistable;
use crate::macros::ttype::ttype;
use crate::macros::uvalue::uvalue;
use crate::records::lua_state::lua_State;
use crate::records::lua_table::LuaTable;
use crate::records::udata::Udata;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_setmetatable(L: *mut lua_State, objindex: c_int) -> c_int {
    api_checknelems!(L, 1);

    let obj: StkId = index2addr(L, objindex);
    api_check!(L, obj != luaO_nilobject as StkId);

    let mut mt: *mut LuaTable = core::ptr::null_mut();
    if !ttisnil!((*L).top.sub(1)) {
        api_check!(L, ttistable!((*L).top.sub(1)));
        mt = hvalue!((*L).top.sub(1));
    }

    match ttype!(obj) {
        x if x == lua_Type::LUA_TTABLE as c_int => {
            let h = hvalue!(obj);
            if (*h).readonly != 0 {
                lua_g_readonlyerror(L);
            }
            (*h).metatable = mt;
            if !mt.is_null() {
                luaC_objbarrier!(L, h, mt);
            }
        }
        x if x == lua_Type::LUA_TUSERDATA as c_int => {
            let u = uvalue!(obj) as *const _ as *mut Udata;
            (*u).metatable = mt;
            if !mt.is_null() {
                luaC_objbarrier!(L, u, mt);
            }
        }
        _ => {
            (*(*L).global).mt[ttype!(obj) as usize] = mt;
        }
    }

    (*L).top = (*L).top.sub(1);
    1
}
