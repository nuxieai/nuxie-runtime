use crate::functions::lua_isnumber::lua_isnumber;
use crate::functions::lua_rawgetfield::lua_rawgetfield;
use crate::macros::lua_l_error::luaL_error;
use crate::macros::lua_pop::lua_pop;
use crate::macros::lua_tointeger::lua_tointeger;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

pub fn getfield(L: *mut lua_State, key: &str, d: i32) -> i32 {
    let key_bytes = key.as_bytes();
    let mut buf = key_bytes.to_vec();
    buf.push(0);
    let key_c: *const c_char = buf.as_ptr() as *const c_char;

    unsafe {
        lua_rawgetfield(L, -1, key_c);

        if lua_isnumber(L, -1) != 0 {
            let res = lua_tointeger!(L, -1) as i32;
            lua_pop(L, 1);
            res
        } else {
            if d < 0 {
                luaL_error!(L, "field '{}' missing in date table", key);
            }
            lua_pop(L, 1);
            d
        }
    }
}
