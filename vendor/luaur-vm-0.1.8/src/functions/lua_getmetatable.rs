use core::ffi::c_int;

use crate::enums::lua_type::lua_Type;
use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::hvalue::hvalue;
use crate::macros::objectvalue::objectvalue;
use crate::macros::sethvalue::sethvalue;
use crate::macros::ttype::ttype;
use crate::macros::uvalue::uvalue;
use crate::records::lua_state::lua_State;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_getmetatable(L: *mut lua_State, objindex: c_int) -> c_int {
    lua_c_threadbarrier_lapi(L);

    let mut mt: *mut LuaTable = core::ptr::null_mut();
    let obj: StkId = index2addr(L, objindex);

    match ttype!(obj) {
        x if x == lua_Type::LUA_TTABLE as c_int => {
            mt = (*hvalue!(obj)).metatable;
        }
        x if x == lua_Type::LUA_TUSERDATA as c_int => {
            mt = (*uvalue!(obj)).metatable;
        }
        x if x == lua_Type::LUA_TOBJECT as c_int => {
            mt = (*(*objectvalue!(obj)).lclass).instancemetatable;
        }
        _ => {
            mt = (*(*L).global).mt[ttype!(obj) as usize];
        }
    }

    if !mt.is_null() {
        sethvalue!(L, (*L).top, mt);
        api_incr_top!(L);
    }

    (!mt.is_null()) as c_int
}
