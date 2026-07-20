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
) -> *mut TValue {
    // The dependency card for lua_h_getp shows a stub signature pub fn lua_h_getp();
    // We must transmute it to the real signature (t, key, tag) -> *const TValue to call it.
    type LuaHGetPFn = unsafe fn(*mut LuaTable, *mut core::ffi::c_void, i32) -> *const TValue;
    let lua_h_getp_ptr =
        core::mem::transmute::<_, LuaHGetPFn>(lua_h_getp as *const core::ffi::c_void);

    let p = lua_h_getp_ptr(t, key, tag);

    if p != luaO_nilobject {
        cast_to!(*mut TValue, p)
    } else {
        let mut k: TValue = core::mem::zeroed();

        // setpvalue(obj, x, tag) logic:
        // i_o->value.p = (x); i_o->extra[0] = (tag); i_o->tt = LUA_TLIGHTUSERDATA;
        k.value.p = key;
        k.extra[0] = tag;
        k.tt = 2; // LUA_TLIGHTUSERDATA

        // The dependency card for newkey shows a stub signature pub fn newkey();
        // We must transmute it to the real signature (L, t, key) -> *mut TValue to call it.
        type NewKeyFn = unsafe fn(*mut lua_State, *mut LuaTable, *const TValue) -> *mut TValue;
        let newkey_ptr = core::mem::transmute::<_, NewKeyFn>(newkey as *const core::ffi::c_void);

        newkey_ptr(L, t, &k)
    }
}
