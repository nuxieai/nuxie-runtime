use crate::enums::lua_type::lua_Type;
use crate::functions::getfunc::getfunc;
use crate::functions::lua_getfenv::lua_getfenv;
use crate::functions::lua_iscfunction::lua_iscfunction;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::lua_setsafeenv::lua_setsafeenv;
use crate::macros::lua_globalsindex::LUA_GLOBALSINDEX;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn lua_b_getfenv(L: *mut lua_State) -> i32 {
    getfunc(L, 1);
    if lua_iscfunction(L, -1) != 0 {
        lua_pushvalue(L, LUA_GLOBALSINDEX);
    } else {
        lua_getfenv(L, -1);
    }
    lua_setsafeenv(L, -1, 0);
    1
}
