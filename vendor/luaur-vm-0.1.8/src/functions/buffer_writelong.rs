use luaur_common::macros::luau_big_endian::LUAU_BIG_ENDIAN;

use crate::functions::buffer_swapbe::buffer_swapbe;
use crate::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_l_checkinteger_64::lua_l_checkinteger_64;
use crate::macros::isoutofbounds::isoutofbounds;
use crate::macros::lua_l_error::luaL_error;
use crate::type_aliases::lua_state::lua_State;

pub fn buffer_writelong(L: *mut lua_State) -> core::ffi::c_int {
    let mut len: usize = 0;
    let buf = lua_l_checkbuffer(L, 1, &mut len) as *mut core::ffi::c_char;
    let offset = lua_l_checkinteger(L, 2);
    let value = unsafe { lua_l_checkinteger_64(L, 3) };

    if isoutofbounds(offset, len, core::mem::size_of::<i64>()) {
        unsafe {
            luaL_error!(L, "buffer access out of bounds");
        }
    }

    let value = if LUAU_BIG_ENDIAN {
        unsafe { buffer_swapbe(value) }
    } else {
        value
    };

    unsafe {
        core::ptr::copy_nonoverlapping(
            &value as *const i64 as *const core::ffi::c_char,
            buf.add(offset as usize),
            core::mem::size_of::<i64>(),
        );
    }

    0
}
