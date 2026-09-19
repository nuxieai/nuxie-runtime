use crate::functions::lua_h_getp::lua_h_getp;
use crate::functions::newkey::newkey;
use crate::macros::cast_to::cast_to;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_h_setp(
    L: *mut lua_State,
    t: *mut LuaTable,
    key: *mut core::ffi::c_void,
    tag: i32,
) -> crate::records::lua_exception::LuaResult<*mut TValue> {
    let p = lua_h_getp(t, key, tag);

    if p != luaO_nilobject {
        Ok(cast_to!(*mut TValue, p))
    } else {
        let mut k: TValue = core::mem::zeroed();

        // setpvalue(obj, x, tag) logic:
        // i_o->value.p = (x); i_o->extra[0] = (tag); i_o->tt = LUA_TLIGHTUSERDATA;
        k.value.p = key;
        k.extra[0] = tag;
        k.tt = 2; // LUA_TLIGHTUSERDATA

        newkey(L, t, &k)
    }
}
