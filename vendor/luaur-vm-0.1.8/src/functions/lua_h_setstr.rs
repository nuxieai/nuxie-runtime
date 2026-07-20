use crate::functions::lua_h_getstr::lua_h_getstr;
use crate::functions::newkey::newkey;
use crate::macros::cast_to::cast_to;
use crate::macros::invalidate_t_mcache::invalidateTMcache;
use crate::macros::setsvalue::setsvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::t_string::TString;
use crate::type_aliases::t_value::TValue;

use crate::macros::lua_o_nilobject::luaO_nilobject;

#[allow(non_snake_case)]
pub unsafe fn lua_h_setstr(L: *mut lua_State, t: *mut LuaTable, key: *mut TString) -> *mut TValue {
    // The dependency card for lua_h_getstr shows a stub signature pub fn lua_h_getstr();
    // We must transmute it to the real signature (t, key) -> *const TValue to call it.
    type LuaHGetStrFn = unsafe fn(*mut LuaTable, *mut TString) -> *const TValue;
    let lua_h_getstr_ptr =
        core::mem::transmute::<_, LuaHGetStrFn>(lua_h_getstr as *const core::ffi::c_void);

    let p = lua_h_getstr_ptr(t, key);
    invalidateTMcache(t);

    if p != luaO_nilobject {
        cast_to!(*mut TValue, p)
    } else {
        let mut k: TValue = core::mem::zeroed();
        setsvalue!(L, &mut k, key);

        // The newkey stub in the context is fn newkey(), but the C++ source and
        // other examples show it takes (L, t, k). We must use the real signature
        // required by the logic.
        type NewKeyFn = unsafe fn(*mut lua_State, *mut LuaTable, *const TValue) -> *mut TValue;
        let newkey_ptr: NewKeyFn = core::mem::transmute(newkey as *const core::ffi::c_void);
        newkey_ptr(L, t, &k)
    }
}
