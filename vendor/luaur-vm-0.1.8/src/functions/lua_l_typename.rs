use core::ffi::c_char;
use core::ffi::c_int;

use crate::functions::lua_a_toobject::luaA_toobject;
use crate::functions::lua_t_objtypename::lua_t_objtypename;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_l_typename(L: *mut lua_State, idx: c_int) -> *const c_char {
    let obj: *const TValue = luaA_toobject(L, idx);

    if obj.is_null() || obj == luaO_nilobject as *const TValue {
        b"no value\0" as *const u8 as *const c_char
    } else {
        lua_t_objtypename(L, obj)
    }
}
