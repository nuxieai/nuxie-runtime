//! Node: `cxx:Function:Luau.VM:VM/src/ldebug.cpp:277:luaG_ordererror`
//! Source: `VM/src/ldebug.cpp:277-284` (hand-ported)

use core::ffi::{c_char, CStr};

use crate::functions::lua_t_objtypename::lua_t_objtypename;
use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;
use crate::type_aliases::tms::TMS;

#[allow(non_snake_case)]
pub unsafe fn luaG_ordererror(
    L: *mut lua_State,
    p1: *const TValue,
    p2: *const TValue,
    op: TMS,
) -> ! {
    let t1: *const c_char = lua_t_objtypename(L, p1);
    let t2: *const c_char = lua_t_objtypename(L, p2);
    let opname: &str = if op == TMS::TM_LT {
        "<"
    } else if op == TMS::TM_LE {
        "<="
    } else {
        "=="
    };

    lua_g_runerror!(
        L,
        "attempt to compare {} {} {}",
        CStr::from_ptr(t1).to_string_lossy(),
        opname,
        CStr::from_ptr(t2).to_string_lossy()
    )
}

#[allow(non_snake_case)]
pub unsafe fn lua_g_ordererror(
    L: *mut lua_State,
    p1: *const TValue,
    p2: *const TValue,
    op: TMS,
) -> ! {
    luaG_ordererror(L, p1, p2, op)
}
