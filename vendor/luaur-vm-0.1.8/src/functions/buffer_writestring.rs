use crate::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_l_checklstring::lua_l_checklstring;
use crate::functions::lua_l_optinteger::lua_l_optinteger;
use crate::macros::isoutofbounds::isoutofbounds;
use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::macros::lua_l_error::luaL_error;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

pub unsafe fn buffer_writestring(L: *mut lua_State) -> c_int {
    let mut len: usize = 0;
    let buf = lua_l_checkbuffer(L, 1, &mut len);
    let offset = lua_l_checkinteger(L, 2);

    let mut size: usize = 0;
    let val = lua_l_checklstring(L, 3, &mut size);
    let count = lua_l_optinteger(L, 4, size as c_int);

    luaL_argcheck!(L, count >= 0, 4, "count");

    if count as usize > size {
        luaL_error!(L, "string length overflow");
    }

    if isoutofbounds(offset, len, count as usize) {
        luaL_error!(L, "buffer access out of bounds");
    }

    core::ptr::copy_nonoverlapping(
        val,
        (buf as *mut core::ffi::c_char).add(offset as usize),
        count as usize,
    );

    0
}
