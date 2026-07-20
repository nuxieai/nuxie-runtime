use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::functions::lua_pushnil::lua_pushnil;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

#[allow(non_snake_case)]
pub unsafe fn lua_pushstring(l: *mut lua_State, s: *const c_char) {
    if s.is_null() {
        lua_pushnil(l);
    } else {
        let mut len: usize = 0;
        while *s.add(len) != 0 {
            len += 1;
        }
        lua_pushlstring(l, s, len);
    }
}
