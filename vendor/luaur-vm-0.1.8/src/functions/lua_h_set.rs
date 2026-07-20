use crate::functions::lua_h_get::lua_h_get;
use crate::functions::lua_h_newkey::lua_h_newkey;
use crate::macros::cast_to::cast_to;
use crate::macros::invalidate_t_mcache::invalidateTMcache;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

use crate::macros::lua_o_nilobject::luaO_nilobject;

#[allow(non_snake_case)]
pub unsafe fn luaH_set(L: *mut lua_State, t: *mut LuaTable, key: *const TValue) -> *mut TValue {
    let p = lua_h_get(t, key);
    invalidateTMcache(t);

    if p != luaO_nilobject {
        cast_to!(*mut TValue, p)
    } else {
        lua_h_newkey(L, t, key)
    }
}
