//! Node: `cxx:Function:Luau.VM:VM/src/lbaselib.cpp:293:luaB_pcally`
//! Source: `VM/src/lbaselib.cpp:293-312` (hand-ported)

use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_l_checkany::lua_l_checkany;
use crate::functions::lua_l_pcallyieldable::lua_l_pcallyieldable;
use crate::macros::lua_multret::LUA_MULTRET;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_b_pcally(L: *mut lua_State) -> i32 {
    lua_l_checkany(L, 1);

    lua_l_pcallyieldable(L, lua_gettop(L) - 1, LUA_MULTRET, 0)
}
