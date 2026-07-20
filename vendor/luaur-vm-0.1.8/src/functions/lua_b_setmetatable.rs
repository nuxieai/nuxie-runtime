//! Node: `cxx:Function:Luau.VM:VM/src/lbaselib.cpp:75:luaB_setmetatable`
//! Source: `VM/src/lbaselib.cpp:75-85` (hand-ported)

use crate::enums::lua_type::lua_Type;
use crate::functions::lua_l_checktype::lua_l_checktype;
use crate::functions::lua_l_error_l::lua_l_error_l;
use crate::functions::lua_l_getmetafield::lua_l_getmetafield;
use crate::functions::lua_setmetatable::lua_setmetatable;
use crate::functions::lua_settop::lua_settop;
use crate::functions::lua_type::lua_type;
use crate::macros::lua_l_argexpected::luaL_argexpected;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_b_setmetatable(L: *mut lua_State) -> i32 {
    let t = lua_type(L, 2);
    lua_l_checktype(L, 1, lua_Type::LUA_TTABLE as i32);
    luaL_argexpected!(
        L,
        t == lua_Type::LUA_TNIL as i32 || t == lua_Type::LUA_TTABLE as i32,
        2,
        "nil or table"
    );
    if lua_l_getmetafield(L, 1, c"__metatable".as_ptr()) != 0 {
        lua_l_error_l(
            L,
            c"cannot change a protected metatable".as_ptr(),
            format_args!("cannot change a protected metatable"),
        );
    }
    lua_settop(L, 2);
    lua_setmetatable(L, 1);
    1
}
