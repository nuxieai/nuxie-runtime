use crate::enums::lua_type::lua_Type;
use crate::functions::lua_tolstring::lua_tolstring;
use crate::functions::tag_error::tag_error;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;
use core::ffi::c_int;

pub unsafe fn lua_l_checklstring(L: *mut lua_State, narg: c_int, len: *mut usize) -> *const c_char {
    let s = lua_tolstring(L, narg, len);
    if s.is_null() {
        tag_error(L, narg, lua_Type::LUA_TSTRING as c_int);
    }
    s
}
