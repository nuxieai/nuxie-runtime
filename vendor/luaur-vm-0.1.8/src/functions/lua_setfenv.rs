use core::ffi::c_int;

use crate::enums::lua_type::lua_Type;
use crate::functions::index_2_addr::index_2_addr;
use crate::macros::api_check::api_check;
use crate::macros::api_checknelems::api_checknelems;
use crate::macros::gcvalue::gcvalue;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_c_objbarrier::luaC_objbarrier;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::ttistable::ttistable;
use crate::macros::ttype::ttype;
use crate::records::closure::Closure;
use crate::records::lua_state::lua_State;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_setfenv(L: *mut lua_State, idx: c_int) -> c_int {
    let mut res: c_int = 1;
    api_checknelems!(L, 1);
    let o: StkId = index_2_addr(L, idx);
    api_check!(L, o != luaO_nilobject as StkId);
    api_check!(L, ttistable!((*L).top.sub(1)));
    match ttype!(o) {
        x if x == lua_Type::LUA_TFUNCTION as c_int => {
            let cl = core::ptr::addr_of_mut!((*(*o).value.gc).cl) as *mut Closure;
            (*cl).env = hvalue!((*L).top.sub(1));
        }
        x if x == lua_Type::LUA_TTHREAD as c_int => {
            let th = core::ptr::addr_of_mut!((*(*o).value.gc).th) as *mut lua_State;
            (*th).gt = hvalue!((*L).top.sub(1));
        }
        _ => {
            res = 0;
        }
    }
    if res != 0 {
        luaC_objbarrier!(L, gcvalue!(o), hvalue!((*L).top.sub(1)) as *mut LuaTable);
    }
    (*L).top = (*L).top.sub(1);
    res
}
