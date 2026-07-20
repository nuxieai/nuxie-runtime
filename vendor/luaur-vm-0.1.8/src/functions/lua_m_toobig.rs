//! Node: `cxx:Function:Luau.VM:VM/src/lmem.cpp:wrapper:luaM_toobig`
//! Source: `VM/src/lmem.cpp` (hand-ported)

use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_m_toobig(L: *mut lua_State) -> ! {
    lua_g_runerror!(L, "memory allocation error: block too big")
}

#[allow(non_snake_case)]
pub unsafe fn luaM_toobig(L: *mut lua_State) -> ! {
    lua_m_toobig(L)
}
