//! Node: `cxx:Function:Luau.VM:VM/src/lbaselib.cpp:108:getfunc`
//!
//! `getfunc` — resolve the function argument for `getfenv`/`setfenv`: either the
//! explicit function at slot 1, or the function at stack `level` (via
//! `lua_getinfo`'s `f` option, which pushes it). Used by the env builtins.

use crate::functions::lua_getinfo::lua_getinfo;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_l_optinteger::lua_l_optinteger;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::macros::lua_isfunction::lua_isfunction;
use crate::macros::lua_isnil::lua_isnil;
use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::macros::lua_l_argerror::luaL_argerror;
use crate::macros::lua_l_error::luaL_error;
use crate::records::lua_debug::LuaDebug;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

pub fn getfunc(L: *mut lua_State, opt: i32) {
    unsafe {
        if lua_isfunction!(L, 1) {
            lua_pushvalue(L, 1);
        } else {
            let mut ar: LuaDebug = core::mem::zeroed();
            let level: c_int = if opt != 0 {
                lua_l_optinteger(L, 1, 1)
            } else {
                lua_l_checkinteger(L, 1)
            };
            luaL_argcheck!(L, level >= 0, 1, "level must be non-negative");
            if lua_getinfo(L, level, c"f".as_ptr(), &mut ar) == 0 {
                luaL_argerror!(L, 1, "invalid level");
            }
            if lua_isnil!(L, -1) {
                luaL_error!(
                    L,
                    "no function environment for tail call at level {}",
                    level
                );
            }
        }
    }
}
