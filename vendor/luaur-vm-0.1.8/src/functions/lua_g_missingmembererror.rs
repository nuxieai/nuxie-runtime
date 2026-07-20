use core::ffi::CStr;

use crate::functions::lua_t_objtypename::lua_t_objtypename;
use crate::macros::getstr::getstr;
use crate::macros::lua_g_runerror::lua_g_runerror;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttisstring::ttisstring;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luaG_missingmembererror(
    L: *mut lua_State,
    p1: *const TValue,
    p2: *const TValue,
) -> ! {
    if !ttisstring!(p2) {
        let t1 = lua_t_objtypename(L, p1);
        let t2 = lua_t_objtypename(L, p2);
        lua_g_runerror!(
            L,
            "cannot index {} with a {}",
            CStr::from_ptr(t1).to_string_lossy(),
            CStr::from_ptr(t2).to_string_lossy(),
        )
    } else {
        let t1 = lua_t_objtypename(L, p1);
        let key = tsvalue!(p2);
        lua_g_runerror!(
            L,
            "this {} does not have a key named '{}'",
            CStr::from_ptr(t1).to_string_lossy(),
            CStr::from_ptr(getstr(key)).to_string_lossy(),
        )
    }
}

#[allow(non_snake_case)]
pub unsafe fn lua_g_missingmembererror(
    L: *mut lua_State,
    p1: *const TValue,
    p2: *const TValue,
) -> ! {
    luaG_missingmembererror(L, p1, p2)
}
