//! Node: `cxx:Function:Luau.VM:VM/src/laux.cpp:159:luaL_checkany`
//! Source: `VM/src/laux.cpp:159-163` (hand-ported)

use core::ffi::c_int;

use crate::functions::lua_l_error_l::lua_l_error_l;
use crate::functions::lua_type::lua_type;
use crate::macros::lua_tnone::LUA_TNONE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_l_checkany(L: *mut lua_State, narg: c_int) {
    if lua_type(L, narg) == LUA_TNONE {
        lua_l_error_l(
            L,
            c"missing argument #%d".as_ptr(),
            format_args!("missing argument #{}", narg),
        );
    }
}
