use crate::functions::lua_getmetatable::lua_getmetatable;
use crate::functions::lua_next::lua_next;
use crate::functions::lua_pushnil::lua_pushnil;
use crate::functions::lua_setreadonly::lua_setreadonly;
use crate::functions::lua_setsafeenv::lua_setsafeenv;
use crate::functions::lua_type::lua_type;
use crate::macros::lua_globalsindex::LUA_GLOBALSINDEX;
use crate::macros::lua_istable::lua_istable;
use crate::macros::lua_pop::lua_pop;
use crate::macros::lua_pushliteral::lua_pushliteral;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn lua_l_sandbox(l: *mut lua_State) {
    // set all libraries to read-only
    lua_pushnil(l);
    while lua_next(l, LUA_GLOBALSINDEX) != 0 {
        // lua_istable! macro uses lua_type internally; we check the type directly.
        if lua_type(l, -1) == crate::enums::lua_type::lua_Type::LUA_TTABLE as i32 {
            lua_setreadonly(l, -1, 1);
        }
        lua_pop(l, 1);
    }

    // set all builtin metatables to read-only
    lua_pushliteral(l as *mut core::ffi::c_void, c"".as_ptr());
    if lua_getmetatable(l, -1) != 0 {
        lua_setreadonly(l, -1, 1);
        lua_pop(l, 2);
    } else {
        lua_pop(l, 1);
    }

    // set globals to readonly and activate safeenv since the env is immutable
    lua_setreadonly(l, LUA_GLOBALSINDEX, 1);
    lua_setsafeenv(l, LUA_GLOBALSINDEX, 1);
}
