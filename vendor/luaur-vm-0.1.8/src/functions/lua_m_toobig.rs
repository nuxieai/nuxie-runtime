//! Node: `cxx:Function:Luau.VM:VM/src/lmem.cpp:wrapper:luaM_toobig`
//! Source: `VM/src/lmem.cpp` (hand-ported)

use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_m_toobig<T>(L: *mut lua_State) -> crate::records::lua_exception::LuaResult<T> {
    lua_g_runerror!(L, "memory allocation error: block too big")
}

#[allow(non_snake_case)]
pub unsafe fn luaM_toobig<T>(L: *mut lua_State) -> crate::records::lua_exception::LuaResult<T> {
    lua_m_toobig(L)
}
