//! Node: `cxx:Function:Luau.VM:VM/src/ldebug.cpp:313:luaG_readonlyerror`
//! Source: `VM/src/ldebug.cpp:313-316` (hand-ported)

use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_g_readonlyerror(L: *mut lua_State) -> ! {
    lua_g_runerror!(L, "attempt to modify a readonly table")
}

#[allow(non_snake_case)]
pub unsafe fn luaG_readonlyerror(L: *mut lua_State) -> ! {
    lua_g_readonlyerror(L)
}
