//! Node: `cxx:Function:Luau.VM:VM/src/ldebug.cpp:264:luaG_aritherror`
//! Source: `VM/src/ldebug.cpp:264-275` (hand-ported; `luaT_eventname[op]`
//! is read from `g->tmname[op]`, built from the same string table)

use core::ffi::{c_char, CStr};

use crate::functions::lua_t_objtypename::lua_t_objtypename;
use crate::macros::getstr::getstr;
use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;
use crate::type_aliases::tms::TMS;

#[allow(non_snake_case)]
pub unsafe fn lua_g_aritherror(
    L: *mut lua_State,
    p1: *const TValue,
    p2: *const TValue,
    op: TMS,
) -> ! {
    let t1: *const c_char = lua_t_objtypename(L, p1);
    let t2: *const c_char = lua_t_objtypename(L, p2);
    // skip __ from metamethod name
    let opname = getstr((*(*L).global).tmname[op as usize]).add(2);

    if t1 == t2 {
        // C++ compares interned typename pointers
        lua_g_runerror!(
            L,
            "attempt to perform arithmetic ({}) on {}",
            CStr::from_ptr(opname).to_string_lossy(),
            CStr::from_ptr(t1).to_string_lossy()
        )
    } else {
        lua_g_runerror!(
            L,
            "attempt to perform arithmetic ({}) on {} and {}",
            CStr::from_ptr(opname).to_string_lossy(),
            CStr::from_ptr(t1).to_string_lossy(),
            CStr::from_ptr(t2).to_string_lossy()
        )
    }
}

#[allow(non_snake_case)]
pub unsafe fn luaG_aritherror(
    L: *mut lua_State,
    p1: *const TValue,
    p2: *const TValue,
    op: TMS,
) -> ! {
    lua_g_aritherror(L, p1, p2, op)
}
