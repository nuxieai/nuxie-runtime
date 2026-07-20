use crate::functions::lua_g_runerror_l::lua_g_runerror_l;
use crate::functions::luai_vecisnan::luai_vecisnan;
use crate::functions::newkey::newkey;
use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::macros::luai_numisnan::luai_numisnan;
use crate::macros::nvalue::nvalue;
use crate::macros::ttisnil::ttisnil;
use crate::macros::ttisnumber::ttisnumber;
use crate::macros::ttisvector::ttisvector;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_h_newkey(l: *mut lua_State, t: *mut LuaTable, key: *const TValue) -> *mut TValue {
    if ttisnil!(key) {
        lua_g_runerror_l(l, core::ptr::null(), format_args!("table index is nil"));
    } else if ttisnumber!(key) && luai_numisnan(nvalue!(key)) {
        lua_g_runerror_l(l, core::ptr::null(), format_args!("table index is NaN"));
    } else if ttisvector!(key) && luai_vecisnan(vvalue!(key).as_ptr()) {
        lua_g_runerror_l(
            l,
            core::ptr::null(),
            format_args!("table index contains NaN"),
        );
    }

    newkey(l, t, key)
}
