//! Node: `cxx:Function:Luau.VM:VM/src/ldebug.cpp:249:luaG_forerrorL`
//! Source: `VM/src/ldebug.cpp:249-254` (hand-ported)

use core::ffi::{c_char, CStr};

use crate::functions::lua_t_objtypename::lua_t_objtypename;
use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luaG_forerror_l(L: *mut lua_State, o: *const TValue, what: *const c_char) -> ! {
    let t: *const c_char = lua_t_objtypename(L, o);

    lua_g_runerror!(
        L,
        "invalid 'for' {} (number expected, got {})",
        CStr::from_ptr(what).to_string_lossy(),
        CStr::from_ptr(t).to_string_lossy()
    )
}

#[allow(non_snake_case)]
pub unsafe fn lua_g_forerror_l(L: *mut lua_State, o: *const TValue, what: *const c_char) -> ! {
    luaG_forerror_l(L, o, what)
}
