//! Node: `cxx:Function:Luau.VM:VM/src/ldebug.cpp:286:luaG_indexerror`
//! Source: `VM/src/ldebug.cpp:286-296` (hand-ported)

use core::ffi::{c_char, CStr};

use crate::functions::lua_t_objtypename::lua_t_objtypename;
use crate::macros::getstr::getstr;
use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttisstring::ttisstring;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_string::TString;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_g_indexerror(L: *mut lua_State, p1: *const TValue, p2: *const TValue) -> ! {
    let t1: *const c_char = lua_t_objtypename(L, p1);
    let t2: *const c_char = lua_t_objtypename(L, p2);
    let key: *const TString = if ttisstring!(p2) {
        tsvalue!(p2)
    } else {
        core::ptr::null()
    };

    // limit length to make sure we don't generate very long error messages for very long keys
    if !key.is_null() && (*key).len <= 64 {
        lua_g_runerror!(
            L,
            "attempt to index {} with '{}'",
            CStr::from_ptr(t1).to_string_lossy(),
            CStr::from_ptr(getstr(key)).to_string_lossy()
        )
    } else {
        lua_g_runerror!(
            L,
            "attempt to index {} with {}",
            CStr::from_ptr(t1).to_string_lossy(),
            CStr::from_ptr(t2).to_string_lossy()
        )
    }
}

#[allow(non_snake_case)]
pub unsafe fn luaG_indexerror(L: *mut lua_State, p1: *const TValue, p2: *const TValue) -> ! {
    lua_g_indexerror(L, p1, p2)
}
