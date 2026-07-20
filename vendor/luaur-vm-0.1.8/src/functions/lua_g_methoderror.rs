//! Node: `cxx:Function:Luau.VM:VM/src/ldebug.cpp:306:luaG_methoderror`
//! Source: `VM/src/ldebug.cpp:306-311` (hand-ported)

use core::ffi::{c_char, CStr};

use crate::functions::lua_t_objtypename::lua_t_objtypename;
use crate::macros::getstr::getstr;
use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_string::TString;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_g_methoderror(L: *mut lua_State, p1: *const TValue, p2: *const TString) -> ! {
    let t1: *const c_char = lua_t_objtypename(L, p1);

    lua_g_runerror!(
        L,
        "attempt to call missing method '{}' of {}",
        CStr::from_ptr(getstr(p2)).to_string_lossy(),
        CStr::from_ptr(t1).to_string_lossy()
    )
}

#[allow(non_snake_case)]
pub unsafe fn luaG_methoderror(L: *mut lua_State, p1: *const TValue, p2: *const TString) -> ! {
    lua_g_methoderror(L, p1, p2)
}
