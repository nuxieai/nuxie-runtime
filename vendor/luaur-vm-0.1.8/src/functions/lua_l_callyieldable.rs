use core::ffi::c_int;

use crate::functions::lua_call::lua_call;
use crate::macros::api_check::api_check;
use crate::macros::c_call_yield::C_CALL_YIELD;
use crate::macros::clvalue::clvalue;
use crate::macros::iscfunction::iscfunction;
use crate::macros::isyielded::isyielded;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_l_callyieldable(L: *mut lua_State, nargs: c_int, nresults: c_int) -> c_int {
    api_check!(L, iscfunction!((*(*L).ci).func));
    let cl = clvalue!((*(*L).ci).func);
    let c = core::ptr::addr_of!((*cl).inner.c).cast::<crate::records::closure::CClosure>();
    api_check!(L, (*c).cont.is_some());

    lua_call(L, nargs, nresults);

    if isyielded(L) {
        return C_CALL_YIELD;
    }

    ((*c).cont.unwrap())(L, crate::enums::lua_status::LuaStatus::LUA_OK as c_int)
}
