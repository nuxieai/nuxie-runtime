use crate::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_l_error_l::lua_l_error_l;
use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::macros::isoutofbounds::isoutofbounds;
use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int, c_void};

pub fn buffer_readstring(L: *mut lua_State) -> c_int {
    let mut len: usize = 0;
    let buf = lua_l_checkbuffer(L, 1, &mut len);
    let offset = lua_l_checkinteger(L, 2);
    let size = lua_l_checkinteger(L, 3);

    luaL_argcheck!(L, size >= 0, 3, "size");

    if isoutofbounds(offset, len, size as usize) {
        let msg = b"buffer access out of bounds\0";
        unsafe {
            lua_l_error_l(
                L,
                msg.as_ptr() as *const c_char,
                core::format_args!("buffer access out of bounds"),
            );
        }
    }

    unsafe {
        let data_ptr = (buf as *const c_char).add(offset as usize);
        lua_pushlstring(L, data_ptr, size as usize);
    }

    1
}
