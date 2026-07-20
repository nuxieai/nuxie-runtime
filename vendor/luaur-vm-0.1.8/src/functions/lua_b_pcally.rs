//! Node: `cxx:Function:Luau.VM:VM/src/lbaselib.cpp:293:luaB_pcally`
//! Source: `VM/src/lbaselib.cpp:293-312` (hand-ported)

use crate::functions::lua_b_pcallrun::lua_b_pcallrun;
use crate::functions::lua_d_pcall::luaD_pcall;
use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_insert::lua_insert;
use crate::functions::lua_l_checkany::lua_l_checkany;
use crate::functions::lua_pushboolean::lua_pushboolean;
use crate::functions::lua_rawcheckstack::lua_rawcheckstack;
use crate::macros::c_call_yield::C_CALL_YIELD;
use crate::macros::expandstacklimit::expandstacklimit;
use crate::macros::isyielded::isyielded;
use crate::macros::lua_callinfo_handle::LUA_CALLINFO_HANDLE;
use crate::macros::savestack::savestack;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_b_pcally(L: *mut lua_State) -> i32 {
    lua_l_checkany(L, 1);

    let func: StkId = (*L).base;
    (*(*L).ci).flags |= LUA_CALLINFO_HANDLE as u32;

    let status = luaD_pcall(
        L,
        Some(lua_b_pcallrun),
        func as *mut core::ffi::c_void,
        savestack!(L, func) as isize,
        0,
    );

    expandstacklimit!(L, (*L).top);

    if status == 0 && isyielded(L) {
        return C_CALL_YIELD;
    }

    lua_rawcheckstack(L, 1);
    lua_pushboolean(L, (status == 0) as i32);
    lua_insert(L, 1);
    lua_gettop(L)
}
