//! Node: `cxx:Function:Luau.VM:VM/src/ldebug.cpp:242:luaG_typeerrorL`
//! Source: `VM/src/ldebug.cpp:242-247` (hand-ported)

use core::ffi::{c_char, CStr};

use crate::functions::lua_t_objtypename::lua_t_objtypename;
use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_g_typeerror_l(L: *mut lua_State, o: *const TValue, op: *const c_char) -> ! {
    let t: *const c_char = lua_t_objtypename(L, o);

    lua_g_runerror!(
        L,
        "attempt to {} a {} value",
        CStr::from_ptr(op).to_string_lossy(),
        CStr::from_ptr(t).to_string_lossy()
    )
}

#[allow(non_snake_case)]
pub unsafe fn luaG_typeerrorL(L: *mut lua_State, o: *const TValue, op: *const c_char) -> ! {
    lua_g_typeerror_l(L, o, op)
}
