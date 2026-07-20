use crate::enums::lua_type::lua_Type;
use crate::functions::lua_newuserdatatagged::lua_newuserdatatagged;
use crate::functions::lua_setmetatable::lua_setmetatable;
use crate::functions::lua_toboolean::lua_toboolean;
use crate::functions::lua_type::lua_type;
use crate::macros::lua_l_argexpected::luaL_argexpected;
use crate::macros::lua_newtable::lua_newtable;
use crate::macros::utag_proxy::UTAG_PROXY;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_b_newproxy(L: *mut lua_State) -> i32 {
    let t = lua_type(L, 1);
    luaL_argexpected!(
        L,
        t == lua_Type::LUA_TNIL as i32
            || t == lua_Type::LUA_TBOOLEAN as i32
            || t == lua_Type::LUA_TNONE as i32,
        1,
        "nil or boolean"
    );

    let needsmt = lua_toboolean(L, 1) != 0;

    lua_newuserdatatagged(L, 0, UTAG_PROXY);

    if needsmt {
        lua_newtable(L);
        lua_setmetatable(L, -2);
    }

    1
}
