use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::lua_replace::lua_replace;
use crate::functions::lua_setfield::lua_setfield;
use crate::functions::lua_setmetatable::lua_setmetatable;
use crate::functions::lua_setreadonly::lua_setreadonly;
use crate::functions::lua_setsafeenv::lua_setsafeenv;
use crate::macros::lua_globalsindex::LUA_GLOBALSINDEX;
use crate::macros::lua_newtable::lua_newtable;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_l_sandboxthread(L: *mut lua_State) {
    // create new global table that proxies reads to original table
    lua_newtable(L);

    lua_newtable(L);

    lua_pushvalue(L, LUA_GLOBALSINDEX);

    lua_setfield(L, -2, c"__index".as_ptr());

    lua_setreadonly(L, -1, 1);

    lua_setmetatable(L, -2);

    // we can set safeenv now although it's important to set it to false if code is loaded twice into the thread
    lua_replace(L, LUA_GLOBALSINDEX);

    lua_setsafeenv(L, LUA_GLOBALSINDEX, 1);
}
