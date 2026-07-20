use core::ffi::c_char;

use crate::functions::lua_t_objtypenamestr::lua_t_objtypenamestr;
use crate::macros::getstr::getstr;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_t_objtypename(L: *mut lua_State, o: *const TValue) -> *const c_char {
    getstr(lua_t_objtypenamestr(L, o))
}
