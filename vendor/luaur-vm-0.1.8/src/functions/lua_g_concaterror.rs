//! Node: `cxx:Function:Luau.VM:VM/src/ldebug.cpp:256:luaG_concaterror`
//! Source: `VM/src/ldebug.cpp:256-262` (hand-ported)

use core::ffi::{c_char, CStr};

use crate::functions::lua_t_objtypename::lua_t_objtypename;
use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_g_concaterror(L: *mut lua_State, p1: StkId, p2: StkId) -> ! {
    let t1: *const c_char = lua_t_objtypename(L, p1);
    let t2: *const c_char = lua_t_objtypename(L, p2);

    lua_g_runerror!(
        L,
        "attempt to concatenate {} with {}",
        CStr::from_ptr(t1).to_string_lossy(),
        CStr::from_ptr(t2).to_string_lossy()
    )
}

#[allow(non_snake_case)]
pub unsafe fn luaG_concaterror(L: *mut lua_State, p1: StkId, p2: StkId) -> ! {
    lua_g_concaterror(L, p1, p2)
}
