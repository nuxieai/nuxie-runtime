//! Node: `cxx:Function:Luau.VM:VM/src/lbaselib.cpp:48:luaB_error`
//! Source: `VM/src/lbaselib.cpp:48-59` (hand-ported)

use crate::functions::lua_concat::lua_concat;
use crate::functions::lua_error::lua_error;
use crate::functions::lua_isstring::lua_isstring;
use crate::functions::lua_l_optinteger::lua_l_optinteger;
use crate::functions::lua_l_where::lua_l_where;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::lua_settop::lua_settop;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_b_error(L: *mut lua_State) -> i32 {
    let level = lua_l_optinteger(L, 2, 1);
    lua_settop(L, 1);
    if lua_isstring(L, 1) != 0 && level > 0 {
        lua_l_where(L, level);
        lua_pushvalue(L, 1);
        lua_concat(L, 2);
    }
    lua_error(L);
}
