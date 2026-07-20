//! Node: `cxx:Function:Luau.VM:VM/src/lcorolib.cpp:144:auxwrapfinish`
//! Source: `VM/src/lcorolib.cpp:144-157` (hand-ported)

use crate::functions::lua_concat::lua_concat;
use crate::functions::lua_error::lua_error;
use crate::functions::lua_insert::lua_insert;
use crate::functions::lua_isstring::lua_isstring;
use crate::functions::lua_l_where::lua_l_where;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn auxwrapfinish(L: *mut lua_State, r: core::ffi::c_int) -> core::ffi::c_int {
    if r < 0 {
        if lua_isstring(L, -1) != 0 {
            lua_l_where(L, 1);
            lua_insert(L, -2);
            lua_concat(L, 2);
        }
        lua_error(L);
    }
    r
}
